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
use crate::proto::v1::{
    HandshakeInit, HandshakeResponse, Envelope, Ping, Pong,
    CloakDescriptor, PublishAck, DescriptorRequest,
    Introduce1, IntroduceAck, Introduce2, RendezvousAck,
    CapabilityVerifyRequest, CapabilityVerifyResponse,
    ChatMessage, FileChunk, TransferAck
};
use crate::routing::dht::{DhtNode, DhtKey};
use crate::routing::circuit::CircuitManager;
use crate::network::bridge::MeshBridge;
use crate::errors::{CloakResult};
use crate::cloak_protocol::address::parse_address;

// ── Node Implementation ──────────────────────────────────────────────────────

pub struct CloakNode {
    dht: Arc<DhtNode>,
    circuit_manager: Arc<CircuitManager>,
    bridge: Arc<MeshBridge>,
    #[allow(dead_code)]
    identity_pubkey: [u8; 32],
}

impl CloakNode {
    pub fn new(identity_pubkey: [u8; 32]) -> Self {
        let local_id = DhtKey(identity_pubkey);
        let dht = Arc::new(DhtNode::new(local_id));
        let circuit_manager = Arc::new(CircuitManager::new());
        let bridge = Arc::new(MeshBridge::new(circuit_manager.clone()));
        
        dht.clone().start_maintenance();
        Self { dht, circuit_manager, bridge, identity_pubkey }
    }
    
    pub async fn start_proxy(&self, port: u16) -> CloakResult<()> {
        // Pre-build a mock circuit for the demo
        self.circuit_manager.build_circuit(3, vec![
            ("relay1".into(), "1.1.1.1".into()),
            ("relay2".into(), "2.2.2.2".into()),
            ("relay3".into(), "3.3.3.3".into()),
        ]).await?;
        
        self.bridge.start_client_proxy(port).await
    }

    /// Internal helper to verify a capability token (Use Case 3)
    fn verify_token_internal(&self, request: &CapabilityVerifyRequest) -> Result<(), Status> {
        let token = request.token.as_ref().ok_or_else(|| Status::invalid_argument("Missing token"))?;
        
        if let Some(expires_at) = &token.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if (expires_at.seconds as u64) < now {
                return Err(Status::unauthenticated("Token expired"));
            }
        }

        if !token.scopes.contains(&request.required_scope) {
            return Err(Status::permission_denied(format!("Missing required scope: {}", request.required_scope)));
        }

        if token.signature.is_empty() {
            return Err(Status::unauthenticated("Token signature missing"));
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
}
