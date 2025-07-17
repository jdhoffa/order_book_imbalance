# order_book_imbalance
This project implements an order book imbalance high-frequency trading strategy in Rust. It leverages real-time level 2 (L2) market data from the Binance WebSocket API.

## Features
- Initial order book snapshot fetching via REST API (stored as a local Parquet file)
- Real-time order book updates, streamed via WebSocket
- Order book imbalance calculation based on real-time data
- Buy/sell signal generation based on imbalance thresholds
- (Util) CLI for reading and analyzing stored parquet snapshots

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
```

The application will:
1. Fetch an initial order book snapshot via REST API
2. Store the snapshot locally in Parquet format
   - File will be named `orderbook_snapshot_btcusdt_{YYYYMMDD}_{HHMMSS}.parquet`
   - Example: `orderbook_snapshot_btcusdt_20250716_123456.parquet`
3. Connect to Binance's WebSocket stream, for the given symbol (default: BTC/USDT)
4. Calculate order book imbalance in real-time on 10-second intervals:
    - "top_bids_qty / (top_bids_qty + top_asks_qty)"
5. Generate buy/sell signals based on imbalance thresholds:
    - Buy signal if imbalance > 0.7
    - Sell signal if imbalance < 0.3
6. Exit the stream by pressing `Ctrl+C`

### (Util) Reading Stored Parquet Snapshots
Use the snapshot reader to view stored order book data:
```bash
cargo run --bin read_snapshot -- -f <filename>.parquet
```

Options:
- `-f, --file`: Path to the Parquet file (required)
- `-r, --rows`: Number of rows to display (default: 10, use 0 for all rows)

Example:
```bash
cargo run --bin read_snapshot -- -f orderbook_snapshot_btcusdt_{YYYYMMDD}_{HHMMSS}.parquet -r 5
```

## Example Output

When running the strategy, you might see output like:

```
Received snapshot with last update ID: 72984315819
Saved snapshot to orderbook_snapshot_btcusdt_20250717_151018.parquet
WebSocket connection established
OrderBook updated (Last Update ID: 72984316237)
Current Imbalance (top 10): 0.7071
📈 BUY SIGNAL (Imbalance: 0.7071)
OrderBook updated (Last Update ID: 72984316606)
Current Imbalance (top 10): 0.7532
📈 BUY SIGNAL (Imbalance: 0.7532)
OrderBook updated (Last Update ID: 72984316956)
Current Imbalance (top 10): 0.7467
📈 BUY SIGNAL (Imbalance: 0.7467)
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
orderbook_snapshot_<symbol>_<timestamp>.parquet
```
Example: `orderbook_snapshot_btcusdt_20250716_123456.parquet`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
