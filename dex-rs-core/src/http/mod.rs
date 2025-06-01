use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};
use serde::{de::DeserializeOwned, Serialize};

use crate::DexError;

#[async_trait]
pub trait HttpTransport: Send + Sync {
    async fn call(&self, req: Request<Vec<u8>>) -> Result<Response<Bytes>, DexError>;
}

/* -------- ReqwestTransport (default) -------- */
#[cfg(feature = "http-reqwest")]
pub mod reqwest_impl {
    use super::*;
    use reqwest::Client;

    pub struct ReqwestTransport {
        client: Client,
    }

    impl ReqwestTransport {
        pub fn new() -> Self {
            Self { client: Client::builder().user_agent("dex-rs").build().unwrap() }
        }
    }

    #[async_trait]
    impl HttpTransport for ReqwestTransport {
        async fn call(&self, req: Request<Vec<u8>>) -> Result<Response<Bytes>, DexError> {
            let (parts, body) = req.into_parts();
            let mut rb = self.client.request(parts.method, parts.uri.to_string());
            rb = rb.headers(parts.headers).body(body);
            let resp = rb.send().await?;
            let status = resp.status();
            let mut builder = Response::builder().status(status);
            for (k, v) in resp.headers() {
                builder = builder.header(k, v);
            }
            let bytes = resp.bytes().await?;
            Ok(builder.body(bytes).unwrap())
        }
    }
}

/* -------- Convenience wrapper -------- */
use std::sync::Arc;

pub struct Http {
    inner: Arc<dyn HttpTransport>,
}

impl Http {
    pub fn new(inner: Arc<dyn HttpTransport>) -> Self { Self { inner } }

    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, DexError> {
        let req = Request::builder().method("GET").uri(url).body(Vec::new()).unwrap();
        let resp = self.inner.call(req).await?;
        Ok(serde_json::from_slice(resp.body())?)
    }

    pub async fn post_json<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R, DexError> {
        let req = Request::builder()
            .method("POST")
            .uri(url)
            .header("content-type", "application/json")
            .body(serde_json::to_vec(body)?)
            .unwrap();

        let resp = self.inner.call(req).await?;
        Ok(serde_json::from_slice(resp.body())?)
    }
}