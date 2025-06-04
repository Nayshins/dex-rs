use dex_rs::prelude::*;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> DexResult<()> {
    env_logger::init();
    
    println!("🔗 Connecting to Hyperliquid testnet...");
    let hl = Hyperliquid::builder().testnet().connect().await?;
    
    let coin = "BTC";
    let depth_display = 5; // Show top 5 levels
    
    println!("📖 Subscribing to live order book updates for {}...", coin);
    println!("Displaying top {} levels on each side", depth_display);
    println!("Press Ctrl+C to exit\n");
    
    let (tx, mut rx) = mpsc::unbounded_channel();
    hl.subscribe(StreamKind::L2Book, Some(coin), tx).await?;
    
    let mut update_count = 0;
    
    loop {
        match timeout(Duration::from_secs(60), rx.recv()).await {
            Ok(Some(event)) => {
                if let StreamEvent::L2(orderbook) = event {
                    update_count += 1;
                    
                    // Clear screen and show updated order book
                    print!("\x1B[2J\x1B[1;1H"); // Clear screen and move cursor to top
                    
                    let timestamp = chrono::DateTime::from_timestamp_millis(orderbook.ts as i64)
                        .unwrap_or_default()
                        .format("%H:%M:%S UTC");
                    
                    println!("📖 Order Book: {} | Update #{} | {}", orderbook.coin, update_count, timestamp);
                    
                    // Calculate mid price and spread
                    let best_bid = orderbook.bids.first().map(|b| b.price.into_inner()).unwrap_or(0.0);
                    let best_ask = orderbook.asks.first().map(|a| a.price.into_inner()).unwrap_or(0.0);
                    
                    if best_bid > 0.0 && best_ask > 0.0 {
                        let mid_price = (best_bid + best_ask) / 2.0;
                        let spread = best_ask - best_bid;
                        let spread_bps = (spread / mid_price) * 10000.0;
                        println!("💰 Mid: ${:.2} | Spread: ${:.2} ({:.1} bps)", mid_price, spread, spread_bps);
                    }
                    
                    println!("\n{:-<75}", "");
                    println!("{:^37} | {:^37}", "BIDS", "ASKS");
                    println!("{:-<37}+{:-<37}", "", "");
                    println!("{:<12} {:<15} {:<6} | {:<6} {:<15} {:<12}", "Quantity", "Price", "Orders", "Orders", "Price", "Quantity");
                    println!("{:-<37}+{:-<37}", "", "");
                    
                    for i in 0..depth_display {
                        let bid_str = if i < orderbook.bids.len() {
                            let bid = &orderbook.bids[i];
                            format!("{:<12.6} ${:<14.2} {:<6}", 
                                   bid.qty.into_inner(), 
                                   bid.price.into_inner(),
                                   bid.n)
                        } else {
                            format!("{:<37}", "")
                        };
                        
                        let ask_str = if i < orderbook.asks.len() {
                            let ask = &orderbook.asks[i];
                            format!("{:<6} ${:<14.2} {:<12.6}", 
                                   ask.n,
                                   ask.price.into_inner(),
                                   ask.qty.into_inner())
                        } else {
                            format!("{:<37}", "")
                        };
                        
                        println!("{} | {}", bid_str, ask_str);
                    }
                    
                    // Show cumulative volumes at top levels
                    let total_bid_vol: f64 = orderbook.bids.iter()
                        .take(depth_display)
                        .map(|b| b.qty.into_inner())
                        .sum();
                    let total_ask_vol: f64 = orderbook.asks.iter()
                        .take(depth_display)
                        .map(|a| a.qty.into_inner())
                        .sum();
                    
                    println!("{:-<37}+{:-<37}", "", "");
                    println!("Sum: {:<30.6} | Sum: {:<30.6}", total_bid_vol, total_ask_vol);
                    
                    println!("\n🔄 Waiting for next update...");
                }
            }
            Ok(None) => {
                println!("❌ WebSocket connection closed");
                break;
            }
            Err(_) => {
                println!("\n⏰ No updates received in the last 60 seconds");
                println!("🔄 Still listening...");
            }
        }
    }
    
    println!("\n📊 Total updates received: {}", update_count);
    
    Ok(())
}