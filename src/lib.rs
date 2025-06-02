//! Public façade crate for the whole SDK.

pub use dex_rs_core::{DexError, PerpDex, StreamEvent, StreamKind};
pub use dex_rs_types as types;
pub type DexResult<T> = Result<T, DexError>;

#[cfg(feature = "hyperliquid")]
pub use dex_rs_hyperliquid::Hyperliquid;

/// Commonly-used imports in a single glob.
pub mod prelude {
    pub use crate::types::*;
    pub use crate::*;
    pub use ordered_float::NotNan;
}
