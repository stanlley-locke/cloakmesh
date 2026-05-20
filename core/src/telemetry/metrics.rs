use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// ── Node metrics ─────────────────────────────────────────────────────────────

/// Thread-safe node metrics counters.
#[derive(Debug, Default)]
pub struct NodeMetrics {
    pub active_circuits: AtomicU64,
    pub total_circuits_built: AtomicU64,
    pub total_circuits_failed: AtomicU64,
    pub bytes_relayed: AtomicU64,
    pub cells_sent: AtomicU64,
    pub cells_received: AtomicU64,
    pub cover_cells_sent: AtomicU64,
    pub dht_entries: AtomicU64,
    pub dht_lookups: AtomicU64,
    pub dht_lookup_failures: AtomicU64,
    pub descriptor_publishes: AtomicU64,
    pub descriptor_fetches: AtomicU64,
    pub handshakes_initiated: AtomicU64,
    pub handshakes_completed: AtomicU64,
    pub handshakes_failed: AtomicU64,
    pub capability_verifications: AtomicU64,
    pub capability_failures: AtomicU64,
}

impl NodeMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn inc_active_circuits(&self) {
        self.active_circuits.fetch_add(1, Ordering::Relaxed);
        self.total_circuits_built.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_active_circuits(&self) {
        self.active_circuits.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn inc_circuit_failure(&self) {
        self.total_circuits_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_bytes_relayed(&self, n: u64) {
        self.bytes_relayed.fetch_add(n, Ordering::Relaxed);
    }

    pub fn inc_cells_sent(&self, is_cover: bool) {
        self.cells_sent.fetch_add(1, Ordering::Relaxed);
        if is_cover {
            self.cover_cells_sent.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn inc_cells_received(&self) {
        self.cells_received.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_dht_entries(&self, n: u64) {
        self.dht_entries.store(n, Ordering::Relaxed);
    }

    pub fn inc_dht_lookup(&self, success: bool) {
        self.dht_lookups.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.dht_lookup_failures.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn inc_handshake(&self, completed: bool) {
        self.handshakes_initiated.fetch_add(1, Ordering::Relaxed);
        if completed {
            self.handshakes_completed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.handshakes_failed.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn inc_capability_verification(&self, success: bool) {
        self.capability_verifications.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.capability_failures.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Snapshot all metrics as a plain struct for serialization.
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            active_circuits: self.active_circuits.load(Ordering::Relaxed),
            total_circuits_built: self.total_circuits_built.load(Ordering::Relaxed),
            total_circuits_failed: self.total_circuits_failed.load(Ordering::Relaxed),
            bytes_relayed: self.bytes_relayed.load(Ordering::Relaxed),
            cells_sent: self.cells_sent.load(Ordering::Relaxed),
            cells_received: self.cells_received.load(Ordering::Relaxed),
            cover_cells_sent: self.cover_cells_sent.load(Ordering::Relaxed),
            dht_entries: self.dht_entries.load(Ordering::Relaxed),
            dht_lookups: self.dht_lookups.load(Ordering::Relaxed),
            dht_lookup_failures: self.dht_lookup_failures.load(Ordering::Relaxed),
            handshakes_completed: self.handshakes_completed.load(Ordering::Relaxed),
            handshakes_failed: self.handshakes_failed.load(Ordering::Relaxed),
            capability_failures: self.capability_failures.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub active_circuits: u64,
    pub total_circuits_built: u64,
    pub total_circuits_failed: u64,
    pub bytes_relayed: u64,
    pub cells_sent: u64,
    pub cells_received: u64,
    pub cover_cells_sent: u64,
    pub dht_entries: u64,
    pub dht_lookups: u64,
    pub dht_lookup_failures: u64,
    pub handshakes_completed: u64,
    pub handshakes_failed: u64,
    pub capability_failures: u64,
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_increment_correctly() {
        let m = NodeMetrics::new();
        m.inc_active_circuits();
        m.inc_active_circuits();
        m.dec_active_circuits();
        let snap = m.snapshot();
        assert_eq!(snap.active_circuits, 1);
        assert_eq!(snap.total_circuits_built, 2);
    }

    #[test]
    fn cover_cell_tracking() {
        let m = NodeMetrics::new();
        m.inc_cells_sent(false);
        m.inc_cells_sent(true);
        m.inc_cells_sent(true);
        let snap = m.snapshot();
        assert_eq!(snap.cells_sent, 3);
        assert_eq!(snap.cover_cells_sent, 2);
    }
}
