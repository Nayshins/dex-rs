# dex-rs

A high-performance Rust library for interacting with cryptocurrency perpetual exchanges. Currently supports Hyperliquid with a modular architecture for easy integration of additional exchanges.

## Features

- **Unified API**: Common trait interface (`PerpDex`) for all supported exchanges
- **Async/Await**: Built on Tokio for high-performance async operations
- **Real-time Streaming**: WebSocket support for live market data and account updates
- **Type Safety**: Comprehensive type definitions with NaN-safe numeric types
- **Production Ready**: TLS support, automatic reconnection, and robust error handling

## Supported Exchanges

- **Hyperliquid** - Complete API implementation with all 24 endpoints

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
dex-rs = "0.1.0"
```

### Basic Usage

```rust
use dex_rs::prelude::*;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> DexResult<()> {
    // Connect to Hyperliquid testnet
    let hl = Hyperliquid::builder().testnet().connect().await?;
    
    // Get recent trades
    let trades = hl.trades("BTC", 10).await?;
    println!("Last trade: {:?}", trades.last());

    // Stream real-time market data
    let (tx, mut rx) = mpsc::unbounded_channel();
    hl.subscribe(StreamKind::Bbo, Some("BTC"), tx).await?;

    while let Some(event) = rx.recv().await {
        if let StreamEvent::Bbo { bid_px, ask_px, .. } = event {
            println!("BTC Bid: {bid_px}, Ask: {ask_px}");
        }
    }
    
    Ok(())
}
```

### Trading Example

```rust
use dex_rs::prelude::*;

#[tokio::main]
async fn main() -> DexResult<()> {
    // Connect with authentication for trading
    let hl = Hyperliquid::builder()
        .testnet()
        .credentials("your_private_key")
        .connect()
        .await?;
    
    // Place a limit order
    let order = OrderReq {
        coin: "BTC".to_string(),
        is_buy: true,
        px: price(65000.0),
        qty: qty(0.001),
        tif: Tif::Gtc,
        reduce_only: false,
        cloid: None,  // Or Some("my_custom_id".to_string())
    };
    
    let resp = hl.place_order(order).await?;
    println!("Order placed - Exchange ID: {}, Client ID: {}", 
             resp.order_id.0, resp.client_order_id);
    
    // Check positions
    let positions = hl.positions().await?;
    for position in positions {
        println!("Position: {} size: {}", position.coin, position.size);
    }
    
    Ok(())
}
```

### Client Order IDs

The library supports client order IDs (clOrdIds) for order tracking:

```rust
use dex_rs::prelude::*;

// Use a custom client order ID
let order = OrderReq {
    coin: "BTC".to_string(),
    cloid: Some("my_order_123".to_string()),
    // ... other fields
};

// Or generate one automatically
let cloid = generate_cloid(); // e.g., "1701234567890123456_42"
let order = OrderReq {
    coin: "BTC".to_string(),
    cloid: Some(cloid.clone()),
    // ... other fields
};

// The response includes both IDs
let resp = hl.place_order(order).await?;
println!("Exchange ID: {}", resp.order_id.0);
println!("Client ID: {}", resp.client_order_id);
```

## API Reference

### Market Data

- `trades(coin, limit)` - Get recent trades
- `orderbook(coin, depth)` - Get order book snapshot
- `all_mids()` - Get mid prices for all assets
- `meta()` - Get perpetual market metadata
- `funding_history(coin, start, end)` - Get funding rate history

### Account Management (Requires Authentication)

- `place_order(order)` - Place a new order
- `cancel(order_id)` - Cancel an existing order
- `positions()` - Get current positions
- `user_state()` - Get account state and balances
- `open_orders()` - Get open orders
- `user_fills()` - Get fill history

### Real-time Streaming

- `subscribe(StreamKind, coin, channel)` - Subscribe to real-time data

Supported stream types:
- `StreamKind::Trades` - Trade updates
- `StreamKind::Bbo` - Best bid/offer updates
- `StreamKind::L2Book` - Level 2 order book updates
- `StreamKind::Orders` - Order status updates (authenticated)
- `StreamKind::Fills` - Fill notifications (authenticated)

## Architecture

The library is organized into several crates:

- **`dex-rs-core`** - Core traits, error handling, and transport layer
- **`dex-rs-types`** - Common type definitions and data structures
- **`dex-rs-exchanges/hyperliquid`** - Hyperliquid exchange implementation
- **`dex-rs-examples`** - Usage examples and demos

## Error Handling

All operations return `DexResult<T>` which is an alias for `Result<T, DexError>`. The error type provides detailed information about failures:

```rust
match hl.trades("BTC", 10).await {
    Ok(trades) => println!("Got {} trades", trades.len()),
    Err(DexError::Network(err)) => eprintln!("Network error: {}", err),
    Err(DexError::Api(err)) => eprintln!("API error: {}", err),
    Err(err) => eprintln!("Other error: {}", err),
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.