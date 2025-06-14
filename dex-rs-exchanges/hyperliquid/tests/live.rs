use dex_rs_core::traits::PerpDex;
use dex_rs_hyperliquid::Hyperliquid;
use dex_rs_types::{price, qty, OrderReq, Tif};

#[tokio::test]
#[ignore] // set HL_PK env var & remove to run
async fn fetch_and_trade() {
    let hl = Hyperliquid::builder()
        .private_key_env("HL_PK")
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
        .private_key_env("HL_PK")
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

#[tokio::test]
#[ignore] // live API calls
async fn test_new_market_data_endpoints() {
    let hl = Hyperliquid::builder().testnet().connect().await.unwrap();

    // Test all mids
    let all_mids = hl.all_mids().await.unwrap();
    assert!(!all_mids.mids.is_empty());
    println!(
        "All mids: {:?}",
        all_mids.mids.keys().take(5).collect::<Vec<_>>()
    );

    // Test meta
    let meta = hl.meta().await.unwrap();
    assert!(!meta.assets.is_empty());
    println!("Assets count: {}", meta.assets.len());

    // Test meta and asset contexts
    let meta_and_ctxs = hl.meta_and_asset_ctxs().await.unwrap();
    assert!(!meta_and_ctxs.meta.assets.is_empty());
    assert!(!meta_and_ctxs.asset_ctxs.is_empty());
    println!("Asset contexts count: {}", meta_and_ctxs.asset_ctxs.len());

    // Test funding history
    let end_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let start_time = end_time - (24 * 60 * 60 * 1000); // 24 hours ago

    let funding_history = hl
        .funding_history("BTC", start_time, Some(end_time))
        .await
        .unwrap();
    println!("Funding history entries: {}", funding_history.len());

    // Test candle snapshot
    let candles = hl
        .candle_snapshot("BTC", "1h", start_time, end_time)
        .await
        .unwrap();
    assert!(!candles.0.is_empty());
    println!("Candles count: {}", candles.0.len());
}

#[tokio::test]
#[ignore] // requires valid wallet and live API calls
async fn test_authenticated_endpoints() {
    let hl = Hyperliquid::builder()
        .private_key_env("HL_PK")
        .testnet()
        .connect()
        .await
        .unwrap();

    // Test user state
    let user_state = hl.user_state().await.unwrap();
    println!(
        "Account value: {}",
        user_state.cross_margin_summary.account_value
    );
    println!("Positions count: {}", user_state.asset_positions.len());

    // Test open orders
    let open_orders = hl.open_orders().await.unwrap();
    println!("Open orders count: {}", open_orders.len());

    // Test user fills
    let user_fills = hl.user_fills().await.unwrap();
    println!("Total fills: {}", user_fills.len());

    // Test user fills by time (last 24 hours)
    let end_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let start_time = end_time - (24 * 60 * 60 * 1000);

    let recent_fills = hl
        .user_fills_by_time(start_time, Some(end_time))
        .await
        .unwrap();
    println!("Recent fills: {}", recent_fills.len());

    // Test user fees
    let user_fees = hl.user_fees().await.unwrap();
    println!("Total fees: {}", user_fees.total_fees);

    // Test user funding (last 7 days)
    let start_time_week = end_time - (7 * 24 * 60 * 60 * 1000);
    let user_funding = hl
        .user_funding(start_time_week, Some(end_time))
        .await
        .unwrap();
    println!("Funding payments: {}", user_funding.delta.len());
}

#[tokio::test]
#[ignore] // live API calls
async fn test_spot_endpoints() {
    let hl = Hyperliquid::builder().testnet().connect().await.unwrap();

    // Test spot meta
    let spot_meta = hl.spot_meta().await.unwrap();
    assert!(!spot_meta.tokens.is_empty());
    println!("Spot tokens count: {}", spot_meta.tokens.len());

    // Test spot meta and asset contexts
    let spot_meta_and_ctxs = hl.spot_meta_and_asset_ctxs().await.unwrap();
    assert!(!spot_meta_and_ctxs.meta.tokens.is_empty());
    println!(
        "Spot asset contexts count: {}",
        spot_meta_and_ctxs.asset_ctxs.len()
    );
}
