use input::InputGateway;

fn main() {
    let gateway = InputGateway::new();
    let _ = gateway.receive();
    println!("input program started");
}
