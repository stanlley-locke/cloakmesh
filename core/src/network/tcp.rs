//! TCP Transport Implementation
//!
//! Provides a robust, length-prefixed async TCP transport. Includes handling for
//! fragmentation, large payloads, timeouts, and graceful connection teardown.

use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, instrument, warn};

use crate::errors::{CloakError, CloakResult};
use crate::network::transport::{Connection, Transport};

const MAX_PAYLOAD_SIZE: usize = 10 * 1024 * 1024; // 10 MB limit to prevent OOM
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const IO_TIMEOUT: Duration = Duration::from_secs(30);

/// A robust TCP connection that frames messages with a 4-byte big-endian length prefix.
pub struct TcpConnection {
    stream: Arc<Mutex<TcpStream>>,
    peer_addr: SocketAddr,
}

impl TcpConnection {
    pub fn new(stream: TcpStream) -> CloakResult<Self> {
        let peer_addr = stream.peer_addr().map_err(|e| {
            CloakError::ConnectionFailed {
                addr: "unknown".into(),
                reason: format!("failed to get peer addr: {}", e),
            }
        })?;

        // Disable Nagle's algorithm for lower latency
        if let Err(e) = stream.set_nodelay(true) {
            warn!(peer = %peer_addr, error = %e, "Failed to set TCP_NODELAY");
        }

        Ok(Self {
            stream: Arc::new(Mutex::new(stream)),
            peer_addr,
        })
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }
}

#[async_trait]
impl Connection for TcpConnection {
    #[instrument(skip(self, data), fields(peer = %self.peer_addr, len = data.len()))]
    async fn send(&self, data: &[u8]) -> CloakResult<()> {
        if data.len() > MAX_PAYLOAD_SIZE {
            return Err(CloakError::Protocol(format!(
                "Payload too large: {} > {}",
                data.len(),
                MAX_PAYLOAD_SIZE
            )));
        }

        let len_prefix = (data.len() as u32).to_be_bytes();
        let mut stream = self.stream.lock().await;

        let write_future = async {
            stream.write_all(&len_prefix).await?;
            stream.write_all(data).await?;
            stream.flush().await?;
            Ok::<(), std::io::Error>(())
        };

        match timeout(IO_TIMEOUT, write_future).await {
            Ok(Ok(_)) => {
                debug!("Successfully sent framed payload");
                Ok(())
            }
            Ok(Err(e)) => {
                error!("I/O error during send: {}", e);
                Err(CloakError::SendFailed(e.to_string()))
            }
            Err(_) => {
                error!("Send timed out");
                Err(CloakError::Timeout { ms: IO_TIMEOUT.as_millis() as u64 })
            }
        }
    }

    #[instrument(skip(self), fields(peer = %self.peer_addr))]
    async fn recv(&self) -> CloakResult<Vec<u8>> {
        let mut stream = self.stream.lock().await;

        let read_future = async {
            let mut len_buf = [0u8; 4];
            stream.read_exact(&mut len_buf).await?;
            let len = u32::from_be_bytes(len_buf) as usize;

            if len > MAX_PAYLOAD_SIZE {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("incoming payload too large: {}", len),
                ));
            }

            let mut data = vec![0u8; len];
            stream.read_exact(&mut data).await?;
            Ok::<Vec<u8>, std::io::Error>(data)
        };

        match timeout(IO_TIMEOUT, read_future).await {
            Ok(Ok(data)) => {
                debug!(len = data.len(), "Successfully received framed payload");
                Ok(data)
            }
            Ok(Err(e)) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    Err(CloakError::ConnectionClosed)
                } else {
                    error!("I/O error during recv: {}", e);
                    Err(CloakError::RecvFailed(e.to_string()))
                }
            }
            Err(_) => {
                error!("Recv timed out");
                Err(CloakError::Timeout { ms: IO_TIMEOUT.as_millis() as u64 })
            }
        }
    }

    async fn close(&self) -> CloakResult<()> {
        let mut stream = self.stream.lock().await;
        stream.shutdown().await.map_err(|e| CloakError::Other(anyhow::anyhow!(e)))?;
        Ok(())
    }
}

/// A robust async TCP Transport manager.
pub struct TcpTransport {}

impl Default for TcpTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl TcpTransport {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Transport for TcpTransport {
    #[instrument(skip(self), fields(addr = %addr))]
    async fn connect(&self, addr: &str) -> CloakResult<Box<dyn Connection>> {
        debug!("Attempting TCP connection");
        let connect_future = TcpStream::connect(addr);

        match timeout(CONNECT_TIMEOUT, connect_future).await {
            Ok(Ok(stream)) => {
                info!("TCP connection established");
                let conn = TcpConnection::new(stream)?;
                Ok(Box::new(conn))
            }
            Ok(Err(e)) => {
                error!("TCP connection failed: {}", e);
                Err(CloakError::ConnectionFailed {
                    addr: addr.to_string(),
                    reason: e.to_string(),
                })
            }
            Err(_) => {
                error!("TCP connection timed out");
                Err(CloakError::Timeout { ms: CONNECT_TIMEOUT.as_millis() as u64 })
            }
        }
    }

    #[instrument(skip(self))]
    async fn listen(&self, port: u16) -> CloakResult<()> {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr).await.map_err(|e| CloakError::Other(anyhow::anyhow!(e)))?;
        info!("TCP Transport listening on {}", addr);

        // Spawn a background task to handle incoming connections.
        // In a full implementation, this would pass the accepted connections to a channel or callback.
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        info!(peer = %peer_addr, "Accepted new TCP connection");
                        if let Ok(_conn) = TcpConnection::new(stream) {
                            // Phase 2: Route this connection to the Noise handshake engine
                        }
                    }
                    Err(e) => {
                        warn!("Failed to accept TCP connection: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_tcp_framing_roundtrip() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let conn = TcpConnection::new(stream).unwrap();
            let msg = conn.recv().await.unwrap();
            assert_eq!(msg, b"ping");
            conn.send(b"pong").await.unwrap();
        });

        let transport = TcpTransport::new();
        let client_conn = transport.connect(&addr.to_string()).await.unwrap();
        
        client_conn.send(b"ping").await.unwrap();
        let response = client_conn.recv().await.unwrap();
        assert_eq!(response, b"pong");

        server_task.await.unwrap();
    }
}
