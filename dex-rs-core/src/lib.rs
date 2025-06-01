pub mod error;
pub use error::DexError;
pub type DexResult<T> = Result<T, DexError>;

pub mod runtime;
pub use runtime::{Sleep, Spawn};

pub mod http;
pub mod rt_tokio; // feature-gated inside file
pub mod traits;
pub mod ws;

pub use traits::{PerpDex, Position, StreamEvent, StreamKind};

/* Re-export types from sibling crate for convenience */
pub use dex_rs_types as types;
