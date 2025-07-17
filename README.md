# order_book_imbalance
[![Rust](https://github.com/jdhoffa/order_book_imbalance/actions/workflows/ci.yml/badge.svg)](https://github.com/jdhoffa/order_book_imbalance/actions/workflows/ci.yml)

This project implements an order book imbalance high-frequency trading strategy in Rust. It leverages real-time level 2 (L2) market data from the Binance WebSocket API.

## Features
- Initial order book snapshot fetching via REST API (snapshot saved as JSON)
- Real-time order book updates, streamed via WebSocket (stream saved as JSONL on exit)
- Order book imbalance calculation based on real-time data
- Buy/sell signal generation based on imbalance thresholds (trade decisions saved as JSON)
- Simulation of trading strategy with a 1000000 USDT starting portfolio balance

## Installation
1. Clone the repository
2. Build the project:
```bash
cargo build --release
```

## Usage

### Fetching Order Book Data
Run the main application to trigger the order book imbalance strategy (on "BTC/USDT" by default):
```bash
cargo run

# specify log level (default: info)
RUST_LOG=debug cargo run
```

The application will:
1. Fetch an initial order book snapshot via REST API
2. Store the snapshot locally in Parquet format
   - File will be named `orderbook_snapshot_btcusdt_{YYYYMMDD}_{HHMMSS}.json`
   - Example: `orderbook_snapshot_btcusdt_20250716_123456.json`
3. Connect to Binance's WebSocket stream, for the given symbol (default: BTC/USDT)
4. Calculate order book imbalance in real-time on 10-second intervals:
    - "top_bids_qty / (top_bids_qty + top_asks_qty)"
5. Generate buy/sell signals based on imbalance thresholds:
    - Buy signal if imbalance > 0.7
    - Sell signal if imbalance < 0.3
6. Simulate trading strategy with a 1000000 USDT starting portfolio balance
7. Exit the stream by pressing `Ctrl+C`

## Example Output

When running the strategy, you might see output like:

```
[2025-07-17T16:38:50Z INFO  order_book_imbalance] Initial portfolio: $1000000.00 USDT, 0.00000000 BTC
[2025-07-17T16:38:50Z INFO  order_book_imbalance] Received snapshot with last update ID: 72995677008
[2025-07-17T16:38:50Z INFO  order_book_imbalance] Saved snapshot to orderbook_snapshot_btcusdt_20250717_183850.json
[2025-07-17T16:38:52Z INFO  order_book_imbalance] WebSocket connection established
[2025-07-17T16:38:52Z INFO  order_book_imbalance] OrderBook updated (Last Update ID: 72995677651)
[2025-07-17T16:38:52Z INFO  order_book_imbalance] Current Imbalance (top 10): 0.1968

...

[2025-07-17T17:08:35Z INFO  order_book_imbalance] Current Imbalance (top 10): 0.8519
^C[2025-07-17T17:08:35Z INFO  order_book_imbalance] Received CTRL+C, shutting down gracefully...
[2025-07-17T17:08:35Z INFO  order_book_imbalance] Saved trades to trades.json
[2025-07-17T17:08:35Z INFO  order_book_imbalance] Final portfolio value: $1020314.66 (USDT: $0.00, BTC: 8.58543368 @ 118842.53 USDT)
```

## Data Format

### Order Book Snapshot
Snapshots are stored in Parquet format with the following schema:
- symbol: String
- update_id: UInt64
- bid_price: Float64
- bid_quantity: Float64
- ask_price: Float64
- ask_quantity: Float64

### Order Book Updates
Each real-time order book update includes:
- Event type
- Event time
- Symbol
- Update IDs (for order book maintenance)
- Bids (price and quantity)
- Asks (price and quantity)

## File Naming Convention
Snapshot files are saved with the format:
```
orderbook_snapshot_<symbol>_<timestamp>.json
```
Example: `orderbook_snapshot_btcusdt_20250716_123456.json`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
