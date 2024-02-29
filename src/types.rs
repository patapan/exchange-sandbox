#[derive(Debug)]
pub struct Order {
    user_id: u32,
    price: u64,
    size: u32,
    side: Side,
}

#[derive(Debug)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug)]
pub enum Request {
    PlaceOrder {
        user_id: u32,
        price: u64,
        size: u64,
        side: Side,
    },
    CancelOrder {
        order_id: u64,
    },
    Deposit {
        amount: u64,
    },
    CreateUser {
        name: String,
    },
}

#[derive(Debug)]
pub enum Update {
    Noop,
    Order { order_id: u32 },
    Trade { price: u64, size: u64 },
    Cancel { order_id: u32 },
    CreateUser { user_id: u32 },
    Deposit { amount: u64 },
}
