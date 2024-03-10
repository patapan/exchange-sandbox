mod exchange;
mod types;

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use env_logger;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use types::{Order, OrderStatus, Request, Side, Update};
use bybit::WebSocketApiClient;
use bybit::ws::spot;
use bybit::ws::response::SpotPublicResponse;

struct PriceData{
    bid: f64,
    ask: f64,
}

impl PriceData {
    pub fn init() -> PriceData {
        Self {
            bid: 0.0,
            ask: 0.0
        }
    }
}

struct Bot {
    sender: mpsc::Sender<Request>,
    receiver: mpsc::Receiver<Update>, // Track bot state.
    user_name: String,
    balance: u64,   // Amount in cash account
    inventory: u64, // Num tokens currently held
    margin_bps: u16,

    // orders 
    bids: BTreeSet<Order>,
    asks: BTreeSet<Order>,

    price_data: Arc<Mutex<PriceData>>,
}

impl Bot {
    pub fn init(sender: mpsc::Sender<Request>, receiver: mpsc::Receiver<Update>, price_data: Arc<Mutex<PriceData>>) -> Bot {
        Self {
            sender,
            receiver,
            user_name: "".to_string(),
            balance: 0,
            inventory: 0,
            margin_bps: 10,
            bids: BTreeSet::new(),
            asks: BTreeSet::new(),
            price_data
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
                },
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

    async fn place_order(&mut self, price: u64, size: u64, side: Side) {
        let r = Request::PlaceOrder {
            user_name: self.user_name.clone(),
            price,
            size,
            side,
        };
        let _ = self.sender.send(r).await;
    }

    async fn cancel_order(&mut self, order_id: usize) {
        let r = Request::CancelOrder { order_id };
        let _ = self.sender.send(r).await;
    }

    fn handle_updates(&mut self) {
        while let Ok(update) = self.receiver.try_recv() {
            match update {               
                Update::CreateUser { user_name, success } => if success { self.user_name = user_name },
                Update::Order { order_id, user_name, status } => if user_name == self.user_name { self.process_order(order_id, user_name, status) },
                Update::Deposit { user_name, amount } => todo!(),
                Update::Trade { price, size } => self.update_price_data(),
            }
        }
    }

    fn process_order(&self, order_id: usize, user_name: String, status: OrderStatus) {
        
    }

    fn update_price_data(&self) {
        
    }

    fn get_exchange_price(self) {
        // get and update price
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
    let bot = Bot::init(tx_request, rx_update, Arc::clone(&price_data));

    bot.start_bybit_poll(Arc::clone(&price_data)).await;


    loop {
        // bot.place_order(100, 20, Side::Bid).await;
        // bot.handle_updates();

        println!("Bid: {:?}, Ask:",  bot.price_data.lock().unwrap().bid);
        sleep(Duration::from_millis(1000)).await;
    }
}
