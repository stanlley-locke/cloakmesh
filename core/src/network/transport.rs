// Transport abstraction — QUIC primary, TCP fallback
// Full implementation: Phase 2.2
use async_trait::async_trait;
use crate::errors::CloakResult;

#[async_trait]
pub trait Transport: Send + Sync {
    async fn connect(&self, addr: &str) -> CloakResult<Box<dyn Connection>>;
    async fn listen(&self, port: u16) -> CloakResult<()>;
}

#[async_trait]
pub trait Connection: Send + Sync {
    async fn send(&self, data: &[u8]) -> CloakResult<()>;
    async fn recv(&self) -> CloakResult<Vec<u8>>;
    async fn close(&self) -> CloakResult<()>;
}
