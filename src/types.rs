use std::cmp::Ordering;

#[derive(Debug, Clone, Copy)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub order_id: usize,
    pub user_id: u32,
    pub price: u64,
    pub size: u64,
    pub side: Side,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Bid,
    Ask,
}

// Implement comparators for Order struct

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.user_id == other.user_id
    }
}

impl Eq for Order {}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.side {
            Side::Bid => other
                .price
                .cmp(&self.price)
                .then_with(|| self.user_id.cmp(&other.user_id)),
            Side::Ask => self
                .price
                .cmp(&other.price)
                .then_with(|| self.user_id.cmp(&other.user_id)),
        }
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Define structs for service

#[derive(Debug)]
pub enum Request {
    PlaceOrder {
        user_id: u32,
        price: u64,
        size: u64,
        side: Side,
    },
    CancelOrder {
        order_id: usize,
    },
    Deposit {
        user: String,
        amount: u64,
    },
    CreateUser {
        name: String,
    },
}

#[derive(Debug)]
pub enum Update {
    Noop,
    Order { order_id: usize },
    Trade { price: u64, size: u64 },
    Cancel { order_id: usize },
    Deposit { amount: u64 },
    CreateUser { user_id: u32 },
}
