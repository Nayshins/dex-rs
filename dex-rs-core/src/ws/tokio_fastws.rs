//! Tokio-based WebSocket transport using fastwebsockets.

use super::*;
use bytes::Bytes;
use fastwebsockets::{Frame, OpCode, Payload, WebSocket};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct FastWsTransport;

#[async_trait]
impl WsTransport for FastWsTransport {
    async fn connect(
        &self,
        url: &str,
    ) -> Result<Box<dyn WsConnection + Send + Sync + Unpin>, DexError> {
        use http::{Request, Uri};
        use http_body_util::Empty;
        use hyper_util::rt::tokio::TokioExecutor;
        use tokio::net::TcpStream;
        use tokio_rustls::{
            rustls::{pki_types::ServerName, ClientConfig, RootCertStore},
            TlsConnector,
        };
        use webpki_roots;

        let uri: Uri = url
            .parse()
            .map_err(|e| DexError::Ws(format!("Invalid URL: {}", e)))?;

        let host = uri.host().unwrap_or("localhost").to_string();
        let port = uri
            .port_u16()
            .unwrap_or(if uri.scheme_str() == Some("wss") {
                443
            } else {
                80
            });
        let is_tls = uri.scheme_str() == Some("wss");

        let tcp_stream = TcpStream::connect((host.as_str(), port))
            .await
            .map_err(|e| DexError::Ws(format!("Connection failed: {}", e)))?;

        // Build WebSocket request with empty body
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Host", &host)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                fastwebsockets::handshake::generate_key(),
            )
            .body(Empty::<Bytes>::new())
            .map_err(|e| DexError::Ws(format!("Failed to build request: {}", e)))?;

        let executor = TokioExecutor::new();

        if is_tls {
            // Set up TLS configuration
            let mut root_store = RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

            let config = ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();

            let connector = TlsConnector::from(Arc::new(config));
            let domain = ServerName::try_from(host.clone())
                .map_err(|e| DexError::Ws(format!("Invalid hostname: {}", e)))?;

            let tls_stream = connector
                .connect(domain, tcp_stream)
                .await
                .map_err(|e| DexError::Ws(format!("TLS connection failed: {}", e)))?;

            let (ws, _) = fastwebsockets::handshake::client(&executor, req, tls_stream)
                .await
                .map_err(|e| DexError::Ws(format!("WebSocket handshake failed: {}", e)))?;

            Ok(Box::new(FastWsConnection {
                ws: Arc::new(Mutex::new(ws)),
            }))
        } else {
            let (ws, _) = fastwebsockets::handshake::client(&executor, req, tcp_stream)
                .await
                .map_err(|e| DexError::Ws(format!("WebSocket handshake failed: {}", e)))?;

            Ok(Box::new(FastWsConnection {
                ws: Arc::new(Mutex::new(ws)),
            }))
        }
    }
}

pub struct FastWsConnection {
    ws: Arc<Mutex<WebSocket<hyper_util::rt::tokio::TokioIo<hyper::upgrade::Upgraded>>>>,
}

#[async_trait]
impl WsConnection for FastWsConnection {
    async fn read_message(&mut self) -> Result<Vec<u8>, DexError> {
        let mut ws = self.ws.lock().await;
        loop {
            let frame = ws
                .read_frame()
                .await
                .map_err(|e| DexError::Ws(format!("Failed to read frame: {}", e)))?;

            match frame.opcode {
                OpCode::Text | OpCode::Binary => {
                    return Ok(frame.payload.to_vec());
                }
                OpCode::Close => {
                    return Err(DexError::Ws("Connection closed by peer".into()));
                }
                OpCode::Ping => {
                    // Auto-respond to ping with pong
                    let pong = Frame::pong(frame.payload);
                    ws.write_frame(pong)
                        .await
                        .map_err(|e| DexError::Ws(format!("Failed to send pong: {}", e)))?;
                }
                OpCode::Pong => {
                    // Ignore pong frames, continue to next frame
                }
                OpCode::Continuation => {
                    // This shouldn't happen with FragmentCollector
                    return Err(DexError::Ws("Unexpected continuation frame".into()));
                }
            }
        }
    }

    async fn send_message(&mut self, data: Bytes) -> Result<(), DexError> {
        let mut ws = self.ws.lock().await;
        let frame = Frame::text(Payload::Owned(data.to_vec()));
        ws.write_frame(frame)
            .await
            .map_err(|e| DexError::Ws(format!("Failed to send message: {}", e)))
    }

    async fn close(&mut self) -> Result<(), DexError> {
        let mut ws = self.ws.lock().await;
        let frame = Frame::close(1000, b"");
        ws.write_frame(frame)
            .await
            .map_err(|e| DexError::Ws(format!("Failed to close connection: {}", e)))
    }
}
