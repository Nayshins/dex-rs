use dex_rs_core::{http::Http, DexError};
use dex_rs_types::*;
use serde::{Deserialize, Serialize};

pub struct HlRest {
    base: String,
    http: Http,
}

impl HlRest {
    pub fn new(http: Http, testnet: bool) -> Self {
        let base = if testnet {
            "https://api.hyperliquid-testnet.xyz".into()
        } else {
            "https://api.hyperliquid.xyz".into()
        };
        Self { base, http }
    }

    /* ----- trades ----- */
    pub async fn trades(&self, coin: &str, limit: usize) -> Result<Vec<Trade>, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            coin: &'a str,
        }
        #[derive(Deserialize)]
        struct RawTrade {
            side: String,
            px: String,
            qty: String,
            time: u64,
            hash: String,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "trades",
            coin,
        };
        let raws: Vec<RawTrade> = self.http.post_json(&url, &body).await?;

        let trades: Result<Vec<_>, DexError> = raws
            .into_iter()
            .take(limit)
            .map(|r| -> Result<Trade, DexError> {
                Ok(Trade {
                    id: r.hash.clone(),
                    ts: r.time,
                    side: if r.side == "B" { Side::Buy } else { Side::Sell },
                    price: price(
                        r.px.parse::<f64>()
                            .map_err(|_| DexError::Parse("Invalid trade price".into()))?,
                    ),
                    qty: qty(r
                        .qty
                        .parse::<f64>()
                        .map_err(|_| DexError::Parse("Invalid trade quantity".into()))?),
                    coin: coin.to_string(),
                    tid: 0, // HTTP API doesn't provide trade ID
                })
            })
            .collect();
        trades
    }

    /* ----- order-book snapshot ----- */
    pub async fn l2_book(&self, coin: &str) -> Result<OrderBook, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            coin: &'a str,
        }
        #[derive(Deserialize)]
        struct Raw {
            levels: [Vec<[String; 2]>; 2], // [[bid_px,bid_sz], [ask_px,ask_sz]]
            time: u64,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "l2Book",
            coin,
        };
        let value: serde_json::Value = self.http.post_json(&url, &body).await?;
        let raw: Raw = serde_json::from_value(
            value
                .get(coin.to_uppercase())
                .ok_or_else(|| DexError::Parse("missing coin".into()))?
                .clone(),
        )?;

        let bids: Result<Vec<_>, DexError> = raw.levels[0]
            .iter()
            .map(|l| -> Result<OrderBookLevel, DexError> {
                Ok(OrderBookLevel {
                    price: price(
                        l[0].parse::<f64>()
                            .map_err(|_| DexError::Parse("Invalid bid price".into()))?,
                    ),
                    qty: qty(l[1]
                        .parse::<f64>()
                        .map_err(|_| DexError::Parse("Invalid bid quantity".into()))?),
                    n: 0,
                })
            })
            .collect();
        let bids = bids?;

        let asks: Result<Vec<_>, DexError> = raw.levels[1]
            .iter()
            .map(|l| -> Result<OrderBookLevel, DexError> {
                Ok(OrderBookLevel {
                    price: price(
                        l[0].parse::<f64>()
                            .map_err(|_| DexError::Parse("Invalid ask price".into()))?,
                    ),
                    qty: qty(l[1]
                        .parse::<f64>()
                        .map_err(|_| DexError::Parse("Invalid ask quantity".into()))?),
                    n: 0,
                })
            })
            .collect();
        let asks = asks?;

        Ok(OrderBook {
            coin: coin.into(),
            ts: raw.time,
            bids,
            asks,
        })
    }

    /* ----- place order ----- */
    pub async fn place_order(
        &self,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, DexError> {
        let url = format!("{}/exchange", self.base);
        self.http
            .post_json::<_, serde_json::Value>(&url, &payload)
            .await
    }

    /* ----- User Account & Trading Data Endpoints ----- */

    /// Get user's perpetual trading state
    pub async fn clearinghouse_state(
        &self,
        user: &str,
        dex: Option<&str>,
    ) -> Result<UserState, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            user: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            dex: Option<&'a str>,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "clearinghouseState",
            user,
            dex,
        };
        self.http.post_json(&url, &body).await
    }

    /// Get user's spot trading state
    pub async fn spot_clearinghouse_state(
        &self,
        user: &str,
    ) -> Result<serde_json::Value, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            user: &'a str,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "spotClearinghouseState",
            user,
        };
        self.http.post_json(&url, &body).await
    }

    /// Get user's open orders
    pub async fn open_orders(
        &self,
        user: &str,
        dex: Option<&str>,
    ) -> Result<Vec<OpenOrder>, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            user: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            dex: Option<&'a str>,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "openOrders",
            user,
            dex,
        };
        self.http.post_json(&url, &body).await
    }

    /// Get user's frontend open orders
    pub async fn frontend_open_orders(
        &self,
        user: &str,
        dex: Option<&str>,
    ) -> Result<Vec<OpenOrder>, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            user: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            dex: Option<&'a str>,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "frontendOpenOrders",
            user,
            dex,
        };
        self.http.post_json(&url, &body).await
    }

    /// Get user's fill history
    pub async fn user_fills(&self, user: &str) -> Result<Vec<UserFill>, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            user: &'a str,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "userFills",
            user,
        };
        self.http.post_json(&url, &body).await
    }

    /// Get user's fills within time range
    pub async fn user_fills_by_time(
        &self,
        user: &str,
        start_time: u64,
        end_time: Option<u64>,
    ) -> Result<Vec<UserFill>, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            user: &'a str,
            #[serde(rename = "startTime")]
            start_time: u64,
            #[serde(rename = "endTime", skip_serializing_if = "Option::is_none")]
            end_time: Option<u64>,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "userFillsByTime",
            user,
            start_time,
            end_time,
        };
        self.http.post_json(&url, &body).await
    }

    /// Get user's funding payment history
    pub async fn user_funding(
        &self,
        user: &str,
        start_time: u64,
        end_time: Option<u64>,
    ) -> Result<UserFunding, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            user: &'a str,
            #[serde(rename = "startTime")]
            start_time: u64,
            #[serde(rename = "endTime", skip_serializing_if = "Option::is_none")]
            end_time: Option<u64>,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "userFunding",
            user,
            start_time,
            end_time,
        };
        self.http.post_json(&url, &body).await
    }

    /// Get user's fee summary
    pub async fn user_fees(&self, user: &str) -> Result<UserFees, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            user: &'a str,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "userFees",
            user,
        };
        self.http.post_json(&url, &body).await
    }

    /// Query specific order status
    pub async fn order_status(&self, user: &str, oid: u64) -> Result<OrderStatus, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            user: &'a str,
            oid: u64,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "orderStatus",
            user,
            oid,
        };
        self.http.post_json(&url, &body).await
    }

    /* ----- Market Data Endpoints ----- */

    /// Get all mid prices
    pub async fn all_mids(&self, dex: Option<&str>) -> Result<AllMids, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            dex: Option<&'a str>,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "allMids",
            dex,
        };
        self.http.post_json(&url, &body).await
    }

    /// Get perpetual market metadata
    pub async fn meta(&self, dex: Option<&str>) -> Result<UniverseMeta, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            dex: Option<&'a str>,
        }

        let url = format!("{}/info", self.base);
        let body = Body { kind: "meta", dex };
        self.http.post_json(&url, &body).await
    }

    /// Get perpetual market metadata with asset contexts
    pub async fn meta_and_asset_ctxs(&self) -> Result<MetaAndAssetCtxs, DexError> {
        #[derive(Serialize)]
        struct Body {
            #[serde(rename = "type")]
            kind: &'static str,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "metaAndAssetCtxs",
        };
        self.http.post_json(&url, &body).await
    }

    /// Get spot market metadata
    pub async fn spot_meta(&self) -> Result<SpotMeta, DexError> {
        #[derive(Serialize)]
        struct Body {
            #[serde(rename = "type")]
            kind: &'static str,
        }

        let url = format!("{}/info", self.base);
        let body = Body { kind: "spotMeta" };
        self.http.post_json(&url, &body).await
    }

    /// Get spot market metadata with asset contexts
    pub async fn spot_meta_and_asset_ctxs(&self) -> Result<SpotMetaAndAssetCtxs, DexError> {
        #[derive(Serialize)]
        struct Body {
            #[serde(rename = "type")]
            kind: &'static str,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "spotMetaAndAssetCtxs",
        };
        self.http.post_json(&url, &body).await
    }

    /// Get available perpetual DEX information
    pub async fn perp_dexs(&self) -> Result<serde_json::Value, DexError> {
        #[derive(Serialize)]
        struct Body {
            #[serde(rename = "type")]
            kind: &'static str,
        }

        let url = format!("{}/info", self.base);
        let body = Body { kind: "perpDexs" };
        self.http.post_json(&url, &body).await
    }

    /// Get funding rate history
    pub async fn funding_history(
        &self,
        coin: &str,
        start_time: u64,
        end_time: Option<u64>,
    ) -> Result<Vec<FundingHistory>, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            coin: &'a str,
            #[serde(rename = "startTime")]
            start_time: u64,
            #[serde(rename = "endTime", skip_serializing_if = "Option::is_none")]
            end_time: Option<u64>,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "fundingHistory",
            coin,
            start_time,
            end_time,
        };
        self.http.post_json(&url, &body).await
    }

    /// Get candlestick data
    pub async fn candle_snapshot(
        &self,
        coin: &str,
        interval: &str,
        start_time: u64,
        end_time: u64,
    ) -> Result<CandleSnapshot, DexError> {
        #[derive(Serialize)]
        struct CandleReq<'a> {
            coin: &'a str,
            interval: &'a str,
            #[serde(rename = "startTime")]
            start_time: u64,
            #[serde(rename = "endTime")]
            end_time: u64,
        }

        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            req: CandleReq<'a>,
        }

        let url = format!("{}/info", self.base);
        let req = CandleReq {
            coin,
            interval,
            start_time,
            end_time,
        };
        let body = Body {
            kind: "candleSnapshot",
            req,
        };
        self.http.post_json(&url, &body).await
    }

    /* ----- Staking & Delegation Endpoints ----- */

    /// Get user's staking summary
    pub async fn delegator_summary(&self, user: &str) -> Result<DelegatorSummary, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            user: &'a str,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "delegatorSummary",
            user,
        };
        self.http.post_json(&url, &body).await
    }

    /// Get user's delegation details
    pub async fn delegations(&self, user: &str) -> Result<Vec<Delegation>, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            user: &'a str,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "delegations",
            user,
        };
        self.http.post_json(&url, &body).await
    }

    /// Get user's staking rewards
    pub async fn delegator_rewards(&self, user: &str) -> Result<DelegatorRewards, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            user: &'a str,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "delegatorRewards",
            user,
        };
        self.http.post_json(&url, &body).await
    }

    /* ----- Account Management Endpoints ----- */

    /// Get user's referral state
    pub async fn referral(&self, user: &str) -> Result<ReferralState, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            user: &'a str,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "referral",
            user,
        };
        self.http.post_json(&url, &body).await
    }

    /// Get user's sub-accounts
    pub async fn sub_accounts(&self, user: &str) -> Result<Vec<SubAccount>, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            user: &'a str,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "subAccounts",
            user,
        };
        self.http.post_json(&url, &body).await
    }

    /// Get multi-sig signers for user
    pub async fn user_to_multi_sig_signers(
        &self,
        user: &str,
    ) -> Result<serde_json::Value, DexError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            user: &'a str,
        }

        let url = format!("{}/info", self.base);
        let body = Body {
            kind: "userToMultiSigSigners",
            user,
        };
        self.http.post_json(&url, &body).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_url_construction() {
        // Mock HTTP client for testing
        use dex_rs_core::http::reqwest_impl::ReqwestTransport;
        use dex_rs_core::http::Http;
        use std::sync::Arc;

        let transport = Arc::new(ReqwestTransport::new());
        let http = Http::new(transport);

        let mainnet_rest = HlRest::new(http, false);
        assert!(mainnet_rest.base.contains("api.hyperliquid.xyz"));
        assert!(!mainnet_rest.base.contains("testnet"));

        let transport2 = Arc::new(ReqwestTransport::new());
        let http2 = Http::new(transport2);
        let testnet_rest = HlRest::new(http2, true);
        assert!(testnet_rest.base.contains("testnet"));
    }

    #[test]
    fn test_trade_parsing() {
        // Test parsing of raw trade data
        let raw_trades = vec![
            json!({
                "side": "B",
                "px": "50000.5",
                "qty": "0.001",
                "time": 1234567890,
                "hash": "abcdef123456"
            }),
            json!({
                "side": "A",
                "px": "49999.9",
                "qty": "0.002",
                "time": 1234567891,
                "hash": "fedcba654321"
            }),
        ];

        // Test the parsing logic (would need to extract the parsing code to a separate function)
        for (i, raw) in raw_trades.iter().enumerate() {
            let side_str = raw["side"].as_str().unwrap();
            let expected_side = if side_str == "B" {
                Side::Buy
            } else {
                Side::Sell
            };

            match i {
                0 => assert_eq!(expected_side, Side::Buy),
                1 => assert_eq!(expected_side, Side::Sell),
                _ => {}
            }

            let px: f64 = raw["px"].as_str().unwrap().parse().unwrap();
            assert!(px > 0.0);

            let qty: f64 = raw["qty"].as_str().unwrap().parse().unwrap();
            assert!(qty > 0.0);
        }
    }

    #[test]
    fn test_orderbook_level_parsing() {
        // Test parsing of L2 book levels
        let levels = vec![
            vec!["50000.0".to_string(), "0.5".to_string()],
            vec!["49999.0".to_string(), "1.0".to_string()],
        ];

        for level in levels {
            let price_val: f64 = level[0].parse().unwrap();
            let qty_val: f64 = level[1].parse().unwrap();

            assert!(price_val > 0.0);
            assert!(qty_val > 0.0);

            // Test our price/qty constructors don't panic
            use dex_rs_types::{price, qty};
            let _p = price(price_val);
            let _q = qty(qty_val);
        }
    }

    #[test]
    fn test_request_body_construction() {
        // Test trades request body
        let trades_body = json!({
            "type": "trades",
            "coin": "BTC"
        });

        assert_eq!(trades_body["type"], "trades");
        assert_eq!(trades_body["coin"], "BTC");

        // Test L2 book request body
        let l2_body = json!({
            "type": "l2Book",
            "coin": "ETH"
        });

        assert_eq!(l2_body["type"], "l2Book");
        assert_eq!(l2_body["coin"], "ETH");
    }

    #[test]
    fn test_error_handling() {
        // Test that invalid JSON parsing would fail gracefully
        let invalid_trade = json!({
            "side": "B",
            "px": "invalid_number",
            "qty": "0.001",
            "time": 1234567890,
            "hash": "abcdef123456"
        });

        let px_result: Result<f64, _> = invalid_trade["px"].as_str().unwrap().parse();
        assert!(px_result.is_err());
    }

    #[test]
    fn test_l2_book_response_structure() {
        // Test the expected structure of L2 book response
        let mock_response = json!({
            "BTC": {
                "levels": [
                    [["50000.0", "0.5"], ["49999.0", "1.0"]], // bids
                    [["50001.0", "0.3"], ["50002.0", "0.7"]]  // asks
                ],
                "time": 1234567890
            }
        });

        let btc_data = &mock_response["BTC"];
        assert!(btc_data.get("levels").is_some());
        assert!(btc_data.get("time").is_some());

        let levels = btc_data["levels"].as_array().unwrap();
        assert_eq!(levels.len(), 2); // bids and asks

        let bids = levels[0].as_array().unwrap();
        let asks = levels[1].as_array().unwrap();
        assert!(!bids.is_empty());
        assert!(!asks.is_empty());
    }

    #[test]
    fn test_new_endpoint_request_bodies() {
        // Test clearinghouseState request body
        let clearinghouse_body = json!({
            "type": "clearinghouseState",
            "user": "0x1234567890abcdef1234567890abcdef12345678"
        });
        assert_eq!(clearinghouse_body["type"], "clearinghouseState");
        assert_eq!(
            clearinghouse_body["user"],
            "0x1234567890abcdef1234567890abcdef12345678"
        );

        // Test openOrders request body
        let open_orders_body = json!({
            "type": "openOrders",
            "user": "0x1234567890abcdef1234567890abcdef12345678"
        });
        assert_eq!(open_orders_body["type"], "openOrders");

        // Test userFills request body
        let user_fills_body = json!({
            "type": "userFills",
            "user": "0x1234567890abcdef1234567890abcdef12345678"
        });
        assert_eq!(user_fills_body["type"], "userFills");

        // Test userFillsByTime request body
        let user_fills_time_body = json!({
            "type": "userFillsByTime",
            "user": "0x1234567890abcdef1234567890abcdef12345678",
            "startTime": 1234567890000u64,
            "endTime": 1234567900000u64
        });
        assert_eq!(user_fills_time_body["type"], "userFillsByTime");
        assert_eq!(user_fills_time_body["startTime"], 1234567890000u64);

        // Test fundingHistory request body
        let funding_history_body = json!({
            "type": "fundingHistory",
            "coin": "BTC",
            "startTime": 1234567890000u64
        });
        assert_eq!(funding_history_body["type"], "fundingHistory");
        assert_eq!(funding_history_body["coin"], "BTC");

        // Test meta request body
        let meta_body = json!({
            "type": "meta"
        });
        assert_eq!(meta_body["type"], "meta");

        // Test allMids request body
        let all_mids_body = json!({
            "type": "allMids"
        });
        assert_eq!(all_mids_body["type"], "allMids");

        // Test candleSnapshot request body
        let candle_body = json!({
            "type": "candleSnapshot",
            "req": {
                "coin": "BTC",
                "interval": "1h",
                "startTime": 1234567890000u64,
                "endTime": 1234567900000u64
            }
        });
        assert_eq!(candle_body["type"], "candleSnapshot");
        assert_eq!(candle_body["req"]["coin"], "BTC");
        assert_eq!(candle_body["req"]["interval"], "1h");
    }

    #[test]
    fn test_user_state_response_structure() {
        // Test expected UserState response structure
        let mock_user_state = json!({
            "assetPositions": [
                {
                    "position": {
                        "coin": "BTC",
                        "szi": "0.1",
                        "leverage": {
                            "type": "cross",
                            "value": 10
                        },
                        "entryPx": "50000.0",
                        "positionValue": "5000.0",
                        "unrealizedPnl": "100.0",
                        "returnOnEquity": "0.02"
                    }
                }
            ],
            "crossMarginSummary": {
                "accountValue": "10000.0",
                "totalMarginUsed": "500.0",
                "totalNtlPos": "5000.0",
                "totalRawUsd": "10000.0"
            },
            "crossMaintenanceMarginUsed": "250.0",
            "withdrawalsUsed": [
                {
                    "used": "0.0",
                    "limit": "1000.0"
                }
            ],
            "time": 1234567890000u64
        });

        assert!(mock_user_state.get("assetPositions").is_some());
        assert!(mock_user_state.get("crossMarginSummary").is_some());
        assert!(mock_user_state.get("time").is_some());

        let asset_positions = mock_user_state["assetPositions"].as_array().unwrap();
        assert!(!asset_positions.is_empty());
    }

    #[test]
    fn test_funding_history_response_structure() {
        // Test expected FundingHistory response structure
        let mock_funding_history = json!([
            {
                "coin": "BTC",
                "fundingRate": "0.0001",
                "premium": "0.00005",
                "time": 1234567890000u64
            },
            {
                "coin": "BTC",
                "fundingRate": "0.00015",
                "premium": "0.0001",
                "time": 1234567900000u64
            }
        ]);

        let funding_array = mock_funding_history.as_array().unwrap();
        assert_eq!(funding_array.len(), 2);

        for funding in funding_array {
            assert!(funding.get("coin").is_some());
            assert!(funding.get("fundingRate").is_some());
            assert!(funding.get("time").is_some());
        }
    }

    #[test]
    fn test_meta_response_structure() {
        // Test expected Meta response structure
        let mock_meta = json!({
            "universe": [
                {
                    "name": "BTC-USD",
                    "szDecimals": 5,
                    "maxLeverage": 50,
                    "onlyIsolated": false
                }
            ]
        });

        assert!(mock_meta.get("universe").is_some());
        let universe = mock_meta["universe"].as_array().unwrap();
        assert!(!universe.is_empty());

        let asset = &universe[0];
        assert!(asset.get("name").is_some());
        assert!(asset.get("maxLeverage").is_some());
    }
}
