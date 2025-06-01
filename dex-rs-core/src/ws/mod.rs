use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;
use futures_util::Sink;

use crate::DexError;

#[async_trait]
pub trait WsTransport: Send + Sync {
    async fn connect(
        &self,
        url: &str,
    ) -> Result<Box<dyn WsStream + Send + Sync + Unpin>, DexError>;
}

pub trait WsStream:
    Stream<Item = Result<Bytes, DexError>> + Sink<Bytes, Error = DexError>
{}

/* ---------- FastWebSocket impl (Tokio) ---------- */
#[cfg(feature = "rt-tokio")]
pub mod tokio_fastws {
    use super::*;

    pub struct FastWsTransport;

    #[async_trait]
    impl WsTransport for FastWsTransport {
        async fn connect(
            &self,
            _url: &str,
        ) -> Result<Box<dyn WsStream + Send + Sync + Unpin>, DexError> {
            // TODO: Implement WebSocket connection
            Err(DexError::Unsupported("WebSocket connection not yet implemented"))
        }
    }
}