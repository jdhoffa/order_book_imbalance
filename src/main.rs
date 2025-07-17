mod models;

use futures_util::StreamExt;
use models::{OrderBook, OrderBookSnapshot, OrderBookUpdate, Trade};
use reqwest;
use serde_json;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use tokio::{select, signal};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

async fn fetch_order_book_snapshot(
    symbol: &str,
) -> Result<OrderBookSnapshot, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.binance.com/api/v3/depth?symbol={}&limit=1000",
        symbol.to_uppercase()
    );

    let response = reqwest::get(&url).await?;
    let snapshot: OrderBookSnapshot = response.json().await?;

    Ok(snapshot)
}

async fn save_snapshot_to_json(
    snapshot: &OrderBookSnapshot,
    symbol: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("orderbook_snapshot_{}_{}.json", symbol, timestamp);

    let file = File::create(&filename)?;
    serde_json::to_writer_pretty(file, snapshot)?;
    Ok(filename)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Portfolio state
    let mut usdt_balance: f64 = 1000000.0;
    let mut btc_balance: f64 = 0.0;
    log::info!(
        "Initial portfolio: ${:.2} USDT, {:.8} BTC",
        usdt_balance,
        btc_balance
    );

    // Fetch the initial order book snapshot
    let symbol = "btcusdt";

    let mut shutdown_signal = Box::pin(signal::ctrl_c());

    // Initialize the snapshot
    let snapshot = fetch_order_book_snapshot(symbol).await?;
    log::info!(
        "Received snapshot with last update ID: {}",
        snapshot.last_update_id
    );

    // Save snapshot to JSON file
    match save_snapshot_to_json(&snapshot, symbol).await {
        Ok(filename) => log::info!("Saved snapshot to {}", filename),
        Err(e) => log::error!("Failed to save snapshot: {}", e),
    }

    // Initialize live OrderBook from snapshot
    let mut order_book = OrderBook::from(snapshot);
    let mut trades = Vec::new();

    // Open file to save diffs
    let diffs_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("orderbook_diffs.jsonl")?;
    let mut diffs_writer = BufWriter::new(diffs_file);

    // Connect to the Binance WebSocket stream
    let url = "wss://stream.binance.com:9443/ws/btcusdt@depth";
    let (ws_stream, _) = connect_async(url).await?;
    log::info!("WebSocket connection established");

    let (_, mut read) = ws_stream.split();

    loop {
        select! {
            maybe_message = read.next() => {
                match maybe_message {
                    Some(Ok(Message::Text(text))) => match serde_json::from_str::<OrderBookUpdate>(&text) {
                        Ok(update) => {
                            if update.final_update_id > order_book.last_update_id {
                                let json = serde_json::to_string(&update)?;
                                writeln!(diffs_writer, "{}", json)?;
                                diffs_writer.flush()?;

                                order_book.apply_update(&update);
                                log::info!(
                                    "OrderBook updated (Last Update ID: {})",
                                    order_book.last_update_id
                                );

                                let top_bids_qty: f64 = order_book
                                    .bids
                                    .iter()
                                    .rev()
                                    .take(10)
                                    .map(|(_, qty)| qty)
                                    .sum();

                                let top_asks_qty: f64 =
                                    order_book.asks.iter().take(10).map(|(_, qty)| qty).sum();

                                let imbalance =
                                    top_bids_qty / (top_bids_qty + top_asks_qty).max(f64::EPSILON);
                                log::info!("Current Imbalance (top 10): {:.4}", imbalance);

                                let mut trade_executed = false;
                                let mut trade_side = "";

                                if imbalance > 0.7 && usdt_balance > 0.0 {
                                    // BUY
                                    if let Some((best_ask, _)) = order_book.asks.iter().next() {
                                        let qty = usdt_balance / **best_ask;
                                        btc_balance += qty;
                                        log::info!("📈 BUY {:.8} BTC at {:.2} USDT", qty, best_ask);
                                        usdt_balance = 0.0;
                                        trade_executed = true;
                                        trade_side = "buy";
                                    }
                                }
                                if imbalance < 0.3 && btc_balance > 0.0 {
                                    // SELL
                                    if let Some((best_bid, _)) = order_book.bids.iter().next_back() {
                                        let proceeds = btc_balance * **best_bid;
                                        log::info!("📉 SELL {:.8} BTC at {:.2} USDT", btc_balance, best_bid);
                                        usdt_balance += proceeds;
                                        btc_balance = 0.0;
                                        trade_executed = true;
                                        trade_side = "sell";
                                    }
                                }
                                if trade_executed {
                                    trades.push(Trade {
                                        side: trade_side.into(),
                                        update_id: update.final_update_id,
                                        imbalance,
                                    });
                                    log::info!(
                                        "Portfolio: ${:.2} USDT, {:.8} BTC",
                                        usdt_balance, btc_balance
                                    );
                                }
                            }
                        }
                        Err(e) => log::error!("Failed to parse update: {}", e),
                    },
                    Some(Ok(Message::Binary(bin))) => log::info!("Received binary message: {:?}", bin),
                    Some(Ok(Message::Ping(_))) => log::info!("Received ping"),
                    Some(Ok(Message::Pong(_))) => log::info!("Received pong"),
                    Some(Ok(Message::Close(_))) => {
                        log::info!("WebSocket closed");
                        break;
                    }
                    Some(Ok(Message::Frame(_))) => log::info!("Received raw frame"),
                    Some(Err(e)) => {
                        log::error!("Error receiving message: {}", e);
                        break;
                    }
                    None => break,
                }
            }
            _ = &mut shutdown_signal => {
                log::info!("Received CTRL+C, shutting down gracefully...");
                break;
            }
        }
    }

    // Save trades to disk as JSON
    let trades_file = File::create("trades.json")?;
    serde_json::to_writer_pretty(trades_file, &trades)?;
    log::info!("Saved trades to trades.json");

    // Calculate and print final portfolio value
    let final_btc_price = order_book
        .bids
        .into_iter()
        .last()
        .map(|(p, _)| *p)
        .unwrap_or(0.0);
    let total_value = usdt_balance + btc_balance * final_btc_price;
    log::info!(
        "Final portfolio value: ${:.2} (USDT: ${:.2}, BTC: {:.8} @ {:.2} USDT)",
        total_value,
        usdt_balance,
        btc_balance,
        final_btc_price
    );

    Ok(())
}
