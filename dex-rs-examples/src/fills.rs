use dex_rs::prelude::*;
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
    
    println!("📝 Fetching recent fill history...");
    let fills = hl.user_fills().await?;
    
    if fills.is_empty() {
        println!("❌ No fills found in your trading history.");
        println!("💡 Execute some trades first to see fill history.");
        
        // Also try fetching fills by time (last 7 days)
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let seven_days_ago = now - (7 * 24 * 60 * 60 * 1000);
        
        println!("\n🔍 Checking last 7 days for any fills...");
        match hl.user_fills_by_time(seven_days_ago, Some(now)).await {
            Ok(time_fills) => {
                if time_fills.is_empty() {
                    println!("❌ No fills found in the last 7 days either.");
                } else {
                    println!("✅ Found {} fills in the last 7 days", time_fills.len());
                }
            }
            Err(e) => {
                println!("⚠️ Could not fetch fills by time: {}", e);
            }
        }
        return Ok(());
    }
    
    println!("\n💵 Fill History ({} total fills):", fills.len());
    println!("{:=<100}", "");
    
    let mut total_volume = 0.0;
    let mut total_fees = 0.0;
    let mut asset_stats: std::collections::HashMap<String, (f64, f64, usize)> = std::collections::HashMap::new();
    
    println!("{:<15} {:<8} {:<15} {:<12} {:<12} {:<12} {:<15}", 
             "Time", "Side", "Asset", "Price", "Size", "Fee", "Trade ID");
    println!("{:-<100}", "");
    
    for fill in fills.iter().take(50) {  // Show last 50 fills
        let datetime = chrono::DateTime::from_timestamp_millis(fill.time as i64)
            .unwrap_or_default()
            .format("%m-%d %H:%M");
        
        let side_emoji = match fill.side.as_str() {
            "B" => "🟢 BUY",
            "A" => "🔴 SELL",
            _ => &fill.side,
        };
        
        let price: f64 = fill.px.parse().unwrap_or(0.0);
        let size: f64 = fill.sz.parse().unwrap_or(0.0);
        let fee: f64 = fill.fee.parse().unwrap_or(0.0);
        
        let volume = price * size;
        total_volume += volume;
        total_fees += fee;
        
        // Update asset statistics
        let stats = asset_stats.entry(fill.coin.clone()).or_insert((0.0, 0.0, 0));
        stats.0 += volume; // total volume
        stats.1 += fee;    // total fees
        stats.2 += 1;      // fill count
        
        println!(
            "{:<15} {:<8} {:<15} ${:<11.2} {:<12.6} ${:<11.4} {}",
            datetime,
            side_emoji,
            fill.coin,
            price,
            size,
            fee,
            fill.tid
        );
        
        // Show additional details for recent fills
        if fills.len() <= 10 {
            println!("              Order ID: {} | Hash: {}", fill.oid, fill.hash);
            if fill.liquidation.unwrap_or(false) {
                println!("              ⚠️ LIQUIDATION");
            }
            if !fill.closed_pnl.is_empty() && fill.closed_pnl != "0" {
                let closed_pnl: f64 = fill.closed_pnl.parse().unwrap_or(0.0);
                let pnl_emoji = if closed_pnl > 0.0 { "🟢" } else { "🔴" };
                println!("              Closed PnL: {} ${:.2}", pnl_emoji, closed_pnl);
            }
            println!();
        }
    }
    
    if fills.len() > 50 {
        println!("... and {} more fills (showing most recent 50)", fills.len() - 50);
    }
    
    // Summary statistics
    println!("\n📊 Trading Statistics:");
    println!("{:=<60}", "");
    println!("Total Volume Traded: ${:.2}", total_volume);
    println!("Total Fees Paid: ${:.4}", total_fees);
    
    if total_volume > 0.0 {
        let fee_rate = (total_fees / total_volume) * 10000.0; // in basis points
        println!("Average Fee Rate: {:.1} bps", fee_rate);
    }
    
    // Asset breakdown
    if asset_stats.len() > 1 {
        println!("\n📈 By Asset:");
        let mut sorted_assets: Vec<_> = asset_stats.iter().collect();
        sorted_assets.sort_by(|a, b| b.1.0.partial_cmp(&a.1.0).unwrap_or(std::cmp::Ordering::Equal));
        
        println!("{:-<60}", "");
        println!("{:<10} {:<12} {:<15} {:<10}", "Asset", "Volume", "Fees", "Fills");
        println!("{:-<60}", "");
        
        for (asset, (volume, fees, count)) in sorted_assets {
            println!("{:<10} ${:<11.2} ${:<14.4} {:<10}", asset, volume, fees, count);
        }
    }
    
    // Recent trading activity
    if !fills.is_empty() {
        let latest_fill = &fills[0];
        let latest_time = chrono::DateTime::from_timestamp_millis(latest_fill.time as i64)
            .unwrap_or_default()
            .format("%Y-%m-%d %H:%M:%S UTC");
        
        println!("\n🕐 Latest Activity:");
        println!("Last fill: {} at {}", latest_fill.coin, latest_time);
        
        // Check if user has been liquidated recently
        let recent_liquidations = fills.iter()
            .take(10)
            .filter(|f| f.liquidation.unwrap_or(false))
            .count();
        
        if recent_liquidations > 0 {
            println!("⚠️ Warning: {} recent liquidation(s) detected", recent_liquidations);
        }
    }
    
    Ok(())
}