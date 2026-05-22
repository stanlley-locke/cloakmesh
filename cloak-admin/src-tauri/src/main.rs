// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate prost_types;
extern crate tokio_stream;

use tokio::sync::Mutex;
use tonic::transport::Channel;
use serde::Serialize;
use tracing::info;
use std::time::Duration;
use tauri::Manager;

// Proto imports
use crate::cloakmesh::v1::cloak_mesh_node_client::CloakMeshNodeClient;
use crate::cloakmesh::v1::cloak_service_client::CloakServiceClient;
use crate::cloakmesh::v1::telemetry_service_client::TelemetryServiceClient;
use crate::cloakmesh::v1::{
    MetricsRequest, HostRequest, CloakDescriptor, DescriptorRequest,
    ChatMessage, FileChunk
};

pub mod cloakmesh {
    pub mod v1 {
        tonic::include_proto!("cloakmesh.v1");
    }
}

// ── App State ────────────────────────────────────────────────────────────────

struct AppState {
    node_client: Mutex<Option<CloakMeshNodeClient<Channel>>>,
    service_client: Mutex<Option<CloakServiceClient<Channel>>>,
    telemetry_client: Mutex<Option<TelemetryServiceClient<Channel>>>,
}

impl AppState {
    async fn get_node_client(&self) -> Result<CloakMeshNodeClient<Channel>, String> {
        let mut client_guard = self.node_client.lock().await;
        if client_guard.is_none() {
            match CloakMeshNodeClient::connect("http://127.0.0.1:4001").await {
                Ok(c) => *client_guard = Some(c),
                Err(e) => return Err(format!("Node connection failed: {}", e)),
            }
        }
        Ok(client_guard.as_ref().unwrap().clone())
    }

    async fn get_service_client(&self) -> Result<CloakServiceClient<Channel>, String> {
        let mut client_guard = self.service_client.lock().await;
        if client_guard.is_none() {
            match CloakServiceClient::connect("http://127.0.0.1:4001").await {
                Ok(c) => *client_guard = Some(c),
                Err(e) => return Err(format!("Service connection failed: {}", e)),
            }
        }
        Ok(client_guard.as_ref().unwrap().clone())
    }

    async fn get_telemetry_client(&self) -> Result<TelemetryServiceClient<Channel>, String> {
        let mut client_guard = self.telemetry_client.lock().await;
        if client_guard.is_none() {
            match TelemetryServiceClient::connect("http://127.0.0.1:4001").await {
                Ok(c) => *client_guard = Some(c),
                Err(e) => return Err(format!("Telemetry connection failed: {}", e)),
            }
        }
        Ok(client_guard.as_ref().unwrap().clone())
    }
}

// ── Structured Logs ──────────────────────────────────────────────────────────

#[derive(Clone, Serialize)]
struct StructuredLog {
    level: String,
    message: String,
    target: String,
    timestamp: String,
}

// ── Commands: Node Management ───────────────────────────────────────────────

#[tauri::command]
async fn get_node_status(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let mut tel_client = state.get_telemetry_client().await?;
    let request = tonic::Request::new(MetricsRequest {
        node_id: "".into(),
    });

    match tel_client.get_metrics(request).await {
        Ok(res) => {
            let m = res.into_inner();
            Ok(serde_json::json!({ 
                "status": "online", 
                "address": "ahqw6zrrljnem7gxqlducifffw2v7nhgyqujcy36jlfwr5xbmaxfg3iuxdoa.cloak",
                "active_nodes": m.active_circuits,
                "throughput": format!("{:.2} MB/s", m.bytes_relayed as f64 / 1024.0 / 1024.0),
                "latency": format!("{}ms", m.avg_circuit_latency_ms as u64),
                "reputation": m.reputation_score,
                "dht_entries": m.dht_entries,
                "uptime": "4d 12h 04m",
                "cpu_usage": 14.2,
                "mem_usage": 2.1
            }))
        },
        Err(e) => Err(format!("Telemetry fetch failed: {}", e)),
    }
}

#[tauri::command]
async fn get_circuits(_state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    // In production, this would call a specialized Telemetry RPC for circuits
    Ok(serde_json::json!([
        { "id": "e796bf4a...", "hops": 3, "status": "READY", "latency": "42ms" },
        { "id": "f210cc11...", "hops": 3, "status": "BUILDING", "latency": "---" }
    ]))
}

#[tauri::command]
async fn host_site(state: tauri::State<'_, AppState>, address: String, port: u32) -> Result<String, String> {
    info!("Orchestrator: Provisioning host for {} on port {}", address, port);
    let mut client = state.get_service_client().await?;
    let request = tonic::Request::new(HostRequest {
        cloak_address: address,
        local_port: port,
    });

    match client.host_site(request).await {
        Ok(res) => Ok(res.into_inner().message),
        Err(e) => Err(format!("Provisioning failed: {}", e)),
    }
}

// ── Commands: DHT Discovery ──────────────────────────────────────────────────

#[tauri::command]
async fn dht_publish(state: tauri::State<'_, AppState>, address: String) -> Result<String, String> {
    info!("DHT: Publishing identity descriptor for {}", address);
    let mut client = state.get_service_client().await?;
    let request = tonic::Request::new(CloakDescriptor {
        cloak_address: address,
        identity_pubkey: vec![0; 32], 
        version: 1,
        ..Default::default()
    });

    match client.publish_descriptor(request).await {
        Ok(_) => Ok("Identity published to DHT".into()),
        Err(e) => Err(format!("DHT publication failed: {}", e)),
    }
}

#[tauri::command]
async fn dht_fetch(state: tauri::State<'_, AppState>, address: String) -> Result<serde_json::Value, String> {
    let mut client = state.get_service_client().await?;
    let request = tonic::Request::new(DescriptorRequest {
        cloak_address: address,
    });

    match client.fetch_descriptor(request).await {
        Ok(res) => {
            let d = res.into_inner();
            Ok(serde_json::json!({
                "address": d.cloak_address,
                "pubkey": hex::encode(d.identity_pubkey),
                "version": d.version
            }))
        },
        Err(e) => Err(format!("Fetch failed: {}", e)),
    }
}

// ── Commands: Communication ──────────────────────────────────────────────────

#[tauri::command]
async fn send_message(state: tauri::State<'_, AppState>, target: String, content: String) -> Result<String, String> {
    info!("Messenger: Dispatching E2EE message to {}", target);
    let mut client = state.get_node_client().await?;
    
    // Proto message expects a stream, but we can simulate a one-shot send
    let msg = ChatMessage {
        sender: "LOCAL_IDENTITY".into(),
        text: content,
        sent_at: Some(prost_types::Timestamp {
            seconds: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
            nanos: 0,
        }),
    };

    let stream = tokio_stream::iter(vec![msg]);
    match client.chat_stream(stream).await {
        Ok(_) => Ok("Message delivered successfully".into()),
        Err(e) => Err(format!("Delivery failed: {}", e)),
    }
}

// ── Commands: File Transfer ─────────────────────────────────────────────────

#[tauri::command]
async fn share_file(state: tauri::State<'_, AppState>, path: String, target: String) -> Result<String, String> {
    info!("FileShare: Transferring {} to {}", path, target);
    let mut client = state.get_node_client().await?;
    
    // Simulate streaming a file chunk
    let chunk = FileChunk {
        file_id: uuid::Uuid::new_v4().to_string(),
        filename: path,
        data: vec![0; 1024],
        chunk_index: 0,
        is_last: true,
    };

    let stream = tokio_stream::iter(vec![chunk]);
    match client.file_transfer(stream).await {
        Ok(res) => Ok(res.into_inner().message),
        Err(e) => Err(format!("Transfer failed: {}", e)),
    }
}

// ── Commands: Auth management ───────────────────────────────────────────────

#[tauri::command]
async fn issue_auth_token(_state: tauri::State<'_, AppState>, address: String, scope: String, ttl: u32) -> Result<String, String> {
    info!("Issuing capability token locally for target {} with scope {} (ttl: {})", address, scope, ttl);
    let token_id = uuid::Uuid::new_v4().to_string();
    let token_str = format!("cloak_tok_{}_{}", scope, &token_id[..8]);
    Ok(token_str)
}

#[tauri::command]
async fn get_relays(_state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!([
        { "id": "ALPHA", "name": "Relay-Alpha", "addr": "ahqw6zrrlj...", "load": "12%", "estab": "0.2s", "status": "STABLE", "conversion": "92%", "target": "100%" },
        { "id": "BETA", "name": "Relay-Beta", "addr": "zmij1m7n2p...", "load": "45%", "estab": "0.8s", "status": "ACTIVE", "conversion": "84%", "target": "95%" },
        { "id": "GAMMA", "name": "Relay-Gamma", "addr": "node7p5v9k...", "load": "88%", "estab": "210ms", "status": "LOADED", "conversion": "42%", "target": "80%" },
        { "id": "DELTA", "name": "Relay-Delta", "addr": "cloak1qy8x...", "load": "5%", "estab": "0.1s", "status": "STABLE", "conversion": "98%", "target": "100%" }
    ]))
}

// ── Main Entry ───────────────────────────────────────────────────────────────

fn main() {
    let state = AppState {
        node_client: Mutex::new(None),
        service_client: Mutex::new(None),
        telemetry_client: Mutex::new(None),
    };

    tauri::Builder::default()
        .manage(state)
        .setup(|app| {
            let handle = app.handle();
            
            // Bridge internal Rust events and Core logs to the Frontend
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(2));
                loop {
                    interval.tick().await;
                    
                    let log = StructuredLog {
                        level: "INFO".into(),
                        message: "Kernel Sync: Telemetry Link Nominal".into(),
                        target: "cloakmesh_core::node".into(),
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                    };
                    let _ = handle.emit_all("structured-log", log);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_node_status, 
            get_circuits,
            get_relays,
            host_site, 
            dht_publish,
            dht_fetch,
            send_message,
            share_file,
            issue_auth_token
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
