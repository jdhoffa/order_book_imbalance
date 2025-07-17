mod models;

use arrow::array::{Float64Array, StringArray, UInt64Array};
use arrow::record_batch::RecordBatch;
use futures_util::StreamExt;
use models::{OrderBookSnapshot, OrderBookUpdate};
use parquet::arrow::arrow_writer::ArrowWriter;
use parquet::file::properties::WriterProperties;
use reqwest;
use std::fs::File;
use std::sync::Arc;
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

async fn save_snapshot_to_parquet(
    snapshot: &OrderBookSnapshot,
    symbol: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Create arrays for our columns
    let mut bid_prices = Vec::new();
    let mut bid_quantities = Vec::new();
    let mut ask_prices = Vec::new();
    let mut ask_quantities = Vec::new();
    let mut update_ids = Vec::new();
    let mut symbols = Vec::new();

    // Convert bids
    for bid in &snapshot.bids {
        bid_prices.push(bid[0].parse::<f64>()?);
        bid_quantities.push(bid[1].parse::<f64>()?);
        update_ids.push(snapshot.last_update_id);
        symbols.push(symbol);
    }

    // Convert asks
    for ask in &snapshot.asks {
        ask_prices.push(ask[0].parse::<f64>()?);
        ask_quantities.push(ask[1].parse::<f64>()?);
    }

    // Create Arrow arrays
    let bid_prices = Float64Array::from(bid_prices);
    let bid_quantities = Float64Array::from(bid_quantities);
    let ask_prices = Float64Array::from(ask_prices);
    let ask_quantities = Float64Array::from(ask_quantities);
    let update_ids = UInt64Array::from(update_ids);
    let symbols = StringArray::from(symbols);

    // Create schema
    let schema = Arc::new(arrow::datatypes::Schema::new(vec![
        arrow::datatypes::Field::new("symbol", arrow::datatypes::DataType::Utf8, false),
        arrow::datatypes::Field::new("update_id", arrow::datatypes::DataType::UInt64, false),
        arrow::datatypes::Field::new("bid_price", arrow::datatypes::DataType::Float64, false),
        arrow::datatypes::Field::new("bid_quantity", arrow::datatypes::DataType::Float64, false),
        arrow::datatypes::Field::new("ask_price", arrow::datatypes::DataType::Float64, false),
        arrow::datatypes::Field::new("ask_quantity", arrow::datatypes::DataType::Float64, false),
    ]));

    // Create RecordBatch
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(symbols),
            Arc::new(update_ids),
            Arc::new(bid_prices),
            Arc::new(bid_quantities),
            Arc::new(ask_prices),
            Arc::new(ask_quantities),
        ],
    )?;

    // Create filename with timestamp
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("orderbook_snapshot_{}_{}.parquet", symbol, timestamp);

    // Create Parquet file
    let file = File::create(&filename)?;
    let props = WriterProperties::builder().build();
    let mut writer = ArrowWriter::try_new(file, schema, Some(props))?;

    // Write batch
    writer.write(&batch)?;
    writer.close()?;

    Ok(filename)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch the initial order book snapshot
    let symbol = "btcusdt";

    // Initialize the snapshot
    let snapshot = fetch_order_book_snapshot(symbol).await?;
    println!(
        "Received snapshot with last update ID: {}",
        snapshot.last_update_id
    );

    // Save snapshot to Parquet file
    match save_snapshot_to_parquet(&snapshot, symbol).await {
        Ok(filename) => println!("Saved snapshot to {}", filename),
        Err(e) => eprintln!("Failed to save snapshot: {}", e),
    }

    // Connect to the Binance WebSocket stream
    let url = "wss://stream.binance.com:9443/ws/btcusdt@depth";
    let (ws_stream, _) = connect_async(url).await?;
    println!("WebSocket connection established");

    let (_, mut read) = ws_stream.split();

    // Handle incoming messages
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => match serde_json::from_str::<OrderBookUpdate>(&text) {
                Ok(update) => {
                    println!("Received order book update:");
                    println!("Event type: {}", update.event_type);
                    println!("Event time: {}", update.event_time);
                    println!("Symbol: {}", update.symbol);
                    println!(
                        "Update IDs: {} to {}",
                        update.first_update_id, update.final_update_id
                    );
                    println!("Bids:");
                    for bid in update.bids {
                        println!("  Price: {}, Quantity: {}", bid[0], bid[1]);
                    }
                    println!("Asks:");
                    for ask in update.asks {
                        println!("  Price: {}, Quantity: {}", ask[0], ask[1]);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse message: {}", e);
                    eprintln!("Raw message: {}", text);
                }
            },
            Ok(Message::Binary(bin)) => println!("Received binary message: {:?}", bin),
            Ok(Message::Ping(_)) => println!("Received ping"),
            Ok(Message::Pong(_)) => println!("Received pong"),
            Ok(Message::Close(_)) => {
                println!("WebSocket closed");
                break;
            }
            Ok(Message::Frame(_)) => println!("Received raw frame"),
            Err(e) => {
                println!("Error receiving message: {}", e);
                break;
            }
        }
    }
    Ok(())
}
