use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::errors::{CloakError, CloakResult};

// ── Top-level node configuration ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_id: String,
    pub listen_port: u16,
    pub bootstrap_peers: Vec<String>,
    pub data_dir: PathBuf,
    pub log_level: LogLevel,
    pub circuit: CircuitConfig,
    pub traffic: TrafficConfig,
    pub crypto: CryptoConfig,
    pub dht: DhtConfig,
    pub telemetry: TelemetryConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogLevel::Error => "error",
            LogLevel::Warn  => "warn",
            LogLevel::Info  => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        };
        write!(f, "{s}")
    }
}

// ── Circuit configuration ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitConfig {
    /// Minimum hops (security floor). Must be ≥ 2.
    pub min_hops: u8,
    /// Maximum hops (latency ceiling). Must be ≤ 5.
    pub max_hops: u8,
    /// Default hops when no threat-model override is active.
    pub default_hops: u8,
    /// Milliseconds before a circuit build attempt times out.
    pub build_timeout_ms: u64,
    /// How many circuits to keep warm in the pool.
    pub pool_size: usize,
}

impl CircuitConfig {
    pub fn validate(&self) -> CloakResult<()> {
        if self.min_hops < 2 {
            return Err(CloakError::ConfigInvalidValue {
                field: "circuit.min_hops".into(),
                reason: "must be ≥ 2 for anonymity guarantees".into(),
            });
        }
        if self.max_hops > 5 {
            return Err(CloakError::ConfigInvalidValue {
                field: "circuit.max_hops".into(),
                reason: "must be ≤ 5".into(),
            });
        }
        if self.default_hops < self.min_hops || self.default_hops > self.max_hops {
            return Err(CloakError::ConfigInvalidValue {
                field: "circuit.default_hops".into(),
                reason: format!(
                    "must be between min_hops ({}) and max_hops ({})",
                    self.min_hops, self.max_hops
                ),
            });
        }
        Ok(())
    }
}

// ── Traffic / padding configuration ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficConfig {
    /// Fixed cell size in bytes. Must be 256 or 1024.
    pub cell_size_bytes: u32,
    /// Cover traffic packets per second during idle periods.
    pub cover_flow_pps: u32,
    /// Maximum random jitter added to each cell send (milliseconds).
    pub jitter_max_ms: u32,
    /// Whether padding and cover traffic are active.
    pub padding_enabled: bool,
    /// Session key rotation interval in cells.
    pub key_rotation_cells: u64,
    /// Session key rotation interval in seconds (whichever comes first).
    pub key_rotation_secs: u64,
}

impl TrafficConfig {
    pub fn validate(&self) -> CloakResult<()> {
        if self.cell_size_bytes != 256 && self.cell_size_bytes != 1024 {
            return Err(CloakError::ConfigInvalidValue {
                field: "traffic.cell_size_bytes".into(),
                reason: "must be 256 or 1024".into(),
            });
        }
        if self.key_rotation_cells == 0 {
            return Err(CloakError::ConfigInvalidValue {
                field: "traffic.key_rotation_cells".into(),
                reason: "must be > 0".into(),
            });
        }
        Ok(())
    }
}

// ── Cryptographic configuration ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    /// Path to the node's Ed25519 identity key file (PEM or raw bytes).
    /// If absent, a new key is generated and saved here on first start.
    pub identity_key_path: PathBuf,
    /// Maximum age of a descriptor nonce before it is considered a replay (seconds).
    pub nonce_window_secs: u64,
    /// Maximum descriptor age before it is considered stale (seconds).
    pub descriptor_max_age_secs: u64,
    /// Whether to enable the hybrid PQ (Kyber-768) key exchange.
    pub pq_hybrid_enabled: bool,
}

// ── DHT configuration ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtConfig {
    /// Kademlia k-bucket size.
    pub k_bucket_size: usize,
    /// Number of parallel lookups (alpha parameter).
    pub alpha: usize,
    /// Descriptor TTL in the DHT (seconds).
    pub descriptor_ttl_secs: u64,
    /// How often to republish our own descriptors (seconds).
    pub republish_interval_secs: u64,
}

// ── Telemetry configuration ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub prometheus_enabled: bool,
    pub prometheus_port: u16,
    pub metrics_interval_secs: u64,
}

// ── Defaults & construction ──────────────────────────────────────────────────

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            node_id: "node-default".into(),
            listen_port: 4001,
            bootstrap_peers: vec![],
            data_dir: PathBuf::from("./data"),
            log_level: LogLevel::Info,
            circuit: CircuitConfig {
                min_hops: 2,
                max_hops: 5,
                default_hops: 3,
                build_timeout_ms: 10_000,
                pool_size: 3,
            },
            traffic: TrafficConfig {
                cell_size_bytes: 256,
                cover_flow_pps: 1,
                jitter_max_ms: 50,
                padding_enabled: true,
                key_rotation_cells: 10_000,
                key_rotation_secs: 300,
            },
            crypto: CryptoConfig {
                identity_key_path: PathBuf::from("./data/identity.key"),
                nonce_window_secs: 300,
                descriptor_max_age_secs: 3600,
                pq_hybrid_enabled: true,
            },
            dht: DhtConfig {
                k_bucket_size: 20,
                alpha: 3,
                descriptor_ttl_secs: 3600,
                republish_interval_secs: 1800,
            },
            telemetry: TelemetryConfig {
                prometheus_enabled: true,
                prometheus_port: 9090,
                metrics_interval_secs: 15,
            },
        }
    }
}

impl NodeConfig {
    /// Load from a TOML file, then apply environment variable overrides.
    pub fn load(path: &str) -> CloakResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| CloakError::ConfigParse(format!("cannot read {path}: {e}")))?;
        let mut cfg: NodeConfig = toml::from_str(&content)
            .map_err(|e| CloakError::ConfigParse(e.to_string()))?;
        cfg.apply_env_overrides();
        cfg.validate()?;
        Ok(cfg)
    }

    /// Apply `CLOAK_*` environment variable overrides.
    fn apply_env_overrides(&mut self) {
        if let Ok(id) = std::env::var("CLOAK_NODE_ID") {
            self.node_id = id;
        }
        if let Ok(port) = std::env::var("CLOAK_PORT") {
            if let Ok(p) = port.parse() {
                self.listen_port = p;
            }
        }
        if let Ok(bootstrap) = std::env::var("CLOAK_BOOTSTRAP") {
            if !bootstrap.is_empty() {
                self.bootstrap_peers = bootstrap.split(',').map(str::to_string).collect();
            }
        }
        if let Ok(level) = std::env::var("CLOAK_LOG_LEVEL") {
            self.log_level = match level.to_lowercase().as_str() {
                "error" => LogLevel::Error,
                "warn"  => LogLevel::Warn,
                "debug" => LogLevel::Debug,
                "trace" => LogLevel::Trace,
                _       => LogLevel::Info,
            };
        }
    }

    /// Validate all sub-configurations.
    pub fn validate(&self) -> CloakResult<()> {
        if self.node_id.is_empty() {
            return Err(CloakError::ConfigInvalidValue {
                field: "node_id".into(),
                reason: "must not be empty".into(),
            });
        }
        if self.listen_port < 1024 {
            return Err(CloakError::ConfigInvalidValue {
                field: "listen_port".into(),
                reason: "must be ≥ 1024 (non-privileged)".into(),
            });
        }
        self.circuit.validate()?;
        self.traffic.validate()?;
        Ok(())
    }
}
