//! Tokio-based WebSocket transport using fastwebsockets.

use super::*;
use fastwebsockets::{ClientBuilder, WebSocket};
use futures_util::{StreamExt, SinkExt};
use tokio::io::BufStream;
use tokio::net::TcpStream;

pub struct FastWsTransport;

#[async_trait]
impl WsTransport for FastWsTransport {
    async fn connect(
        &self,
        url: &str,
    ) -> Result<Box<dyn WsStream + Send + Sync + Unpin>, DexError> {
        let (ws, _response) = ClientBuilder::new(url)
            .connect()
            .await
            .map_err(DexError::from)?;
        Ok(Box::new(ws))
    }
}

// Blanket impl ties fastwebsockets::WebSocket to our alias.
impl<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static> WsStream
    for WebSocket<BufStream<S>>
{}