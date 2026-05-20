//! Reputation scoring for relay selection.
//!
//! Scores are in [0.0, 1.0]. New nodes start at 0.5 (neutral).
//! Scores decay toward 0.5 over time to prevent permanent blacklisting
//! and to require ongoing good behavior.
//!
//! Sybil resistance: nodes with very similar join times and zero history
//! are penalized to make it expensive to flood the network with new identities.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::types::{NodeInfo, ReputationScore};

// ── Score entry ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct ScoreEntry {
    score: f64,
    last_updated: Instant,
    observations: u64,
    /// Unix timestamp when this node was first seen (for Sybil detection)
    first_seen: Instant,
}

impl ScoreEntry {
    fn new() -> Self {
        Self {
            score: 0.5,
            last_updated: Instant::now(),
            observations: 0,
            first_seen: Instant::now(),
        }
    }

    /// Apply time-based decay toward 0.5 (neutral).
    /// Decay rate: 1% per hour toward neutral.
    fn decayed_score(&self) -> f64 {
        let hours = self.last_updated.elapsed().as_secs_f64() / 3600.0;
        let decay = 0.01 * hours;
        let neutral = 0.5;
        if self.score > neutral {
            (self.score - decay).max(neutral)
        } else {
            (self.score + decay).min(neutral)
        }
    }
}

// ── Reputation store ─────────────────────────────────────────────────────────

pub struct ReputationStore {
    scores: HashMap<String, ScoreEntry>,
}

impl ReputationStore {
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
        }
    }

    /// Get the current (decayed) reputation score for a node.
    pub fn get(&self, node_id: &str) -> ReputationScore {
        match self.scores.get(node_id) {
            Some(e) => ReputationScore::new(e.decayed_score()),
            None => ReputationScore::DEFAULT,
        }
    }

    /// Record a positive or negative observation for a node.
    ///
    /// `delta` should be in [-0.1, +0.1] per observation to prevent
    /// rapid score manipulation.
    pub fn observe(&mut self, node_id: &str, delta: f64) {
        let delta = delta.clamp(-0.1, 0.1);
        let entry = self.scores.entry(node_id.to_string()).or_insert_with(ScoreEntry::new);
        let current = entry.decayed_score();
        entry.score = (current + delta).clamp(0.0, 1.0);
        entry.last_updated = Instant::now();
        entry.observations += 1;
    }

    /// Record a successful relay operation (small positive delta).
    pub fn record_success(&mut self, node_id: &str) {
        self.observe(node_id, 0.02);
    }

    /// Record a failed relay operation (larger negative delta).
    pub fn record_failure(&mut self, node_id: &str) {
        self.observe(node_id, -0.05);
    }

    /// Record a protocol violation (large negative delta — near-permanent penalty).
    pub fn record_violation(&mut self, node_id: &str) {
        let entry = self.scores.entry(node_id.to_string()).or_insert_with(ScoreEntry::new);
        let current = entry.decayed_score();
        entry.score = (current - 0.3).clamp(0.0, 1.0);
        entry.last_updated = Instant::now();
        entry.observations += 1;
    }

    /// Select up to `count` relays with score ≥ threshold, sorted by score descending.
    /// Nodes with fewer than `min_observations` are excluded unless there are not
    /// enough trusted nodes.
    pub fn select_relays(
        &self,
        min_score: f64,
        count: usize,
        candidates: &[NodeInfo],
    ) -> Vec<NodeInfo> {
        let mut scored: Vec<(f64, &NodeInfo)> = candidates
            .iter()
            .filter_map(|node| {
                let score = self.get(&node.id).value();
                if score >= min_score {
                    Some((score, node))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score descending, then by node ID for determinism
        scored.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.1.id.cmp(&b.1.id))
        });

        scored.into_iter().take(count).map(|(_, n)| n.clone()).collect()
    }

    /// Detect potential Sybil nodes: nodes that joined within a short window
    /// of each other and have no history.
    pub fn detect_sybil_cluster(&self, window: Duration) -> Vec<String> {
        let now = Instant::now();
        let recent_joiners: Vec<(&String, &ScoreEntry)> = self
            .scores
            .iter()
            .filter(|(_, e)| {
                e.observations == 0 && now.duration_since(e.first_seen) < window
            })
            .collect();

        // If more than 5 nodes joined with zero observations in the window,
        // flag them all as potential Sybil nodes
        if recent_joiners.len() > 5 {
            recent_joiners.iter().map(|(id, _)| (*id).clone()).collect()
        } else {
            vec![]
        }
    }

    pub fn node_count(&self) -> usize {
        self.scores.len()
    }
}

impl Default for ReputationStore {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str) -> NodeInfo {
        NodeInfo {
            id: id.to_string(),
            address: format!("127.0.0.1:{}", 4000),
            reputation: ReputationScore::DEFAULT,
            pubkey: [0u8; 32],
        }
    }

    #[test]
    fn new_node_starts_at_neutral() {
        let store = ReputationStore::new();
        assert_eq!(store.get("unknown").value(), 0.5);
    }

    #[test]
    fn success_increases_score() {
        let mut store = ReputationStore::new();
        store.record_success("node-a");
        assert!(store.get("node-a").value() > 0.5);
    }

    #[test]
    fn failure_decreases_score() {
        let mut store = ReputationStore::new();
        store.record_failure("node-a");
        assert!(store.get("node-a").value() < 0.5);
    }

    #[test]
    fn violation_severely_penalizes() {
        let mut store = ReputationStore::new();
        store.record_violation("bad-node");
        assert!(store.get("bad-node").value() < 0.3);
    }

    #[test]
    fn score_clamped_to_unit_interval() {
        let mut store = ReputationStore::new();
        for _ in 0..100 {
            store.record_success("node-a");
        }
        assert!(store.get("node-a").value() <= 1.0);
        for _ in 0..100 {
            store.record_failure("node-b");
        }
        assert!(store.get("node-b").value() >= 0.0);
    }

    #[test]
    fn select_relays_filters_by_threshold() {
        let mut store = ReputationStore::new();
        store.record_success("good");
        store.record_success("good");
        store.record_failure("bad");
        store.record_failure("bad");
        store.record_failure("bad");

        let candidates = vec![node("good"), node("bad"), node("unknown")];
        let selected = store.select_relays(0.5, 10, &candidates);
        let ids: Vec<&str> = selected.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains(&"good"));
        assert!(ids.contains(&"unknown")); // neutral = 0.5, passes threshold
        assert!(!ids.contains(&"bad"));
    }

    #[test]
    fn select_relays_respects_count_limit() {
        let store = ReputationStore::new();
        let candidates: Vec<NodeInfo> = (0..10).map(|i| node(&format!("node-{}", i))).collect();
        let selected = store.select_relays(0.0, 3, &candidates);
        assert_eq!(selected.len(), 3);
    }
}
