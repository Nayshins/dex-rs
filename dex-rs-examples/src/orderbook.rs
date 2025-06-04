use dex_rs::prelude::*;

#[tokio::main]
async fn main() -> DexResult<()> {
    env_logger::init();
    
    println!("🔗 Connecting to Hyperliquid testnet...");
    let hl = Hyperliquid::builder().testnet().connect().await?;
    
    let coin = "BTC";
    let depth = 10;
    
    println!("📖 Fetching order book for {} (depth: {})...", coin, depth);
    let orderbook = hl.orderbook(coin, depth).await?;
    
    println!("\n📊 Order Book for {} ({})", coin, orderbook.coin);
    println!("Timestamp: {}", chrono::DateTime::from_timestamp_millis(orderbook.ts as i64)
        .unwrap_or_default()
        .format("%Y-%m-%d %H:%M:%S UTC"));
    
    // Calculate mid price
    let best_bid = orderbook.bids.first().map(|b| b.price.into_inner()).unwrap_or(0.0);
    let best_ask = orderbook.asks.first().map(|a| a.price.into_inner()).unwrap_or(0.0);
    let mid_price = if best_bid > 0.0 && best_ask > 0.0 {
        (best_bid + best_ask) / 2.0
    } else {
        0.0
    };
    
    println!("Mid Price: ${:.2}", mid_price);
    if best_bid > 0.0 && best_ask > 0.0 {
        let spread = best_ask - best_bid;
        let spread_bps = (spread / mid_price) * 10000.0;
        println!("Spread: ${:.2} ({:.1} bps)", spread, spread_bps);
    }
    
    println!("\n{:-<70}", "");
    println!("{:^35} | {:^35}", "BIDS", "ASKS");
    println!("{:-<35}+{:-<35}", "", "");
    println!("{:<15} {:<15} {:<3} | {:<3} {:<15} {:<15}", "Quantity", "Price", "Ords", "Ords", "Price", "Quantity");
    println!("{:-<35}+{:-<35}", "", "");
    
    let max_levels = std::cmp::max(orderbook.bids.len(), orderbook.asks.len());
    
    for i in 0..max_levels {
        let bid_str = if i < orderbook.bids.len() {
            let bid = &orderbook.bids[i];
            format!("{:<15.6} ${:<14.2} {:<3}", 
                   bid.qty.into_inner(), 
                   bid.price.into_inner(),
                   bid.n)
        } else {
            format!("{:<35}", "")
        };
        
        let ask_str = if i < orderbook.asks.len() {
            let ask = &orderbook.asks[i];
            format!("{:<3} ${:<14.2} {:<15.6}", 
                   ask.n,
                   ask.price.into_inner(),
                   ask.qty.into_inner())
        } else {
            format!("{:<35}", "")
        };
        
        println!("{} | {}", bid_str, ask_str);
    }
    
    // Show totals
    let total_bid_qty: f64 = orderbook.bids.iter().map(|b| b.qty.into_inner()).sum();
    let total_ask_qty: f64 = orderbook.asks.iter().map(|a| a.qty.into_inner()).sum();
    
    println!("{:-<35}+{:-<35}", "", "");
    println!("Total: {:<23.6} | Total: {:<23.6}", total_bid_qty, total_ask_qty);
    
    Ok(())
}