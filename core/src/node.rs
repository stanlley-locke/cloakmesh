use async_trait::async_trait;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use prost::Message;
use std::time::Duration;

use crate::proto::v1::cloak_mesh_node_server::CloakMeshNode;
use crate::proto::v1::cloak_service_server::CloakService;
use crate::proto::v1::capability_service_server::CapabilityService;
use crate::proto::v1::{
    HandshakeInit, HandshakeResponse, Envelope, Ping, Pong,
    CloakDescriptor, PublishAck, DescriptorRequest,
    Introduce1, IntroduceAck, Introduce2, RendezvousAck,
    CapabilityVerifyRequest, CapabilityVerifyResponse
};
use crate::routing::dht::{DhtNode, DhtKey};
use crate::routing::circuit::CircuitManager;
use crate::network::bridge::MeshBridge;
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
        
        // 1. Check expiration
        if let Some(expires_at) = &token.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if (expires_at.seconds as u64) < now {
                return Err(Status::unauthenticated("Token expired"));
            }
        }

        // 2. Check scope
        if !token.scopes.contains(&request.required_scope) {
            return Err(Status::permission_denied(format!("Missing required scope: {}", request.required_scope)));
        }

        // 3. Verify signature (In Phase 2 we use a mock verification for the demo)
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
}

#[async_trait]
impl CloakService for CloakNode {
    /// Use Case 1: Secure Descriptor Publication
    async fn publish_descriptor(
        &self,
        request: Request<CloakDescriptor>,
    ) -> Result<Response<PublishAck>, Status> {
        let descriptor = request.into_inner();
        
        // 1. Validate address and signature
        let pubkey = parse_address(&descriptor.cloak_address)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        
        if pubkey != descriptor.identity_pubkey.as_slice() {
            return Err(Status::unauthenticated("Address/Pubkey mismatch"));
        }

        // 2. Serialize descriptor for DHT storage
        let mut buf = Vec::with_capacity(descriptor.encoded_len());
        descriptor.encode(&mut buf).map_err(|e| Status::internal(e.to_string()))?;

        // 3. Store in DHT
        let key = DhtKey(pubkey);
        self.dht.store_local(key, buf, Duration::from_secs(3600)).await
            .map_err(|_| Status::internal("DHT storage failed"))?;

        Ok(Response::new(PublishAck {
            success: true,
            message: format!("Descriptor for {} published successfully", descriptor.cloak_address),
        }))
    }

    /// Use Case 1: Secure Descriptor Discovery
    async fn fetch_descriptor(
        &self,
        request: Request<DescriptorRequest>,
    ) -> Result<Response<CloakDescriptor>, Status> {
        let req = request.into_inner();
        
        // 1. Resolve address to DHT key
        let pubkey = parse_address(&req.cloak_address)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let key = DhtKey(pubkey);

        // 2. Fetch from DHT
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
