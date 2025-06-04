# dex-rs Examples

This directory contains comprehensive examples demonstrating how to use the dex-rs library with Hyperliquid.

## Prerequisites

1. **Rust Environment**: Make sure you have Rust installed
2. **Private Key**: For authenticated examples, set your private key as an environment variable:
   ```bash
   export HYPERLIQUID_PRIVATE_KEY="your_private_key_here"
   ```
3. **Testnet**: All examples use Hyperliquid testnet by default for safety

## Running Examples

Use `cargo run --bin <example_name>` to run any example:

```bash
cd dex-rs-examples
cargo run --bin trades
```

## Available Examples

### 🔍 Market Data Examples (No Authentication Required)

#### `trades`
Fetches and displays recent trades for a specific asset.
```bash
cargo run --bin trades
```
Shows: Recent trade history with timestamps, prices, quantities, and trade IDs.

#### `orderbook`
Fetches and displays the current order book snapshot.
```bash
cargo run --bin orderbook
```
Shows: Best bids/asks, order book depth, spreads, and market statistics.

#### `funding_history`
Fetches and displays funding rate history for perpetual contracts.
```bash
cargo run --bin funding_history
```
Shows: Historical funding rates, premiums, and annualized rates.

#### `market_info`
Displays comprehensive market metadata and asset information.
```bash
cargo run --bin market_info
```
Shows: Available assets, prices, leverage limits, and volume statistics.

### 📡 WebSocket Streaming Examples (No Authentication Required)

#### `ws_trades`
Live stream of trade executions.
```bash
cargo run --bin ws_trades
```
Features: Real-time trade feed with statistics and rate calculations.

#### `ws_orderbook`
Live order book updates with visual display.
```bash
cargo run --bin ws_orderbook
```
Features: Real-time order book visualization with spread calculations.

#### `ws_bbo`
Live best bid/offer updates with price change tracking.
```bash
cargo run --bin ws_bbo
```
Features: BBO stream with price history and trend analysis.

#### `ws_user_events` ⚠️ (Requires Authentication)
Live stream of user-specific events (orders and fills).
```bash
cargo run --bin ws_user_events
```
Features: Real-time order updates and fill notifications.

#### `ws_all_streams` 🔥 (Comprehensive Test)
**THE ULTIMATE TEST** - Subscribes to ALL WebSocket streams simultaneously!
```bash
cargo run --bin ws_all_streams
```
Features: 
- Tests all 5 stream types at once
- Multi-asset streaming (BTC + ETH)
- Real-time statistics and health monitoring
- Works with or without authentication
- Perfect for verifying WebSocket stability

### 💱 Trading Examples ⚠️ (Requires Authentication)

#### `place_order`
Places a limit order and verifies its status.
```bash
cargo run --bin place_order
```
Features: Order placement with market context and status verification.

#### `cancel_order`
Cancels existing open orders.
```bash
cargo run --bin cancel_order
```
Features: Lists open orders and cancels selected orders with confirmation.

### 👤 Account Examples ⚠️ (Requires Authentication)

#### `user_state`
Displays comprehensive account information.
```bash
cargo run --bin user_state
```
Shows: Account balance, positions, margin usage, and risk metrics.

#### `positions`
Shows detailed position information and portfolio summary.
```bash
cargo run --bin positions
```
Shows: Position details, PnL, portfolio statistics, and performance metrics.

#### `fills`
Displays trading history and fill details.
```bash
cargo run --bin fills
```
Shows: Recent fills, trading statistics, and fee analysis.

## Authentication Setup

For examples that require authentication (marked with ⚠️), you need to:

1. **Get a Private Key**: Generate or export your Hyperliquid private key
2. **Set Environment Variable**:
   ```bash
   export HYPERLIQUID_PRIVATE_KEY="0x1234567890abcdef..."
   ```
3. **Testnet Safety**: All examples use testnet - your mainnet funds are safe

### Security Notes

- **Never commit private keys** to version control
- **Use testnet** for development and testing
- **Store keys securely** in environment variables or secure key management
- **Rotate keys regularly** for production use

## Example Output

### Market Data Example
```
🔗 Connecting to Hyperliquid testnet...
📈 Fetching recent 10 trades for BTC...

📊 Recent Trades for BTC:
────────────────────────────────────────────────────────────────────────────────
Time                 Side       Price           Quantity        Trade ID
────────────────────────────────────────────────────────────────────────────────
14:32:15             🟢 BUY     $65,420.50      0.001500        1234567
14:32:10             🔴 SELL    $65,415.25      0.002100        1234566
...
```

### Trading Example
```
🔗 Connecting to Hyperliquid testnet with authentication...
📊 Fetching current market data for BTC...
💰 Current market: Bid $65,420.50 | Ask $65,425.00 | Mid $65,422.75

📝 Placing limit buy order:
   Asset: BTC
   Side: BUY
   Price: $64,420.50
   Size: 0.001 BTC
   Type: Good Till Cancel (GTC)

✅ Order placed successfully!
🆔 Order ID: 1234567890
```

## Troubleshooting

### Common Issues

1. **Authentication Errors**
   - Verify your private key is correctly set
   - Ensure you're using testnet keys for testnet

2. **Network Errors**
   - Check your internet connection
   - Verify Hyperliquid services are operational

3. **Compilation Errors**
   - Make sure you're in the `dex-rs-examples` directory
   - Run `cargo update` to refresh dependencies

### Logging

Set `RUST_LOG=debug` for detailed logging:
```bash
RUST_LOG=debug cargo run --bin trades
```

## Educational Purpose

These examples are designed for:
- **Learning** the dex-rs API
- **Testing** integration patterns
- **Development** reference implementations
- **Prototyping** trading strategies

**Not for production trading without proper testing and risk management!**

## Contributing

Feel free to:
- Add new examples
- Improve existing examples
- Fix bugs or enhance documentation
- Share interesting use cases

See the main project README for contribution guidelines.