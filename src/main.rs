mod exchange;
mod types;

use core::time;
use std::borrow::BorrowMut;
use std::cmp::Ordering;
use std::collections::{BTreeSet, BinaryHeap, HashMap, HashSet};
use std::sync::{Arc, Mutex};

use bybit::ws::response::SpotPublicResponse;
use bybit::ws::spot;
use bybit::WebSocketApiClient;
use env_logger;
use std::collections::LinkedList;
use std::time::SystemTime;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use types::{Order, OrderStatus, Request, Side, Update};

struct PriceData {
    bid: f64,
    ask: f64,
}

impl PriceData {
    pub fn init() -> PriceData {
        Self { bid: 0.0, ask: 0.0 }
    }
}

struct TimestampedOrder {
    order: Order,
}

impl PartialEq for TimestampedOrder {
    fn eq(&self, other: &Self) -> bool {
        self.order.created == other.order.created
    }
}

impl Eq for TimestampedOrder {}

impl PartialOrd for TimestampedOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimestampedOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order.created.cmp(&other.order.created)
    }
}

struct Bot {
    sender: mpsc::Sender<Request>,
    receiver: mpsc::Receiver<Update>,
    user_name: String,
    balance: f64,   // Amount in USDC
    inventory: f64, // Num tokens currently held
    edge_bps: f64,  // Num bps to offer above bid/ask
    bybit_data: Arc<Mutex<PriceData>>,
    sandbox_price: f64,

    // Handle orders
    completed_orders: HashSet<usize>, // Orders are completed if they are filled, failed, or cancelled
    active_orders: BinaryHeap<TimestampedOrder>, // orders sorted by timestamp
}

impl Bot {
    pub fn init(
        sender: mpsc::Sender<Request>,
        receiver: mpsc::Receiver<Update>,
        bybit_data: Arc<Mutex<PriceData>>,
    ) -> Bot {
        Self {
            sender,
            receiver,
            user_name: "".to_string(),
            balance: 0.0,
            inventory: 0.0,
            edge_bps: 5.0,
            bybit_data,
            sandbox_price: 0.0,
            completed_orders: HashSet::new(),
            active_orders: BinaryHeap::new(),
        }
    }

    async fn start_bybit_poll(&self, price_data: Arc<Mutex<PriceData>>) {
        let price_data_clone = Arc::clone(&price_data);

        tokio::spawn(async move {
            let mut client = WebSocketApiClient::spot().build();
            client.subscribe_orderbook("SOLUSDT", spot::OrderbookDepth::Level1);

            let callback = move |res: SpotPublicResponse| match res {
                SpotPublicResponse::Orderbook(res) => {
                    if let Some(bid) = res.data.b.first() {
                        if let Some(ask) = res.data.a.first() {
                            let bid_price = bid.0.parse::<f64>().unwrap_or_default();
                            let ask_price = ask.0.parse::<f64>().unwrap_or_default();

                            let mut data = price_data_clone.lock().unwrap();
                            data.bid = bid_price;
                            data.ask = ask_price;

                            println!("Updated PriceData: bid = {}, ask = {}", data.bid, data.ask);
                        }
                    }
                }
                _ => (),
            };

            match client.run(callback) {
                Ok(_) => {}
                Err(e) => println!("Error running WebSocket client: {}", e),
            }
        });
    }

    async fn initialize_user_id(&self) {
        let request = Request::CreateUser {
            name: "bot".to_string(),
        };
        let _ = self.sender.send(request).await;
    }

    async fn cancel_order(&mut self, order_id: usize) {
        let r = Request::CancelOrder { order_id };
        let _ = self.sender.send(r).await;
    }

    fn handle_updates(&mut self) {
        while let Ok(update) = self.receiver.try_recv() {
            match update {
                Update::CreateUser { user_name, success } => {
                    if success {
                        self.user_name = user_name
                    }
                }
                Update::Order { order } => {
                    if order.user_name == self.user_name {
                        // We only really care about our own orders (atm)
                        self.process_order(order);
                    }
                }
                Update::Deposit {
                    user_name,
                    amount,
                    success,
                } => {
                    if user_name == self.user_name && success {
                        self.balance += amount
                    }
                }
                Update::Trade { price, size } => self.sandbox_price = price,
            }
        }
    }

    fn process_order(&mut self, order: Order) {
        // add pending orders to list
        match order.status {
            OrderStatus::Pending => {
                // we've hit the book, record order
                self.active_orders.push(TimestampedOrder { order });
            }
            OrderStatus::Filled => {
                self.completed_orders.insert(order.order_id);
            }
            OrderStatus::Cancelled => {
                // update balance with volume
                self.balance += order.price * order.size;
                self.completed_orders.insert(order.order_id);
            }
            OrderStatus::Failed => {
                // Failed means it could not be converted to pending/filled
                self.completed_orders.insert(order.order_id);
            }
        }
    }

    async fn update_positions(&mut self) {
        // open new bid position and record it.
        let bid_price = self.bybit_data.lock().unwrap().bid * (1.0 + self.edge_bps);
        let bid_size = ((self.balance / bid_price) * 0.01).ceil(); // use 1% of existing balance on this trade

        let _ = self.sender.send(Request::PlaceOrder {
            user_name: self.user_name.clone(),
            price: bid_price,
            size: bid_size,
            side: Side::Bid,
        }).await;

        let ask_price = self.bybit_data.lock().unwrap().ask * (1.0 + self.edge_bps);
        let ask_size = ((self.balance / ask_price) * 0.01).ceil(); // use 1% of existing balance on this trade

        // open asks if we have enough inventory
        if self.inventory >= ask_size {
            let _ = self.sender.send(Request::PlaceOrder {
                user_name: self.user_name.clone(),
                price: ask_price,
                size: ask_size,
                side: Side::Ask,
            }).await;
        }

        loop {
            // cancel any pending orders older than 60 seconds
            if self.active_orders.peek().unwrap().order.created < SystemTime::now() - Duration::new(10,0) {
                self.cancel_order(self.active_orders.peek().unwrap().order.order_id).await;
                self.active_orders.pop();
            // pop off any old orders (filled, cancelled, etc)
            } else if self.completed_orders.contains(&self.active_orders.peek().unwrap().order.order_id) {
                self.active_orders.pop();
            } else {
                break;
            }
        }
    }
}

/**
 * TODO::
 * Add logic for a bot to trade around some price point for any asset.
 *
 * Bot has to perform the following functionality:
 * - Grab some reference price: price can be grabbed from any public API / websocket i.e. Bybit SDK has functionality for this.
 * - Place and cancel - manage it's orders within some configurable spread from a price.
 * - Have some risk limit i.e. Cannot trade over $X position value.
 * - Whatever other logic you feel is relevant.
 */
#[tokio::main]
async fn main() {
    env_logger::init();

    let (tx_update, rx_update) = mpsc::channel::<Update>(1000);
    let (tx_request, rx_request) = mpsc::channel::<Request>(1000);

    tokio::spawn(async move { exchange::start(rx_request, tx_update).await });

    // Handle price data socket feed from bybit
    let price_data = Arc::new(Mutex::new(PriceData { bid: 0.0, ask: 0.0 }));
    let mut bot = Bot::init(tx_request, rx_update, Arc::clone(&price_data));

    bot.start_bybit_poll(Arc::clone(&price_data)).await;
    bot.initialize_user_id().await;

    loop {
        bot.handle_updates();
        bot.update_positions().await;

        // println!("Bid: {:?}, Ask:",  bot.price_data.lock().unwrap().bid);
        sleep(Duration::from_millis(1000)).await;
    }
}
