use rand::prelude::*;
use tokio::sync::mpsc;

use crate::types::*;

/*
 * TODO:
 * Implement functionality here to track the state required for an exchange
 * The sample code here is a starting point but functionality includes:
 * - Tracking order state correctly
 * - Simple order matching engine logic
 * - Tracking user state
 * Feel free to use any data structures you wish.
 */

struct Exchange {}

pub async fn start(mut receiver: mpsc::Receiver<Request>, sender: mpsc::Sender<Update>) {
    let exchange = Exchange {};
    while let Some(r) = receiver.recv().await {
        println!("Received {:?}", r);
        let mut results = vec![];
        match r {
            Request::CreateUser { .. } => {}
            Request::Deposit { .. } => {}
            Request::PlaceOrder { price, size, .. } => {
                // Synthetically generate a trade event for the bot as a hack.
                // If doing margining piece, store the user state appropriately.
                match generate_trade_event(price, size) {
                    Some(x) => results.push(x),
                    None => {}
                };
                // Handle remaining order tracking / matching.
            }
            Request::CancelOrder { .. } => {}
        }

        for x in results {
            let _ = sender.send(x).await;
        }
    }
}

fn generate_trade_event(price: u64, size: u64) -> Option<Update> {
    match rand::random() {
        true => {
            let size = rand::thread_rng().gen_range(1..size);
            Some(Update::Trade { price, size })
        }
        false => None,
    }
}
