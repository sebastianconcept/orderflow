use engine_types::{Order, OrderId};
use sequencer::Sequencer;

fn main() {
    let sequencer = Sequencer::new();
    let order = Order {
        id: OrderId::new(1),
        quantity: 10.0,
    };
    let _ = sequencer.process(order);
    println!("sequencer program started");
}
