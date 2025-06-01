use dex_rs_core::traits::PerpDex;
use dex_rs_hyperliquid::Hyperliquid;
use dex_rs_types::{price, qty, OrderReq, Tif};

#[tokio::test]
#[ignore] // set HL_PK env var & remove to run
async fn fetch_and_trade() {
    let hl = Hyperliquid::builder()
        .wallet_env("HL_PK")
        .testnet()
        .connect()
        .await
        .unwrap();

    let t = hl.trades("BTC", 3).await.unwrap();
    assert_eq!(t.len(), 3);

    let ob = hl.orderbook("BTC", 2).await.unwrap();
    assert_eq!(ob.bids.len(), 2);

    let _id = hl
        .place_order(OrderReq {
            coin: "BTC".into(),
            is_buy: true,
            px: price(100.0),
            qty: qty(0.0001),
            tif: Tif::Ioc,
            reduce_only: false,
        })
        .await
        .unwrap();
}

#[tokio::test]
#[ignore] // live API calls
async fn test_market_data() {
    let hl = Hyperliquid::builder().testnet().connect().await.unwrap();

    // Test trades fetch
    let trades = hl.trades("BTC", 5).await.unwrap();
    assert!(!trades.is_empty());
    assert!(trades.len() <= 5);

    // Test orderbook fetch
    let ob = hl.orderbook("BTC", 3).await.unwrap();
    assert_eq!(ob.coin, "BTC");
    assert!(!ob.bids.is_empty());
    assert!(!ob.asks.is_empty());
    assert!(ob.bids.len() <= 3);
    assert!(ob.asks.len() <= 3);
}

#[tokio::test]
#[ignore] // requires valid wallet
async fn test_websocket_streams() {
    use dex_rs_core::traits::{StreamEvent, StreamKind};
    use tokio::sync::mpsc;

    let hl = Hyperliquid::builder()
        .wallet_env("HL_PK")
        .testnet()
        .connect()
        .await
        .unwrap();

    let (tx, mut rx) = mpsc::unbounded_channel();

    // Subscribe to BTC trades
    hl.subscribe(StreamKind::Trades, Some("BTC"), tx.clone())
        .await
        .unwrap();

    // Wait for some messages
    let mut count = 0;
    while count < 3 {
        if let Some(event) = rx.recv().await {
            match event {
                StreamEvent::Trade(trade) => {
                    assert_eq!(trade.id.len() > 0, true);
                    count += 1;
                }
                _ => {}
            }
        }
    }
}
