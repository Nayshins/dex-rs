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
    
    // First, get all open orders
    println!("📋 Fetching current open orders...");
    let orders = hl.open_orders().await?;
    
    if orders.is_empty() {
        println!("❌ No open orders found to cancel.");
        println!("💡 Try running the 'place_order' example first to create an order.");
        return Ok(());
    }
    
    println!("\n📊 Found {} open order(s):", orders.len());
    println!("{:-<80}", "");
    println!("{:<12} {:<8} {:<15} {:<15} {:<15}", "Order ID", "Side", "Asset", "Price", "Size");
    println!("{:-<80}", "");
    
    for order in &orders {
        println!("{:<12} {:<8} {:<15} ${:<14} {:<15}",
                order.oid,
                order.side.to_uppercase(),
                order.coin,
                order.limit_px,
                order.sz);
    }
    
    // For this example, let's cancel the first order
    let order_to_cancel = &orders[0];
    let order_id = OrderId(order_to_cancel.oid.to_string());
    
    println!("\n🎯 Canceling order ID: {}", order_to_cancel.oid);
    println!("   Asset: {}", order_to_cancel.coin);
    println!("   Side: {}", order_to_cancel.side.to_uppercase());
    println!("   Price: ${}", order_to_cancel.limit_px);
    println!("   Size: {}", order_to_cancel.sz);
    
    match hl.cancel(order_id).await {
        Ok(()) => {
            println!("✅ Order canceled successfully!");
            
            // Wait a moment then verify cancellation
            println!("\n⏳ Waiting 2 seconds then verifying cancellation...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            match hl.open_orders().await {
                Ok(remaining_orders) => {
                    let was_canceled = !remaining_orders.iter()
                        .any(|o| o.oid == order_to_cancel.oid);
                    
                    if was_canceled {
                        println!("✅ Confirmed: Order {} has been canceled", order_to_cancel.oid);
                    } else {
                        println!("⚠️ Warning: Order {} still appears in open orders", order_to_cancel.oid);
                    }
                    
                    println!("\n📋 Remaining open orders: {}", remaining_orders.len());
                    if !remaining_orders.is_empty() {
                        for order in &remaining_orders {
                            println!("   🆔 {}: {} {} @ ${}", 
                                    order.oid,
                                    order.side.to_uppercase(),
                                    order.coin,
                                    order.limit_px);
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️ Could not verify cancellation: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to cancel order: {}", e);
            println!("\n🔍 Possible reasons:");
            println!("   - Order already filled or canceled");
            println!("   - Order ID not found");
            println!("   - Network connectivity issues");
            println!("   - Authentication problems");
        }
    }
    
    Ok(())
}