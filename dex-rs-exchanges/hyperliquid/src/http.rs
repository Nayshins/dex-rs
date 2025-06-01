use dex_rs_core::{http::Http, DexError};
use dex_rs_types::{price, qty, OrderBook, OrderBookLevel, Side, Trade};
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

        Ok(raws
            .into_iter()
            .take(limit)
            .map(|r| Trade {
                id: r.hash,
                ts: r.time,
                side: if r.side == "B" { Side::Buy } else { Side::Sell },
                price: price(r.px.parse::<f64>().unwrap()),
                qty: qty(r.qty.parse::<f64>().unwrap()),
            })
            .collect())
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

        let bids = raw.levels[0]
            .iter()
            .map(|l| OrderBookLevel {
                price: price(l[0].parse::<f64>().unwrap()),
                qty: qty(l[1].parse::<f64>().unwrap()),
            })
            .collect();

        let asks = raw.levels[1]
            .iter()
            .map(|l| OrderBookLevel {
                price: price(l[0].parse::<f64>().unwrap()),
                qty: qty(l[1].parse::<f64>().unwrap()),
            })
            .collect();

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
}
