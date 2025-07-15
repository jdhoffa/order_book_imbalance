# order_book_imbalance
This project explores an implementation of the order book imbalance high-frequency trading strategy. The strategy is coded in Rust, and tested against free publicly available real-time level 2 (L2) data using the Binance WebSocket API. 

# (WIP) Usage
1. Clone the repository
2. Run the application:
``` bash
cargo run
```

The application will connect to Binance's WebSocket stream and display order book updates in real-time. 

4. Exit the stream by pressing `Ctrl+C`.

## Output Format

Each order book update includes:
- Event type
- Event time
- Symbol
- Update IDs (for order book maintenance)
- Bids (price and quantity)
- Asks (price and quantity)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.