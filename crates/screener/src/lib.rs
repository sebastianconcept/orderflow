// Screener placeholder

use engine_types::Order;

pub struct Screener;

impl Screener {
    pub fn new() -> Self {
        Self
    }

    pub fn allow(&self, _order: &Order) -> bool {
        true
    }
}

impl Default for Screener {
    fn default() -> Self {
        Self::new()
    }
}
