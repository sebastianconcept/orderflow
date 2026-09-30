//! Client requests that drive order lifecycle.
//!
//! When a matching engine receives work, it uses this module so NewLimit, NewMarket,
//! cancel, and replace are distinct shapes. NewLimit and NewMarket carry client
//! identifiers; the engine assigns OrderId on accept.

use crate::identity::{AccountId, ClientOrderId, InstrumentId, OrderId};
use crate::order::Side;
use crate::price::Price;
use crate::quantity::Quantity;

/// EngineCommand is a client request to change order state.
/// When a matching engine receives work, it uses this type so the variant is the
/// layout: NewLimit and NewMarket are separate, and New has no engine OrderId.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum EngineCommand {
    /// Request to open a limit order.
    /// The client names account, instrument, side, price, and size.
    NewLimit {
        account_id: AccountId,
        client_order_id: ClientOrderId,
        instrument_id: InstrumentId,
        side: Side,
        price: Price,
        quantity: Quantity,
    },

    /// Request to open a market order.
    /// The client names account, instrument, side, and size; Market has no price field.
    NewMarket {
        account_id: AccountId,
        client_order_id: ClientOrderId,
        instrument_id: InstrumentId,
        side: Side,
        quantity: Quantity,
    },

    /// Request to cancel by engine OrderId.
    /// The client names a resting order after accept.
    CancelByOrder { order_id: OrderId },

    /// Request to cancel by the opening New identifiers.
    /// The client names the original AccountId and ClientOrderId.
    CancelByClient {
        account_id: AccountId,
        client_order_id: ClientOrderId,
    },

    /// Request to amend price or quantity on a resting order.
    /// The client sends a fresh ClientOrderId for the replace.
    Replace {
        order_id: OrderId,
        client_order_id: ClientOrderId,
        price: Price,
        quantity: Quantity,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_copy<T: Copy + PartialEq + std::fmt::Debug>(value: T) {
        let copy = value;
        assert_eq!(value, copy);
    }

    #[test]
    fn engine_command_new_limit_has_client_order_id_and_no_engine_order_id() {
        // Given a NewLimit command for a limit buy order
        let account = AccountId::new(1);
        let client_order = ClientOrderId::new(7);
        let instrument = InstrumentId::new(2);
        let side = Side::Buy;
        let price = Price::new(100);
        let quantity = Quantity::new(10);

        // When we create a NewLimit command
        let command = EngineCommand::NewLimit {
            account_id: account,
            client_order_id: client_order,
            instrument_id: instrument,
            side,
            price,
            quantity,
        };

        // Then the command holds the client request fields and no OrderId
        let EngineCommand::NewLimit {
            account_id,
            client_order_id,
            instrument_id,
            side,
            price,
            quantity,
        } = command
        else {
            panic!("expected NewLimit");
        };
        assert_eq!(account_id.inner(), 1);
        assert_eq!(client_order_id.inner(), 7);
        assert_eq!(instrument_id.inner(), 2);
        assert_eq!(side, Side::Buy);
        assert_eq!(price.inner(), 100);
        assert_eq!(quantity.inner(), 10);
    }

    #[test]
    fn engine_command_new_market_has_no_price() {
        // Given a NewMarket command
        let command = EngineCommand::NewMarket {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(8),
            instrument_id: InstrumentId::new(2),
            side: Side::Sell,
            quantity: Quantity::new(5),
        };

        // When we read the command
        let EngineCommand::NewMarket {
            account_id,
            client_order_id,
            instrument_id,
            side,
            quantity,
        } = command
        else {
            panic!("expected NewMarket");
        };

        // Then the market request has size and no price field
        assert_eq!(account_id.inner(), 1);
        assert_eq!(client_order_id.inner(), 8);
        assert_eq!(instrument_id.inner(), 2);
        assert_eq!(side, Side::Sell);
        assert_eq!(quantity.inner(), 5);
    }

    #[test]
    fn engine_command_cancel_by_client_uses_account_and_client_order_id() {
        // Given a CancelByClient command
        let account = AccountId::new(1);
        let client_order = ClientOrderId::new(7);

        // When we create a CancelByClient command
        let command = EngineCommand::CancelByClient {
            account_id: account,
            client_order_id: client_order,
        };

        // Then the command holds the opening New identifiers
        let EngineCommand::CancelByClient {
            account_id,
            client_order_id,
        } = command
        else {
            panic!("expected CancelByClient");
        };
        assert_eq!(account_id.inner(), 1);
        assert_eq!(client_order_id.inner(), 7);
    }

    #[test]
    fn engine_command_replace_carries_order_id_and_fresh_client_order_id() {
        // Given a Replace command with a fresh client request identifier
        let order_id = OrderId::new(42);
        let fresh_client_order = ClientOrderId::new(99);

        // When we create a Replace command
        let command = EngineCommand::Replace {
            order_id,
            client_order_id: fresh_client_order,
            price: Price::new(105),
            quantity: Quantity::new(12),
        };

        // Then the command holds both the engine OrderId and the fresh ClientOrderId
        let EngineCommand::Replace {
            order_id,
            client_order_id,
            price,
            quantity,
        } = command
        else {
            panic!("expected Replace");
        };
        assert_eq!(order_id.inner(), 42);
        assert_eq!(client_order_id.inner(), 99);
        assert_eq!(price.inner(), 105);
        assert_eq!(quantity.inner(), 12);
        assert_ne!(order_id.inner(), fresh_client_order.inner());
    }

    #[test]
    fn engine_command_variants_implement_copy() {
        // Given each EngineCommand variant
        let new_limit = EngineCommand::NewLimit {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };
        let new_market = EngineCommand::NewMarket {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(8),
            instrument_id: InstrumentId::new(2),
            side: Side::Sell,
            quantity: Quantity::new(5),
        };
        let cancel_by_order = EngineCommand::CancelByOrder {
            order_id: OrderId::new(42),
        };
        let cancel_by_client = EngineCommand::CancelByClient {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
        };
        let replace = EngineCommand::Replace {
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(99),
            price: Price::new(105),
            quantity: Quantity::new(12),
        };

        // When we copy each command
        // Then the original and the copy compare equal
        assert_copy(new_limit);
        assert_copy(new_market);
        assert_copy(cancel_by_order);
        assert_copy(cancel_by_client);
        assert_copy(replace);
    }

    #[test]
    fn engine_command_equality() {
        // Given two NewLimit commands with the same fields
        let command_one = EngineCommand::NewLimit {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };
        let command_two = EngineCommand::NewLimit {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };
        let different_account = EngineCommand::NewLimit {
            account_id: AccountId::new(2),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };

        // When we compare them
        // Then equal commands match and a different account does not
        assert_eq!(command_one, command_two);
        assert_ne!(command_one, different_account);
    }
}
