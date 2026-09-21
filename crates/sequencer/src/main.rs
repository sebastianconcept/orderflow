//! Sequencer process entry.
//!
//! When this process starts, it stamps one EngineCommand and exits.

use engine_types::{AccountId, ClientOrderId, EngineCommand, InstrumentId, Price, Quantity, Side};
use sequencer::Sequencer;

fn main() {
    let mut sequencer = Sequencer::new();
    let command = EngineCommand::NewLimit {
        account_id: AccountId::new(50),
        client_order_id: ClientOrderId::new(100),
        instrument_id: InstrumentId::new(1),
        side: Side::Buy,
        price: Price::new(1000),
        quantity: Quantity::new(10),
    };
    let _ = sequencer.stamp(command);
    println!("sequencer program started");
}
