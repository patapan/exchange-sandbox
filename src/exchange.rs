use tokio::sync::mpsc;

use crate::types::*;
use std::collections::{BTreeSet, HashMap};

/*
 * TODO:
 * Implement functionality here to track the state required for an exchange
 * The sample code here is a starting point but functionality includes:
 * - Tracking order state correctly
 * - Simple order matching engine logic
 * - Tracking user state
 * Feel free to use any data structures you wish.
 */

struct Exchange {
    // Red-black tree representing the bids in the order book.
    pub bids: BTreeSet<Order>,

    // Red-black tree representing the asks in the order book.
    pub asks: BTreeSet<Order>,

    // Map from user name to account balance
    pub deposits: HashMap<String, u64>,

    // Record of all active and completed orders through the book
    pub orders: Vec<Order>,
}

impl Exchange {
    // Return a vector of Orders
    fn place_order(&mut self, user_id: u32, price: u64, size: u64, side: Side) -> Vec<Update> {
        // Add order to DB
        let order_id = self.orders.len();
        self.orders.push(Order {
            order_id,
            user_id,
            price,
            size,
            side,
            status: OrderStatus::Pending,
        });

        // Track updates for event propogation
        let mut updates = Vec::new();

        let mut order_size_remaining = size;

        // Choose the book to match against based on the side of the incoming order
        let book_to_match = match side {
            Side::Bid => &mut self.asks,
            Side::Ask => &mut self.bids,
        };

        // Attempt to match the order with orders in the opposite book
        let matched_orders: Vec<usize> = Vec::new();
        for order in book_to_match.iter() {
            // Match condition: For bids, incoming price must be >= book price; for asks, <=
            let match_condition = match side {
                Side::Bid => price >= order.price,
                Side::Ask => price <= order.price,
            };

            if match_condition && order_size_remaining > 0 {
                let trade_size = std::cmp::min(order_size_remaining, order.size);
                order_size_remaining -= trade_size;

                // Record trade event
                updates.push(Update::Trade { price, size });

                if order_size_remaining == 0 {
                    break;
                }
            }
        }

        // Remove matched orders from the book
        for matched_order_id in matched_orders {
            book_to_match.remove(&self.orders[matched_order_id]);
            self.orders[matched_order_id as usize].status = OrderStatus::Filled;
        }

        // If there's a remaining unmatched portion of the order, add it to the correct book
        if order_size_remaining > 0 {
            let remaining_order = Order {
                order_id,
                user_id,
                price,
                size: order_size_remaining,
                side,
                status: OrderStatus::Pending,
            };
            match side {
                Side::Bid => {
                    self.bids.insert(remaining_order.clone());
                }
                Side::Ask => {
                    self.asks.insert(remaining_order.clone());
                }
            }
            self.orders[order_id] = remaining_order;
        } else {
            self.orders[order_id].status = OrderStatus::Filled;
        }

        updates.push(Update::Order { order_id: order_id });

        return updates;
    }

    fn deposit(&mut self, user: String, amount: u64) -> Vec<Update> {
        *self.deposits.entry(user).or_insert(0) += amount;
        return vec![Update::Deposit { amount }];
    }

    fn cancel_order(&mut self, order_id: usize) -> Vec<Update> {
        // currently O(lg N) - Could also use a map to optimize further, however I will leave that as tech debt for now.
        if let Some(order) = self.orders.get_mut(order_id) {
            match order.side {
                Side::Bid => {
                    self.bids.remove(&order);
                }
                Side::Ask => {
                    self.asks.remove(&order);
                }
            }
            order.status = OrderStatus::Cancelled;
        }

        return vec![Update::Cancel { order_id }];
    }
}

pub async fn start(mut receiver: mpsc::Receiver<Request>, sender: mpsc::Sender<Update>) {
    let mut exchange = Exchange {
        bids: BTreeSet::new(),
        asks: BTreeSet::new(),
        deposits: HashMap::new(),
        orders: Vec::new(),
    };
    while let Some(r) = receiver.recv().await {
        println!("Received {:?}", r);

        let response = match r {
            Request::CancelOrder { order_id } => exchange.cancel_order(order_id),
            Request::CreateUser { name } => exchange.deposit(name, 0),
            Request::Deposit { user, amount } => exchange.deposit(user, amount),
            Request::PlaceOrder {
                user_id,
                price,
                size,
                side,
            } => exchange.place_order(user_id, price, size, side),
        };

        for x in response {
            let _ = sender.send(x).await;
        }
    }
}
