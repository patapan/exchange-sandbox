use crate::types::*;
use std::collections::{BTreeSet, HashMap};
use tokio::sync::mpsc;

struct Exchange {
    // Red-black tree representing the bids in the order book.
    pub bids: BTreeSet<Order>,

    // Red-black tree representing the asks in the order book.
    pub asks: BTreeSet<Order>,

    // Map from user name to account balance
    pub deposits: HashMap<String, f64>,

    // Record of all active and completed orders through the book
    pub orders: Vec<Order>,
}

impl Exchange {
    // Return a vector of Orders
    fn place_order(&mut self, user_name: String, price: f64, size: f64, side: Side) -> Vec<Update> {
        let order_id = self.orders.len();
        if *self.deposits.entry(user_name.clone()).or_insert(0.0) < price * size {
            // User doesn't have enough money to place an order
            return vec![Update::Order{user_name: user_name.clone(), order_id, status: OrderStatus::Failed}];
        }

        self.orders.push(Order {
            order_id,
            user_name: user_name.clone(),
            price,
            size,
            side,
            status: OrderStatus::Pending,
        });

        // Track updates for event propogation
        let mut updates = vec![Update::Order {
            user_name: user_name.clone(),
            order_id,
            status: OrderStatus::Pending,
        }];

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

            if match_condition && order_size_remaining > 0.0 {
                let trade_size = order_size_remaining.min(order.size);
                order_size_remaining -= trade_size;

                // Record trade event
                updates.push(Update::Trade { price, size });

                if order_size_remaining == 0.0 {
                    break;
                }
            }
        }

        // Remove matched orders from the book
        for matched_order_id in matched_orders {
            book_to_match.remove(&self.orders[matched_order_id]);
            self.orders[matched_order_id].status = OrderStatus::Filled;
            updates.push(Update::Order {
                user_name: self.orders[matched_order_id].user_name.clone(),
                order_id: matched_order_id,
                status: OrderStatus::Filled,
            })
        }

        // If there's a remaining unmatched portion of the order, add it to the correct book
        if order_size_remaining > 0.0 {
            let remaining_order = Order {
                order_id,
                user_name,
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
            updates.push(Update::Order {
                user_name: user_name,
                order_id: order_id,
                status: OrderStatus::Filled,
            });
        }

        return updates;
    }

    fn create_user(&mut self, user_name: String) -> Vec<Update> {
        let user_already_exists = self.deposits.contains_key(&user_name);
        if !user_already_exists {
            self.deposits.insert(user_name.clone(), 0.0);
        }
        vec![Update::CreateUser {
            user_name: user_name,
            success: !user_already_exists,
        }]
    }

    fn deposit(&mut self, user_name: String, amount: f64) -> Vec<Update> {
        let success = amount >= 0.0;
        if success {
            *self.deposits.entry(user_name.clone()).or_insert(0.0) += amount;
        }
        vec![Update::Deposit {
            user_name,
            amount,
            success,
        }]
    }

    fn cancel_order(&mut self, order_id: usize) -> Vec<Update> {
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
            return vec![Update::Order {
                order_id,
                user_name: self.orders[order_id].user_name.clone(),
                status: OrderStatus::Cancelled,
            }];
        }
        // else unsuccessful cancel, Noop
        return Vec::new();
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
            Request::CreateUser { name } => exchange.create_user(name),
            Request::Deposit { user, amount } => exchange.deposit(user, amount),
            Request::PlaceOrder {
                user_name,
                price,
                size,
                side,
            } => exchange.place_order(user_name, price, size, side),
        };

        for x in response {
            let _ = sender.send(x).await;
        }
    }
}
