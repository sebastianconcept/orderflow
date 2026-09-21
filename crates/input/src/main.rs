//! Input process entry.
//!
//! When this process starts, it polls InputGateway once and exits.

use input::InputGateway;

fn main() {
    let gateway = InputGateway::new();
    let _ = gateway.receive();
    println!("input program started");
}
