use thiserror::Error;

#[derive(Error, Debug)]
pub enum DexError {
    #[error("HTTP transport: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("WebSocket: {0}")]
    Ws(#[from] fastwebsockets::Error),

    #[error("Parse: {0}")]
    Parse(String),

    #[error("Exchange error {code:?}: {msg}")]
    Exchange { code: Option<i64>, msg: String },

    #[error("Timeout")]
    Timeout,

    #[error("Unsupported feature: {0}")]
    Unsupported(&'static str),

    #[error("Other: {0}")]
    Other(String),
}

/* Blanket From impls for common libs */
impl From<serde_json::Error> for DexError {
    fn from(e: serde_json::Error) -> Self {
        Self::Parse(e.to_string())
    }
}
