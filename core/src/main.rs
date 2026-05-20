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

    #[arg(long, help = "Bootstrap peer address (host:port)")]
    bootstrap: Option<String>,

    #[arg(long, default_value = "false", help = "Output logs as JSON")]
    json_logs: bool,
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
    if let Some(bootstrap) = cli.bootstrap {
        if !bootstrap.is_empty() {
            config.bootstrap_peers.push(bootstrap);
        }
    }

    // Initialize structured logging
    init_tracing(config.log_level, cli.json_logs);

    info!(
        node_id = %config.node_id,
        port = config.listen_port,
        bootstrap_peers = config.bootstrap_peers.len(),
        "CloakMesh node starting"
    );

    // Ensure data directory exists
    std::fs::create_dir_all(&config.data_dir)?;

    // Load or generate Ed25519 identity key
    let identity = Ed25519KeyPair::load_or_generate(&config.crypto.identity_key_path)?;
    let pubkey = identity.public_key_bytes();
    let cloak_address = derive_address(&pubkey);

    info!(
        cloak_address = %cloak_address,
        pubkey = %hex::encode(pubkey),
        "Identity loaded"
    );

    // Initialize metrics
    let metrics = NodeMetrics::new();

    // Validate final config
    config.validate()?;

    info!(
        cell_size = config.traffic.cell_size_bytes,
        padding_enabled = config.traffic.padding_enabled,
        default_hops = config.circuit.default_hops,
        pq_hybrid = config.crypto.pq_hybrid_enabled,
        "Node configuration validated"
    );

    // Initialize Node state
    let node = Arc::new(cloakmesh_core::node::CloakNode::new(pubkey));

    let addr = format!("0.0.0.0:{}", config.listen_port).parse()?;
    info!(addr = %addr, "gRPC server starting");

    let server = tonic::transport::Server::builder()
        .add_service(cloakmesh_core::proto::v1::cloak_mesh_node_server::CloakMeshNodeServer::from_arc(node.clone()))
        .add_service(cloakmesh_core::proto::v1::cloak_service_server::CloakServiceServer::from_arc(node.clone()))
        .add_service(cloakmesh_core::proto::v1::capability_service_server::CapabilityServiceServer::from_arc(node.clone()))
        .serve(addr);

    // ── Phase 2 will add: ────────────────────────────────────────────────────
    // - QUIC transport initialization
    // - DHT bootstrap and peer discovery
    // - Circuit pool warm-up
    // - Descriptor publication
    // ────────────────────────────────────────────────────────────────────────

    info!("Node fully initialized and listening");

    // Start Phase 2 Client Proxy (Demo Mode)
    let node_clone = node.clone();
    tokio::spawn(async move {
        if let Err(e) = node_clone.start_proxy(9050).await {
            eprintln!("Failed to start client proxy: {e}");
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
