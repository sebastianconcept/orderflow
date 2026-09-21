//! Screener process entry.
//!
//! When this process starts, it checks one Order and exits.

use engine_types::{Order, OrderId};
use screener::Screener;

fn main() {
    let screener = Screener::new();
    let order = Order {
        id: OrderId::new(1),
        client_order_id: engine_types::ClientOrderId::new(100),
        instrument_id: engine_types::InstrumentId::new(1),
        account_id: engine_types::AccountId::new(50),
        side: engine_types::Side::Buy,
        order_type: engine_types::OrderType::Limit,
        price: engine_types::Price::new(1000),
        quantity: engine_types::Quantity::new(10),
    };
    let _ = screener.allow(&order);
    println!("screener program started");
}
