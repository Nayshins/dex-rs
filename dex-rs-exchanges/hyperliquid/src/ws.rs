use bytes::Bytes;
use dex_rs_core::traits::{FillEvent, OrderEvent, StreamEvent, StreamKind};
use dex_rs_core::{ws::WsTransport, DexError};
use dex_rs_types::{price, qty, OrderBook, OrderBookLevel, Side, Trade};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

pub struct HlWs<T: WsTransport + Clone + 'static> {
    txp: T,
    url: String,
}

#[derive(Deserialize, Debug)]
struct TradeData {
    coin: String,
    side: String,
    px: String,
    sz: String,
    time: u64,
    hash: String,
    tid: u64,
}

#[derive(Deserialize, Debug)]
struct L2BookData {
    coin: String,
    time: u64,
    levels: [Vec<L2Level>; 2], // [bids, asks]
}

#[derive(Deserialize, Debug)]
struct L2Level {
    px: String,
    sz: String,
    n: u32,
}

#[derive(Deserialize, Debug)]
struct BboData {
    coin: String,
    time: u64,
    #[serde(rename = "bestBid")]
    best_bid: String,
    #[serde(rename = "bestAsk")]
    best_ask: String,
}

#[derive(Deserialize, Debug)]
struct OrderUpdate {
    order: BasicOrder,
    status: String,
    #[serde(rename = "statusTimestamp")]
    status_timestamp: u64,
}

#[derive(Deserialize, Debug)]
struct BasicOrder {
    coin: String,
    side: String,
    #[serde(rename = "limitPx")]
    limit_px: String,
    sz: String,
    oid: u64,
    timestamp: u64,
}

#[derive(Deserialize, Debug)]
struct UserFillsData {
    user: String,
    fills: Vec<UserFill>,
}

#[derive(Deserialize, Debug)]
struct UserFill {
    coin: String,
    px: String,
    sz: String,
    side: String,
    time: u64,
    hash: String,
    oid: u64,
    tid: u64,
    fee: String,
}

impl<T: WsTransport + Clone + 'static> HlWs<T> {
    pub fn new(txp: T, testnet: bool) -> Self {
        let url = if testnet {
            "wss://api.hyperliquid-testnet.xyz/ws"
        } else {
            "wss://api.hyperliquid.xyz/ws"
        };
        Self {
            txp,
            url: url.into(),
        }
    }

    pub async fn subscribe(
        &self,
        kind: StreamKind,
        coin: Option<&str>,
        out: mpsc::UnboundedSender<StreamEvent>,
        address_hex: Option<&str>,
    ) -> Result<(), DexError> {
        let subscription = match kind {
            StreamKind::Bbo => json!({
                "type": "bbo",
                "coin": coin.ok_or(DexError::Other("coin required for BBO".into()))?
            }),
            StreamKind::Trades => json!({
                "type": "trades",
                "coin": coin.ok_or(DexError::Other("coin required for trades".into()))?
            }),
            StreamKind::L2Book => json!({
                "type": "l2Book",
                "coin": coin.ok_or(DexError::Other("coin required for l2Book".into()))?
            }),
            StreamKind::Orders => json!({
                "type": "orderUpdates",
                "user": address_hex.ok_or(DexError::Other("address required for orders".into()))?
            }),
            StreamKind::Fills => json!({
                "type": "userFills",
                "user": address_hex.ok_or(DexError::Other("address required for fills".into()))?
            }),
        };

        let msg = json!({
            "method": "subscribe",
            "subscription": subscription
        });

        // Clone necessary data for the reconnection loop
        let txp = self.txp.clone();
        let url = self.url.clone();
        let stream_kind = kind;
        let msg_bytes = Bytes::from(msg.to_string());

        tokio::spawn(async move {
            let mut retry_count = 0;
            const MAX_RETRIES: u32 = 10;
            const BASE_DELAY_MS: u64 = 1000;
            const MAX_DELAY_MS: u64 = 30000;

            loop {
                match Self::connect_and_subscribe(&txp, &url, &msg_bytes, &out, stream_kind).await {
                    Ok(_) => {
                        // Connection ended normally, reset retry count
                        retry_count = 0;
                        eprintln!("WebSocket connection ended, attempting to reconnect...");
                    }
                    Err(e) => {
                        retry_count += 1;
                        eprintln!("WebSocket error (attempt {}): {:?}", retry_count, e);

                        if retry_count >= MAX_RETRIES {
                            eprintln!("Max retries reached, giving up on WebSocket connection");
                            break;
                        }
                    }
                }

                // Exponential backoff with simple jitter
                let delay_ms = std::cmp::min(
                    BASE_DELAY_MS * 2_u64.pow(retry_count.saturating_sub(1)),
                    MAX_DELAY_MS,
                );
                // Simple jitter using retry_count for deterministic but varied delays
                let jitter = (retry_count as u64 * 137) % (delay_ms / 4 + 1); // Add up to 25% jitter
                let total_delay = delay_ms + jitter;

                eprintln!("Waiting {}ms before reconnecting...", total_delay);
                sleep(Duration::from_millis(total_delay)).await;
            }
        });

        Ok(())
    }

    async fn connect_and_subscribe<U: WsTransport + 'static>(
        txp: &U,
        url: &str,
        msg_bytes: &Bytes,
        out: &mpsc::UnboundedSender<StreamEvent>,
        stream_kind: StreamKind,
    ) -> Result<(), DexError> {
        let mut ws = txp.connect(url).await?;
        ws.send_message(msg_bytes.clone()).await?;

        loop {
            match ws.read_message().await {
                Ok(bytes) => {
                    if let Err(e) = Self::handle_message(&bytes, out, stream_kind).await {
                        eprintln!("Error handling WebSocket message: {}", e);
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    async fn handle_message(
        bytes: &[u8],
        out: &mpsc::UnboundedSender<StreamEvent>,
        kind: StreamKind,
    ) -> Result<(), DexError> {
        let val: Value = serde_json::from_slice(bytes)?;

        if val.get("method") == Some(&json!("subscriptionResponse")) {
            return Ok(());
        }

        let event = match kind {
            StreamKind::Bbo => Self::parse_bbo(&val)?,
            StreamKind::Trades => Self::parse_trades(&val)?,
            StreamKind::L2Book => Self::parse_l2_book(&val)?,
            StreamKind::Orders => Self::parse_orders(&val)?,
            StreamKind::Fills => Self::parse_fills(&val)?,
        };

        if let Some(ev) = event {
            let _ = out.send(ev);
        }

        Ok(())
    }

    fn parse_bbo(val: &Value) -> Result<Option<StreamEvent>, DexError> {
        if let Ok(bbo) = serde_json::from_value::<BboData>(val["data"].clone()) {
            Ok(Some(StreamEvent::Bbo {
                coin: bbo.coin,
                bid_px: bbo
                    .best_bid
                    .parse()
                    .map_err(|_| DexError::Parse("Invalid bid price".into()))?,
                ask_px: bbo
                    .best_ask
                    .parse()
                    .map_err(|_| DexError::Parse("Invalid ask price".into()))?,
                timestamp: bbo.time,
            }))
        } else {
            Ok(None)
        }
    }

    fn parse_trades(val: &Value) -> Result<Option<StreamEvent>, DexError> {
        if let Ok(trades) = serde_json::from_value::<Vec<TradeData>>(val["data"].clone()) {
            if let Some(trade_data) = trades.into_iter().next() {
                let trade = Trade {
                    id: trade_data.hash.clone(),
                    ts: trade_data.time,
                    side: if trade_data.side == "B" {
                        Side::Buy
                    } else {
                        Side::Sell
                    },
                    price: price(
                        trade_data
                            .px
                            .parse()
                            .map_err(|_| DexError::Parse("Invalid trade price".into()))?,
                    ),
                    qty: qty(trade_data
                        .sz
                        .parse()
                        .map_err(|_| DexError::Parse("Invalid trade size".into()))?),
                    coin: trade_data.coin,
                    tid: trade_data.tid,
                };
                return Ok(Some(StreamEvent::Trade(trade)));
            }
        }
        Ok(None)
    }

    fn parse_l2_book(val: &Value) -> Result<Option<StreamEvent>, DexError> {
        if let Ok(book) = serde_json::from_value::<L2BookData>(val["data"].clone()) {
            let bids: Result<Vec<_>, DexError> = book.levels[0]
                .iter()
                .map(|level| -> Result<OrderBookLevel, DexError> {
                    Ok(OrderBookLevel {
                        price: price(
                            level
                                .px
                                .parse()
                                .map_err(|_| DexError::Parse("Invalid L2 bid price".into()))?,
                        ),
                        qty: qty(level
                            .sz
                            .parse()
                            .map_err(|_| DexError::Parse("Invalid L2 bid quantity".into()))?),
                        n: level.n,
                    })
                })
                .collect();
            let bids = bids?;

            let asks: Result<Vec<_>, DexError> = book.levels[1]
                .iter()
                .map(|level| -> Result<OrderBookLevel, DexError> {
                    Ok(OrderBookLevel {
                        price: price(
                            level
                                .px
                                .parse()
                                .map_err(|_| DexError::Parse("Invalid L2 ask price".into()))?,
                        ),
                        qty: qty(level
                            .sz
                            .parse()
                            .map_err(|_| DexError::Parse("Invalid L2 ask quantity".into()))?),
                        n: level.n,
                    })
                })
                .collect();
            let asks = asks?;

            let orderbook = OrderBook {
                coin: book.coin,
                ts: book.time,
                bids,
                asks,
            };

            Ok(Some(StreamEvent::L2(orderbook)))
        } else {
            Ok(None)
        }
    }

    fn parse_orders(val: &Value) -> Result<Option<StreamEvent>, DexError> {
        if let Ok(order_updates) = serde_json::from_value::<Vec<OrderUpdate>>(val["data"].clone()) {
            if let Some(update) = order_updates.into_iter().next() {
                let order_event = OrderEvent {
                    coin: update.order.coin,
                    side: update.order.side,
                    limit_px: update.order.limit_px,
                    sz: update.order.sz,
                    oid: update.order.oid,
                    status: update.status,
                    timestamp: update.status_timestamp,
                    order_timestamp: update.order.timestamp,
                };
                return Ok(Some(StreamEvent::Order(order_event)));
            }
        }
        Ok(None)
    }

    fn parse_fills(val: &Value) -> Result<Option<StreamEvent>, DexError> {
        if let Ok(fills_data) = serde_json::from_value::<UserFillsData>(val["data"].clone()) {
            if let Some(fill) = fills_data.fills.into_iter().next() {
                let fill_event = FillEvent {
                    coin: fill.coin,
                    side: fill.side,
                    px: fill.px,
                    sz: fill.sz,
                    oid: fill.oid,
                    tid: fill.tid,
                    time: fill.time,
                    fee: fill.fee,
                    hash: fill.hash,
                    user: fills_data.user,
                };
                return Ok(Some(StreamEvent::Fill(fill_event)));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;

    #[test]
    fn test_subscription_message_construction() {
        // Test BBO subscription
        let bbo_sub = json!({
            "type": "bbo",
            "coin": "BTC"
        });
        assert_eq!(bbo_sub["type"], "bbo");
        assert_eq!(bbo_sub["coin"], "BTC");

        // Test trades subscription
        let trades_sub = json!({
            "type": "trades",
            "coin": "ETH"
        });
        assert_eq!(trades_sub["type"], "trades");
        assert_eq!(trades_sub["coin"], "ETH");

        // Test order updates subscription
        let orders_sub = json!({
            "type": "orderUpdates",
            "user": "0x1234567890abcdef1234567890abcdef12345678"
        });
        assert_eq!(orders_sub["type"], "orderUpdates");
        assert!(orders_sub["user"].as_str().unwrap().starts_with("0x"));

        // Test user fills subscription
        let fills_sub = json!({
            "type": "userFills",
            "user": "0x1234567890abcdef1234567890abcdef12345678"
        });
        assert_eq!(fills_sub["type"], "userFills");
        assert!(fills_sub["user"].as_str().unwrap().starts_with("0x"));
    }

    #[test]
    fn test_bbo_parsing() {
        let mock_message = json!({
            "data": {
                "coin": "BTC",
                "time": 1234567890,
                "bestBid": "50000.5",
                "bestAsk": "50001.2"
            }
        });

        let result = HlWs::<DummyTransport>::parse_bbo(&mock_message).unwrap();

        if let Some(StreamEvent::Bbo {
            coin,
            bid_px,
            ask_px,
            timestamp,
        }) = result
        {
            assert_eq!(coin, "BTC");
            assert_eq!(bid_px, 50000.5);
            assert_eq!(ask_px, 50001.2);
            assert_eq!(timestamp, 1234567890);
        } else {
            panic!("Expected BBO event");
        }
    }

    #[test]
    fn test_trades_parsing() {
        let mock_message = json!({
            "data": [{
                "coin": "BTC",
                "side": "B",
                "px": "50000.0",
                "sz": "0.001",
                "time": 1234567890,
                "hash": "abcdef123456",
                "tid": 12345
            }]
        });

        let result = HlWs::<DummyTransport>::parse_trades(&mock_message).unwrap();

        if let Some(StreamEvent::Trade(trade)) = result {
            assert_eq!(trade.id, "abcdef123456");
            assert_eq!(trade.ts, 1234567890);
            assert_eq!(trade.side, Side::Buy);
            assert_eq!(*trade.price, 50000.0);
            assert_eq!(*trade.qty, 0.001);
        } else {
            panic!("Expected Trade event");
        }
    }

    #[test]
    fn test_l2_book_parsing() {
        let mock_message = json!({
            "data": {
                "coin": "BTC",
                "time": 1234567890,
                "levels": [
                    [{"px": "50000.0", "sz": "0.5", "n": 1}, {"px": "49999.0", "sz": "1.0", "n": 2}], // bids
                    [{"px": "50001.0", "sz": "0.3", "n": 1}, {"px": "50002.0", "sz": "0.7", "n": 1}]  // asks
                ]
            }
        });

        let result = HlWs::<DummyTransport>::parse_l2_book(&mock_message).unwrap();

        if let Some(StreamEvent::L2(orderbook)) = result {
            assert_eq!(orderbook.coin, "BTC");
            assert_eq!(orderbook.ts, 1234567890);
            assert_eq!(orderbook.bids.len(), 2);
            assert_eq!(orderbook.asks.len(), 2);

            // Check bid levels
            assert_eq!(*orderbook.bids[0].price, 50000.0);
            assert_eq!(*orderbook.bids[0].qty, 0.5);

            // Check ask levels
            assert_eq!(*orderbook.asks[0].price, 50001.0);
            assert_eq!(*orderbook.asks[0].qty, 0.3);
        } else {
            panic!("Expected L2 orderbook event");
        }
    }

    #[test]
    fn test_order_updates_parsing() {
        let mock_message = json!({
            "data": [{
                "order": {
                    "coin": "BTC",
                    "side": "B",
                    "limitPx": "50000.0",
                    "sz": "0.001",
                    "oid": 12345,
                    "timestamp": 1234567890
                },
                "status": "open",
                "statusTimestamp": 1234567891
            }]
        });

        let result = HlWs::<DummyTransport>::parse_orders(&mock_message).unwrap();

        if let Some(StreamEvent::Order(order_event)) = result {
            assert_eq!(order_event.coin, "BTC");
            assert_eq!(order_event.side, "B");
            assert_eq!(order_event.limit_px, "50000.0");
            assert_eq!(order_event.sz, "0.001");
            assert_eq!(order_event.oid, 12345);
            assert_eq!(order_event.status, "open");
            assert_eq!(order_event.timestamp, 1234567891);
        } else {
            panic!("Expected Order event");
        }
    }

    #[test]
    fn test_user_fills_parsing() {
        let mock_message = json!({
            "data": {
                "user": "0x1234567890abcdef1234567890abcdef12345678",
                "fills": [{
                    "coin": "BTC",
                    "side": "B",
                    "px": "50000.0",
                    "sz": "0.001",
                    "oid": 12345,
                    "tid": 67890,
                    "time": 1234567890,
                    "fee": "0.50",
                    "hash": "abcdef123456"
                }]
            }
        });

        let result = HlWs::<DummyTransport>::parse_fills(&mock_message).unwrap();

        if let Some(StreamEvent::Fill(fill_event)) = result {
            assert_eq!(fill_event.coin, "BTC");
            assert_eq!(fill_event.side, "B");
            assert_eq!(fill_event.px, "50000.0");
            assert_eq!(fill_event.sz, "0.001");
            assert_eq!(fill_event.oid, 12345);
            assert_eq!(fill_event.tid, 67890);
            assert_eq!(fill_event.time, 1234567890);
            assert_eq!(fill_event.fee, "0.50");
            assert_eq!(fill_event.hash, "abcdef123456");
        } else {
            panic!("Expected Fill event");
        }
    }

    #[test]
    fn test_invalid_message_handling() {
        // Test empty data
        let empty_message = json!({});
        let result = HlWs::<DummyTransport>::parse_bbo(&empty_message).unwrap();
        assert!(result.is_none());

        // Test malformed data
        let malformed_message = json!({
            "data": {
                "coin": "BTC"
                // missing required fields
            }
        });
        let result = HlWs::<DummyTransport>::parse_bbo(&malformed_message).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_subscription_response_filtering() {
        let subscription_response = json!({
            "method": "subscriptionResponse",
            "subscription": { "type": "bbo", "coin": "BTC" }
        });

        // This should be filtered out in handle_message
        assert_eq!(subscription_response["method"], "subscriptionResponse");
    }

    // Dummy transport for testing parsing functions
    #[derive(Clone)]
    struct DummyTransport;

    #[async_trait]
    impl WsTransport for DummyTransport {
        async fn connect(
            &self,
            _url: &str,
        ) -> Result<Box<dyn dex_rs_core::ws::WsConnection + Send + Sync + Unpin>, DexError>
        {
            Err(DexError::Unsupported("DummyTransport"))
        }
    }
}
