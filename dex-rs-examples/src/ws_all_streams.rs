use dex_rs::prelude::*;
use tokio::sync::mpsc;
use std::env;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> DexResult<()> {
    env_logger::init();
    
    println!("🔗 Testing simultaneous WebSocket subscriptions...");
    println!("This example subscribes to ALL stream types at once to verify they work together.");
    
    // Check if we have authentication for user streams
    let has_auth = env::var("HYPERLIQUID_PRIVATE_KEY").is_ok();
    
    let hl = if has_auth {
        println!("🔐 Found private key - will test authenticated streams too");
        let private_key = env::var("HYPERLIQUID_PRIVATE_KEY").unwrap();
        Hyperliquid::builder()
            .testnet()
            .wallet_hex(&private_key)
            .connect()
            .await?
    } else {
        println!("⚠️ No private key found - testing public streams only");
        println!("💡 Set HYPERLIQUID_PRIVATE_KEY to test authenticated streams");
        Hyperliquid::builder().testnet().connect().await?
    };
    
    let coins = vec!["BTC", "ETH"]; // Test with multiple assets
    
    // Create channels for each stream type
    let (trades_tx, mut trades_rx) = mpsc::unbounded_channel();
    let (bbo_tx, mut bbo_rx) = mpsc::unbounded_channel();
    let (l2_tx, mut l2_rx) = mpsc::unbounded_channel();
    let (orders_tx, mut orders_rx) = mpsc::unbounded_channel();
    let (fills_tx, mut fills_rx) = mpsc::unbounded_channel();
    
    // Statistics tracking
    let mut stats = StreamStats::new();
    let start_time = tokio::time::Instant::now();
    
    println!("\n📡 Subscribing to streams...");
    
    // Subscribe to public streams for each coin
    for coin in &coins {
        println!("  📈 Trades for {}", coin);
        hl.subscribe(StreamKind::Trades, Some(coin), trades_tx.clone()).await?;
        
        println!("  💹 BBO for {}", coin);
        hl.subscribe(StreamKind::Bbo, Some(coin), bbo_tx.clone()).await?;
        
        println!("  📖 L2 Book for {}", coin);
        hl.subscribe(StreamKind::L2Book, Some(coin), l2_tx.clone()).await?;
    }
    
    // Subscribe to authenticated streams if available
    if has_auth {
        println!("  📋 Order updates");
        hl.subscribe(StreamKind::Orders, None, orders_tx).await?;
        
        println!("  💵 Fill updates");
        hl.subscribe(StreamKind::Fills, None, fills_tx).await?;
    }
    
    println!("\n✅ All subscriptions completed successfully!");
    println!("🎯 Listening for events... (Press Ctrl+C to exit)\n");
    
    // Display header
    print_header();
    
    loop {
        tokio::select! {
            Some(event) = trades_rx.recv() => {
                if let StreamEvent::Trade(trade) = event {
                    stats.increment("Trades");
                    print_trade_event(&trade);
                }
            }
            
            Some(event) = bbo_rx.recv() => {
                if let StreamEvent::Bbo { coin, bid_px, ask_px, timestamp } = event {
                    stats.increment("BBO");
                    print_bbo_event(&coin, bid_px, ask_px, timestamp);
                }
            }
            
            Some(event) = l2_rx.recv() => {
                if let StreamEvent::L2(orderbook) = event {
                    stats.increment("L2Book");
                    print_l2_event(&orderbook);
                }
            }
            
            Some(event) = orders_rx.recv() => {
                if let StreamEvent::Order(order) = event {
                    stats.increment("Orders");
                    print_order_event(&order.coin, &order.side, &order.limit_px, &order.sz, order.oid, &order.status, order.timestamp);
                }
            }
            
            Some(event) = fills_rx.recv() => {
                if let StreamEvent::Fill(fill) = event {
                    stats.increment("Fills");
                    print_fill_event(&fill.coin, &fill.side, &fill.px, &fill.sz, fill.oid, fill.tid, fill.time, &fill.fee);
                }
            }
            
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                print_statistics(&stats, start_time.elapsed());
            }
        }
    }
}

#[derive(Default)]
struct StreamStats {
    counts: HashMap<String, u64>,
    last_events: HashMap<String, String>,
}

impl StreamStats {
    fn new() -> Self {
        Self::default()
    }
    
    fn increment(&mut self, stream_type: &str) {
        *self.counts.entry(stream_type.to_string()).or_insert(0) += 1;
        let now = chrono::Utc::now().format("%H:%M:%S").to_string();
        self.last_events.insert(stream_type.to_string(), now);
    }
}

fn print_header() {
    println!("{:=<100}", "");
    println!("{:^100}", "LIVE WEBSOCKET STREAM MONITOR");
    println!("{:=<100}", "");
    println!("{:<12} {:<8} {:<10} {:<15} {:<30} {:<15}", "Time", "Type", "Asset", "Key Info", "Details", "Status");
    println!("{:-<100}", "");
}

fn print_trade_event(trade: &Trade) {
    let time = chrono::DateTime::from_timestamp_millis(trade.ts as i64)
        .unwrap_or_default()
        .format("%H:%M:%S");
    
    let side_display = match trade.side {
        Side::Buy => "🟢 BUY",
        Side::Sell => "🔴 SELL",
    };
    
    println!(
        "{:<12} {:<8} {:<10} {:<15} {:<30} {:<15}",
        time,
        "TRADE",
        trade.coin,
        format!("${:.2}", trade.price.into_inner()),
        format!("{} {:.6}", side_display, trade.qty.into_inner()),
        format!("TID:{}", trade.tid)
    );
}

fn print_bbo_event(coin: &str, bid_px: f64, ask_px: f64, timestamp: u64) {
    let time = chrono::DateTime::from_timestamp_millis(timestamp as i64)
        .unwrap_or_default()
        .format("%H:%M:%S");
    
    let mid_price = (bid_px + ask_px) / 2.0;
    let spread = ask_px - bid_px;
    
    println!(
        "{:<12} {:<8} {:<10} {:<15} {:<30} {:<15}",
        time,
        "BBO",
        coin,
        format!("${:.2}", mid_price),
        format!("Bid:{:.2} Ask:{:.2}", bid_px, ask_px),
        format!("Spr:${:.2}", spread)
    );
}

fn print_l2_event(orderbook: &OrderBook) {
    let time = chrono::DateTime::from_timestamp_millis(orderbook.ts as i64)
        .unwrap_or_default()
        .format("%H:%M:%S");
    
    let bid_levels = orderbook.bids.len();
    let ask_levels = orderbook.asks.len();
    let total_bid_qty: f64 = orderbook.bids.iter().map(|b| b.qty.into_inner()).sum();
    let total_ask_qty: f64 = orderbook.asks.iter().map(|a| a.qty.into_inner()).sum();
    
    println!(
        "{:<12} {:<8} {:<10} {:<15} {:<30} {:<15}",
        time,
        "L2BOOK",
        orderbook.coin,
        format!("{}B/{}A", bid_levels, ask_levels),
        format!("BidVol:{:.3} AskVol:{:.3}", total_bid_qty, total_ask_qty),
        "UPDATED"
    );
}

fn print_order_event(coin: &str, side: &str, limit_px: &str, sz: &str, oid: u64, status: &str, timestamp: u64) {
    let time = chrono::DateTime::from_timestamp_millis(timestamp as i64)
        .unwrap_or_default()
        .format("%H:%M:%S");
    
    let status_emoji = match status {
        "open" => "🟡",
        "filled" => "🟢",
        "canceled" => "🔴",
        "rejected" => "❌",
        _ => "⚪",
    };
    
    println!(
        "{:<12} {:<8} {:<10} {:<15} {:<30} {:<15}",
        time,
        "ORDER",
        coin,
        format!("${}", limit_px),
        format!("{} {} {}", side.to_uppercase(), sz, status_emoji),
        format!("OID:{}", oid)
    );
}

fn print_fill_event(coin: &str, side: &str, px: &str, sz: &str, _oid: u64, tid: u64, time: u64, fee: &str) {
    let formatted_time = chrono::DateTime::from_timestamp_millis(time as i64)
        .unwrap_or_default()
        .format("%H:%M:%S");
    
    let side_display = match side {
        "B" => "🟢 BUY",
        "A" => "🔴 SELL",
        _ => side,
    };
    
    println!(
        "{:<12} {:<8} {:<10} {:<15} {:<30} {:<15}",
        formatted_time,
        "FILL",
        coin,
        format!("${}", px),
        format!("{} {} Fee:${}", side_display, sz, fee),
        format!("TID:{}", tid)
    );
}

fn print_statistics(stats: &StreamStats, elapsed: tokio::time::Duration) {
    println!("\n📊 STREAM STATISTICS ({:.0}s elapsed):", elapsed.as_secs());
    println!("{:-<60}", "");
    
    let total_events: u64 = stats.counts.values().sum();
    
    for (stream, count) in &stats.counts {
        let rate = if elapsed.as_secs() > 0 {
            *count as f64 / elapsed.as_secs() as f64
        } else {
            0.0
        };
        
        let last_event = stats.last_events.get(stream)
            .map(|s| s.as_str())
            .unwrap_or("Never");
        
        println!(
            "{:<12} {:<8} events ({:>6.2}/s) - Last: {}",
            stream, count, rate, last_event
        );
    }
    
    println!("{:-<60}", "");
    println!("TOTAL:       {} events ({:.2}/s)", 
             total_events, 
             total_events as f64 / elapsed.as_secs() as f64);
    
    // Health check
    if total_events == 0 {
        println!("⚠️ WARNING: No events received yet");
    } else if stats.counts.len() >= 3 {
        println!("✅ HEALTHY: Multiple stream types active");
    } else {
        println!("🟡 PARTIAL: Some stream types may be inactive");
    }
    
    println!("{:=<100}", "");
    println!("{:^100}", "CONTINUING STREAM MONITOR...");
    println!("{:=<100}", "");
    println!("{:<12} {:<8} {:<10} {:<15} {:<30} {:<15}", "Time", "Type", "Asset", "Key Info", "Details", "Status");
    println!("{:-<100}", "");
}