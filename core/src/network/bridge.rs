//! Mesh Bridge Implementation
//!
//! Connects local applications to the CloakMesh network.
//! Features:
//! - Site Hosting: Map local port to .cloak address.
//! - Client Proxy: SOCKS5 gateway for browsing .cloak addresses.

use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, instrument, warn, debug};
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::errors::{CloakError, CloakResult};
use crate::routing::circuit::CircuitManager;
use crate::routing::dht::{DhtNode, DhtKey};
use crate::cloak_protocol::address::parse_address;
use crate::proto::v1::cloak_mesh_node_client::CloakMeshNodeClient;
use crate::proto::v1::{TunnelData, CloakDescriptor};
use prost::Message;

/// Manages local-to-mesh and mesh-to-local traffic bridging.
pub struct MeshBridge {
    circuit_manager: Arc<CircuitManager>,
    pub hosted_sites: RwLock<HashMap<String, u16>>,
    dht: Arc<DhtNode>,
}

impl MeshBridge {
    pub fn new(circuit_manager: Arc<CircuitManager>, dht: Arc<DhtNode>) -> Self {
        Self { 
            circuit_manager,
            hosted_sites: RwLock::new(HashMap::new()),
            dht,
        }
    }

    /// Start hosting a local service on a .cloak address.
    #[instrument(skip(self))]
    pub async fn host_service(&self, local_port: u16, cloak_address: &str) -> CloakResult<()> {
        info!("[200] Hosting service on CloakMesh, port: {}, addr: {}", local_port, cloak_address);
        self.hosted_sites.write().await.insert(cloak_address.to_string(), local_port);
        Ok(())
    }

    /// Start a local SOCKS5 proxy to "visit" .cloak addresses.
    #[instrument(skip(self))]
    pub async fn start_client_proxy(self: Arc<Self>, proxy_port: u16) -> CloakResult<()> {
        let addr = format!("127.0.0.1:{}", proxy_port);
        let listener = TcpListener::bind(&addr).await.map_err(|e| CloakError::Other(anyhow::anyhow!(e)))?;
        
        info!("[200] SOCKS5 Proxy started. Visit .cloak addresses via this gateway. addr: {}", addr);

        let bridge = self.clone();
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((mut stream, peer_addr)) => {
                        debug!(peer = %peer_addr, "New incoming SOCKS5 connection");
                        let bridge_inner = bridge.clone();
                        tokio::spawn(async move {
                            if let Err(e) = bridge_inner.handle_socks5(&mut stream).await {
                                warn!(peer = %peer_addr, error = %e, "SOCKS5 session failed");
                            }
                        });
                    }
                    Err(e) => warn!("Proxy accept failed: {}", e),
                }
            }
        });

        Ok(())
    }

    /// Implements RFC 1928 SOCKS5 Handshake and Request parsing.
    async fn handle_socks5(&self, stream: &mut TcpStream) -> CloakResult<()> {
        // 1. Negotiation
        let mut header = [0u8; 2];
        stream.read_exact(&mut header).await.map_err(|_| CloakError::ConnectionClosed)?;
        
        if header[0] != 0x05 {
            return Err(CloakError::Protocol("Invalid SOCKS version".into()));
        }
        
        let nmethods = header[1] as usize;
        let mut methods = vec![0u8; nmethods];
        stream.read_exact(&mut methods).await.map_err(|_| CloakError::ConnectionClosed)?;
        
        // Respond with NO AUTH (0x00)
        stream.write_all(&[0x05, 0x00]).await.map_err(|_| CloakError::ConnectionClosed)?;

        // 2. Request
        let mut request_head = [0u8; 4];
        stream.read_exact(&mut request_head).await.map_err(|_| CloakError::ConnectionClosed)?;
        
        if request_head[0] != 0x05 || request_head[1] != 0x01 { // Only CONNECT supported
            return Err(CloakError::Protocol("Unsupported SOCKS command".into()));
        }

        let address = match request_head[3] {
            0x01 => { // IPv4
                let mut addr = [0u8; 4];
                stream.read_exact(&mut addr).await.map_err(|_| CloakError::ConnectionClosed)?;
                format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3])
            }
            0x03 => { // Domain Name
                let mut len_buf = [0u8; 1];
                stream.read_exact(&mut len_buf).await.map_err(|_| CloakError::ConnectionClosed)?;
                let mut domain = vec![0u8; len_buf[0] as usize];
                stream.read_exact(&mut domain).await.map_err(|_| CloakError::ConnectionClosed)?;
                String::from_utf8_lossy(&domain).into_owned()
            }
            _ => return Err(CloakError::Protocol("Unsupported SOCKS address type".into())),
        };

        let address = address.trim().to_string();

        let mut port_buf = [0u8; 2];
        stream.read_exact(&mut port_buf).await.map_err(|_| CloakError::ConnectionClosed)?;
        let _port = u16::from_be_bytes(port_buf);

        info!(target = %address, "SOCKS5 CONNECT request received");

        // 3. Select an onion circuit and simulate connection
        let circuit = self.circuit_manager.select_random_circuit().await
            .ok_or_else(|| CloakError::InsufficientRelays { need: 1, have: 0 })?;
        
        debug!(circuit = %circuit.id, "Tunneling proxy traffic through circuit");

        // 4. Respond with Success
        let mut response = vec![0x05, 0x00, 0x00, 0x01]; // Success, RSV, IPv4
        response.extend_from_slice(&[0, 0, 0, 0]); // BND.ADDR
        response.extend_from_slice(&[0, 0]);       // BND.PORT
        stream.write_all(&response).await.map_err(|_| CloakError::ConnectionClosed)?;

        // 5. Transfer data (Actually proxy if hosted locally for demo)
        if address.ends_with(".cloak") {
            let guard = self.hosted_sites.read().await;
            if let Some(&local_port) = guard.get(&address) {
                // Address is hosted on this node! We can actually proxy the traffic.
                drop(guard); // release lock
                match TcpStream::connect(format!("127.0.0.1:{}", local_port)).await {
                    Ok(mut target_stream) => {
                        info!("Proxying traffic to local hosted site at 127.0.0.1:{}", local_port);
                        let _ = tokio::io::copy_bidirectional(stream, &mut target_stream).await;
                    }
                    Err(e) => {
                        warn!("Failed to connect to local hosted site: {}", e);
                    }
                }
            } else {
                drop(guard);
                info!("Address {} not hosted locally. Initiating remote proxy via TunnelStream...", address);
                
                // Kademlia DHT Network Lookup
                let pubkey = parse_address(&address)
                    .map_err(|e| CloakError::Protocol(format!("Invalid .cloak address: {}", e)))?;
                let key = DhtKey(pubkey);
                
                let target_ip = match self.dht.find_value_network(&key).await {
                    Ok(Some(data)) => {
                        let descriptor = CloakDescriptor::decode(&data[..])
                            .map_err(|_| CloakError::Other(anyhow::anyhow!("Failed to decode DHT descriptor")))?;
                        if descriptor.intro_points.is_empty() {
                            return Err(CloakError::Other(anyhow::anyhow!("No intro points found for address — descriptor was published without routing info")));
                        }
                        descriptor.intro_points[0].address.clone()
                    }
                    _ => return Err(CloakError::Other(anyhow::anyhow!("Failed to locate address in distributed DHT"))),
                };

                info!("Routing to intro point {} for {}", target_ip, address);

                // Open gRPC directly to the intro point (the hosting node).
                // In a full onion routing setup, this would go through guard/middle relays.
                // The intro_point IS the node hosting the site, so we send a TunnelStream request
                // with an empty path (meaning: "you are the destination").
                let channel = tonic::transport::Channel::from_shared(format!("http://{}", target_ip))
                    .map_err(|e| CloakError::Other(anyhow::anyhow!(e)))?
                    .connect().await
                    .map_err(|e| CloakError::Other(anyhow::anyhow!("Cannot reach intro point {}: {}", target_ip, e)))?;
                    
                let mut client = CloakMeshNodeClient::new(channel);
                let (tx, rx) = tokio::sync::mpsc::channel(100);
                
                let req_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
                
                // Send first chunk with routing info — empty path means we ARE the destination
                let first_msg = TunnelData {
                    circuit_id: circuit.id.clone(),
                    target_address: address.clone(),
                    path: vec![],  // empty path = intro point is the destination
                    chunk: vec![],
                    is_eof: false,
                };
                let _ = tx.send(first_msg).await;
                
                // Send gRPC request to get the response stream
                if let Ok(response) = client.tunnel_stream(req_stream).await {
                    let mut resp_stream = response.into_inner();
                    let target_addr_clone = address.clone();
                    let circ_clone = circuit.id.clone();
                    let mut buf = [0u8; 8192];

                    loop {
                        tokio::select! {
                            res = tokio::io::AsyncReadExt::read(stream, &mut buf) => {
                                match res {
                                    Ok(n) if n > 0 => {
                                        let _ = tx.send(TunnelData {
                                            circuit_id: circ_clone.clone(),
                                            target_address: target_addr_clone.clone(),
                                            path: vec![],
                                            chunk: buf[..n].to_vec(),
                                            is_eof: false,
                                        }).await;
                                    }
                                    _ => {
                                        let _ = tx.send(TunnelData {
                                            circuit_id: circ_clone.clone(),
                                            target_address: target_addr_clone.clone(),
                                            path: vec![], chunk: vec![], is_eof: true
                                        }).await;
                                        break;
                                    }
                                }
                            }
                            msg_opt = tokio_stream::StreamExt::next(&mut resp_stream) => {
                                match msg_opt {
                                    Some(Ok(msg)) => {
                                        if msg.is_eof { break; }
                                        if tokio::io::AsyncWriteExt::write_all(stream, &msg.chunk).await.is_err() {
                                            break;
                                        }
                                    }
                                    _ => break,
                                }
                            }
                        }
                    }
                } else {
                    return Err(CloakError::Other(anyhow::anyhow!("Failed to open TunnelStream to intro point {}", target_ip)));
                }
            }
        } else {
            stream.write_all(b"HTTP/1.1 403 Forbidden\r\n\r\nOnly .cloak addresses are allowed through this gateway.\n").await
                .map_err(|_| CloakError::ConnectionClosed)?;
        }

        Ok(())
    }
}
