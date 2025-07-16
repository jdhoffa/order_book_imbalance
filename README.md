# order_book_imbalance
This project explores an implementation of the order book imbalance high-frequency trading strategy. The strategy is coded in Rust, and tested against free publicly available real-time level 2 (L2) data using the Binance WebSocket API. 

## Features
- Initial order book snapshot fetching via REST API
- Parquet file storage of order book snapshots
- Real-time order book data streaming via WebSocket
- CLI for reading and analyzing stored parquet snapshots

## Installation
1. Clone the repository
2. Build the project:
```bash
cargo build --release
```

## Usage

### Fetching Order Book Data
Run the main application to fetch and store order book data:
```bash
cargo run --bin order_book_imbalance
```

The application will:
1. Fetch an initial order book snapshot via REST API (will automatically be saved as a timestamped parquet file)
3. Connect to Binance's WebSocket stream
4. Display order book updates in real-time
5. Exit the stream by pressing `Ctrl+C`

### Reading Stored Snapshots
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

## Data Format

### Order Book Updates
Each real-time order book update includes:
- Event type
- Event time
- Symbol
- Update IDs (for order book maintenance)
- Bids (price and quantity)
- Asks (price and quantity)

### Stored Snapshots
Snapshots are stored in Parquet format with the following schema:
- symbol: String
- update_id: UInt64
- bid_price: Float64
- bid_quantity: Float64
- ask_price: Float64
- ask_quantity: Float64

## File Naming Convention
Snapshot files are saved with the format:
```
orderbook_snapshot_<symbol>_<timestamp>.parquet
```
Example: `orderbook_snapshot_btcusdt_20250716_123456.parquet`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.