use std::cmp::Ordering;

#[derive(Debug, Clone, Copy)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    Failed
}

#[derive(Debug, Clone)]
pub struct Order {
    pub order_id: usize,
    pub user_name: String,
    pub price: f64,
    pub size: f64,
    pub side: Side,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Bid,
    Ask,
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.user_name == other.user_name
    }
}

impl Eq for Order {}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.side {
            Side::Bid => other
                .price
                .partial_cmp(&self.price)
                .unwrap_or(Ordering::Less)
                .then_with(|| self.user_name.cmp(&other.user_name)),
            Side::Ask => self
                .price
                .partial_cmp(&other.price)
                .unwrap_or(Ordering::Greater)
                .then_with(|| self.user_name.cmp(&other.user_name)),
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
        user_name: String,
        price: f64,
        size: f64,
        side: Side,
    },
    CancelOrder {
        order_id: usize,
    },
    Deposit {
        user: String,
        amount: f64,
    },
    CreateUser {
        name: String,
    },
}

#[derive(Debug)]
pub enum Update {
    Order {
        user_name: String,
        order_id: usize,
        status: OrderStatus,
    }, // Change to order state
    Trade {
        price: f64,
        size: f64,
    }, // A trade has occurred
    Deposit {
        user_name: String,
        amount: f64,
        success: bool,
    },
    CreateUser {
        user_name: String,
        success: bool,
    },
}
