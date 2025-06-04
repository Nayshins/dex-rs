use dex_rs::prelude::*;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> DexResult<()> {
    env_logger::init();
    
    println!("🔗 Connecting to Hyperliquid testnet...");
    let hl = Hyperliquid::builder().testnet().connect().await?;
    
    let coin = "BTC";
    println!("📈 Subscribing to live trades for {}...", coin);
    println!("Press Ctrl+C to exit\n");
    
    let (tx, mut rx) = mpsc::unbounded_channel();
    hl.subscribe(StreamKind::Trades, Some(coin), tx).await?;
    
    let mut trade_count = 0;
    let start_time = tokio::time::Instant::now();
    
    println!("{:-<80}", "");
    println!("{:<12} {:<8} {:<15} {:<15} {:<15}", "Time", "Side", "Price", "Quantity", "Trade ID");
    println!("{:-<80}", "");
    
    loop {
        // Use timeout to periodically show statistics
        match timeout(Duration::from_secs(30), rx.recv()).await {
            Ok(Some(event)) => {
                if let StreamEvent::Trade(trade) = event {
                    trade_count += 1;
                    
                    let time = chrono::DateTime::from_timestamp_millis(trade.ts as i64)
                        .unwrap_or_default()
                        .format("%H:%M:%S");
                    
                    let side_display = match trade.side {
                        Side::Buy => "🟢 BUY",
                        Side::Sell => "🔴 SELL",
                    };
                    
                    println!(
                        "{:<12} {:<8} ${:<14.2} {:<15.6} {}",
                        time,
                        side_display,
                        trade.price.into_inner(),
                        trade.qty.into_inner(),
                        trade.tid
                    );
                }
            }
            Ok(None) => {
                println!("❌ WebSocket connection closed");
                break;
            }
            Err(_) => {
                // Timeout occurred, show statistics
                let elapsed = start_time.elapsed().as_secs();
                let rate = if elapsed > 0 { trade_count as f64 / elapsed as f64 } else { 0.0 };
                println!("\n📊 Stats: {} trades in {}s ({:.1} trades/sec)", 
                         trade_count, elapsed, rate);
                println!("🔄 Still listening for trades...\n");
            }
        }
    }
    
    let elapsed = start_time.elapsed().as_secs();
    println!("\n📈 Final stats: {} trades received in {}s", trade_count, elapsed);
    
    Ok(())
}