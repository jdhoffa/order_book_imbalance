use futures_util::StreamExt;
use reqwest;
use serde::Deserialize;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Debug, Deserialize)]
struct OrderBookSnapshot {
    #[serde(rename = "lastUpdateId")]
    last_update_id: u64,
    #[serde(rename = "bids")]
    _bids: Vec<[String; 2]>,
    #[serde(rename = "asks")]
    _asks: Vec<[String; 2]>,
}

#[derive(Debug, Deserialize)]
struct OrderBookUpdate {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "E")]
    event_time: u64,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "U")]
    first_update_id: u64,
    #[serde(rename = "u")]
    final_update_id: u64,
    #[serde(rename = "b")]
    bids: Vec<[String; 2]>,
    #[serde(rename = "a")]
    asks: Vec<[String; 2]>,
}

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
