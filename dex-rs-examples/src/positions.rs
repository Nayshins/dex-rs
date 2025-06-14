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
        .private_key(&private_key)
        .connect()
        .await?;

    println!("📊 Fetching current positions...");
    let positions = hl.positions().await?;

    if positions.is_empty() {
        println!("❌ No open positions found.");
        println!("💡 Try placing some orders first to establish positions.");
        return Ok(());
    }

    println!("\n📊 Current Positions ({} total):", positions.len());
    println!("{:=<90}", "");

    let mut total_unrealized_pnl = 0.0;
    let mut total_position_value = 0.0;

    for (i, position) in positions.iter().enumerate() {
        println!("\n🏷️ Position #{} - {}", i + 1, position.coin);
        println!("{:-<50}", "");

        let size_display = if position.size > 0.0 {
            format!("🟢 LONG {:.6}", position.size)
        } else if position.size < 0.0 {
            format!("🔴 SHORT {:.6}", position.size.abs())
        } else {
            format!("⚪ FLAT {:.6}", position.size)
        };

        println!("   Position: {}", size_display);

        if let Some(entry_px) = position.entry_px {
            println!("   Entry Price: ${:.2}", entry_px);

            // Calculate current position value (approximate)
            let position_value = position.size.abs() * entry_px;
            total_position_value += position_value;
            println!("   Position Value: ${:.2}", position_value);
        } else {
            println!("   Entry Price: Not available");
        }

        let pnl_emoji = if position.unrealized_pnl > 0.0 {
            "🟢"
        } else if position.unrealized_pnl < 0.0 {
            "🔴"
        } else {
            "⚪"
        };

        println!(
            "   Unrealized PnL: {} ${:.2}",
            pnl_emoji, position.unrealized_pnl
        );
        total_unrealized_pnl += position.unrealized_pnl;

        // Calculate ROE if we have entry price
        if let Some(entry_px) = position.entry_px {
            if entry_px > 0.0 && position.size != 0.0 {
                let roe = (position.unrealized_pnl / (position.size.abs() * entry_px)) * 100.0;
                let roe_emoji = if roe > 0.0 {
                    "🟢"
                } else if roe < 0.0 {
                    "🔴"
                } else {
                    "⚪"
                };
                println!("   Return on Equity: {} {:.2}%", roe_emoji, roe);
            }
        }
    }

    // Summary
    println!("\n📈 Portfolio Summary:");
    println!("{:=<50}", "");

    let total_pnl_emoji = if total_unrealized_pnl > 0.0 {
        "🟢"
    } else if total_unrealized_pnl < 0.0 {
        "🔴"
    } else {
        "⚪"
    };

    println!("Total Position Value: ${:.2}", total_position_value);
    println!(
        "Total Unrealized PnL: {} ${:.2}",
        total_pnl_emoji, total_unrealized_pnl
    );

    if total_position_value > 0.0 {
        let portfolio_roe = (total_unrealized_pnl / total_position_value) * 100.0;
        println!("Portfolio ROE: {:.2}%", portfolio_roe);
    }

    // Position distribution
    println!("\n📊 Position Distribution:");
    let long_positions = positions.iter().filter(|p| p.size > 0.0).count();
    let short_positions = positions.iter().filter(|p| p.size < 0.0).count();
    let flat_positions = positions.iter().filter(|p| p.size == 0.0).count();

    println!("   🟢 Long Positions: {}", long_positions);
    println!("   🔴 Short Positions: {}", short_positions);
    println!("   ⚪ Flat Positions: {}", flat_positions);

    // Largest positions by absolute value
    let mut sorted_positions = positions.clone();
    sorted_positions.sort_by(|a, b| {
        b.unrealized_pnl
            .abs()
            .partial_cmp(&a.unrealized_pnl.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if sorted_positions.len() > 1 {
        println!("\n🎯 Top Performers by PnL:");
        for (i, position) in sorted_positions.iter().take(3).enumerate() {
            let pnl_emoji = if position.unrealized_pnl > 0.0 {
                "🟢"
            } else {
                "🔴"
            };
            println!(
                "   {}. {} {} ${:.2}",
                i + 1,
                position.coin,
                pnl_emoji,
                position.unrealized_pnl
            );
        }
    }

    Ok(())
}
