//! Frozen OFL1 commands and events for codec checks.
//!
//! When an allocation test needs one sample of every kind, it uses this module
//! so session 1 and sequence zero stay stable.

use engine_types::{
    AccountId, ClientOrderId, CommandSequence, EngineCommand, EngineEvent, EngineEventRejectReason,
    EventSequence, InstrumentId, OrderId, SequencedCommand, SessionId, Side, TimestampNanos,
};

/// Upper bound for a stream-encoded command or event frame.
pub const MAX_ENCODED_STREAM_FRAME: usize = 128;

/// Answers the SessionId stamped on every fixture journal and datagram.
pub fn session_id() -> SessionId {
    SessionId::new(1)
}

/// Answers a NewLimit SequencedCommand with fixed prices and the given sequence.
pub fn new_limit_sequenced_command(command_sequence: CommandSequence) -> SequencedCommand {
    SequencedCommand::new(
        command_sequence,
        EngineCommand::NewLimit {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            price: engine_types::Price::new(100),
            quantity: engine_types::Quantity::new(10),
        },
    )
}

/// Answers a NewMarket SequencedCommand with the given sequence.
pub fn new_market_sequenced_command(command_sequence: CommandSequence) -> SequencedCommand {
    SequencedCommand::new(
        command_sequence,
        EngineCommand::NewMarket {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(8),
            instrument_id: InstrumentId::new(3),
            side: Side::Sell,
            quantity: engine_types::Quantity::new(5),
        },
    )
}

/// Answers a CancelByOrder SequencedCommand with the given sequence.
pub fn cancel_by_order_sequenced_command(command_sequence: CommandSequence) -> SequencedCommand {
    SequencedCommand::new(
        command_sequence,
        EngineCommand::CancelByOrder {
            order_id: OrderId::new(100),
        },
    )
}

/// Answers a CancelByClient SequencedCommand with the given sequence.
pub fn cancel_by_client_sequenced_command(command_sequence: CommandSequence) -> SequencedCommand {
    SequencedCommand::new(
        command_sequence,
        EngineCommand::CancelByClient {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
        },
    )
}

/// Answers a Replace SequencedCommand with the given sequence.
pub fn replace_sequenced_command(command_sequence: CommandSequence) -> SequencedCommand {
    SequencedCommand::new(
        command_sequence,
        EngineCommand::Replace {
            order_id: OrderId::new(100),
            client_order_id: ClientOrderId::new(99),
            price: engine_types::Price::new(105),
            quantity: engine_types::Quantity::new(12),
        },
    )
}

/// Answers an Accepted event with the given clocks.
pub fn accepted_engine_event(
    event_sequence: EventSequence,
    command_sequence: CommandSequence,
) -> EngineEvent {
    EngineEvent::Accepted {
        event_sequence,
        command_sequence,
        timestamp_nanos: TimestampNanos::new(1000),
        order_id: OrderId::new(100),
        client_order_id: ClientOrderId::new(7),
        account_id: AccountId::new(1),
    }
}

/// Answers a Rejected event with the given clocks.
pub fn rejected_engine_event(
    event_sequence: EventSequence,
    command_sequence: CommandSequence,
) -> EngineEvent {
    EngineEvent::Rejected {
        event_sequence,
        command_sequence,
        timestamp_nanos: TimestampNanos::new(2000),
        client_order_id: ClientOrderId::new(7),
        reason: EngineEventRejectReason::InvalidQuantity,
    }
}

/// Answers a Replaced event with the given clocks.
pub fn replaced_engine_event(
    event_sequence: EventSequence,
    command_sequence: CommandSequence,
) -> EngineEvent {
    EngineEvent::Replaced {
        event_sequence,
        command_sequence,
        timestamp_nanos: TimestampNanos::new(3000),
        order_id: OrderId::new(100),
        client_order_id: ClientOrderId::new(99),
    }
}

/// Answers a Canceled event with the given clocks.
pub fn canceled_engine_event(
    event_sequence: EventSequence,
    command_sequence: CommandSequence,
) -> EngineEvent {
    EngineEvent::Canceled {
        event_sequence,
        command_sequence,
        timestamp_nanos: TimestampNanos::new(4000),
        order_id: OrderId::new(100),
    }
}

/// Answers a Trade event with the given clocks.
pub fn trade_engine_event(
    event_sequence: EventSequence,
    command_sequence: CommandSequence,
) -> EngineEvent {
    EngineEvent::Trade {
        event_sequence,
        command_sequence,
        timestamp_nanos: TimestampNanos::new(5000),
        maker_order_id: OrderId::new(100),
        taker_order_id: OrderId::new(101),
        instrument_id: InstrumentId::new(2),
        price: engine_types::Price::new(100),
        quantity: engine_types::Quantity::new(3),
    }
}

/// Answers every command kind with sequence zero.
pub fn all_command_kinds() -> [(&'static str, SequencedCommand); 5] {
    let zero = CommandSequence::new(0);
    [
        ("new_limit", new_limit_sequenced_command(zero)),
        ("new_market", new_market_sequenced_command(zero)),
        ("cancel_by_order", cancel_by_order_sequenced_command(zero)),
        ("cancel_by_client", cancel_by_client_sequenced_command(zero)),
        ("replace", replace_sequenced_command(zero)),
    ]
}

/// Answers every event kind with clocks at zero.
pub fn all_event_kinds() -> [(&'static str, EngineEvent); 5] {
    let zero_command = CommandSequence::new(0);
    let zero_event = EventSequence::new(0);
    [
        ("accepted", accepted_engine_event(zero_event, zero_command)),
        ("rejected", rejected_engine_event(zero_event, zero_command)),
        ("replaced", replaced_engine_event(zero_event, zero_command)),
        ("canceled", canceled_engine_event(zero_event, zero_command)),
        ("trade", trade_engine_event(zero_event, zero_command)),
    ]
}
