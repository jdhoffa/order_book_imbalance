use ordered_float::OrderedFloat;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct OrderBookSnapshot {
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64,
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
}

#[derive(Debug, Deserialize)]
pub struct OrderBookUpdate {
    #[serde(rename = "e")]
    pub _event_type: String,
    #[serde(rename = "E")]
    pub _event_time: u64,
    #[serde(rename = "s")]
    pub _symbol: String,
    #[serde(rename = "U")]
    pub _first_update_id: u64,
    #[serde(rename = "u")]
    pub final_update_id: u64,
    #[serde(rename = "b")]
    pub bids: Vec<[String; 2]>,
    #[serde(rename = "a")]
    pub asks: Vec<[String; 2]>,
}

pub struct OrderBook {
    pub bids: BTreeMap<OrderedFloat<f64>, f64>, // price => quantity
    pub asks: BTreeMap<OrderedFloat<f64>, f64>,
    pub last_update_id: u64,
}

impl From<OrderBookSnapshot> for OrderBook {
    fn from(snapshot: OrderBookSnapshot) -> Self {
        let bids = snapshot
            .bids
            .iter()
            .map(|b| {
                (
                    OrderedFloat(b[0].parse::<f64>().unwrap()),
                    b[1].parse::<f64>().unwrap(),
                )
            })
            .collect();
        let asks = snapshot
            .asks
            .iter()
            .map(|a| {
                (
                    OrderedFloat(a[0].parse::<f64>().unwrap()),
                    a[1].parse::<f64>().unwrap(),
                )
            })
            .collect();

        OrderBook {
            bids,
            asks,
            last_update_id: snapshot.last_update_id,
        }
    }
}

impl OrderBook {
    pub fn apply_update(&mut self, update: &OrderBookUpdate) {
        // Apply bids
        for bid in &update.bids {
            let price = bid[0].parse::<f64>().unwrap();
            let qty = bid[1].parse::<f64>().unwrap();
            if qty == 0.0 {
                self.bids.remove(&OrderedFloat(price));
            } else {
                self.bids.insert(OrderedFloat(price), qty);
            }
        }
        // Apply asks
        for ask in &update.asks {
            let price = ask[0].parse::<f64>().unwrap();
            let qty = ask[1].parse::<f64>().unwrap();
            if qty == 0.0 {
                self.asks.remove(&OrderedFloat(price));
            } else {
                self.asks.insert(OrderedFloat(price), qty);
            }
        }

        // Update last update ID
        self.last_update_id = update.final_update_id;
    }
}
