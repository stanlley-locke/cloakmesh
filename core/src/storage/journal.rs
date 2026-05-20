//! Append-only write-ahead journal for crash recovery.
//!
//! Each record is: `[4-byte LE length][payload][4-byte CRC32 checksum]`
//! On recovery, records with invalid checksums are truncated.

use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::errors::{CloakError, CloakResult};

// ── CRC32 ────────────────────────────────────────────────────────────────────

fn crc32(data: &[u8]) -> u32 {
    // Simple CRC32 using the standard polynomial
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

// ── Journal ──────────────────────────────────────────────────────────────────

pub struct Journal {
    #[allow(dead_code)]
    path: PathBuf,
    file: File,
    records_written: u64,
}

impl Journal {
    /// Open or create a journal file.
    pub fn open(path: &Path) -> CloakResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            file,
            records_written: 0,
        })
    }

    /// Append a record to the journal. Flushes and syncs to disk.
    pub fn append(&mut self, data: &[u8]) -> CloakResult<u64> {
        let len = data.len() as u32;
        let checksum = crc32(data);

        self.file.write_all(&len.to_le_bytes())?;
        self.file.write_all(data)?;
        self.file.write_all(&checksum.to_le_bytes())?;
        self.file.flush()?;
        // fsync for durability
        self.file.sync_data()?;

        self.records_written += 1;
        Ok(self.records_written)
    }

    /// Read all valid records from the journal. Stops at the first corrupt record.
    pub fn read_all(path: &Path) -> CloakResult<Vec<Vec<u8>>> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(e.into()),
        };
        let mut reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut offset: u64 = 0;

        loop {
            // Read length prefix
            let mut len_buf = [0u8; 4];
            match reader.read_exact(&mut len_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }
            let len = u32::from_le_bytes(len_buf) as usize;
            offset += 4;

            // Read payload
            let mut payload = vec![0u8; len];
            match reader.read_exact(&mut payload) {
                Ok(()) => {}
                Err(_) => return Err(CloakError::JournalCorrupted(offset)),
            }
            offset += len as u64;

            // Read and verify checksum
            let mut csum_buf = [0u8; 4];
            match reader.read_exact(&mut csum_buf) {
                Ok(()) => {}
                Err(_) => return Err(CloakError::JournalCorrupted(offset)),
            }
            let stored_csum = u32::from_le_bytes(csum_buf);
            let computed_csum = crc32(&payload);
            if stored_csum != computed_csum {
                return Err(CloakError::JournalCorrupted(offset));
            }
            offset += 4;

            records.push(payload);
        }

        Ok(records)
    }

    /// Truncate the journal (e.g. after a successful checkpoint).
    pub fn truncate(&mut self) -> CloakResult<()> {
        self.file.set_len(0)?;
        self.file.seek(SeekFrom::Start(0))?;
        self.records_written = 0;
        Ok(())
    }

    pub fn records_written(&self) -> u64 {
        self.records_written
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_and_read_records() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.journal");

        let mut j = Journal::open(&path).unwrap();
        j.append(b"record one").unwrap();
        j.append(b"record two").unwrap();
        j.append(b"record three").unwrap();

        let records = Journal::read_all(&path).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], b"record one");
        assert_eq!(records[1], b"record two");
        assert_eq!(records[2], b"record three");
    }

    #[test]
    fn empty_journal_returns_empty_vec() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.journal");
        let records = Journal::read_all(&path).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn truncate_clears_journal() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("trunc.journal");
        let mut j = Journal::open(&path).unwrap();
        j.append(b"data").unwrap();
        j.truncate().unwrap();
        let records = Journal::read_all(&path).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn corrupt_checksum_detected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("corrupt.journal");
        let mut j = Journal::open(&path).unwrap();
        j.append(b"good record").unwrap();
        drop(j);

        // Corrupt the last byte (part of the checksum)
        let mut data = std::fs::read(&path).unwrap();
        let last = data.len() - 1;
        data[last] ^= 0xFF;
        std::fs::write(&path, &data).unwrap();

        let result = Journal::read_all(&path);
        assert!(result.is_err());
    }
}
