//! Engine commands for order lifecycle management.
//!
//! This module provides the command types used to interact with the matching engine.
//! Commands represent client requests (New, Cancel, Replace) that drive order lifecycle
//! state transitions.
//!
//! # Commands Overview
//!
//! - [`EngineCommand::New`]: Submit a new order to the matching engine.
//! - [`EngineCommand::CancelByOrder`]: Cancel an existing order by its engine-assigned ID.
//! - [`EngineCommand::CancelByClient`]: Cancel an order by its client-assigned ID.
//! - [`EngineCommand::Replace`]: Replace an existing order with modified price/quantity.
//!
//! # Key Design Decisions
//!
//! - [`EngineCommand::New`] does NOT include an `OrderId` field because the engine assigns
//!   this identifier upon acceptance. The client provides only a `ClientOrderId` for
//!   request correlation.
//! - [`EngineCommand::CancelByClient`] uses the original `(AccountId, ClientOrderId)` pair
//!   from the opening `New` command to identify which order to cancel.
//! - [`EngineCommand::Replace`] takes a fresh `ClientOrderId` because replacement creates
//!   a new logical request, even though it amends the same order.
//!
//! # Copy Semantics
//!
//! All command variants implement [`Copy`] because they contain only integer types that
//! are `Copy`. This allows efficient reuse without ownership transfer.
//!
//! # Examples
//!
//! ```ignore
//! use engine_types::{
//!     AccountId, ClientOrderId, EngineCommand, InstrumentId, OrderId,
//!     OrderType, Price, Quantity, Side,
//! };
//!
//! // Create a new limit buy order
//! let new_order = EngineCommand::New {
//!     account_id: AccountId::new(1),
//!     client_order_id: ClientOrderId::new(7),
//!     instrument_id: InstrumentId::new(2),
//!     side: Side::Buy,
//!     order_type: OrderType::Limit,
//!     price: Price::new(100),
//!     quantity: Quantity::new(10),
//! };
//!
//! // Cancel by client order ID
//! let cancel_order = EngineCommand::CancelByClient {
//!     account_id: AccountId::new(1),
//!     client_order_id: ClientOrderId::new(7),
//! };
//!
//! // Replace an order with new price and quantity
//! let replace_order = EngineCommand::Replace {
//!     order_id: OrderId::new(1),
//!     client_order_id: ClientOrderId::new(8), // fresh ID for this replace request
//!     price: Price::new(105),
//!     quantity: Quantity::new(12),
//! };
//! ```

use crate::identity::{AccountId, ClientOrderId, InstrumentId, OrderId};
use crate::order::{OrderType, Side};
use crate::price::Price;
use crate::quantity::Quantity;

/// Engine command representing an order lifecycle operation.
///
/// Commands are submitted by clients to request state changes in the matching engine.
/// Each command variant serves a distinct purpose:
///
/// - [`EngineCommand::New`]: Submit a new order for execution.
/// - [`EngineCommand::CancelByOrder`]: Cancel an existing order by its engine ID.
/// - [`EngineCommand::CancelByClient`]: Cancel an order using the original client request ID.
/// - [`EngineCommand::Replace`]: Amended a resting order with new price and/or quantity.
///
/// # Usage
///
/// Commands are processed by the [`MatchingEngine`](crate::MatchingEngine) trait's
/// `process` method, which appends resulting [`EngineEvent`](crate::EngineEvent)s
/// to a caller-owned buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EngineCommand {
    /// Submit a new order to the matching engine.
    ///
    /// # Fields
    ///
    /// * `account_id` - Identifier for the trading account placing the order.
    /// * `client_order_id` - Client-assigned unique identifier for this request.
    ///   The engine does NOT assign an order ID at this stage; that happens on acceptance.
    /// * `instrument_id` - Identifier for the tradable instrument.
    /// * `side` - Whether this is a buy or sell order.
    /// * `order_type` - Whether this is a limit or market order.
    /// * `price` - Price in ticks (unused for market orders; use `Price(0)`).
    /// * `quantity` - Quantity in lots.
    ///
    /// # Notes
    ///
    /// This variant intentionally does NOT include an `OrderId` field because the engine
    /// assigns order IDs only after acceptance. Including a client-provided order ID here
    /// would blur the distinction between client request IDs and engine-assigned order IDs.
    New {
        account_id: AccountId,
        client_order_id: ClientOrderId,
        instrument_id: InstrumentId,
        side: Side,
        order_type: OrderType,
        price: Price,
        quantity: Quantity,
    },

    /// Cancel an existing order by its engine-assigned order ID.
    ///
    /// # Fields
    ///
    /// * `order_id` - The engine-assigned identifier of the order to cancel.
    ///
    /// # Notes
    ///
    /// Use this when you have received an [`EngineEvent::Accepted`](crate::EngineEvent::Accepted)
    /// and need to cancel that specific order.
    CancelByOrder { order_id: OrderId },

    /// Cancel an existing order using the original client request identifier.
    ///
    /// # Fields
    ///
    /// * `account_id` - Identifier for the trading account that placed the original order.
    /// * `client_order_id` - The client-assigned identifier from the opening [`EngineCommand::New`].
    ///
    /// # Notes
    ///
    /// This variant uses the `(AccountId, ClientOrderId)` pair from the opening `New` command.
    /// This pattern supports scenarios where the client only retains the original request ID
    /// but not necessarily the engine-assigned order ID.
    CancelByClient {
        account_id: AccountId,
        client_order_id: ClientOrderId,
    },

    /// Replace an existing order with modified price and/or quantity.
    ///
    /// # Fields
    ///
    /// * `order_id` - The engine-assigned identifier of the order to replace.
    /// * `client_order_id` - A fresh client-assigned identifier for this replace request.
    ///   This is NOT the original `ClientOrderId` from the `New` command; each replace
    ///   is a distinct logical request.
    /// * `price` - New price in ticks (unused if only modifying quantity).
    /// * `quantity` - New quantity in lots.
    ///
    /// # Notes
    ///
    /// Replace does NOT include an `instrument_id` field because the order cannot change
    /// its instrument. Replace amends only price and quantity on the same instrument.
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

    /// Test that New command has client_order_id and no engine order_id.
    #[test]
    fn engine_command_new_has_client_order_id_and_no_engine_order_id() {
        // Given: a New command for a limit buy order
        let account = AccountId::new(1);
        let client_order = ClientOrderId::new(7);
        let instrument = InstrumentId::new(2);
        let side = Side::Buy;
        let order_type = OrderType::Limit;
        let price = Price::new(100);
        let quantity = Quantity::new(10);

        // When: we create a New command
        let command = EngineCommand::New {
            account_id: account,
            client_order_id: client_order,
            instrument_id: instrument,
            side,
            order_type,
            price,
            quantity,
        };

        // Then: the command has all required fields
        match command {
            EngineCommand::New {
                account_id,
                client_order_id,
                instrument_id,
                side,
                order_type,
                price,
                quantity,
            } => {
                assert_eq!(account_id.inner(), 1);
                assert_eq!(client_order_id.inner(), 7);
                assert_eq!(instrument_id.inner(), 2);
                assert_eq!(side, Side::Buy);
                assert_eq!(order_type, OrderType::Limit);
                assert_eq!(price.inner(), 100);
                assert_eq!(quantity.inner(), 10);
            }
            _ => panic!("Expected New variant"),
        }

        // Verify we cannot access order_id (it doesn't exist on New)
        let _ = client_order; // Use to avoid unused warning
    }

    /// Test that CancelByClient uses account_id and client_order_id.
    #[test]
    fn engine_command_cancel_by_client_uses_account_and_client_order_id() {
        // Given: a CancelByClient command
        let account = AccountId::new(1);
        let client_order = ClientOrderId::new(7);

        // When: we create a CancelByClient command
        let command = EngineCommand::CancelByClient {
            account_id: account,
            client_order_id: client_order,
        };

        // Then: the command has both required fields
        match command {
            EngineCommand::CancelByClient {
                account_id,
                client_order_id,
            } => {
                assert_eq!(account_id.inner(), 1);
                assert_eq!(client_order_id.inner(), 7);
            }
            _ => panic!("Expected CancelByClient variant"),
        }
    }

    /// Test that Replace carries order_id and a fresh client_order_id.
    #[test]
    fn engine_command_replace_carries_order_id_and_fresh_client_order_id() {
        // Given: a Replace command
        let order_id = OrderId::new(42);
        let fresh_client_order = ClientOrderId::new(99); // This is the REPLACE request ID, not original

        // When: we create a Replace command
        let command = EngineCommand::Replace {
            order_id,
            client_order_id: fresh_client_order,
            price: Price::new(105),
            quantity: Quantity::new(12),
        };

        // Then: the command has both order_id and fresh client_order_id
        match command {
            EngineCommand::Replace {
                order_id,
                client_order_id,
                price,
                quantity,
            } => {
                assert_eq!(order_id.inner(), 42);
                assert_eq!(client_order_id.inner(), 99);
                assert_eq!(price.inner(), 105);
                assert_eq!(quantity.inner(), 12);
            }
            _ => panic!("Expected Replace variant"),
        }

        // Verify order_id and client_order_id are distinct concepts
        assert_ne!(order_id.inner(), fresh_client_order.inner());
    }

    /// Test that all EngineCommand variants implement Copy.
    #[test]
    fn engine_command_variants_implement_copy() {
        // Test New is Copy
        let original_new = EngineCommand::New {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            order_type: OrderType::Limit,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };

        let _copied_new = original_new;
        // Both can be used after assignment
        match original_new {
            EngineCommand::New {
                client_order_id, ..
            } => {
                assert_eq!(client_order_id.inner(), 7);
            }
            _ => unreachable!(),
        }

        // Test CancelByOrder is Copy
        let original_cancel = EngineCommand::CancelByOrder {
            order_id: OrderId::new(123),
        };

        let _copied_cancel = original_cancel;
        match original_cancel {
            EngineCommand::CancelByOrder { order_id } => {
                assert_eq!(order_id.inner(), 123);
            }
            _ => unreachable!(),
        }

        // Test CancelByClient is Copy
        let original_cancel_client = EngineCommand::CancelByClient {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
        };

        let _copied_cancel_client = original_cancel_client;
        match original_cancel_client {
            EngineCommand::CancelByClient {
                client_order_id, ..
            } => {
                assert_eq!(client_order_id.inner(), 7);
            }
            _ => unreachable!(),
        }

        // Test Replace is Copy
        let original_replace = EngineCommand::Replace {
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(99),
            price: Price::new(105),
            quantity: Quantity::new(12),
        };

        let _copied_replace = original_replace;
        match original_replace {
            EngineCommand::Replace {
                client_order_id, ..
            } => {
                assert_eq!(client_order_id.inner(), 99);
            }
            _ => unreachable!(),
        }
    }

    /// Test equality of EngineCommand variants.
    #[test]
    fn engine_command_equality() {
        // Test New equality
        let cmd1 = EngineCommand::New {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            order_type: OrderType::Limit,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };

        let cmd2 = EngineCommand::New {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            order_type: OrderType::Limit,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };

        assert_eq!(cmd1, cmd2);

        // Test inequality
        let cmd3 = EngineCommand::New {
            account_id: AccountId::new(2), // Different account
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            order_type: OrderType::Limit,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };

        assert_ne!(cmd1, cmd3);

        // Test CancelByOrder equality
        let cancel1 = EngineCommand::CancelByOrder {
            order_id: OrderId::new(42),
        };

        let cancel2 = EngineCommand::CancelByOrder {
            order_id: OrderId::new(42),
        };

        assert_eq!(cancel1, cancel2);

        // Test CancelByClient equality
        let cancel_client1 = EngineCommand::CancelByClient {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
        };

        let cancel_client2 = EngineCommand::CancelByClient {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
        };

        assert_eq!(cancel_client1, cancel_client2);

        // Test Replace equality
        let replace1 = EngineCommand::Replace {
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(99),
            price: Price::new(105),
            quantity: Quantity::new(12),
        };

        let replace2 = EngineCommand::Replace {
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(99),
            price: Price::new(105),
            quantity: Quantity::new(12),
        };

        assert_eq!(replace1, replace2);
    }
}
