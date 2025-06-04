use dex_rs::prelude::*;

#[tokio::main]
async fn main() -> DexResult<()> {
    env_logger::init();
    
    println!("🔗 Connecting to Hyperliquid testnet...");
    let hl = Hyperliquid::builder().testnet().connect().await?;
    
    let coin = "BTC";
    
    // Get funding history for the last 7 days
    let now = chrono::Utc::now().timestamp_millis() as u64;
    let seven_days_ago = now - (7 * 24 * 60 * 60 * 1000);
    
    println!("💰 Fetching funding history for {} (last 7 days)...", coin);
    let funding_history = hl.funding_history(coin, seven_days_ago, Some(now)).await?;
    
    println!("\n📊 Funding History for {}:", coin);
    println!("{:-<80}", "");
    println!("{:<20} {:<15} {:<15} {:<15}", "Time", "Funding Rate", "Premium", "Time");
    println!("{:-<80}", "");
    
    let mut total_funding = 0.0;
    
    for funding in funding_history.iter().take(20) {  // Show last 20 entries
        let datetime = chrono::DateTime::from_timestamp_millis(funding.time as i64)
            .unwrap_or_default()
            .format("%m-%d %H:%M UTC");
        
        let funding_rate: f64 = funding.funding_rate.parse().unwrap_or(0.0);
        let premium: f64 = funding.premium.parse().unwrap_or(0.0);
        
        total_funding += funding_rate;
        
        // Color code based on funding rate
        let rate_display = if funding_rate > 0.0 {
            format!("🔴 {:.6}%", funding_rate * 100.0)
        } else if funding_rate < 0.0 {
            format!("🟢 {:.6}%", funding_rate * 100.0)
        } else {
            format!("⚪ {:.6}%", funding_rate * 100.0)
        };
        
        println!(
            "{:<20} {:<15} {:<15.6} {}",
            datetime,
            rate_display,
            premium,
            funding.time
        );
    }
    
    if !funding_history.is_empty() {
        let avg_funding = total_funding / funding_history.len().min(20) as f64;
        println!("\n📈 Average funding rate (last {} periods): {:.6}%", 
                 funding_history.len().min(20), 
                 avg_funding * 100.0);
        
        // Calculate annualized rate (funding typically happens every 8 hours)
        let annualized_rate = avg_funding * 365.0 * 3.0; // 3 times per day
        println!("📊 Annualized funding rate: {:.2}%", annualized_rate * 100.0);
        
        if let Some(latest) = funding_history.first() {
            let latest_rate: f64 = latest.funding_rate.parse().unwrap_or(0.0);
            println!("🕐 Latest funding rate: {:.6}%", latest_rate * 100.0);
        }
    } else {
        println!("No funding history found for the specified period.");
    }
    
    Ok(())
}