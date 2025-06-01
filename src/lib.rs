//! Public façade crate for the whole SDK.

pub use dex_rs_types  as types;
pub use dex_rs_core::{DexError, PerpDex, StreamKind, StreamEvent};
pub type DexResult<T> = Result<T, DexError>;

#[cfg(feature = "hyperliquid")]
pub use dex_rs_hyperliquid::Hyperliquid;

/// Commonly-used imports in a single glob.
pub mod prelude {
    pub use crate::*;
    pub use crate::types::*;
    pub use ordered_float::NotNan;
}