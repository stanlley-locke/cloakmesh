use anyhow::{Context, Result};
use clap::Parser;
use tracing::info;
use std::{path::Path, sync::Arc};

use cloakmesh_core::{
    config::NodeConfig,
    crypto::Ed25519KeyPair,
    cloak_protocol::address::derive_address,
    telemetry::{init_tracing, NodeMetrics},
};

// ── Bootstrap peer address normalization ────────────────────────────────────

/// Normalize a bootstrap peer address into `host:port` form.
///
/// Accepts:
/// - Plain `host:port`              → returned as-is
/// - `https://host:port/path`       → `host:port`
/// - `http://host:port/path`        → `host:port`
/// - GitHub Codespaces-style URLs:
///   `https://ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev/`
///   The gRPC port is embedded as the last numeric segment before `.app.github.dev`.
///   Result: `ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev:4001`
/// - Generic domain without port:
///   `https://example.com/` → uses explicit HTTPS port 443 as fallback
fn normalize_peer_addr(raw: &str) -> Result<String> {
    let trimmed = raw.trim().trim_end_matches('/');

    // If it doesn't look like a URL, treat as host:port directly
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        // Validate it contains a colon (host:port)
        if trimmed.contains(':') {
            return Ok(trimmed.to_string());
        }
        anyhow::bail!("Bootstrap peer '{}' is not a valid host:port or URL", raw);
    }

    let parsed = url::Url::parse(trimmed)
        .with_context(|| format!("Cannot parse bootstrap URL: '{}'", raw))?;

    let host = parsed
        .host_str()
        .with_context(|| format!("No host in bootstrap URL: '{}'", raw))?;

    // Case 1: Explicit port in URL — use it directly
    if let Some(port) = parsed.port() {
        return Ok(format!("{}:{}", host, port));
    }

    // Case 2: GitHub Codespaces / similar: port embedded in subdomain
    // Pattern: <name>-<PORT>.app.github.dev  or  <name>-<PORT>.preview.app.github.dev
    // We extract the last dash-separated segment before the TLD zone.
    if let Some(port) = extract_port_from_subdomain(host) {
        tracing::debug!(
            "Extracted gRPC port {} from Codespaces-style host '{}'",
            port, host
        );
        return Ok(format!("{}:{}", host, port));
    }

    // Case 3: Fallback — use HTTPS default port 443 (TLS gRPC)
    let default_port: u16 = if trimmed.starts_with("https://") { 443 } else { 80 };
    tracing::warn!(
        "Could not determine gRPC port for '{}'; falling back to port {}",
        host,
        default_port
    );
    Ok(format!("{}:{}", host, default_port))
}

/// Try to extract a port number embedded in a hostname's subdomain label.
///
/// GitHub Codespaces URL format:
///   `<random-workspace-id>-<PORT>.app.github.dev`
///
/// We scan each `-`-separated segment right-to-left looking for a valid u16.
fn extract_port_from_subdomain(host: &str) -> Option<u16> {
    // Take only the first label (before the first '.')
    let first_label = host.split('.').next()?;
    // Scan dash-separated parts right to left
    for part in first_label.rsplit('-') {
        if let Ok(port) = part.parse::<u16>() {
            if port > 0 {
                return Some(port);
            }
        }
    }
    None
}

/// Parse a peer list from a JSON file.
///
/// Supported formats:
///
/// Format A — array of strings:
/// ```json
/// ["127.0.0.1:4001", "https://host-4002.app.github.dev/"]
/// ```
///
/// Format B — object with "peers" array:
/// ```json
/// {
///   "network": "cloakmesh-mainnet",
///   "peers": [
///     {"address": "127.0.0.1:4001", "id": "node-alpha", "note": "bootstrap authority"},
///     {"address": "https://host-4002.app.github.dev/"},
///     "raw-string-also-works:4001"
///   ]
/// }
/// ```
fn load_peers_file(path: &Path) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Cannot read peers file: {}", path.display()))?;

    let value: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("Cannot parse peers file as JSON: {}", path.display()))?;

    let peer_strings: Vec<String> = match &value {
        // Format A: top-level array of strings
        serde_json::Value::Array(arr) => {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        }
        // Format B: object with "peers" key
        serde_json::Value::Object(map) => {
            let peers = map.get("peers")
                .with_context(|| "Peers file object must have a 'peers' key")?;

            match peers {
                serde_json::Value::Array(arr) => {
                    arr.iter().filter_map(|v| {
                        // Each entry can be a string OR an object with "address" key
                        if let Some(s) = v.as_str() {
                            Some(s.to_string())
                        } else if let Some(addr) = v.get("address").and_then(|a| a.as_str()) {
                            Some(addr.to_string())
                        } else {
                            tracing::warn!("Skipping unrecognized peer entry in peers file: {}", v);
                            None
                        }
                    }).collect()
                }
                _ => anyhow::bail!("'peers' key must be an array"),
            }
        }
        _ => anyhow::bail!("Peers file must be a JSON array or object"),
    };

    Ok(peer_strings)
}

// ── CLI definition ───────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "cloakmesh",
    version,
    about = "CloakMesh privacy network node",
    long_about = "Start a CloakMesh node. Bootstrap peer addresses can be specified as:\n  \
                  - plain host:port (e.g. 127.0.0.1:4001)\n  \
                  - HTTPS URL with explicit port (e.g. https://example.com:4001/)\n  \
                  - GitHub Codespaces URL (e.g. https://name-4001.app.github.dev/)\n  \
                  - JSON peer list file via --peers-file"
)]
struct Cli {
    #[arg(long, default_value = "configs/default.toml", help = "Path to TOML config file")]
    config: String,

    #[arg(long, help = "Override listen port")]
    port: Option<u16>,

    #[arg(long, help = "Override node ID")]
    id: Option<String>,

    /// Bootstrap peer address. May be specified multiple times.
    /// Accepts: host:port, https://host:port/, GitHub Codespaces URLs.
    ///
    /// Examples:
    ///   --bootstrap 127.0.0.1:4001
    ///   --bootstrap https://ideal-orbit-abc123-4001.app.github.dev/
    ///   --bootstrap https://myserver.com:4001/
    #[arg(long, num_args = 0..)]
    bootstrap: Vec<String>,

    /// Path to a JSON file containing a list of bootstrap peers.
    ///
    /// Supports two formats:
    ///   Array: ["127.0.0.1:4001", "https://host-4002.app.github.dev/"]
    ///   Object: {"network": "...", "peers": [{"address": "..."}, "..."]}
    #[arg(long, help = "Path to JSON peers file (see --help for format)")]
    peers_file: Option<String>,

    /// Public address that other nodes should use to reach this node.
    /// If not set, uses 127.0.0.1:<port> (only reachable locally).
    ///
    /// Examples:
    ///   --public-addr 203.0.113.10:4001
    ///   --public-addr https://ideal-orbit-abc123-4001.app.github.dev/
    ///   --public-addr mynode.example.com:4001
    #[arg(long, help = "Public address this node announces to peers (host:port or URL)")]
    public_addr: Option<String>,

    #[arg(long, default_value_t = true, help = "Output logs as JSON")]
    json_logs: bool,

    #[arg(long, default_value_t = false, help = "Enable persistent storage and mine ATK tokens")]
    mine_atk: bool,
}

// ── Entry point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration (falls back to defaults if file not found)
    let mut config = NodeConfig::load(&cli.config).unwrap_or_else(|e| {
        eprintln!("Config load warning: {e} — using defaults");
        NodeConfig::default()
    });

    // Apply CLI overrides
    if let Some(port) = cli.port {
        config.listen_port = port;
    }
    if let Some(id) = cli.id {
        config.node_id = id;
    }

    // ── Collect all bootstrap peers ──────────────────────────────────────────
    let mut raw_peers: Vec<String> = Vec::new();

    // 1. From --bootstrap flags
    raw_peers.extend(cli.bootstrap.into_iter().filter(|p| !p.is_empty()));

    // 2. From --peers-file
    if let Some(peers_path) = &cli.peers_file {
        match load_peers_file(Path::new(peers_path)) {
            Ok(file_peers) => {
                eprintln!(
                    "Loaded {} peers from '{}'",
                    file_peers.len(),
                    peers_path
                );
                raw_peers.extend(file_peers);
            }
            Err(e) => {
                eprintln!("Warning: Could not load peers file '{}': {}", peers_path, e);
            }
        }
    }

    // 3. Normalize all addresses (URL → host:port)
    for raw in raw_peers {
        match normalize_peer_addr(&raw) {
            Ok(normalized) => {
                if normalized != raw {
                    eprintln!("Bootstrap '{}' → '{}'", raw, normalized);
                }
                if !config.bootstrap_peers.contains(&normalized) {
                    config.bootstrap_peers.push(normalized);
                }
            }
            Err(e) => {
                eprintln!("Warning: Skipping invalid bootstrap peer '{}': {}", raw, e);
            }
        }
    }

    // ── Public address announcement ──────────────────────────────────────────
    let public_addr: Option<String> = if let Some(raw) = cli.public_addr {
        match normalize_peer_addr(&raw) {
            Ok(addr) => {
                if addr != raw {
                    eprintln!("Public addr '{}' → '{}'", raw, addr);
                }
                Some(addr)
            }
            Err(e) => {
                eprintln!("Warning: Invalid --public-addr '{}': {}", raw, e);
                None
            }
        }
    } else {
        // Default: announce the local gRPC address
        Some(format!("127.0.0.1:{}", config.listen_port))
    };

    // Initialize structured logging
    init_tracing(config.log_level, cli.json_logs);

    info!(
        "[200] CloakMesh node starting, node_id: {}, port: {}, bootstrap_peers: {}, public_addr: {}",
        config.node_id,
        config.listen_port,
        config.bootstrap_peers.len(),
        public_addr.as_deref().unwrap_or("none"),
    );

    if !config.bootstrap_peers.is_empty() {
        info!("[200] Bootstrap peers: {:?}", config.bootstrap_peers);
    }

    // Isolate data directories for local testing of multiple nodes
    if config.data_dir == std::path::Path::new("data") {
        config.data_dir = format!("data_{}", config.node_id).into();
    }
    if config.crypto.identity_key_path == std::path::Path::new("data/identity.pem") {
        config.crypto.identity_key_path =
            format!("{}/identity.pem", config.data_dir.display()).into();
    }

    // Ensure data directory exists
    std::fs::create_dir_all(&config.data_dir)?;

    // Load or generate Ed25519 identity key
    let identity = Ed25519KeyPair::load_or_generate(&config.crypto.identity_key_path)?;
    let pubkey = identity.public_key_bytes();
    let cloak_address = derive_address(&pubkey);

    info!(
        "[200] Identity loaded, cloak_address: {}, pubkey: {}, public_addr: {}",
        cloak_address,
        hex::encode(pubkey),
        public_addr.as_deref().unwrap_or("127.0.0.1:?"),
    );

    // Initialize metrics
    let metrics = NodeMetrics::new();

    // Validate final config
    config.validate()?;

    info!(
        "[200] Node configuration validated, cell_size: {}, padding_enabled: {}, default_hops: {}, pq_hybrid: {}",
        config.traffic.cell_size_bytes,
        config.traffic.padding_enabled,
        config.circuit.default_hops,
        config.crypto.pq_hybrid_enabled,
    );

    // Initialize Node state (pass public_addr so descriptors announce the right address)
    let node = Arc::new(cloakmesh_core::node::CloakNode::new(
        pubkey,
        config.node_id.clone(),
        cli.mine_atk,
        config.bootstrap_peers.clone(),
        config.data_dir.to_string_lossy().into_owned(),
        config.listen_port,
    ));

    let addr = format!("0.0.0.0:{}", config.listen_port).parse()?;
    info!("[200] gRPC server starting, addr: {}", addr);

    let server = tonic::transport::Server::builder()
        .add_service(cloakmesh_core::proto::v1::cloak_mesh_node_server::CloakMeshNodeServer::from_arc(node.clone()))
        .add_service(cloakmesh_core::proto::v1::cloak_service_server::CloakServiceServer::from_arc(node.clone()))
        .add_service(cloakmesh_core::proto::v1::capability_service_server::CapabilityServiceServer::from_arc(node.clone()))
        .add_service(cloakmesh_core::proto::v1::telemetry_service_server::TelemetryServiceServer::from_arc(node.clone()))
        .serve(addr);

    info!("[200] Node fully initialized and listening");

    // Start SOCKS5 proxy
    let node_clone = node.clone();
    let proxy_port = config.listen_port + 5049; // 4001 → 9050, 4002 → 9051
    tokio::spawn(async move {
        if let Err(e) = node_clone.start_proxy(proxy_port).await {
            eprintln!("Failed to start client proxy: {}", e);
        }
    });

    // Run server and handle graceful shutdown
    tokio::select! {
        res = server => {
            if let Err(e) = res {
                eprintln!("Server error: {e}");
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received SIGINT, shutting down gracefully");
        }
    }

    info!(
        cells_sent = metrics.cells_sent.load(std::sync::atomic::Ordering::Relaxed),
        "Node shutdown complete"
    );

    Ok(())
}
