use dex_rs::prelude::*;
use tokio::sync::mpsc;
use std::env;

#[tokio::main]
async fn main() -> DexResult<()> {
    env_logger::init();
    
    // Get private key from environment variable
    let private_key = env::var("HYPERLIQUID_PRIVATE_KEY")
        .expect("Please set HYPERLIQUID_PRIVATE_KEY environment variable");
    
    println!("🔗 Connecting to Hyperliquid testnet with authentication...");
    let hl = Hyperliquid::builder()
        .testnet()
        .credentials(&private_key)
        .connect()
        .await?;
    
    println!("👤 Subscribing to user events (orders and fills)...");
    println!("Press Ctrl+C to exit\n");
    
    // Subscribe to both order updates and fills
    let (order_tx, mut order_rx) = mpsc::unbounded_channel();
    let (fill_tx, mut fill_rx) = mpsc::unbounded_channel();
    
    hl.subscribe(StreamKind::Orders, None, order_tx).await?;
    hl.subscribe(StreamKind::Fills, None, fill_tx).await?;
    
    println!("🎯 Listening for order updates and fills...");
    println!("{:=<80}", "");
    
    loop {
        tokio::select! {
            order_event = order_rx.recv() => {
                match order_event {
                    Some(StreamEvent::Order(order)) => {
                        let time = chrono::DateTime::from_timestamp_millis(order.timestamp as i64)
                            .unwrap_or_default()
                            .format("%H:%M:%S");
                        
                        let status_emoji = match order.status.as_str() {
                            "open" => "🟡",
                            "filled" => "🟢",
                            "canceled" => "🔴",
                            "rejected" => "❌",
                            _ => "⚪",
                        };
                        
                        println!("📋 ORDER UPDATE [{}]", time);
                        println!("   {} Status: {}", status_emoji, order.status.to_uppercase());
                        println!("   🪙 Asset: {}", order.coin);
                        println!("   📊 Side: {}", order.side.to_uppercase());
                        println!("   💰 Price: ${}", order.limit_px);
                        println!("   📏 Size: {}", order.sz);
                        println!("   🆔 Order ID: {}", order.oid);
                        
                        let order_time = chrono::DateTime::from_timestamp_millis(order.order_timestamp as i64)
                            .unwrap_or_default()
                            .format("%H:%M:%S");
                        println!("   🕐 Order Time: {}", order_time);
                        println!("{:-<50}", "");
                    }
                    Some(_) => {} // Ignore other events
                    None => {
                        println!("❌ Order WebSocket connection closed");
                        break;
                    }
                }
            }
            
            fill_event = fill_rx.recv() => {
                match fill_event {
                    Some(StreamEvent::Fill(fill)) => {
                        let time = chrono::DateTime::from_timestamp_millis(fill.time as i64)
                            .unwrap_or_default()
                            .format("%H:%M:%S");
                        
                        let side_emoji = match fill.side.as_str() {
                            "B" => "🟢 BUY",
                            "A" => "🔴 SELL",
                            _ => &fill.side,
                        };
                        
                        println!("💵 FILL EXECUTED [{}]", time);
                        println!("   {} {}", side_emoji, fill.coin);
                        println!("   💰 Fill Price: ${}", fill.px);
                        println!("   📏 Fill Size: {}", fill.sz);
                        println!("   💸 Fee: ${}", fill.fee);
                        println!("   🆔 Order ID: {}", fill.oid);
                        println!("   🔗 Trade ID: {}", fill.tid);
                        println!("   🏦 Hash: {}", fill.hash);
                        println!("   👤 User: {}", fill.user);
                        println!("{:-<50}", "");
                    }
                    Some(_) => {} // Ignore other events
                    None => {
                        println!("❌ Fill WebSocket connection closed");
                        break;
                    }
                }
            }
        }
    }
    
    Ok(())
}