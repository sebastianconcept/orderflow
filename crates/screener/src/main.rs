use engine_types::{Order, OrderId};
use screener::Screener;

fn main() {
    let screener = Screener::new();
    let order = Order {
        id: OrderId::new(1),
        quantity: 10.0,
    };
    let _ = screener.allow(&order);
    println!("screener program started");
}
