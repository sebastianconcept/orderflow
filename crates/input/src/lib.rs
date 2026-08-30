// Input gateway – placeholder

use engine_types::Order;

pub struct InputGateway;
impl InputGateway {
    pub fn new() -> Self {
        Self
    }
    pub fn receive(&self) -> Option<Order> {
        None
    }
}
