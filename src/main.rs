mod exchange;
mod types;

use env_logger;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use types::{Request, Side, Update};
struct Bot {
    sender: mpsc::Sender<Request>,
    receiver: mpsc::Receiver<Update>, // Track bot state.
    user_name: String,
    balance: u64,   // Amount in cash account
    inventory: u64, // Num tokens currently held
}

impl Bot {
    pub fn init(sender: mpsc::Sender<Request>, receiver: mpsc::Receiver<Update>) -> Bot {
        Self {
            sender,
            receiver,
            user_name: "",
            balance: 0,
            inventory: 0,
        }
    }

    async fn initialize_user_id(&self) {
        let request = Request::CreateUser {
            name: "bot".to_string(),
        };
        let _ = self.sender.send(request).await;
    }

    async fn place_order(self, price: u64, size: u64, side: Side) {
        let r = Request::PlaceOrder {
            user_name: self.user_name,
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
                Update::Order { order_id, user_name, status } => if user_name == self.user_name { process_order(order_id, user_name, status) },
                Update::Deposit { user_name, amount } => todo!(),
                Update::Trade { price, size } => update_price(),
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

    let mut bot = Bot::init(tx_request, rx_update);

    loop {
        bot.place_order(100, 20, Side::Bid).await;
        bot.handle_updates();
        sleep(Duration::from_millis(500)).await;
    }
}
