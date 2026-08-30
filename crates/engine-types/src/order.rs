//! Order and order-related types for the matching engine.
//!
//! This module provides domain types for representing orders in the matching
//! engine, including side (buy/sell), order type (limit/market), and the
//! complete Order struct with all necessary fields.

use crate::identity::{AccountId, ClientOrderId, InstrumentId, OrderId};
use crate::price::Price;
use crate::quantity::Quantity;

/// Order representation with integer fields for economic precision.
///
/// An `Order` represents a trading request in the matching engine. It contains
/// all necessary information to identify, route, and process the order.
///
/// # Fields
///
/// * `id` - Engine-assigned unique identifier for the order (populated after acceptance)
/// * `client_order_id` - Client-assigned unique identifier for the request
/// * `instrument_id` - Identifier for the tradable instrument
/// * `account_id` - Identifier for the trading account
/// * `side` - Whether this is a buy or sell order
/// * `order_type` - Whether this is a limit or market order
/// * `price` - Price in ticks (for limit orders; unused for market orders)
/// * `quantity` - Quantity in lots
///
/// # Examples
///
/// ```
/// use engine_types::{
///     AccountId, ClientOrderId, InstrumentId, Order, OrderType, Price, Quantity, Side,
/// };
///
/// // Create a limit buy order
/// let limit_order = Order::new(
///     1,
///     7,
///     2,
///     10,
///     Side::Buy,
///     OrderType::Limit,
///     Price::new(100),
///     Quantity::new(10),
/// );
///
/// // Create a market sell order
/// let market_order = Order::new(
///     2,
///     8,
///     3,
///     20,
///     Side::Sell,
///     OrderType::Market,
///     Price::new(0), // unused for market orders
///     Quantity::new(5),
/// );
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Order {
    /// Engine-assigned unique identifier for the order.
    pub id: OrderId,
    /// Client-assigned unique identifier for the request.
    pub client_order_id: ClientOrderId,
    /// Identifier for the tradable instrument.
    pub instrument_id: InstrumentId,
    /// Identifier for the trading account.
    pub account_id: AccountId,
    /// Whether this is a buy or sell order.
    pub side: Side,
    /// Whether this is a limit or market order.
    pub order_type: OrderType,
    /// Price in ticks (for limit orders; unused for market orders).
    pub price: Price,
    /// Quantity in lots.
    pub quantity: Quantity,
}

impl Order {
    /// Create a new Order with the given fields.
    ///
    /// # Arguments
    ///
    /// * `id` - Engine-assigned order ID (use 0 for new orders)
    /// * `client_order_id` - Client-request identifier
    /// * `instrument_id` - Instrument being traded
    /// * `account_id` - Account placing the order
    /// * `side` - Buy or Sell
    /// * `order_type` - Limit or Market
    /// * `price` - Price in ticks (0 for market orders)
    /// * `quantity` - Quantity in lots
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u64,
        client_order_id: u64,
        instrument_id: u64,
        account_id: u64,
        side: Side,
        order_type: OrderType,
        price: Price,
        quantity: Quantity,
    ) -> Self {
        Order {
            id: OrderId::new(id),
            client_order_id: ClientOrderId::new(client_order_id),
            instrument_id: InstrumentId::new(instrument_id),
            account_id: AccountId::new(account_id),
            side,
            order_type,
            price,
            quantity,
        }
    }

    /// Check if this is a limit order.
    ///
    /// # Examples
    ///
    /// ```
    /// use engine_types::{Order, OrderType, Side, Price, Quantity};
    ///
    /// let order = Order::new(1, 7, 2, 10, Side::Buy, OrderType::Limit, Price::new(100), Quantity::new(10));
    /// assert!(order.is_limit());
    ///
    /// let market_order = Order::new(2, 8, 3, 20, Side::Sell, OrderType::Market, Price::new(0), Quantity::new(5));
    /// assert!(!market_order.is_limit());
    /// ```
    pub fn is_limit(&self) -> bool {
        matches!(self.order_type, OrderType::Limit)
    }

    /// Check if this is a market order.
    ///
    /// # Examples
    ///
    /// ```
    /// use engine_types::{Order, OrderType, Side, Price, Quantity};
    ///
    /// let market_order = Order::new(2, 8, 3, 20, Side::Sell, OrderType::Market, Price::new(0), Quantity::new(5));
    /// assert!(market_order.is_market());
    ///
    /// let limit_order = Order::new(1, 7, 2, 10, Side::Buy, OrderType::Limit, Price::new(100), Quantity::new(10));
    /// assert!(!limit_order.is_market());
    /// ```
    pub fn is_market(&self) -> bool {
        matches!(self.order_type, OrderType::Market)
    }
}

/// Side of an order (buy or sell).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Side {
    /// Buy side of the market.
    Buy,
    /// Sell side of the market.
    Sell,
}

impl Side {
    /// Convert the side to a string representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use engine_types::Side;
    ///
    /// assert_eq!(Side::Buy.as_str(), "BUY");
    /// assert_eq!(Side::Sell.as_str(), "SELL");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Side::Buy => "BUY",
            Side::Sell => "SELL",
        }
    }
}

/// Type of order (limit or market).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OrderType {
    /// Limit order with a specific price.
    Limit,
    /// Market order executed at best available price.
    Market,
}

impl OrderType {
    /// Convert the order type to a string representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use engine_types::OrderType;
    ///
    /// assert_eq!(OrderType::Limit.as_str(), "LIMIT");
    /// assert_eq!(OrderType::Market.as_str(), "MARKET");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderType::Limit => "LIMIT",
            OrderType::Market => "MARKET",
        }
    }

    /// Check if this order type has a price field.
    ///
    /// # Examples
    ///
    /// ```
    /// use engine_types::OrderType;
    ///
    /// assert!(OrderType::Limit.has_price());
    /// assert!(!OrderType::Market.has_price());
    /// ```
    pub fn has_price(&self) -> bool {
        matches!(self, OrderType::Limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_order_holds_instrument_side_price_quantity() {
        // Given: a limit buy order for 10 lots at 100 ticks on instrument 2, account 10
        let expected_id = 1u64;
        let expected_client_order_id = 7u64;
        let expected_instrument_id = 2u64;
        let expected_account_id = 10u64;
        let expected_side = Side::Buy;
        let expected_order_type = OrderType::Limit;
        let expected_price = Price::new(100);
        let expected_quantity = Quantity::new(10);

        // When: we create the order
        let order = Order::new(
            expected_id,
            expected_client_order_id,
            expected_instrument_id,
            expected_account_id,
            expected_side,
            expected_order_type,
            expected_price,
            expected_quantity,
        );

        // Then: all fields are set correctly
        assert_eq!(order.id.inner(), expected_id);
        assert_eq!(order.client_order_id.inner(), expected_client_order_id);
        assert_eq!(order.instrument_id.inner(), expected_instrument_id);
        assert_eq!(order.account_id.inner(), expected_account_id);
        assert_eq!(order.side, expected_side);
        assert_eq!(order.order_type, expected_order_type);
        assert_eq!(order.price, expected_price);
        assert_eq!(order.quantity, expected_quantity);

        // Verify it's a limit order
        assert!(order.is_limit());
        assert!(!order.is_market());
    }

    #[test]
    fn market_order_type_uses_price_zero_as_unused() {
        // Given: a market sell order
        let expected_id = 2u64;
        let expected_client_order_id = 8u64;
        let expected_instrument_id = 3u64;
        let expected_account_id = 20u64;
        let expected_side = Side::Sell;
        let expected_order_type = OrderType::Market;
        let expected_price = Price::new(0); // unused for market orders
        let expected_quantity = Quantity::new(5);

        // When: we create the market order
        let order = Order::new(
            expected_id,
            expected_client_order_id,
            expected_instrument_id,
            expected_account_id,
            expected_side,
            expected_order_type,
            expected_price,
            expected_quantity,
        );

        // Then: all fields are set correctly
        assert_eq!(order.id.inner(), expected_id);
        assert_eq!(order.client_order_id.inner(), expected_client_order_id);
        assert_eq!(order.instrument_id.inner(), expected_instrument_id);
        assert_eq!(order.account_id.inner(), expected_account_id);
        assert_eq!(order.side, expected_side);
        assert_eq!(order.order_type, expected_order_type);
        assert_eq!(order.price, expected_price);
        assert_eq!(order.quantity, expected_quantity);

        // Verify it's a market order
        assert!(!order.is_limit());
        assert!(order.is_market());

        // Verify market orders use Price(0)
        assert_eq!(order.price.inner(), 0);
    }

    #[test]
    fn order_is_copy() {
        // Given: an Order instance
        let original_order = Order::new(
            1,
            7,
            2,
            10,
            Side::Buy,
            OrderType::Limit,
            Price::new(100),
            Quantity::new(10),
        );

        // When: we assign it to another variable
        let copied_order = original_order;

        // Then: both the original and copy can be used (Copy trait)
        // Verify original is still usable
        assert_eq!(original_order.id.inner(), 1);
        assert_eq!(original_order.side, Side::Buy);
        assert_eq!(original_order.order_type, OrderType::Limit);

        // Verify copy is correct
        assert_eq!(copied_order.id.inner(), 1);
        assert_eq!(copied_order.side, Side::Buy);
        assert_eq!(copied_order.order_type, OrderType::Limit);

        // Verify both are equal
        assert_eq!(original_order, copied_order);
    }

    #[test]
    fn side_enum_is_copy() {
        // Given: a Side instance
        let original_side = Side::Buy;

        // When: we assign it to another variable
        let copied_side = original_side;

        // Then: both can be used (Copy trait)
        assert_eq!(original_side, Side::Buy);
        assert_eq!(copied_side, Side::Buy);
    }

    #[test]
    fn order_type_enum_is_copy() {
        // Given: an OrderType instance
        let original_order_type = OrderType::Limit;

        // When: we assign it to another variable
        let copied_order_type = original_order_type;

        // Then: both can be used (Copy trait)
        assert_eq!(original_order_type, OrderType::Limit);
        assert_eq!(copied_order_type, OrderType::Limit);
    }

    #[test]
    fn side_as_str_returns_correct_value() {
        // Given: both sides
        let buy_side = Side::Buy;
        let sell_side = Side::Sell;

        // When: we convert to string
        let buy_str = buy_side.as_str();
        let sell_str = sell_side.as_str();

        // Then: correct string representations
        assert_eq!(buy_str, "BUY");
        assert_eq!(sell_str, "SELL");
    }

    #[test]
    fn order_type_as_str_returns_correct_value() {
        // Given: both order types
        let limit_order = OrderType::Limit;
        let market_order = OrderType::Market;

        // When: we convert to string
        let limit_str = limit_order.as_str();
        let market_str = market_order.as_str();

        // Then: correct string representations
        assert_eq!(limit_str, "LIMIT");
        assert_eq!(market_str, "MARKET");
    }

    #[test]
    fn order_type_has_price_check() {
        // Given: both order types
        let limit_order = OrderType::Limit;
        let market_order = OrderType::Market;

        // When: we check if they have price
        assert!(limit_order.has_price());
        assert!(!market_order.has_price());
    }
}
