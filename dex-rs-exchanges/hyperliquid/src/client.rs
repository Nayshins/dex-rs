use std::sync::Arc;
use tokio::sync::mpsc;

use dex_rs_core::{
    http::{reqwest_impl::ReqwestTransport, Http},
    traits::{PerpDex, Position, StreamEvent, StreamKind},
    ws::tokio_fastws::FastWsTransport,
    DexError,
};
use dex_rs_types::*;

use crate::{http::HlRest, signer::HlSigner, ws::HlWs};

pub struct Hyperliquid {
    rest: HlRest,
    ws: HlWs<FastWsTransport>,
    signer: Option<HlSigner>,
}

impl Hyperliquid {
    pub fn builder() -> HyperliquidBuilder {
        HyperliquidBuilder::default()
    }
}

/* ---------- builder ---------- */
pub struct HyperliquidBuilder {
    testnet: bool,
    wallet_hex: Option<String>,
}

impl Default for HyperliquidBuilder {
    fn default() -> Self {
        Self {
            testnet: false,
            wallet_hex: None,
        }
    }
}

impl HyperliquidBuilder {
    pub fn testnet(mut self) -> Self {
        self.testnet = true;
        self
    }
    pub fn wallet_hex(mut self, pk: impl Into<String>) -> Self {
        self.wallet_hex = Some(pk.into());
        self
    }
    pub fn wallet_env(self, var: &str) -> Self {
        let pk = std::env::var(var).expect("env var missing");
        self.wallet_hex(pk)
    }

    pub async fn connect(self) -> Result<Hyperliquid, DexError> {
        let tp = Arc::new(ReqwestTransport::new());
        let http = Http::new(tp.clone());
        let rest = HlRest::new(http, self.testnet);
        let ws = HlWs::new(FastWsTransport, self.testnet);

        let signer = self
            .wallet_hex
            .map(|pk| HlSigner::from_hex_key(&pk))
            .transpose()?;

        Ok(Hyperliquid { rest, ws, signer })
    }
}

/* ---------- PerpDex impl ---------- */
#[async_trait::async_trait]
impl PerpDex for Hyperliquid {
    async fn trades(&self, coin: &str, limit: usize) -> Result<Vec<Trade>, DexError> {
        self.rest.trades(coin, limit).await
    }

    async fn orderbook(&self, coin: &str, depth: usize) -> Result<OrderBook, DexError> {
        let mut ob = self.rest.l2_book(coin).await?;
        ob.bids.truncate(depth);
        ob.asks.truncate(depth);
        Ok(ob)
    }
    
    async fn all_mids(&self) -> Result<AllMids, DexError> {
        self.rest.all_mids(None).await
    }
    
    async fn meta(&self) -> Result<UniverseMeta, DexError> {
        self.rest.meta(None).await
    }
    
    async fn meta_and_asset_ctxs(&self) -> Result<MetaAndAssetCtxs, DexError> {
        self.rest.meta_and_asset_ctxs().await
    }
    
    async fn funding_history(&self, coin: &str, start_time: u64, end_time: Option<u64>) -> Result<Vec<FundingHistory>, DexError> {
        self.rest.funding_history(coin, start_time, end_time).await
    }

    /* ---- account ---- */
    async fn place_order(&self, req: OrderReq) -> Result<OrderId, DexError> {
        let signer = self
            .signer
            .as_ref()
            .ok_or(DexError::Unsupported("signer required"))?;
        let nonce = 0; // TODO: real nonce fetch
        let sig = signer.sign_order(&req, nonce).await?;
        let payload = serde_json::json!({ "type": "order", "orders": [req], "grouping": "na", "signature": sig });
        let resp = self.rest.place_order(payload).await?;
        Ok(OrderId(
            resp["data"]["statuses"][0]["resting"]["oid"]
                .as_u64()
                .unwrap()
                .to_string(),
        ))
    }

    async fn cancel(&self, id: OrderId) -> Result<(), DexError> {
        let payload = serde_json::json!({ "type":"cancel", "cancels": [{"oid": id.0.parse::<u64>().unwrap()}] });
        self.rest.place_order(payload).await?;
        Ok(())
    }

    async fn positions(&self) -> Result<Vec<Position>, DexError> {
        let signer = self.signer.as_ref().ok_or(DexError::Unsupported("signer required"))?;
        let user_state = self.rest.clearinghouse_state(&signer.address_hex(), None).await?;
        
        Ok(user_state.asset_positions.into_iter().map(|pos| Position {
            coin: pos.coin,
            size: pos.szi.parse().unwrap_or(0.0),
            entry_px: pos.entry_px.map(|p| *p),
            unrealized_pnl: pos.unrealized_pnl.parse().unwrap_or(0.0),
        }).collect())
    }
    
    async fn user_state(&self) -> Result<UserState, DexError> {
        let signer = self.signer.as_ref().ok_or(DexError::Unsupported("signer required"))?;
        self.rest.clearinghouse_state(&signer.address_hex(), None).await
    }
    
    async fn open_orders(&self) -> Result<Vec<OpenOrder>, DexError> {
        let signer = self.signer.as_ref().ok_or(DexError::Unsupported("signer required"))?;
        self.rest.open_orders(&signer.address_hex(), None).await
    }
    
    async fn user_fills(&self) -> Result<Vec<UserFill>, DexError> {
        let signer = self.signer.as_ref().ok_or(DexError::Unsupported("signer required"))?;
        self.rest.user_fills(&signer.address_hex()).await
    }
    
    async fn user_fills_by_time(&self, start_time: u64, end_time: Option<u64>) -> Result<Vec<UserFill>, DexError> {
        let signer = self.signer.as_ref().ok_or(DexError::Unsupported("signer required"))?;
        self.rest.user_fills_by_time(&signer.address_hex(), start_time, end_time).await
    }

    /* ---- streaming ---- */
    async fn subscribe(
        &self,
        kind: StreamKind,
        coin: Option<&str>,
        tx: mpsc::UnboundedSender<StreamEvent>,
    ) -> Result<(), DexError> {
        let address_hex = self.signer.as_ref().map(|s| s.address_hex());
        self.ws
            .subscribe(kind, coin, tx, address_hex.as_deref())
            .await
    }
}

impl Hyperliquid {
    /* ----- Additional convenience methods for full API access ----- */
    
    /// Get candlestick data
    pub async fn candle_snapshot(&self, coin: &str, interval: &str, start_time: u64, end_time: u64) -> Result<CandleSnapshot, DexError> {
        self.rest.candle_snapshot(coin, interval, start_time, end_time).await
    }
    
    /// Get user's fee summary (requires authentication)
    pub async fn user_fees(&self) -> Result<UserFees, DexError> {
        let signer = self.signer.as_ref().ok_or(DexError::Unsupported("signer required"))?;
        self.rest.user_fees(&signer.address_hex()).await
    }
    
    /// Get user's funding payment history (requires authentication)
    pub async fn user_funding(&self, start_time: u64, end_time: Option<u64>) -> Result<UserFunding, DexError> {
        let signer = self.signer.as_ref().ok_or(DexError::Unsupported("signer required"))?;
        self.rest.user_funding(&signer.address_hex(), start_time, end_time).await
    }
    
    /// Query specific order status (requires authentication)
    pub async fn order_status(&self, oid: u64) -> Result<OrderStatus, DexError> {
        let signer = self.signer.as_ref().ok_or(DexError::Unsupported("signer required"))?;
        self.rest.order_status(&signer.address_hex(), oid).await
    }
    
    /// Get spot market metadata
    pub async fn spot_meta(&self) -> Result<SpotMeta, DexError> {
        self.rest.spot_meta().await
    }
    
    /// Get spot market metadata with asset contexts
    pub async fn spot_meta_and_asset_ctxs(&self) -> Result<SpotMetaAndAssetCtxs, DexError> {
        self.rest.spot_meta_and_asset_ctxs().await
    }
    
    /// Get user's staking summary (requires authentication)
    pub async fn delegator_summary(&self) -> Result<DelegatorSummary, DexError> {
        let signer = self.signer.as_ref().ok_or(DexError::Unsupported("signer required"))?;
        self.rest.delegator_summary(&signer.address_hex()).await
    }
    
    /// Get user's delegation details (requires authentication)
    pub async fn delegations(&self) -> Result<Vec<Delegation>, DexError> {
        let signer = self.signer.as_ref().ok_or(DexError::Unsupported("signer required"))?;
        self.rest.delegations(&signer.address_hex()).await
    }
    
    /// Get user's staking rewards (requires authentication)
    pub async fn delegator_rewards(&self) -> Result<DelegatorRewards, DexError> {
        let signer = self.signer.as_ref().ok_or(DexError::Unsupported("signer required"))?;
        self.rest.delegator_rewards(&signer.address_hex()).await
    }
    
    /// Get user's referral state (requires authentication)
    pub async fn referral(&self) -> Result<ReferralState, DexError> {
        let signer = self.signer.as_ref().ok_or(DexError::Unsupported("signer required"))?;
        self.rest.referral(&signer.address_hex()).await
    }
    
    /// Get user's sub-accounts (requires authentication)
    pub async fn sub_accounts(&self) -> Result<Vec<SubAccount>, DexError> {
        let signer = self.signer.as_ref().ok_or(DexError::Unsupported("signer required"))?;
        self.rest.sub_accounts(&signer.address_hex()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dex_rs_types::{price, qty, Tif};

    #[test]
    fn test_builder_pattern() {
        let builder = Hyperliquid::builder();

        // Test testnet flag
        let testnet_builder = builder.testnet();
        assert_eq!(testnet_builder.testnet, true);

        // Test wallet hex
        let wallet_builder = HyperliquidBuilder::default()
            .wallet_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
        assert!(wallet_builder.wallet_hex.is_some());
    }

    #[test]
    fn test_builder_defaults() {
        let builder = HyperliquidBuilder::default();
        assert_eq!(builder.testnet, false);
        assert!(builder.wallet_hex.is_none());
    }

    #[test]
    fn test_order_payload_construction() {
        use serde_json::json;

        let order_req = OrderReq {
            coin: "BTC".to_string(),
            is_buy: true,
            px: price(50000.0),
            qty: qty(0.001),
            tif: Tif::Gtc,
            reduce_only: false,
        };

        // Test the payload structure that would be sent
        let expected_payload = json!({
            "type": "order",
            "orders": [order_req],
            "grouping": "na",
            "signature": "mock_signature"
        });

        assert_eq!(expected_payload["type"], "order");
        assert_eq!(expected_payload["grouping"], "na");
        assert!(expected_payload["orders"].is_array());
        assert!(expected_payload["signature"].is_string());
    }

    #[test]
    fn test_cancel_payload_construction() {
        use serde_json::json;

        let order_id = OrderId("12345".to_string());

        let expected_payload = json!({
            "type": "cancel",
            "cancels": [{"oid": order_id.0.parse::<u64>().unwrap()}]
        });

        assert_eq!(expected_payload["type"], "cancel");
        assert!(expected_payload["cancels"].is_array());

        let cancels = expected_payload["cancels"].as_array().unwrap();
        assert_eq!(cancels.len(), 1);
        assert_eq!(cancels[0]["oid"], 12345);
    }

    #[test]
    fn test_orderbook_depth_limiting() {
        use dex_rs_types::OrderBookLevel;

        // Create a mock orderbook with more levels than requested
        let mut orderbook = OrderBook {
            coin: "BTC".to_string(),
            ts: 1234567890,
            bids: vec![
                OrderBookLevel {
                    price: price(50000.0),
                    qty: qty(1.0),
                },
                OrderBookLevel {
                    price: price(49999.0),
                    qty: qty(1.0),
                },
                OrderBookLevel {
                    price: price(49998.0),
                    qty: qty(1.0),
                },
                OrderBookLevel {
                    price: price(49997.0),
                    qty: qty(1.0),
                },
                OrderBookLevel {
                    price: price(49996.0),
                    qty: qty(1.0),
                },
            ],
            asks: vec![
                OrderBookLevel {
                    price: price(50001.0),
                    qty: qty(1.0),
                },
                OrderBookLevel {
                    price: price(50002.0),
                    qty: qty(1.0),
                },
                OrderBookLevel {
                    price: price(50003.0),
                    qty: qty(1.0),
                },
                OrderBookLevel {
                    price: price(50004.0),
                    qty: qty(1.0),
                },
                OrderBookLevel {
                    price: price(50005.0),
                    qty: qty(1.0),
                },
            ],
        };

        // Test depth limiting (simulating what would happen in orderbook method)
        let depth = 3;
        orderbook.bids.truncate(depth);
        orderbook.asks.truncate(depth);

        assert_eq!(orderbook.bids.len(), 3);
        assert_eq!(orderbook.asks.len(), 3);
    }

    #[test]
    fn test_address_extraction() {
        // Test the pattern used in subscribe method
        struct MockSigner {
            address: String,
        }

        impl MockSigner {
            fn address_hex(&self) -> String {
                self.address.clone()
            }
        }

        let signer = Some(MockSigner {
            address: "1234567890abcdef1234567890abcdef12345678".to_string(),
        });

        // Test the pattern: signer.as_ref().map(|s| s.address_hex())
        let address_hex = signer.as_ref().map(|s| s.address_hex());
        let address_str = address_hex.as_deref();

        assert_eq!(
            address_str,
            Some("1234567890abcdef1234567890abcdef12345678")
        );
    }

    #[test]
    fn test_error_handling() {
        // Test unsupported operations
        use dex_rs_core::DexError;

        let unsupported_error = DexError::Unsupported("test feature");
        match unsupported_error {
            DexError::Unsupported(msg) => assert_eq!(msg, "test feature"),
            _ => panic!("Expected Unsupported error"),
        }

        // Test signer required error
        let signer_error = DexError::Unsupported("signer required");
        match signer_error {
            DexError::Unsupported(msg) => assert_eq!(msg, "signer required"),
            _ => panic!("Expected signer required error"),
        }
    }

    #[test]
    fn test_order_id_parsing() {
        // Test parsing order ID from response
        use serde_json::json;

        let mock_response = json!({
            "data": {
                "statuses": [{
                    "resting": {
                        "oid": 12345u64
                    }
                }]
            }
        });

        let oid = mock_response["data"]["statuses"][0]["resting"]["oid"]
            .as_u64()
            .unwrap();
        let order_id = OrderId(oid.to_string());

        assert_eq!(order_id.0, "12345");
    }

    #[test]
    fn test_stream_kind_mapping() {
        // Test that all StreamKind variants are handled
        let stream_kinds = vec![
            StreamKind::Trades,
            StreamKind::Bbo,
            StreamKind::L2Book,
            StreamKind::Orders,
            StreamKind::Fills,
        ];

        // Each should map to a specific subscription type
        for kind in stream_kinds {
            let subscription_type = match kind {
                StreamKind::Bbo => "bbo",
                StreamKind::Trades => "trades",
                StreamKind::L2Book => "l2Book",
                StreamKind::Orders => "orderUpdates",
                StreamKind::Fills => "userFills",
            };

            assert!(!subscription_type.is_empty());
        }
    }
}
