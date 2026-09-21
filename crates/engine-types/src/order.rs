//! Resting or incoming order with integer economic fields.
//!
//! When a pipeline stage carries work, it uses this module so side, order type,
//! price, and quantity travel as one Order.

use crate::identity::{AccountId, ClientOrderId, InstrumentId, OrderId};
use crate::price::Price;
use crate::quantity::Quantity;

/// Order is a trading request with integer price ticks and quantity lots.
/// When a pipeline stage carries work between stages, it uses this type so the
/// request is one Order.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Order {
    pub id: OrderId,
    pub client_order_id: ClientOrderId,
    pub instrument_id: InstrumentId,
    pub account_id: AccountId,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Price,
    pub quantity: Quantity,
}

impl Order {
    /// Answers an Order from its fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: OrderId,
        client_order_id: ClientOrderId,
        instrument_id: InstrumentId,
        account_id: AccountId,
        side: Side,
        order_type: OrderType,
        price: Price,
        quantity: Quantity,
    ) -> Self {
        Order {
            id,
            client_order_id,
            instrument_id,
            account_id,
            side,
            order_type,
            price,
            quantity,
        }
    }

    /// Answers whether this order is a limit order (price is live).
    pub fn is_limit(&self) -> bool {
        matches!(self.order_type, OrderType::Limit)
    }

    /// Answers whether this order is a market order (price is unused).
    pub fn is_market(&self) -> bool {
        matches!(self.order_type, OrderType::Market)
    }
}

/// Side is whether an order buys or sells.
/// When a command names the book side it targets, it uses this type so Buy is
/// the bid and Sell is the offer.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Side {
    /// Bid side of the book.
    Buy,
    /// Offer side of the book.
    Sell,
}

impl Side {
    /// Answers the display label of this Side (`BUY` or `SELL`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Side::Buy => "BUY",
            Side::Sell => "SELL",
        }
    }
}

/// OrderType is whether an order is limit or market.
/// When a command says whether price is live, it uses this type so Limit rests
/// at a price and Market takes the best available price.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OrderType {
    /// Order that rests at a price.
    Limit,
    /// Order that takes the best available price.
    Market,
}

impl OrderType {
    /// Answers the display label of this OrderType (`LIMIT` or `MARKET`).
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderType::Limit => "LIMIT",
            OrderType::Market => "MARKET",
        }
    }

    /// Answers whether this order type uses a live price (Limit yes, Market no).
    pub fn has_price(&self) -> bool {
        matches!(self, OrderType::Limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_order_holds_instrument_side_price_quantity() {
        // Given a limit buy order for 10 lots at 100 ticks
        let expected_id = 1u64;
        let expected_client_order_id = 7u64;
        let expected_instrument_id = 2u64;
        let expected_account_id = 10u64;
        let expected_side = Side::Buy;
        let expected_order_type = OrderType::Limit;
        let expected_price = Price::new(100);
        let expected_quantity = Quantity::new(10);

        // When we create the order
        let order = Order::new(
            OrderId::new(expected_id),
            ClientOrderId::new(expected_client_order_id),
            InstrumentId::new(expected_instrument_id),
            AccountId::new(expected_account_id),
            expected_side,
            expected_order_type,
            expected_price,
            expected_quantity,
        );

        // Then all fields match and the order is a limit order
        assert_eq!(order.id.inner(), expected_id);
        assert_eq!(order.client_order_id.inner(), expected_client_order_id);
        assert_eq!(order.instrument_id.inner(), expected_instrument_id);
        assert_eq!(order.account_id.inner(), expected_account_id);
        assert_eq!(order.side, expected_side);
        assert_eq!(order.order_type, expected_order_type);
        assert_eq!(order.price, expected_price);
        assert_eq!(order.quantity, expected_quantity);
        assert!(order.is_limit());
        assert!(!order.is_market());
    }

    #[test]
    fn market_order_type_uses_price_zero_as_unused() {
        // Given a market sell order with unused price zero
        let order = Order::new(
            OrderId::new(2),
            ClientOrderId::new(8),
            InstrumentId::new(3),
            AccountId::new(20),
            Side::Sell,
            OrderType::Market,
            Price::new(0),
            Quantity::new(5),
        );

        // When we inspect the order
        // Then it is a market order and price is zero
        assert!(!order.is_limit());
        assert!(order.is_market());
        assert_eq!(order.price.inner(), 0);
        assert_eq!(order.quantity.inner(), 5);
    }

    #[test]
    fn order_is_copy() {
        // Given an Order
        let original_order = Order::new(
            OrderId::new(1),
            ClientOrderId::new(7),
            InstrumentId::new(2),
            AccountId::new(10),
            Side::Buy,
            OrderType::Limit,
            Price::new(100),
            Quantity::new(10),
        );

        // When we assign it to another variable
        let copied_order = original_order;

        // Then both the original and copy can be used
        assert_eq!(original_order, copied_order);
        assert_eq!(original_order.id.inner(), 1);
        assert_eq!(copied_order.id.inner(), 1);
    }

    #[test]
    fn side_and_order_type_are_copy() {
        // Given a Side and an OrderType
        let original_side = Side::Buy;
        let original_order_type = OrderType::Limit;

        // When we assign each to another variable
        let copied_side = original_side;
        let copied_order_type = original_order_type;

        // Then both the original and copy can be used
        assert_eq!(original_side, copied_side);
        assert_eq!(original_order_type, copied_order_type);
    }

    #[test]
    fn side_as_str_returns_correct_value() {
        // Given both sides
        // When we convert them to strings
        // Then the labels are BUY and SELL
        assert_eq!(Side::Buy.as_str(), "BUY");
        assert_eq!(Side::Sell.as_str(), "SELL");
    }

    #[test]
    fn order_type_as_str_and_has_price() {
        // Given both order types
        // When we convert them to strings and check has_price
        // Then limit has a live price and market does not
        assert_eq!(OrderType::Limit.as_str(), "LIMIT");
        assert_eq!(OrderType::Market.as_str(), "MARKET");
        assert!(OrderType::Limit.has_price());
        assert!(!OrderType::Market.has_price());
    }
}
