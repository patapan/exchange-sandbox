mod exchange;
mod types;

use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use types::{Request, Side, Update};

struct Bot {
    sender: mpsc::Sender<Request>,
    receiver: mpsc::Receiver<Update>, // Track bot state.
}

impl Bot {
    pub fn init(sender: mpsc::Sender<Request>, receiver: mpsc::Receiver<Update>) -> Bot {
        Self { sender, receiver }
    }

    fn user_id(&self) -> u32 {
        // Hack.
        0
    }

    async fn place_order(&mut self, price: u64, size: u64, side: Side) {
        let r = Request::PlaceOrder {
            user_id: self.user_id(),
            price,
            size,
            side,
        };
        let _ = self.sender.send(r).await;
    }

    async fn cancel_order(&mut self, order_id: u64) {
        let r = Request::CancelOrder { order_id };
        let _ = self.sender.send(r).await;
    }

    fn handle_updates(&mut self) {
        while let Ok(update) = self.receiver.try_recv() {
            // TODO:: Handle updates
            println!("Update = {:?}", update);
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
