use dex_rs::prelude::*;
use std::collections::VecDeque;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> DexResult<()> {
    env_logger::init();

    println!("🔗 Connecting to Hyperliquid testnet...");
    let hl = Hyperliquid::builder().testnet().connect().await?;

    let coin = "BTC";
    println!(
        "💹 Subscribing to live BBO (Best Bid/Offer) for {}...",
        coin
    );
    println!("Press Ctrl+C to exit\n");

    let (tx, mut rx) = mpsc::unbounded_channel();
    hl.subscribe(StreamKind::Bbo, Some(coin), tx).await?;

    let mut update_count = 0;
    let mut price_history: VecDeque<f64> = VecDeque::with_capacity(100);
    let mut last_mid_price = 0.0;

    println!("{:-<90}", "");
    println!(
        "{:<12} {:<15} {:<15} {:<15} {:<10} {:<15}",
        "Time", "Bid", "Ask", "Mid", "Spread", "Change"
    );
    println!("{:-<90}", "");

    loop {
        match rx.recv().await {
            Some(StreamEvent::Bbo {
                coin: _coin,
                bid_px,
                ask_px,
                timestamp,
            }) => {
                update_count += 1;

                let mid_price = (bid_px + ask_px) / 2.0;
                let spread = ask_px - bid_px;
                let spread_bps = (spread / mid_price) * 10000.0;

                // Calculate price change
                let change = if last_mid_price > 0.0 {
                    mid_price - last_mid_price
                } else {
                    0.0
                };

                let change_emoji = if change > 0.0 {
                    "🟢"
                } else if change < 0.0 {
                    "🔴"
                } else {
                    "⚪"
                };

                // Store price for trend analysis
                price_history.push_back(mid_price);
                if price_history.len() > 100 {
                    price_history.pop_front();
                }

                let time = chrono::DateTime::from_timestamp_millis(timestamp as i64)
                    .unwrap_or_default()
                    .format("%H:%M:%S");

                println!(
                    "{:<12} ${:<14.2} ${:<14.2} ${:<14.2} {:<10.1} {} ${:+.2}",
                    time, bid_px, ask_px, mid_price, spread_bps, change_emoji, change
                );

                last_mid_price = mid_price;

                // Show periodic statistics
                if update_count % 50 == 0 {
                    show_statistics(&price_history, update_count);
                }
            }
            Some(_) => {
                // Ignore other event types
            }
            None => {
                println!("❌ WebSocket connection closed");
                break;
            }
        }
    }

    if !price_history.is_empty() {
        show_statistics(&price_history, update_count);
    }

    Ok(())
}

fn show_statistics(price_history: &VecDeque<f64>, update_count: usize) {
    if price_history.len() < 2 {
        return;
    }

    let current_price = *price_history.back().unwrap();
    let oldest_price = *price_history.front().unwrap();
    let total_change = current_price - oldest_price;
    let change_pct = (total_change / oldest_price) * 100.0;

    let min_price = price_history.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_price = price_history
        .iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let avg_price = price_history.iter().sum::<f64>() / price_history.len() as f64;

    println!("\n📊 Statistics (last {} updates):", price_history.len());
    println!("   Current: ${:.2}", current_price);
    println!("   Average: ${:.2}", avg_price);
    println!("   Range: ${:.2} - ${:.2}", min_price, max_price);
    println!("   Change: ${:+.2} ({:+.2}%)", total_change, change_pct);
    println!("   Total updates: {}", update_count);
    println!("{:-<90}", "");
}
