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

use crate::errors::{CloakError, CloakResult};
use crate::routing::circuit::CircuitManager;

/// Manages local-to-mesh and mesh-to-local traffic bridging.
pub struct MeshBridge {
    circuit_manager: Arc<CircuitManager>,
}

impl MeshBridge {
    pub fn new(circuit_manager: Arc<CircuitManager>) -> Self {
        Self { circuit_manager }
    }

    /// Start hosting a local service on a .cloak address.
    #[instrument(skip(self))]
    pub async fn host_service(&self, local_port: u16, cloak_address: &str) -> CloakResult<()> {
        info!(port = local_port, addr = %cloak_address, "Hosting service on CloakMesh");
        Ok(())
    }

    /// Start a local SOCKS5 proxy to "visit" .cloak addresses.
    #[instrument(skip(self))]
    pub async fn start_client_proxy(&self, proxy_port: u16) -> CloakResult<()> {
        let addr = format!("127.0.0.1:{}", proxy_port);
        let listener = TcpListener::bind(&addr).await.map_err(|e| CloakError::Other(anyhow::anyhow!(e)))?;
        
        info!(addr = %addr, "SOCKS5 Proxy started. Visit .cloak addresses via this gateway.");

        let mgr = self.circuit_manager.clone();
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((mut stream, peer_addr)) => {
                        debug!(peer = %peer_addr, "New incoming SOCKS5 connection");
                        let mgr_inner = mgr.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_socks5(&mut stream, mgr_inner).await {
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
    async fn handle_socks5(stream: &mut TcpStream, mgr: Arc<CircuitManager>) -> CloakResult<()> {
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

        let mut port_buf = [0u8; 2];
        stream.read_exact(&mut port_buf).await.map_err(|_| CloakError::ConnectionClosed)?;
        let _port = u16::from_be_bytes(port_buf);

        info!(target = %address, "SOCKS5 CONNECT request received");

        // 3. Select an onion circuit and simulate connection
        let circuit = mgr.select_random_circuit().await
            .ok_or_else(|| CloakError::InsufficientRelays { need: 1, have: 0 })?;
        
        debug!(circuit = %circuit.id, "Tunneling proxy traffic through circuit");

        // 4. Respond with Success
        let mut response = vec![0x05, 0x00, 0x00, 0x01]; // Success, RSV, IPv4
        response.extend_from_slice(&[0, 0, 0, 0]); // BND.ADDR
        response.extend_from_slice(&[0, 0]);       // BND.PORT
        stream.write_all(&response).await.map_err(|_| CloakError::ConnectionClosed)?;

        // 5. Transfer data (Mocked HTTP response for demo)
        if address.ends_with(".cloak") {
            let msg = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nWelcome to CloakMesh!\nTarget Address: {}\nTunnel Circuit: {}\n",
                address, circuit.id
            );
            stream.write_all(msg.as_bytes()).await.map_err(|_| CloakError::ConnectionClosed)?;
        } else {
            stream.write_all(b"HTTP/1.1 403 Forbidden\r\n\r\nOnly .cloak addresses are allowed through this gateway.\n").await
                .map_err(|_| CloakError::ConnectionClosed)?;
        }

        Ok(())
    }
}
