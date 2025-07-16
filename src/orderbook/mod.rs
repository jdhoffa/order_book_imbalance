use crate::models::{OrderBookSnapshot, OrderBookUpdate};
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::time::interval;

#[derive(Debug)]
pub struct OrderBook {
    bids: BTreeMap<f64, f64>,
    asks: BTreeMap<f64, f64>,
    depth: usize,
    update_frequency: Duration,
}

impl OrderBook {
    pub fn new(depth: usize, update_frequency: Duration) -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            depth,
            update_frequency,
        }
    }

    // Initialize from snapshot
    pub fn initialize_from_snapshot(&mut self, snapshot: &OrderBookSnapshot) {
        self.bids.clear();
        self.asks.clear();

        for bid in &snapshot.bids {
            let price = bid[0].parse::<f64>().unwrap();
            let quantity = bid[1].parse::<f64>().unwrap();
            if quantity > 0.0 {
                self.bids.insert(price, quantity);
            }
        }

        for ask in &snapshot.asks {
            let price = ask[0].parse::<f64>().unwrap();
            let quantity = ask[1].parse::<f64>().unwrap();
            if quantity > 0.0 {
                self.asks.insert(price, quantity);
            }
        }
    }

    // Apply update from WebSocket
    pub fn apply_update(&mut self, update: &OrderBookUpdate) {
        // Process bids
        for bid in &update.bids {
            let price = bid[0].parse::<f64>().unwrap();
            let quantity = bid[1].parse::<f64>().unwrap();

            if quantity > 0.0 {
                self.bids.insert(price, quantity);
            } else {
                self.bids.remove(&price);
            }
        }

        // Process asks
        for ask in &update.asks {
            let price = ask[0].parse::<f64>().unwrap();
            let quantity = ask[1].parse::<f64>().unwrap();

            if quantity > 0.0 {
                self.asks.insert(price, quantity);
            } else {
                self.asks.remove(&price);
            }
        }
    }

    // Calculate imbalance
    pub fn calculate_imbalance(&self) -> f64 {
        let bid_volume: f64 = self
            .bids
            .iter()
            .rev() // Reverse for highest bids first
            .take(self.depth)
            .map(|(_, &quantity)| quantity)
            .sum();

        let ask_volume: f64 = self
            .asks
            .iter() // Already sorted by lowest asks first
            .take(self.depth)
            .map(|(_, &quantity)| quantity)
            .sum();

        let total_volume = bid_volume + ask_volume;
        if total_volume > 0.0 {
            (bid_volume - ask_volume) / total_volume
        } else {
            0.0
        }
    }

    // Start imbalance monitoring
    pub async fn monitor_imbalance(&self) {
        let mut interval = interval(self.update_frequency);

        loop {
            interval.tick().await;
            let imbalance = self.calculate_imbalance();
            println!(
                "Order Book Imbalance (top {} levels): {:.4}",
                self.depth, imbalance
            );
        }
    }

    // Debug printing
    pub fn print_state(&self) {
        println!("Order Book State:");
        println!("Top {} Bids:", self.depth);
        for (price, quantity) in self.bids.iter().rev().take(self.depth) {
            println!("  Price: {:.2}, Quantity: {:.8}", price, quantity);
        }
        println!("Top {} Asks:", self.depth);
        for (price, quantity) in self.asks.iter().take(self.depth) {
            println!("  Price: {:.2}, Quantity: {:.8}", price, quantity);
        }
    }
}
