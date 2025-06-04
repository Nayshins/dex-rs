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
    
    let coin = "BTC";
    
    // Get current market price for reference
    println!("📊 Fetching current market data for {}...", coin);
    let orderbook = hl.orderbook(coin, 1).await?;
    let best_bid = orderbook.bids.first().map(|b| b.price.into_inner()).unwrap_or(0.0);
    let best_ask = orderbook.asks.first().map(|a| a.price.into_inner()).unwrap_or(0.0);
    let mid_price = (best_bid + best_ask) / 2.0;
    
    println!("💰 Current market: Bid ${:.2} | Ask ${:.2} | Mid ${:.2}", best_bid, best_ask, mid_price);
    
    // Place a limit buy order well below market (to avoid accidental execution)
    let order_price = best_bid - 1000.0; // $1000 below best bid
    let order_size = 0.001; // Small size for testing
    
    println!("\n📝 Placing limit buy order:");
    println!("   Asset: {}", coin);
    println!("   Side: BUY");
    println!("   Price: ${:.2}", order_price);
    println!("   Size: {} {}", order_size, coin);
    println!("   Type: Good Till Cancel (GTC)");
    
    let order_req = OrderReq {
        coin: coin.to_string(),
        is_buy: true,
        px: price(order_price),
        qty: qty(order_size),
        tif: Tif::Gtc,
        reduce_only: false,
    };
    
    match hl.place_order(order_req).await {
        Ok(order_id) => {
            println!("✅ Order placed successfully!");
            println!("🆔 Order ID: {}", order_id.0);
            
            // Wait a moment then check order status
            println!("\n⏳ Waiting 2 seconds then checking order status...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            // Get open orders to verify
            match hl.open_orders().await {
                Ok(orders) => {
                    println!("\n📋 Current open orders:");
                    if orders.is_empty() {
                        println!("   No open orders found");
                    } else {
                        for order in &orders {
                            if order.coin == coin {
                                println!("   🆔 {}: {} {} @ ${} (size: {})", 
                                        order.oid,
                                        order.side.to_uppercase(),
                                        order.coin,
                                        order.limit_px,
                                        order.sz);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️ Could not fetch open orders: {}", e);
                }
            }
            
            println!("\n💡 Note: This order is placed well below market price and should remain open.");
            println!("💡 You can cancel it using the cancel_order example with order ID: {}", order_id.0);
        }
        Err(e) => {
            println!("❌ Failed to place order: {}", e);
            println!("\n🔍 Possible reasons:");
            println!("   - Insufficient balance");
            println!("   - Invalid price/size");
            println!("   - Network connectivity issues");
            println!("   - Authentication problems");
        }
    }
    
    Ok(())
}