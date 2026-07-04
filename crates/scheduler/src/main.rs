use engine_types::{Order, OrderId};
use scheduler::Scheduler;

fn main() {
    let scheduler = Scheduler::new();
    let order = Order {
        id: OrderId::new(1),
        quantity: 10.0,
    };
    let _ = scheduler.schedule(order);
    println!("scheduler program started");
}
