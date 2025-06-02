use async_trait::async_trait;
use bytes::Bytes;

use crate::DexError;

#[async_trait]
pub trait WsTransport: Send + Sync {
    async fn connect(
        &self,
        url: &str,
    ) -> Result<Box<dyn WsConnection + Send + Sync + Unpin>, DexError>;
}

#[async_trait]
pub trait WsConnection: Send + Sync {
    /// Read the next message from the WebSocket
    async fn read_message(&mut self) -> Result<Vec<u8>, DexError>;

    /// Send a message to the WebSocket
    async fn send_message(&mut self, data: Bytes) -> Result<(), DexError>;

    /// Close the WebSocket connection
    async fn close(&mut self) -> Result<(), DexError>;
}

/* ---------- FastWebSocket impl (Tokio) ---------- */
#[cfg(feature = "rt-tokio")]
pub mod tokio_fastws;
