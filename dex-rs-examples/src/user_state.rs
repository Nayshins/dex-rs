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
    
    println!("👤 Fetching user account state...");
    let user_state = hl.user_state().await?;
    
    let timestamp = chrono::DateTime::from_timestamp_millis(user_state.time as i64)
        .unwrap_or_default()
        .format("%Y-%m-%d %H:%M:%S UTC");
    
    println!("\n🏦 Account Overview (as of {})", timestamp);
    println!("{:=<80}", "");
    
    // Account summary
    let account_value: f64 = user_state.cross_margin_summary.account_value.parse().unwrap_or(0.0);
    let margin_used: f64 = user_state.cross_margin_summary.total_margin_used.parse().unwrap_or(0.0);
    let ntl_pos: f64 = user_state.cross_margin_summary.total_ntl_pos.parse().unwrap_or(0.0);
    let raw_usd: f64 = user_state.cross_margin_summary.total_raw_usd.parse().unwrap_or(0.0);
    let maintenance_margin: f64 = user_state.cross_maintenance_margin_used.parse().unwrap_or(0.0);
    
    println!("\n💰 Account Summary:");
    println!("   Account Value:       ${:>15.2}", account_value);
    println!("   Raw USD Balance:     ${:>15.2}", raw_usd);
    println!("   Total Margin Used:   ${:>15.2}", margin_used);
    println!("   Maintenance Margin:  ${:>15.2}", maintenance_margin);
    println!("   Notional Position:   ${:>15.2}", ntl_pos);
    
    // Calculate available margin
    let available_margin = account_value - margin_used;
    println!("   Available Margin:    ${:>15.2}", available_margin);
    
    // Margin utilization
    let margin_util = if account_value > 0.0 {
        (margin_used / account_value) * 100.0
    } else {
        0.0
    };
    println!("   Margin Utilization:  {:>15.1}%", margin_util);
    
    // Asset positions
    if !user_state.asset_positions.is_empty() {
        println!("\n📊 Asset Positions ({} total):", user_state.asset_positions.len());
        println!("{:-<80}", "");
        println!("{:<8} {:<12} {:<15} {:<15} {:<15}", "Asset", "Size", "Entry Price", "Position Value", "PnL");
        println!("{:-<80}", "");
        
        let mut total_unrealized_pnl = 0.0;
        
        for position in &user_state.asset_positions {
            let size: f64 = position.szi.parse().unwrap_or(0.0);
            let position_value: f64 = position.position_value.parse().unwrap_or(0.0);
            let unrealized_pnl: f64 = position.unrealized_pnl.parse().unwrap_or(0.0);
            let entry_price = position.entry_px.map(|p| p.into_inner()).unwrap_or(0.0);
            
            total_unrealized_pnl += unrealized_pnl;
            
            let pnl_emoji = if unrealized_pnl > 0.0 {
                "🟢"
            } else if unrealized_pnl < 0.0 {
                "🔴"
            } else {
                "⚪"
            };
            
            println!(
                "{:<8} {:<12.6} ${:<14.2} ${:<14.2} {} ${:.2}",
                position.coin,
                size,
                entry_price,
                position_value.abs(),
                pnl_emoji,
                unrealized_pnl
            );
            
            // Show leverage if available
            if let Some(leverage) = &position.leverage {
                println!("         Leverage: {:.1}x", leverage.into_inner());
            }
            
            // Show return on equity if available
            if let Some(roe) = &position.return_on_equity {
                let roe_value: f64 = roe.parse().unwrap_or(0.0);
                println!("         ROE: {:.2}%", roe_value * 100.0);
            }
        }
        
        println!("{:-<80}", "");
        let total_pnl_emoji = if total_unrealized_pnl > 0.0 {
            "🟢"
        } else if total_unrealized_pnl < 0.0 {
            "🔴"
        } else {
            "⚪"
        };
        println!("Total Unrealized PnL: {} ${:.2}", total_pnl_emoji, total_unrealized_pnl);
    } else {
        println!("\n📊 No open positions");
    }
    
    // Withdrawal information
    if !user_state.withdrawals_used.is_empty() {
        println!("\n💸 Withdrawal Limits:");
        for withdrawal in &user_state.withdrawals_used {
            let used: f64 = withdrawal.used.parse().unwrap_or(0.0);
            let limit: f64 = withdrawal.limit.parse().unwrap_or(0.0);
            let remaining = limit - used;
            let utilization = if limit > 0.0 { (used / limit) * 100.0 } else { 0.0 };
            
            println!("   Used: ${:.2} / ${:.2} ({:.1}%)", used, limit, utilization);
            println!("   Remaining: ${:.2}", remaining);
        }
    }
    
    // Risk metrics
    println!("\n⚠️ Risk Metrics:");
    let health_ratio = if maintenance_margin > 0.0 {
        account_value / maintenance_margin
    } else {
        f64::INFINITY
    };
    
    if health_ratio < 1.2 {
        println!("   🔴 Account Health: {:.2} (DANGER - close to liquidation)", health_ratio);
    } else if health_ratio < 2.0 {
        println!("   🟡 Account Health: {:.2} (WARNING - monitor closely)", health_ratio);
    } else {
        println!("   🟢 Account Health: {:.2} (SAFE)", health_ratio);
    }
    
    Ok(())
}