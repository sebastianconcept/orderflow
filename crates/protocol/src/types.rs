// use rkyv::{Archive, Deserialize, Serialize};

// #[derive(Archive, Deserialize, Serialize, Debug)]
// pub struct OrderId(u64);

// impl OrderId {
//     pub fn new(id: u64) -> Self {
//         OrderId(id)
//     }
// }

// #[derive(Archive, Deserialize, Serialize, Debug)]
// pub struct Order {
//     pub id: OrderId,
//     pub quantity: f64,
// }

// #[derive(Archive, Deserialize, Serialize, Debug)]
// pub struct Execution {
//     pub order_id: OrderId,
//     pub filled_quantity: f64,
// }
