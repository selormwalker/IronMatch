use std::collections::{BTreeMap, VecDeque};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: Uuid,
    pub price: u64,
    pub quantity: u64,
    pub side: Side,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub price: u64,
    pub quantity: u64,
    pub timestamp: DateTime<Utc>,
}

pub struct OrderBook {
    pub symbol: String,
    pub bids: BTreeMap<u64, VecDeque<Order>>, // Descending: Key is price
    pub asks: BTreeMap<u64, VecDeque<Order>>, // Ascending: Key is price
}

impl OrderBook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn add_order(&mut self, mut order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        match order.side {
            Side::Buy => {
                while order.quantity > 0 {
                    if let Some(mut entry) = self.asks.first_entry() {
                        let ask_price = *entry.key();
                        if ask_price > order.price {
                            break;
                        }

                        let orders_at_price = entry.get_mut();
                        while order.quantity > 0 && !orders_at_price.is_empty() {
                            let mut matching_order = orders_at_price.pop_front().unwrap();
                            let fill_qty = std::cmp::min(order.quantity, matching_order.quantity);

                            trades.push(Trade {
                                buy_order_id: order.id,
                                sell_order_id: matching_order.id,
                                price: ask_price,
                                quantity: fill_qty,
                                timestamp: Utc::now(),
                            });

                            order.quantity -= fill_qty;
                            matching_order.quantity -= fill_qty;

                            if matching_order.quantity > 0 {
                                orders_at_price.push_front(matching_order);
                            }
                        }

                        if orders_at_price.is_empty() {
                            entry.remove();
                        }
                    } else {
                        break;
                    }
                }

                if order.quantity > 0 {
                    self.bids
                        .entry(order.price)
                        .or_insert_with(VecDeque::new)
                        .push_back(order);
                }
            }
            Side::Sell => {
                while order.quantity > 0 {
                    if let Some(mut entry) = self.bids.last_entry() {
                        let bid_price = *entry.key();
                        if bid_price < order.price {
                            break;
                        }

                        let orders_at_price = entry.get_mut();
                        while order.quantity > 0 && !orders_at_price.is_empty() {
                            let mut matching_order = orders_at_price.pop_front().unwrap();
                            let fill_qty = std::cmp::min(order.quantity, matching_order.quantity);

                            trades.push(Trade {
                                buy_order_id: matching_order.id,
                                sell_order_id: order.id,
                                price: bid_price,
                                quantity: fill_qty,
                                timestamp: Utc::now(),
                            });

                            order.quantity -= fill_qty;
                            matching_order.quantity -= fill_qty;

                            if matching_order.quantity > 0 {
                                orders_at_price.push_front(matching_order);
                            }
                        }

                        if orders_at_price.is_empty() {
                            entry.remove();
                        }
                    } else {
                        break;
                    }
                }

                if order.quantity > 0 {
                    self.asks
                        .entry(order.price)
                        .or_insert_with(VecDeque::new)
                        .push_back(order);
                }
            }
        }

        trades
    }
}

#[tokio::main]
async fn main() {
    println!("--- IronMatch High-Performance Matching Engine ---");
    let mut lob = OrderBook::new("BTC/USD".to_string());

    let o1 = Order {
        id: Uuid::new_v4(),
        price: 50000,
        quantity: 10,
        side: Side::Sell,
        timestamp: Utc::now(),
    };

    let o2 = Order {
        id: Uuid::new_v4(),
        price: 50000,
        quantity: 5,
        side: Side::Buy,
        timestamp: Utc::now(),
    };

    println!("Adding Sell Order: {} @ {}", o1.quantity, o1.price);
    lob.add_order(o1);

    println!("Adding Buy Order: {} @ {}", o2.quantity, o2.price);
    let trades = lob.add_order(o2);

    for trade in trades {
        println!(
            "TRADE EXECUTED: {} units @ {} (Buy ID: {}, Sell ID: {})",
            trade.quantity, trade.price, trade.buy_order_id, trade.sell_order_id
        );
    }

    println!("IronMatch Engine Standby. Foundation Established.");
}
