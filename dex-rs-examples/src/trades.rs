use dex_rs::prelude::*;

#[tokio::main]
async fn main() -> DexResult<()> {
    env_logger::init();
    
    println!("🔗 Connecting to Hyperliquid testnet...");
    let hl = Hyperliquid::builder().testnet().connect().await?;
    
    let coin = "BTC";
    let limit = 10;
    
    println!("📈 Fetching recent {} trades for {}...", limit, coin);
    let trades = hl.trades(coin, limit).await?;
    
    println!("\n📊 Recent Trades for {}:", coin);
    println!("{:-<80}", "");
    println!("{:<20} {:<10} {:<15} {:<15} {:<10}", "Time", "Side", "Price", "Quantity", "Trade ID");
    println!("{:-<80}", "");
    
    for trade in trades.iter().rev() {
        let datetime = chrono::DateTime::from_timestamp_millis(trade.ts as i64)
            .unwrap_or_default()
            .format("%H:%M:%S");
        
        let side_emoji = match trade.side {
            Side::Buy => "🟢 BUY ",
            Side::Sell => "🔴 SELL",
        };
        
        println!(
            "{:<20} {:<10} ${:<14.2} {:<15.6} {}",
            datetime,
            side_emoji,
            trade.price.into_inner(),
            trade.qty.into_inner(),
            trade.tid
        );
    }
    
    if let Some(latest) = trades.first() {
        println!("\n💡 Latest trade: {} {} at ${:.2}", 
                 latest.qty.into_inner(),
                 coin,
                 latest.price.into_inner());
    }
    
    Ok(())
}