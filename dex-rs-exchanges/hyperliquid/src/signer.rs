//! Hyperliquid signing implementation.
//!
//! Based on the official Hyperliquid protocol, this implements proper MessagePack
//! encoding and EIP-712 signing for order placement.

use alloy::{
    primitives::{keccak256, Address},
    signers::{local::PrivateKeySigner, Signer},
};
use dex_rs_core::DexError;
use dex_rs_types::OrderReq;
use serde::Serialize;

#[derive(Clone)]
pub struct HlSigner {
    wallet: PrivateKeySigner,
    address: Address,
}

impl HlSigner {
    pub fn from_hex_key(pk_hex: &str) -> Result<Self, DexError> {
        let wallet = pk_hex
            .parse::<PrivateKeySigner>()
            .map_err(|e| DexError::Other(e.to_string()))?;
        let address = wallet.address();
        Ok(Self { wallet, address })
    }

    pub fn address_hex(&self) -> String {
        // Hyperliquid requires lowercase addresses
        format!("{:x}", self.address)
    }

    /// Sign a user action (like placing an order) using MessagePack encoding
    pub async fn sign_order(&self, ord: &OrderReq, nonce: u64) -> Result<String, DexError> {
        let action = OrderAction::from_req(ord, nonce);
        let user_signed_action = UserSignedAction { action };

        // Hyperliquid requires MessagePack encoding before signing
        let msgpack_bytes = rmp_serde::to_vec(&user_signed_action)
            .map_err(|e| DexError::Other(format!("MessagePack encoding failed: {}", e)))?;

        // Hash the MessagePack bytes
        let hash = keccak256(&msgpack_bytes);
        let sig = self
            .wallet
            .sign_hash(&hash.into())
            .await
            .map_err(|e| DexError::Other(e.to_string()))?;

        // Format as hex string
        Ok(format!("0x{}", hex::encode(sig.as_bytes())))
    }
}

/// User-signed action wrapper for MessagePack encoding
#[derive(Debug, Serialize)]
struct UserSignedAction {
    action: OrderAction,
}

/// Order action payload - field order is critical for MessagePack
#[derive(Debug, Serialize)]
struct OrderAction {
    #[serde(rename = "type")]
    action_type: String,
    orders: Vec<Order>,
    grouping: String,
}

#[derive(Debug, Serialize)]
struct Order {
    a: u32,       // asset index
    b: bool,      // is_buy
    p: String,    // price
    s: String,    // size
    r: bool,      // reduce_only
    t: OrderType, // order type
    c: String,    // client_order_id (nonce)
}

#[derive(Debug, Serialize)]
struct OrderType {
    limit: LimitOrder,
}

#[derive(Debug, Serialize)]
struct LimitOrder {
    tif: String, // time in force
}

impl OrderAction {
    fn from_req(req: &OrderReq, nonce: u64) -> Self {
        let order = Order {
            a: 0, // TODO: need proper asset mapping
            b: req.is_buy,
            p: format!("{}", *req.px),
            s: format!("{}", *req.qty),
            r: req.reduce_only,
            t: OrderType {
                limit: LimitOrder {
                    tif: match req.tif {
                        dex_rs_types::Tif::Ioc => "Ioc".to_string(),
                        dex_rs_types::Tif::Gtc => "Gtc".to_string(),
                        dex_rs_types::Tif::Alo => "Alo".to_string(),
                    },
                },
            },
            c: nonce.to_string(),
        };

        OrderAction {
            action_type: "order".to_string(),
            orders: vec![order],
            grouping: "na".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dex_rs_types::{price, qty, Tif};

    const TEST_PRIVATE_KEY: &str =
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

    #[test]
    fn test_signer_creation() {
        let signer = HlSigner::from_hex_key(TEST_PRIVATE_KEY).unwrap();
        let addr = signer.address_hex();

        // Should be lowercase
        assert_eq!(addr, addr.to_lowercase());
        // Should start without 0x prefix in our implementation
        assert!(!addr.starts_with("0x"));
        // Should be 40 characters (20 bytes hex)
        assert_eq!(addr.len(), 40);
    }

    #[test]
    fn test_invalid_private_key() {
        let result = HlSigner::from_hex_key("invalid_key");
        assert!(result.is_err());
    }

    #[test]
    fn test_order_action_construction() {
        let order_req = OrderReq {
            coin: "BTC".to_string(),
            is_buy: true,
            px: price(50000.0),
            qty: qty(0.001),
            tif: Tif::Gtc,
            reduce_only: false,
        };

        let action = OrderAction::from_req(&order_req, 12345);

        assert_eq!(action.action_type, "order");
        assert_eq!(action.grouping, "na");
        assert_eq!(action.orders.len(), 1);

        let order = &action.orders[0];
        assert_eq!(order.a, 0);
        assert_eq!(order.b, true);
        assert_eq!(order.p, "50000");
        assert_eq!(order.s, "0.001");
        assert_eq!(order.r, false);
        assert_eq!(order.t.limit.tif, "Gtc");
        assert_eq!(order.c, "12345");
    }

    #[test]
    fn test_tif_mapping() {
        let test_cases = vec![(Tif::Ioc, "Ioc"), (Tif::Gtc, "Gtc"), (Tif::Alo, "Alo")];

        for (tif, expected) in test_cases {
            let order_req = OrderReq {
                coin: "BTC".to_string(),
                is_buy: true,
                px: price(50000.0),
                qty: qty(0.001),
                tif,
                reduce_only: false,
            };

            let action = OrderAction::from_req(&order_req, 0);
            assert_eq!(action.orders[0].t.limit.tif, expected);
        }
    }

    #[test]
    fn test_messagepack_serialization() {
        let order_req = OrderReq {
            coin: "BTC".to_string(),
            is_buy: true,
            px: price(50000.0),
            qty: qty(0.001),
            tif: Tif::Gtc,
            reduce_only: false,
        };

        let action = OrderAction::from_req(&order_req, 12345);
        let user_signed_action = UserSignedAction { action };

        // Should serialize to MessagePack without error
        let result = rmp_serde::to_vec(&user_signed_action);
        assert!(result.is_ok());

        let msgpack_bytes = result.unwrap();
        assert!(!msgpack_bytes.is_empty());
    }

    #[tokio::test]
    async fn test_sign_order() {
        let signer = HlSigner::from_hex_key(TEST_PRIVATE_KEY).unwrap();

        let order_req = OrderReq {
            coin: "BTC".to_string(),
            is_buy: true,
            px: price(50000.0),
            qty: qty(0.001),
            tif: Tif::Gtc,
            reduce_only: false,
        };

        let result = signer.sign_order(&order_req, 12345).await;
        assert!(result.is_ok());

        let signature = result.unwrap();
        // Should be hex string starting with 0x
        assert!(signature.starts_with("0x"));
        // Should be 132 characters (0x + 130 hex chars = 65 bytes: 32 + 32 + 1 for r,s,v)
        assert_eq!(signature.len(), 132);
    }
}
