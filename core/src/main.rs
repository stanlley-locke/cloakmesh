use anyhow::Result;
use clap::Parser;
use tracing::info;
use std::sync::Arc;

use cloakmesh_core::{
    config::NodeConfig,
    crypto::Ed25519KeyPair,
    cloak_protocol::address::derive_address,
    telemetry::{init_tracing, NodeMetrics},
};

// ── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "cloakmesh", version, about = "CloakMesh privacy network node")]
struct Cli {
    #[arg(long, default_value = "configs/default.toml", help = "Path to TOML config file")]
    config: String,

    #[arg(long, help = "Override listen port")]
    port: Option<u16>,

    #[arg(long, help = "Override node ID")]
    id: Option<String>,

    #[arg(long, help = "Bootstrap peer address (host:port); may be specified multiple times", num_args = 0..)]
    bootstrap: Vec<String>,

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
    // Extend bootstrap peers from CLI args (all --bootstrap flags)
    for peer in cli.bootstrap {
        if !peer.is_empty() {
            config.bootstrap_peers.push(peer);
        }
    }

    // Initialize structured logging
    init_tracing(config.log_level, cli.json_logs);

    info!("[200] CloakMesh node starting, node_id: {}, port: {}, bootstrap_peers: {}", config.node_id, config.listen_port, config.bootstrap_peers.len());

    // Isolate data directories for local testing of multiple nodes
    if config.data_dir == std::path::Path::new("data") {
        config.data_dir = format!("data_{}", config.node_id).into();
    }
    if config.crypto.identity_key_path == std::path::Path::new("data/identity.pem") {
        config.crypto.identity_key_path = format!("{}/identity.pem", config.data_dir.display()).into();
    }

    // Ensure data directory exists
    std::fs::create_dir_all(&config.data_dir)?;

    // Load or generate Ed25519 identity key
    let identity = Ed25519KeyPair::load_or_generate(&config.crypto.identity_key_path)?;
    let pubkey = identity.public_key_bytes();
    let cloak_address = derive_address(&pubkey);

    info!("[200] Identity loaded, cloak_address: {}, pubkey: {}", cloak_address, hex::encode(pubkey));

    // Initialize metrics
    let metrics = NodeMetrics::new();

    // Validate final config
    config.validate()?;

    info!("[200] Node configuration validated, cell_size: {}, padding_enabled: {}, default_hops: {}, pq_hybrid: {}", config.traffic.cell_size_bytes, config.traffic.padding_enabled, config.circuit.default_hops, config.crypto.pq_hybrid_enabled);

    // Initialize Node state
    let node = Arc::new(cloakmesh_core::node::CloakNode::new(pubkey, config.node_id.clone(), cli.mine_atk, config.bootstrap_peers.clone(), config.data_dir.to_string_lossy().into_owned(), config.listen_port));

    let addr = format!("0.0.0.0:{}", config.listen_port).parse()?;
    info!("[200] gRPC server starting, addr: {}", addr);

    let server = tonic::transport::Server::builder()
        .add_service(cloakmesh_core::proto::v1::cloak_mesh_node_server::CloakMeshNodeServer::from_arc(node.clone()))
        .add_service(cloakmesh_core::proto::v1::cloak_service_server::CloakServiceServer::from_arc(node.clone()))
        .add_service(cloakmesh_core::proto::v1::capability_service_server::CapabilityServiceServer::from_arc(node.clone()))
        .add_service(cloakmesh_core::proto::v1::telemetry_service_server::TelemetryServiceServer::from_arc(node.clone()))
        .serve(addr);

    // ── Phase 2 will add: ────────────────────────────────────────────────────
    // - QUIC transport initialization
    // - DHT bootstrap and peer discovery
    // - Circuit pool warm-up
    // - Descriptor publication
    // ────────────────────────────────────────────────────────────────────────

    info!("[200] Node fully initialized and listening");

    // Start Phase 2 Client Proxy (Demo Mode)
    let node_clone = node.clone();
    let proxy_port = config.listen_port + 5049; // 4001 -> 9050, 4002 -> 9051
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
