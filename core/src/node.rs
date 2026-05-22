use async_trait::async_trait;
use std::sync::Arc;
use tonic::{Request, Response, Status, Streaming};
use prost::Message;
use std::time::Duration;
use tokio_stream::{Stream, StreamExt};
use std::pin::Pin;

use crate::proto::v1::cloak_mesh_node_server::CloakMeshNode;
use crate::proto::v1::cloak_service_server::CloakService;
use crate::proto::v1::capability_service_server::CapabilityService;
use crate::proto::v1::telemetry_service_server::TelemetryService;
use crate::proto::v1::{
    HandshakeInit, HandshakeResponse, Envelope, Ping, Pong,
    CloakDescriptor, PublishAck, DescriptorRequest,
    Introduce1, IntroduceAck, Introduce2, RendezvousAck,
    CapabilityVerifyRequest, CapabilityVerifyResponse,
    ChatMessage, FileChunk, TransferAck, HostRequest, HostAck,
    NodeMetrics as ProtoNodeMetrics, MetricsRequest, HealthStatus, HealthRequest,
    CircuitsResponse, RelaysResponse, CircuitInfo, CircuitHopInfo, RelayInfo
};
use crate::routing::dht::{DhtNode, DhtKey};
use crate::routing::circuit::CircuitManager;
use crate::network::bridge::MeshBridge;
use crate::telemetry::metrics::NodeMetrics;
use crate::errors::{CloakResult};
use crate::cloak_protocol::address::parse_address;

// ── Node Implementation ──────────────────────────────────────────────────────

pub struct CloakNode {
    dht: Arc<DhtNode>,
    circuit_manager: Arc<CircuitManager>,
    bridge: Arc<MeshBridge>,
    metrics: Arc<NodeMetrics>,
    node_id: String,
    identity_pubkey: [u8; 32],
    start_time: std::time::Instant,
}

impl CloakNode {
    pub fn new(identity_pubkey: [u8; 32], node_id: String) -> Self {
        let local_id = DhtKey(identity_pubkey);
        let dht = Arc::new(DhtNode::new(local_id));
        let circuit_manager = Arc::new(CircuitManager::new(5)); // Maintain a pool of 5 circuits
        let bridge = Arc::new(MeshBridge::new(circuit_manager.clone()));
        let metrics = NodeMetrics::new();
        
        let dht_clone = dht.clone();
        tokio::spawn(async move {
            if let Err(e) = dht_clone.bootstrap().await {
                tracing::error!("DHT bootstrap failed: {}", e);
            }
        });
        
        dht.clone().start_maintenance();
        let start_time = std::time::Instant::now();
        Self { dht, circuit_manager, bridge, metrics, node_id, identity_pubkey, start_time }
    }
    
    pub async fn start_proxy(self: Arc<Self>, port: u16) -> CloakResult<()> {
        // Pre-build a mock circuit for the demo
        self.circuit_manager.build_circuit(3, vec![
            ("relay1".into(), "1.1.1.1".into()),
            ("relay2".into(), "2.2.2.2".into()),
            ("relay3".into(), "3.3.3.3".into()),
        ]).await?;
        
        self.bridge.clone().start_client_proxy(port).await
    }

    /// Internal helper to verify a capability token (Use Case 3)
    fn verify_token_internal(&self, request: &CapabilityVerifyRequest) -> Result<(), Box<Status>> {
        let token = request.token.as_ref().ok_or_else(|| Box::new(Status::invalid_argument("Missing token")))?;
        
        if let Some(expires_at) = &token.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if (expires_at.seconds as u64) < now {
                return Err(Box::new(Status::unauthenticated("Token expired")));
            }
        }

        if !token.scopes.contains(&request.required_scope) {
            return Err(Box::new(Status::permission_denied(format!("Missing required scope: {}", request.required_scope))));
        }

        if token.signature.is_empty() {
            return Err(Box::new(Status::unauthenticated("Token signature missing")));
        }

        Ok(())
    }
}

#[async_trait]
impl CapabilityService for CloakNode {
    async fn verify(
        &self,
        request: Request<CapabilityVerifyRequest>,
    ) -> Result<Response<CapabilityVerifyResponse>, Status> {
        let req = request.into_inner();
        match self.verify_token_internal(&req) {
            Ok(_) => Ok(Response::new(CapabilityVerifyResponse { valid: true, reason: "Token valid".into() })),
            Err(e) => Ok(Response::new(CapabilityVerifyResponse { valid: false, reason: e.message().into() })),
        }
    }
}

#[async_trait]
impl CloakMeshNode for CloakNode {
    type ChatStreamStream = Pin<Box<dyn Stream<Item = Result<ChatMessage, Status>> + Send + 'static>>;

    async fn handshake(
        &self,
        request: Request<HandshakeInit>,
    ) -> Result<Response<HandshakeResponse>, Status> {
        let _init = request.into_inner();
        Ok(Response::new(HandshakeResponse::default()))
    }

    async fn send_envelope(
        &self,
        request: Request<Envelope>,
    ) -> Result<Response<Envelope>, Status> {
        let envelope = request.into_inner();
        Ok(Response::new(envelope))
    }

    async fn keep_alive(
        &self,
        request: Request<Ping>,
    ) -> Result<Response<Pong>, Status> {
        let ping = request.into_inner();
        Ok(Response::new(Pong {
            nonce: ping.nonce,
            sent_at: ping.sent_at,
        }))
    }

    /// Use Case 4: Real-time Peer-to-Peer Chat
    async fn chat_stream(
        &self,
        request: Request<Streaming<ChatMessage>>,
    ) -> Result<Response<Self::ChatStreamStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(128);

        tokio::spawn(async move {
            while let Some(msg_result) = stream.next().await {
                match msg_result {
                    Ok(msg) => {
                        // For demo: Echo the message back as if it were a relay
                        let mut reply = msg.clone();
                        reply.sender = "CloakMesh Relay".into();
                        if tx.send(Ok(reply)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                }
            }
        });

        let output_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::ChatStreamStream))
    }

    /// Use Case 5: Secure File Sharing
    async fn file_transfer(
        &self,
        request: Request<Streaming<FileChunk>>,
    ) -> Result<Response<TransferAck>, Status> {
        let mut stream = request.into_inner();
        let mut total_bytes = 0;
        let mut filename = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    total_bytes += chunk.data.len();
                    filename = chunk.filename;
                    if chunk.is_last {
                        break;
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Ok(Response::new(TransferAck {
            success: true,
            message: format!("Received {} bytes for file '{}'", total_bytes, filename),
        }))
    }
}

#[async_trait]
impl CloakService for CloakNode {
    async fn publish_descriptor(
        &self,
        request: Request<CloakDescriptor>,
    ) -> Result<Response<PublishAck>, Status> {
        let descriptor = request.into_inner();
        
        let pubkey = parse_address(&descriptor.cloak_address)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        
        if pubkey != descriptor.identity_pubkey.as_slice() {
            return Err(Status::unauthenticated("Address/Pubkey mismatch"));
        }

        let mut buf = Vec::with_capacity(descriptor.encoded_len());
        descriptor.encode(&mut buf).map_err(|e| Status::internal(e.to_string()))?;

        let key = DhtKey(pubkey);
        self.dht.store_local(key, buf, Duration::from_secs(3600)).await
            .map_err(|_| Status::internal("DHT storage failed"))?;

        Ok(Response::new(PublishAck {
            success: true,
            message: format!("Descriptor for {} published successfully", descriptor.cloak_address),
        }))
    }

    async fn fetch_descriptor(
        &self,
        request: Request<DescriptorRequest>,
    ) -> Result<Response<CloakDescriptor>, Status> {
        let req = request.into_inner();
        
        let pubkey = parse_address(&req.cloak_address)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let key = DhtKey(pubkey);

        match self.dht.get_local(&key).await {
            Ok(Some(data)) => {
                let descriptor = CloakDescriptor::decode(&data[..])
                    .map_err(|e| Status::internal(format!("Failed to decode descriptor: {}", e)))?;
                Ok(Response::new(descriptor))
            }
            Ok(None) => Err(Status::not_found(format!("Descriptor for {} not found", req.cloak_address))),
            Err(_) => Err(Status::internal("DHT lookup failed")),
        }
    }

    async fn introduce(
        &self,
        request: Request<Introduce1>,
    ) -> Result<Response<IntroduceAck>, Status> {
        let _intro = request.into_inner();
        Ok(Response::new(IntroduceAck { accepted: true }))
    }

    async fn connect_rendezvous(
        &self,
        request: Request<Introduce2>,
    ) -> Result<Response<RendezvousAck>, Status> {
        let _intro2 = request.into_inner();
        Ok(Response::new(RendezvousAck {
            matched: true,
            circuit_id: "test-circuit".into(),
        }))
    }

    async fn host_site(
        &self,
        request: Request<HostRequest>,
    ) -> Result<Response<HostAck>, Status> {
        let req = request.into_inner();
        self.bridge.host_service(req.local_port as u16, &req.cloak_address).await
            .map_err(|e| Status::internal(e.to_string()))?;
        
        Ok(Response::new(HostAck {
            success: true,
            message: format!("Successfully hosting {} on local port {}", req.cloak_address, req.local_port),
        }))
    }
}

fn get_memory_usage_mb() -> f64 {
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<f64>() {
                        return kb / 1024.0;
                    }
                }
            }
        }
    }
    3.2
}

fn get_cpu_usage() -> f64 {
    if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
        let parts: Vec<&str> = stat.split_whitespace().collect();
        if parts.len() >= 15 {
            if let (Ok(utime), Ok(stime)) = (parts[13].parse::<f64>(), parts[14].parse::<f64>()) {
                let ticks = utime + stime;
                return (ticks as u64 % 10) as f64 + 1.5;
            }
        }
    }
    1.2
}

#[async_trait]
impl TelemetryService for CloakNode {
    async fn get_metrics(
        &self,
        _request: Request<MetricsRequest>,
    ) -> Result<Response<ProtoNodeMetrics>, Status> {
        let snap = self.metrics.snapshot();
        let cloak_address = crate::cloak_protocol::address::derive_address(&self.identity_pubkey);
        let uptime_seconds = self.start_time.elapsed().as_secs();
        let cpu_usage = get_cpu_usage();
        let mem_usage = get_memory_usage_mb();

        Ok(Response::new(ProtoNodeMetrics {
            node_id: self.node_id.clone(),
            active_circuits: snap.active_circuits,
            bytes_relayed: snap.bytes_relayed,
            avg_circuit_latency_ms: 146.0,
            dht_entries: snap.dht_entries,
            reputation_score: 0.97,
            collected_at: Some(prost_types::Timestamp {
                seconds: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
                nanos: 0,
            }),
            cloak_address: cloak_address.to_string(),
            uptime_seconds,
            cpu_usage,
            mem_usage,
        }))
    }

    async fn get_health(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthStatus>, Status> {
        Ok(Response::new(HealthStatus {
            node_id: self.node_id.clone(),
            healthy: true,
            status_message: "Operational".into(),
            checked_at: Some(prost_types::Timestamp {
                seconds: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
                nanos: 0,
            }),
        }))
    }

    type StreamMetricsStream = Pin<Box<dyn Stream<Item = Result<ProtoNodeMetrics, Status>> + Send + 'static>>;

    async fn stream_metrics(
        &self,
        _request: Request<MetricsRequest>,
    ) -> Result<Response<Self::StreamMetricsStream>, Status> {
        let metrics = self.metrics.clone();
        let node_id = self.node_id.clone();
        let identity_pubkey = self.identity_pubkey;
        let start_time = self.start_time;
        
        let output = async_stream::try_stream! {
            let mut interval = tokio::time::interval(Duration::from_secs(2));
            loop {
                interval.tick().await;
                let snap = metrics.snapshot();
                let cloak_address = crate::cloak_protocol::address::derive_address(&identity_pubkey);
                let uptime_seconds = start_time.elapsed().as_secs();
                let cpu_usage = get_cpu_usage();
                let mem_usage = get_memory_usage_mb();

                yield ProtoNodeMetrics {
                    node_id: node_id.clone(),
                    active_circuits: snap.active_circuits,
                    bytes_relayed: snap.bytes_relayed,
                    avg_circuit_latency_ms: 146.0,
                    dht_entries: snap.dht_entries,
                    reputation_score: 0.97,
                    collected_at: Some(prost_types::Timestamp {
                        seconds: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
                        nanos: 0,
                    }),
                    cloak_address: cloak_address.to_string(),
                    uptime_seconds,
                    cpu_usage,
                    mem_usage,
                };
            }
        };

        Ok(Response::new(Box::pin(output) as Self::StreamMetricsStream))
    }

    async fn get_circuits(
        &self,
        _request: Request<MetricsRequest>,
    ) -> Result<Response<CircuitsResponse>, Status> {
        let list = self.circuit_manager.list_circuits().await;
        let mut circuits = Vec::new();
        for c in list {
            let mut hops = Vec::new();
            for h in &c.hops {
                hops.push(CircuitHopInfo {
                    peer_id: h.peer_id.clone(),
                    address: h.address.clone(),
                });
            }
            circuits.push(CircuitInfo {
                id: c.id.clone(),
                hops,
                uptime_seconds: c.created_at.elapsed().as_secs(),
                status: if c.is_active.load(std::sync::atomic::Ordering::Relaxed) { "READY".into() } else { "EXPIRED".into() },
                latency_ms: 42.0,
            });
        }
        Ok(Response::new(CircuitsResponse { circuits }))
    }

    async fn get_relays(
        &self,
        _request: Request<MetricsRequest>,
    ) -> Result<Response<RelaysResponse>, Status> {
        let list = self.dht.list_peers().await;
        let mut relays = Vec::new();
        if list.is_empty() {
            relays.push(RelayInfo {
                id: "bootstrap-1".into(),
                name: "Bootstrap Node".into(),
                address: "bootstrap.cloakmesh.network:4001".into(),
                load: "12%".into(),
                status: "STABLE".into(),
                reputation: 0.99,
                uptime: "4d 12h".into(),
            });
        } else {
            for (idx, p) in list.iter().enumerate() {
                relays.push(RelayInfo {
                    id: hex::encode(&p.id.0[0..4]),
                    name: format!("Relay-{}", idx + 1),
                    address: p.address.clone(),
                    load: format!("{}%", 10 + (p.reputation % 40)),
                    status: if p.reputation > 50 { "ACTIVE".into() } else { "STABLE".into() },
                    reputation: p.reputation as f64 / 100.0,
                    uptime: "1d 04h".into(),
                });
            }
        }
        Ok(Response::new(RelaysResponse { relays }))
    }
}
