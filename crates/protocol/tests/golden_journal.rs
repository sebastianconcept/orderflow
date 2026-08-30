//! Golden journal fixture and size cap tests for the protocol.
//!
//! This module provides integration tests that verify:
//!
//! - Mixed commands and events round-trip through encode/decode
//! - Encoded New and Trade frames are at most 128 bytes (excluding header)
//! - Two New commands with the same AccountId and ClientOrderId but different
//!   Sequence round-trip correctly (idempotency fixture)
//!
//! # Background
//!
//! A golden journal is a compact, binary log of commands and events that can be
//! replayed to reproduce matching results. Each frame consists of:
//!
//! - `frame_len` (u16): bytes following this field (kind + payload)
//! - `kind` (u8): opcode (command or event)
//! - `payload`: variable-length data
//!
//! The journal header precedes all frames and contains:
//!
//! - `magic` (4 bytes): "OFL1"
//! - `schema_version` (u16): 1
//! - `SessionId` (u64)
//! - `clock_kind` (u8): 0 logical, 1 wall
//!
//! # Usage
//!
//! These tests assert that the protocol cannot regress in compactness (AC-7)
//! and provide fixture bytes for idempotency testing.

use engine_types::{
    AccountId, ClientOrderId, EngineCommand, EngineEvent, InstrumentId, OrderId, Side,
    TimestampNanos,
};
use protocol::{
    command_codec::{decode_command, encode_command},
    event_codec::{decode_event, encode_event},
    journal_header::{ClockKind, JournalHeader},
};

/// Test that a mixed sequence of commands and events round-trips correctly.
///
/// This test simulates a realistic journal with:
/// - A header (SessionId, clock kind)
/// - New command
/// - Accepted event
/// - Another New command (for duplicate detection testing)
/// - Trade event
/// - Cancel command
/// - Canceled event
#[test]
fn golden_journal_mixed_commands_and_events_round_trip() {
    // Given: A sequence of mixed commands and events
    let session_id = 42u64;
    let clock_kind = ClockKind::Logical;

    // Create journal header
    let mut header_bytes = [0u8; 15];
    let header = JournalHeader::new(session_id, clock_kind);
    header.encode(&mut header_bytes);

    // Build a mixed sequence:
    // 1. New command (account 1, client 7, instrument 2, buy limit)
    let new_cmd_1 = EngineCommand::New {
        account_id: AccountId::new(1),
        client_order_id: ClientOrderId::new(7),
        instrument_id: InstrumentId::new(2),
        side: Side::Buy,
        order_type: engine_types::OrderType::Limit,
        price: engine_types::Price::new(100),
        quantity: engine_types::Quantity::new(10),
    };

    // 2. Accepted event (sequence 1, order_id assigned by engine)
    let accepted_evt = EngineEvent::Accepted {
        sequence: engine_types::Sequence::new(1),
        timestamp_nanos: TimestampNanos::new(1000),
        order_id: OrderId::new(100),
        client_order_id: ClientOrderId::new(7),
        account_id: AccountId::new(1),
    };

    // 3. New command (another order, same account/client but different instrument)
    let new_cmd_2 = EngineCommand::New {
        account_id: AccountId::new(1),
        client_order_id: ClientOrderId::new(7), // same as new_cmd_1 for duplicate test
        instrument_id: InstrumentId::new(3),    // different instrument
        side: Side::Sell,
        order_type: engine_types::OrderType::Market,
        price: engine_types::Price::new(0),
        quantity: engine_types::Quantity::new(5),
    };

    // 4. Trade event (maker=100, taker=101)
    let trade_evt = EngineEvent::Trade {
        sequence: engine_types::Sequence::new(2),
        timestamp_nanos: TimestampNanos::new(2000),
        maker_order_id: OrderId::new(100),
        taker_order_id: OrderId::new(101),
        instrument_id: InstrumentId::new(2),
        price: engine_types::Price::new(100),
        quantity: engine_types::Quantity::new(3),
    };

    // 5. Cancel command
    let cancel_cmd = EngineCommand::CancelByOrder {
        order_id: OrderId::new(100),
    };

    // 6. Canceled event
    let canceled_evt = EngineEvent::Canceled {
        sequence: engine_types::Sequence::new(3),
        timestamp_nanos: TimestampNanos::new(3000),
        order_id: OrderId::new(100),
    };

    // When: We encode all frames into a journal
    let mut journal_bytes = Vec::new();

    // Add header
    journal_bytes.extend_from_slice(&header_bytes);

    // Encode and add New command 1
    let mut frame_buf = [0u8; 128];
    let encoded_len_1 = encode_command(&new_cmd_1, &mut frame_buf);
    journal_bytes.extend_from_slice(&frame_buf[..encoded_len_1]);

    // Encode and add Accepted event
    let encoded_evt_len = encode_event(&accepted_evt, &mut frame_buf);
    journal_bytes.extend_from_slice(&frame_buf[..encoded_evt_len]);

    // Encode and add New command 2
    let encoded_len_2 = encode_command(&new_cmd_2, &mut frame_buf);
    journal_bytes.extend_from_slice(&frame_buf[..encoded_len_2]);

    // Encode and add Trade event
    let encoded_trade_len = encode_event(&trade_evt, &mut frame_buf);
    journal_bytes.extend_from_slice(&frame_buf[..encoded_trade_len]);

    // Encode and add Cancel command
    let encoded_cancel_len = encode_command(&cancel_cmd, &mut frame_buf);
    journal_bytes.extend_from_slice(&frame_buf[..encoded_cancel_len]);

    // Encode and add Canceled event
    let encoded_canceled_len = encode_event(&canceled_evt, &mut frame_buf);
    journal_bytes.extend_from_slice(&frame_buf[..encoded_canceled_len]);

    // When: We decode the journal
    let mut offset = 0;

    // Decode header
    let decoded_header =
        JournalHeader::decode(&journal_bytes[offset..offset + 15]).expect("header decode failed");
    offset += 15;
    assert_eq!(decoded_header.session_id(), session_id);
    assert_eq!(decoded_header.clock_kind(), clock_kind);

    // Decode New command 1
    let (decoded_cmd_1, consumed_1) =
        decode_command(&journal_bytes[offset..]).expect("command 1 decode failed");
    offset += consumed_1;
    assert_eq!(decoded_cmd_1, new_cmd_1);

    // Decode Accepted event
    let (decoded_evt_1, consumed_evt_1) =
        decode_event(&journal_bytes[offset..]).expect("event 1 decode failed");
    offset += consumed_evt_1;
    assert_eq!(decoded_evt_1, accepted_evt);

    // Decode New command 2
    let (decoded_cmd_2, consumed_2) =
        decode_command(&journal_bytes[offset..]).expect("command 2 decode failed");
    offset += consumed_2;
    assert_eq!(decoded_cmd_2, new_cmd_2);

    // Decode Trade event
    let (decoded_trade, consumed_trade) =
        decode_event(&journal_bytes[offset..]).expect("trade decode failed");
    offset += consumed_trade;
    assert_eq!(decoded_trade, trade_evt);

    // Decode Cancel command
    let (decoded_cancel, consumed_cancel) =
        decode_command(&journal_bytes[offset..]).expect("cancel decode failed");
    offset += consumed_cancel;
    assert_eq!(decoded_cancel, cancel_cmd);

    // Decode Canceled event
    let (decoded_canceled, consumed_canceled) =
        decode_event(&journal_bytes[offset..]).expect("canceled decode failed");
    offset += consumed_canceled;
    assert_eq!(decoded_canceled, canceled_evt);

    // Then: We consumed exactly the bytes we wrote
    assert_eq!(offset, journal_bytes.len());
}

/// Test that New and Trade frames are at most 128 bytes (excluding header).
#[test]
fn golden_journal_new_and_trade_frames_are_at_most_128_bytes() {
    // Given: A New command and a Trade event
    let new_cmd = EngineCommand::New {
        account_id: AccountId::new(1),
        client_order_id: ClientOrderId::new(7),
        instrument_id: InstrumentId::new(2),
        side: Side::Buy,
        order_type: engine_types::OrderType::Limit,
        price: engine_types::Price::new(100),
        quantity: engine_types::Quantity::new(10),
    };

    let trade_evt = EngineEvent::Trade {
        sequence: engine_types::Sequence::new(1),
        timestamp_nanos: TimestampNanos::new(1000),
        maker_order_id: OrderId::new(10),
        taker_order_id: OrderId::new(20),
        instrument_id: InstrumentId::new(2),
        price: engine_types::Price::new(100),
        quantity: engine_types::Quantity::new(5),
    };

    // When: We encode both
    let mut frame_buf = [0u8; 128];

    // New command
    let new_encoded_len = encode_command(&new_cmd, &mut frame_buf);

    // Trade event
    let trade_encoded_len = encode_event(&trade_evt, &mut frame_buf);

    // Then: Both are at most 128 bytes (excluding header)
    // Per AC-7: "Encoded New and Trade frames are ≤ 128 bytes each (header excluded)"
    assert!(
        new_encoded_len <= 128,
        "New frame is {} bytes, expected at most 128",
        new_encoded_len
    );
    assert!(
        trade_encoded_len <= 128,
        "Trade frame is {} bytes, expected at most 128",
        trade_encoded_len
    );

    // Verify the actual sizes (per SPECS):
    // New: 3 (header) + 50 (payload) = 53 bytes
    // Trade: 3 (header) + 64 (payload) = 67 bytes
    assert_eq!(
        new_encoded_len, 53,
        "New command should be exactly 53 bytes"
    );
    assert_eq!(
        trade_encoded_len, 67,
        "Trade event should be exactly 67 bytes"
    );
}

/// Test that two New commands with the same AccountId and ClientOrderId
/// but different Sequence round-trip correctly.
///
/// This is a fixture for duplicate New detection (idempotency). The two
/// commands have identical payloads except for Sequence/TimestampNanos on
/// the events. Commands themselves do not include Sequence.
#[test]
fn golden_journal_two_new_frames_share_account_and_client_order_id_with_different_sequence() {
    // Given: Two New commands with same account and client order id
    let new_cmd = EngineCommand::New {
        account_id: AccountId::new(1),
        client_order_id: ClientOrderId::new(7),
        instrument_id: InstrumentId::new(2),
        side: Side::Buy,
        order_type: engine_types::OrderType::Limit,
        price: engine_types::Price::new(100),
        quantity: engine_types::Quantity::new(10),
    };

    // When: We encode the same command twice
    let mut frame_buf1 = [0u8; 128];
    let encoded_len_1 = encode_command(&new_cmd, &mut frame_buf1);

    let mut frame_buf2 = [0u8; 128];
    let encoded_len_2 = encode_command(&new_cmd, &mut frame_buf2);

    // Then: The encodings are identical (commands don't include Sequence)
    assert_eq!(encoded_len_1, encoded_len_2);
    assert_eq!(&frame_buf1[..encoded_len_1], &frame_buf2[..encoded_len_2]);

    // Decode both to verify they match
    let (decoded_cmd_1, _consumed_1) =
        decode_command(&frame_buf1).expect("first command decode failed");
    let (decoded_cmd_2, _consumed_2) =
        decode_command(&frame_buf2).expect("second command decode failed");

    assert_eq!(decoded_cmd_1, new_cmd);
    assert_eq!(decoded_cmd_2, new_cmd);

    // Now test with Accepted events that have different Sequence
    let accepted_evt_1 = EngineEvent::Accepted {
        sequence: engine_types::Sequence::new(1), // different Sequence
        timestamp_nanos: TimestampNanos::new(1000),
        order_id: OrderId::new(100),
        client_order_id: ClientOrderId::new(7),
        account_id: AccountId::new(1),
    };

    let accepted_evt_2 = EngineEvent::Accepted {
        sequence: engine_types::Sequence::new(2), // different Sequence
        timestamp_nanos: TimestampNanos::new(1001),
        order_id: OrderId::new(100), // same order_id (engine-assigned)
        client_order_id: ClientOrderId::new(7),
        account_id: AccountId::new(1),
    };

    // Encode the two Accepted events
    let mut event_buf1 = [0u8; 128];
    let encoded_evt_1 = encode_event(&accepted_evt_1, &mut event_buf1);

    let mut event_buf2 = [0u8; 128];
    let encoded_evt_2 = encode_event(&accepted_evt_2, &mut event_buf2);

    // Decode both events
    let (decoded_evt_1, _consumed_evt_1) =
        decode_event(&event_buf1).expect("first event decode failed");
    let (decoded_evt_2, _consumed_evt_2) =
        decode_event(&event_buf2).expect("second event decode failed");

    // Then: Events round-trip correctly with different Sequence
    assert_eq!(decoded_evt_1, accepted_evt_1);
    assert_eq!(decoded_evt_2, accepted_evt_2);

    // And the events have different encodings due to Sequence
    assert_ne!(&event_buf1[..encoded_evt_1], &event_buf2[..encoded_evt_2]);

    // And both are within the 128 byte limit
    assert!(encoded_evt_1 <= 128);
    assert!(encoded_evt_2 <= 128);

    // Verify exact sizes (per SPECS):
    // Accepted: 3 (header) + 40 (payload) = 43 bytes
    assert_eq!(encoded_evt_1, 43);
    assert_eq!(encoded_evt_2, 43);
}
