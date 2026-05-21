//! Traffic analysis resistance: fixed-cell padding, cover flow, and jitter.
//!
//! All cells are padded to a fixed size (256B or 1KB) before transmission.
//! During idle periods, cover traffic cells are injected at a configured rate.
//! Random jitter is added to each send to defeat timing correlation.

use rand::{Rng, RngCore};
use std::time::Duration;

use crate::errors::{CloakError, CloakResult};

// ── Cell sizes ───────────────────────────────────────────────────────────────

pub const CELL_514B: usize = 514;
pub const CELL_1KB: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellSize {
    Bytes514,
    Bytes1K,
}

impl CellSize {
    pub fn bytes(self) -> usize {
        match self {
            CellSize::Bytes514 => CELL_514B,
            CellSize::Bytes1K => CELL_1KB,
        }
    }
}

// ── Cell kind ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellKind {
    /// Real payload cell
    Data,
    /// Cover traffic cell (random bytes, indistinguishable from data)
    Cover,
}

// ── Padded cell ──────────────────────────────────────────────────────────────

/// A fixed-size cell ready for transmission.
#[derive(Debug, Clone)]
pub struct PaddedCell {
    pub bytes: Vec<u8>,
    pub kind: CellKind,
    pub size: CellSize,
}

impl PaddedCell {
    /// Verify the cell is exactly the expected size.
    pub fn assert_size(&self) {
        assert_eq!(
            self.bytes.len(),
            self.size.bytes(),
            "cell size invariant violated"
        );
    }
}

// ── Padding engine ───────────────────────────────────────────────────────────

/// Stateless traffic padding engine.
pub struct TrafficEngine {
    cell_size: CellSize,
    cover_flow_pps: u32,
    jitter_max_ms: u32,
    enabled: bool,
}

impl TrafficEngine {
    pub fn new(cell_size: CellSize, cover_flow_pps: u32, jitter_max_ms: u32, enabled: bool) -> Self {
        Self { cell_size, cover_flow_pps, jitter_max_ms, enabled }
    }

    /// Pad `payload` to the configured cell size.
    ///
    /// Padding bytes are cryptographically random to prevent compression-based
    /// side channels (CRIME/BREACH-style attacks).
    ///
    /// Returns an error if `payload` exceeds the cell size.
    pub fn pad(&self, payload: &[u8]) -> CloakResult<PaddedCell> {
        let size = self.cell_size.bytes();
        if payload.len() > size {
            return Err(CloakError::Protocol(format!(
                "payload {} bytes exceeds cell size {} bytes",
                payload.len(),
                size
            )));
        }
        let mut cell = vec![0u8; size];
        cell[..payload.len()].copy_from_slice(payload);
        // Fill padding with random bytes (not zeros) to prevent compression attacks
        if payload.len() < size {
            rand::rngs::OsRng.fill_bytes(&mut cell[payload.len()..]);
        }
        Ok(PaddedCell { bytes: cell, kind: CellKind::Data, size: self.cell_size })
    }

    /// Generate a cover traffic cell (entirely random bytes).
    pub fn cover_cell(&self) -> PaddedCell {
        let size = self.cell_size.bytes();
        let mut bytes = vec![0u8; size];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        PaddedCell { bytes, kind: CellKind::Cover, size: self.cell_size }
    }

    /// Return the jitter delay to apply before sending a cell.
    /// Returns `Duration::ZERO` if padding is disabled.
    pub fn jitter_delay(&self) -> Duration {
        if !self.enabled || self.jitter_max_ms == 0 {
            return Duration::ZERO;
        }
        let ms = rand::thread_rng().gen_range(0..=self.jitter_max_ms) as u64;
        Duration::from_millis(ms)
    }

    /// Returns how many cover cells should be sent per second during idle.
    pub fn cover_flow_rate(&self) -> u32 {
        if self.enabled { self.cover_flow_pps } else { 0 }
    }

    /// Determine whether a cover cell should be sent given the elapsed idle
    /// duration. Uses a Poisson-like schedule: on average `cover_flow_pps`
    /// cells per second.
    pub fn should_send_cover(&self, idle_ms: u64) -> bool {
        if !self.enabled || self.cover_flow_pps == 0 {
            return false;
        }
        let interval_ms = 1000u64 / self.cover_flow_pps as u64;
        idle_ms >= interval_ms
    }

    pub fn cell_size(&self) -> CellSize {
        self.cell_size
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// ── Cover flow scheduler ─────────────────────────────────────────────────────

/// Tracks idle time and decides when to inject cover cells.
pub struct CoverFlowScheduler {
    engine: TrafficEngine,
    last_real_cell_ms: u64,
}

impl CoverFlowScheduler {
    pub fn new(engine: TrafficEngine) -> Self {
        Self { engine, last_real_cell_ms: 0 }
    }

    /// Call this whenever a real data cell is sent.
    pub fn record_data_cell(&mut self, now_ms: u64) {
        self.last_real_cell_ms = now_ms;
    }

    /// Call this on a timer tick. Returns cover cells to send (may be empty).
    pub fn tick(&self, now_ms: u64) -> Vec<PaddedCell> {
        let idle = now_ms.saturating_sub(self.last_real_cell_ms);
        if self.engine.should_send_cover(idle) {
            vec![self.engine.cover_cell()]
        } else {
            vec![]
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> TrafficEngine {
        TrafficEngine::new(CellSize::Bytes514, 1, 50, true)
    }

    #[test]
    fn pad_produces_correct_size() {
        let e = engine();
        let cell = e.pad(b"hello world").unwrap();
        assert_eq!(cell.bytes.len(), CELL_514B);
        assert_eq!(&cell.bytes[..11], b"hello world");
        assert_eq!(cell.kind, CellKind::Data);
    }

    #[test]
    fn pad_empty_payload() {
        let e = engine();
        let cell = e.pad(b"").unwrap();
        assert_eq!(cell.bytes.len(), CELL_514B);
    }

    #[test]
    fn pad_exact_size_payload() {
        let e = engine();
        let payload = vec![0xABu8; CELL_514B];
        let cell = e.pad(&payload).unwrap();
        assert_eq!(cell.bytes, payload);
    }

    #[test]
    fn oversized_payload_rejected() {
        let e = engine();
        let payload = vec![0u8; CELL_514B + 1];
        assert!(e.pad(&payload).is_err());
    }

    #[test]
    fn cover_cell_correct_size() {
        let e = engine();
        let cell = e.cover_cell();
        assert_eq!(cell.bytes.len(), CELL_514B);
        assert_eq!(cell.kind, CellKind::Cover);
    }

    #[test]
    fn padding_bytes_are_random_not_zero() {
        let e = engine();
        let cell = e.pad(b"x").unwrap();
        // With overwhelming probability, 513 random bytes are not all zero
        let all_zero = cell.bytes[1..].iter().all(|&b| b == 0);
        assert!(!all_zero, "padding should be random, not zero");
    }

    #[test]
    fn cover_flow_scheduler_triggers_after_idle() {
        let e = TrafficEngine::new(CellSize::Bytes514, 1, 0, true);
        let mut sched = CoverFlowScheduler::new(e);
        sched.record_data_cell(0);
        // 999ms idle — not yet
        assert!(sched.tick(999).is_empty());
        // 1000ms idle — should trigger
        assert!(!sched.tick(1000).is_empty());
    }

    #[test]
    fn disabled_engine_no_cover_cells() {
        let e = TrafficEngine::new(CellSize::Bytes514, 10, 50, false);
        let sched = CoverFlowScheduler::new(e);
        assert!(sched.tick(99999).is_empty());
    }

    #[test]
    fn jitter_within_bounds() {
        let e = engine();
        for _ in 0..100 {
            let d = e.jitter_delay();
            assert!(d.as_millis() <= 50);
        }
    }

    #[test]
    fn cell_size_1kb() {
        let e = TrafficEngine::new(CellSize::Bytes1K, 1, 0, true);
        let cell = e.pad(b"data").unwrap();
        assert_eq!(cell.bytes.len(), CELL_1KB);
    }
}
