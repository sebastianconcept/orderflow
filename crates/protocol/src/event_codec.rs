//! Event encoding and decoding for the protocol.
//!
//! This module provides serialization and deserialization of [`EngineEvent`]
//! values into the compact binary format used in replayable event logs.
//!
//! # Wire Format
//!
//! Each event is encoded as a frame:
//!
//! - `frame_len` (u16, little-endian): Length of kind + payload
//! - `kind` (u8): Opcode identifying the event type
//! - `payload` (variable): Event-specific data
//!
//! # Opcodes
//!
//! | Kind | Name     |
//! |------|----------|
//! | 0x81 | Accepted |
//! | 0x82 | Rejected |
//! | 0x83 | Replaced |
//! | 0x84 | Canceled |
//! | 0x85 | Trade    |
//!
//! # Examples
//!
//! ```
//! use engine_types::{AccountId, ClientOrderId, EngineEvent, InstrumentId, OrderId, Sequence, Side, TimestampNanos};
//! use protocol::event_codec;
//!
//! // Create an Accepted event
//! let event = EngineEvent::Accepted {
//!     sequence: Sequence::new(1),
//!     timestamp_nanos: TimestampNanos::new(1000),
//!     order_id: OrderId::new(42),
//!     client_order_id: ClientOrderId::new(7),
//!     account_id: AccountId::new(1),
//! };
//!
//! // Encode to bytes
//! let mut buf = [0u8; 128];
//! let encoded_len = event_codec::encode_event(&event, &mut buf);
//!
//! // Decode back
//! let (decoded_evt, consumed) = event_codec::decode_event(&buf).expect("should decode successfully");
//! assert_eq!(event, decoded_evt);
//! ```
//!
//! # Error Handling
//!
//! Unknown opcodes return [`DecodeError::UnknownKind`]. Malformed data
//! returns appropriate errors describing the failure.

use engine_types::{
    AccountId, ClientOrderId, EngineEvent, EngineEventRejectReason, InstrumentId, OrderId,
    Sequence, TimestampNanos,
};

use crate::frame::{DecodeError as FrameDecodeError, Frame, FrameKind};
use thiserror::Error;

/// Error types for event decoding.
#[derive(Error, Debug)]
pub enum DecodeError {
    /// The input buffer is too short to contain a valid frame.
    #[error(transparent)]
    Frame(#[from] FrameDecodeError),

    /// The event kind is not recognized.
    #[error("unknown event kind: {0:#04x}")]
    UnknownKind(u8),

    /// The payload is too short for the expected event type.
    #[error("payload too short for event kind {kind:#04x}: expected at least {expected} bytes, got {actual}")]
    PayloadTooShort {
        kind: u8,
        expected: usize,
        actual: usize,
    },

    /// Failed to decode a required field from the payload.
    #[error("failed to decode {field} from payload")]
    FieldDecode { field: &'static str },

    /// Invalid rejection reason value.
    #[error("invalid rejection reason value: {0}")]
    InvalidRejectionReason(u8),
}

/// Encode an EngineEvent to a byte buffer.
///
/// # Arguments
///
/// * `event` - The event to encode.
/// * `buf` - Mutable buffer to write the encoded frame.
///
/// # Returns
///
/// The number of bytes written (frame header + payload).
pub fn encode_event(event: &EngineEvent, buf: &mut [u8]) -> usize {
    let kind = match event {
        EngineEvent::Accepted { .. } => FrameKind::ACCEPTED,
        EngineEvent::Rejected { .. } => FrameKind::REJECTED,
        EngineEvent::Replaced { .. } => FrameKind::REPLACED,
        EngineEvent::Canceled { .. } => FrameKind::CANCELED,
        EngineEvent::Trade { .. } => FrameKind::TRADE,
    };

    let payload = encode_event_payload(event);
    Frame::encode(kind, &payload, buf);

    FRAME_HEADER_SIZE + payload.len()
}

/// Encode an event's payload into bytes.
fn encode_event_payload(event: &EngineEvent) -> Vec<u8> {
    let mut payload = Vec::new();
    match event {
        EngineEvent::Accepted {
            sequence,
            timestamp_nanos,
            order_id,
            client_order_id,
            account_id,
        } => {
            // Accepted payload: sequence u64, timestamp_nanos u64,
            // order_id u64, client_order_id u64, account_id u64
            payload.extend_from_slice(&sequence.inner().to_le_bytes());
            payload.extend_from_slice(&timestamp_nanos.inner().to_le_bytes());
            payload.extend_from_slice(&order_id.inner().to_le_bytes());
            payload.extend_from_slice(&client_order_id.inner().to_le_bytes());
            payload.extend_from_slice(&account_id.inner().to_le_bytes());
        }
        EngineEvent::Rejected {
            sequence,
            timestamp_nanos,
            client_order_id,
            reason,
        } => {
            // Rejected payload: sequence u64, timestamp_nanos u64,
            // client_order_id u64, reason u8
            payload.extend_from_slice(&sequence.inner().to_le_bytes());
            payload.extend_from_slice(&timestamp_nanos.inner().to_le_bytes());
            payload.extend_from_slice(&client_order_id.inner().to_le_bytes());

            // Encode rejection reason as u8
            let reason_val = encode_rejection_reason(reason);
            payload.push(reason_val);
        }
        EngineEvent::Replaced {
            sequence,
            timestamp_nanos,
            order_id,
            client_order_id,
        } => {
            // Replaced payload: sequence u64, timestamp_nanos u64,
            // order_id u64, client_order_id u64
            payload.extend_from_slice(&sequence.inner().to_le_bytes());
            payload.extend_from_slice(&timestamp_nanos.inner().to_le_bytes());
            payload.extend_from_slice(&order_id.inner().to_le_bytes());
            payload.extend_from_slice(&client_order_id.inner().to_le_bytes());
        }
        EngineEvent::Canceled {
            sequence,
            timestamp_nanos,
            order_id,
        } => {
            // Canceled payload: sequence u64, timestamp_nanos u64, order_id u64
            payload.extend_from_slice(&sequence.inner().to_le_bytes());
            payload.extend_from_slice(&timestamp_nanos.inner().to_le_bytes());
            payload.extend_from_slice(&order_id.inner().to_le_bytes());
        }
        EngineEvent::Trade {
            sequence,
            timestamp_nanos,
            maker_order_id,
            taker_order_id,
            instrument_id,
            price,
            quantity,
        } => {
            // Trade payload: sequence u64, timestamp_nanos u64,
            // maker_order_id u64, taker_order_id u64, instrument_id u64,
            // price i64, quantity u128
            payload.extend_from_slice(&sequence.inner().to_le_bytes());
            payload.extend_from_slice(&timestamp_nanos.inner().to_le_bytes());
            payload.extend_from_slice(&maker_order_id.inner().to_le_bytes());
            payload.extend_from_slice(&taker_order_id.inner().to_le_bytes());
            payload.extend_from_slice(&instrument_id.inner().to_le_bytes());
            payload.extend_from_slice(&price.inner().to_le_bytes());
            payload.extend_from_slice(&quantity.inner().to_le_bytes());
        }
    }

    payload
}

/// Encode a rejection reason to its wire value.
fn encode_rejection_reason(reason: &EngineEventRejectReason) -> u8 {
    match reason {
        EngineEventRejectReason::InvalidQuantity => 0,
        EngineEventRejectReason::UnknownInstrument => 1,
        EngineEventRejectReason::OrderNotFound => 2,
        EngineEventRejectReason::InvalidPrice => 3,
        EngineEventRejectReason::Other => 255,
    }
}

/// Decode an EngineEvent from a byte slice.
///
/// # Arguments
///
/// * `buf` - The byte slice containing an encoded event frame.
///
/// # Returns
///
/// * `Ok((EngineEvent, consumed))` - The decoded event and number of bytes consumed.
/// * `Err(DecodeError)` - If decoding fails (unknown kind, truncated payload, etc.).
pub fn decode_event(buf: &[u8]) -> Result<(EngineEvent, usize), DecodeError> {
    // First decode the frame to get kind and payload
    let result = Frame::decode(buf);

    // Handle the frame decode result - re-raise UnknownKind directly
    let (kind, payload) = match result {
        Ok((k, p)) => (k, p),
        Err(FrameDecodeError::UnknownKind(kind_val)) => {
            // Re-raise UnknownKind as our own error
            return Err(DecodeError::UnknownKind(kind_val));
        }
        Err(e) => {
            // Wrap other errors
            return Err(DecodeError::Frame(e));
        }
    };

    // Extract kind value
    let kind_val = kind.inner();

    // Decode the event based on kind
    match kind_val {
        0x81 => decode_accepted_event(payload).map(|evt| (evt, FRAME_HEADER_SIZE + payload.len())),
        0x82 => decode_rejected_event(payload).map(|evt| (evt, FRAME_HEADER_SIZE + payload.len())),
        0x83 => decode_replaced_event(payload).map(|evt| (evt, FRAME_HEADER_SIZE + payload.len())),
        0x84 => decode_canceled_event(payload).map(|evt| (evt, FRAME_HEADER_SIZE + payload.len())),
        0x85 => decode_trade_event(payload).map(|evt| (evt, FRAME_HEADER_SIZE + payload.len())),
        _ => Err(DecodeError::UnknownKind(kind_val)),
    }
}

/// Decode an Accepted event from its payload.
fn decode_accepted_event(payload: &[u8]) -> Result<EngineEvent, DecodeError> {
    const EXPECTED_LEN: usize = 40; // sequence(8) + timestamp_nanos(8) + order_id(8) +
                                    // client_order_id(8) + account_id(8)

    if payload.len() < EXPECTED_LEN {
        return Err(DecodeError::PayloadTooShort {
            kind: 0x81,
            expected: EXPECTED_LEN,
            actual: payload.len(),
        });
    }

    let sequence = u64::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3], payload[4], payload[5], payload[6],
        payload[7],
    ]);

    let timestamp_nanos = u64::from_le_bytes([
        payload[8],
        payload[9],
        payload[10],
        payload[11],
        payload[12],
        payload[13],
        payload[14],
        payload[15],
    ]);

    let order_id = u64::from_le_bytes([
        payload[16],
        payload[17],
        payload[18],
        payload[19],
        payload[20],
        payload[21],
        payload[22],
        payload[23],
    ]);

    let client_order_id = u64::from_le_bytes([
        payload[24],
        payload[25],
        payload[26],
        payload[27],
        payload[28],
        payload[29],
        payload[30],
        payload[31],
    ]);

    let account_id = u64::from_le_bytes([
        payload[32],
        payload[33],
        payload[34],
        payload[35],
        payload[36],
        payload[37],
        payload[38],
        payload[39],
    ]);

    Ok(EngineEvent::Accepted {
        sequence: Sequence::new(sequence),
        timestamp_nanos: TimestampNanos::new(timestamp_nanos),
        order_id: OrderId::new(order_id),
        client_order_id: ClientOrderId::new(client_order_id),
        account_id: AccountId::new(account_id),
    })
}

/// Decode a Rejected event from its payload.
fn decode_rejected_event(payload: &[u8]) -> Result<EngineEvent, DecodeError> {
    const EXPECTED_LEN: usize = 25; // sequence(8) + timestamp_nanos(8) + client_order_id(8) +
                                    // reason(1)

    if payload.len() < EXPECTED_LEN {
        return Err(DecodeError::PayloadTooShort {
            kind: 0x82,
            expected: EXPECTED_LEN,
            actual: payload.len(),
        });
    }

    let sequence = u64::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3], payload[4], payload[5], payload[6],
        payload[7],
    ]);

    let timestamp_nanos = u64::from_le_bytes([
        payload[8],
        payload[9],
        payload[10],
        payload[11],
        payload[12],
        payload[13],
        payload[14],
        payload[15],
    ]);

    let client_order_id = u64::from_le_bytes([
        payload[16],
        payload[17],
        payload[18],
        payload[19],
        payload[20],
        payload[21],
        payload[22],
        payload[23],
    ]);

    let reason_val = payload[24];
    let reason = decode_rejection_reason(reason_val)?;

    Ok(EngineEvent::Rejected {
        sequence: Sequence::new(sequence),
        timestamp_nanos: TimestampNanos::new(timestamp_nanos),
        client_order_id: ClientOrderId::new(client_order_id),
        reason,
    })
}

/// Decode a Replaced event from its payload.
fn decode_replaced_event(payload: &[u8]) -> Result<EngineEvent, DecodeError> {
    const EXPECTED_LEN: usize = 32; // sequence(8) + timestamp_nanos(8) + order_id(8) +
                                    // client_order_id(8)

    if payload.len() < EXPECTED_LEN {
        return Err(DecodeError::PayloadTooShort {
            kind: 0x83,
            expected: EXPECTED_LEN,
            actual: payload.len(),
        });
    }

    let sequence = u64::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3], payload[4], payload[5], payload[6],
        payload[7],
    ]);

    let timestamp_nanos = u64::from_le_bytes([
        payload[8],
        payload[9],
        payload[10],
        payload[11],
        payload[12],
        payload[13],
        payload[14],
        payload[15],
    ]);

    let order_id = u64::from_le_bytes([
        payload[16],
        payload[17],
        payload[18],
        payload[19],
        payload[20],
        payload[21],
        payload[22],
        payload[23],
    ]);

    let client_order_id = u64::from_le_bytes([
        payload[24],
        payload[25],
        payload[26],
        payload[27],
        payload[28],
        payload[29],
        payload[30],
        payload[31],
    ]);

    Ok(EngineEvent::Replaced {
        sequence: Sequence::new(sequence),
        timestamp_nanos: TimestampNanos::new(timestamp_nanos),
        order_id: OrderId::new(order_id),
        client_order_id: ClientOrderId::new(client_order_id),
    })
}

/// Decode a Canceled event from its payload.
fn decode_canceled_event(payload: &[u8]) -> Result<EngineEvent, DecodeError> {
    const EXPECTED_LEN: usize = 24; // sequence(8) + timestamp_nanos(8) + order_id(8)

    if payload.len() < EXPECTED_LEN {
        return Err(DecodeError::PayloadTooShort {
            kind: 0x84,
            expected: EXPECTED_LEN,
            actual: payload.len(),
        });
    }

    let sequence = u64::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3], payload[4], payload[5], payload[6],
        payload[7],
    ]);

    let timestamp_nanos = u64::from_le_bytes([
        payload[8],
        payload[9],
        payload[10],
        payload[11],
        payload[12],
        payload[13],
        payload[14],
        payload[15],
    ]);

    let order_id = u64::from_le_bytes([
        payload[16],
        payload[17],
        payload[18],
        payload[19],
        payload[20],
        payload[21],
        payload[22],
        payload[23],
    ]);

    Ok(EngineEvent::Canceled {
        sequence: Sequence::new(sequence),
        timestamp_nanos: TimestampNanos::new(timestamp_nanos),
        order_id: OrderId::new(order_id),
    })
}

/// Decode a Trade event from its payload.
fn decode_trade_event(payload: &[u8]) -> Result<EngineEvent, DecodeError> {
    const EXPECTED_LEN: usize = 64; // sequence(8) + timestamp_nanos(8) + maker_order_id(8) +
                                    // taker_order_id(8) + instrument_id(8) + price(8) + quantity(16)

    if payload.len() < EXPECTED_LEN {
        return Err(DecodeError::PayloadTooShort {
            kind: 0x85,
            expected: EXPECTED_LEN,
            actual: payload.len(),
        });
    }

    let sequence = u64::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3], payload[4], payload[5], payload[6],
        payload[7],
    ]);

    let timestamp_nanos = u64::from_le_bytes([
        payload[8],
        payload[9],
        payload[10],
        payload[11],
        payload[12],
        payload[13],
        payload[14],
        payload[15],
    ]);

    let maker_order_id = u64::from_le_bytes([
        payload[16],
        payload[17],
        payload[18],
        payload[19],
        payload[20],
        payload[21],
        payload[22],
        payload[23],
    ]);

    let taker_order_id = u64::from_le_bytes([
        payload[24],
        payload[25],
        payload[26],
        payload[27],
        payload[28],
        payload[29],
        payload[30],
        payload[31],
    ]);

    let instrument_id = u64::from_le_bytes([
        payload[32],
        payload[33],
        payload[34],
        payload[35],
        payload[36],
        payload[37],
        payload[38],
        payload[39],
    ]);

    let price = i64::from_le_bytes([
        payload[40],
        payload[41],
        payload[42],
        payload[43],
        payload[44],
        payload[45],
        payload[46],
        payload[47],
    ]);

    let quantity_bytes = [
        payload[48],
        payload[49],
        payload[50],
        payload[51],
        payload[52],
        payload[53],
        payload[54],
        payload[55],
        payload[56],
        payload[57],
        payload[58],
        payload[59],
        payload[60],
        payload[61],
        payload[62],
        payload[63],
    ];
    let quantity = u128::from_le_bytes(quantity_bytes);

    Ok(EngineEvent::Trade {
        sequence: Sequence::new(sequence),
        timestamp_nanos: TimestampNanos::new(timestamp_nanos),
        maker_order_id: OrderId::new(maker_order_id),
        taker_order_id: OrderId::new(taker_order_id),
        instrument_id: InstrumentId::new(instrument_id),
        price: engine_types::Price::from_i64(price),
        quantity: engine_types::Quantity::from_u128(quantity),
    })
}

/// Size of the frame header (frame_len + kind).
pub const FRAME_HEADER_SIZE: usize = 3;

/// Decode a rejection reason from its wire value.
fn decode_rejection_reason(value: u8) -> Result<EngineEventRejectReason, DecodeError> {
    match value {
        0 => Ok(EngineEventRejectReason::InvalidQuantity),
        1 => Ok(EngineEventRejectReason::UnknownInstrument),
        2 => Ok(EngineEventRejectReason::OrderNotFound),
        3 => Ok(EngineEventRejectReason::InvalidPrice),
        255 => Ok(EngineEventRejectReason::Other),
        _ => Err(DecodeError::InvalidRejectionReason(value)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that Accepted event round-trips through encode/decode.
    #[test]
    fn event_codec_accepted_round_trips() {
        // Given: an Accepted event
        let event = EngineEvent::Accepted {
            sequence: Sequence::new(1),
            timestamp_nanos: TimestampNanos::new(1000),
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(7),
            account_id: AccountId::new(1),
        };

        // When: we encode and decode
        let mut buf = [0u8; 128];
        let encoded_len = encode_event(&event, &mut buf);

        let (decoded_evt, consumed) = decode_event(&buf).expect("decode should succeed");

        // Then: round trip preserves event
        assert_eq!(event, decoded_evt);
        assert_eq!(encoded_len, consumed);

        // Verify size is within limit (128 bytes max per SPECS)
        assert!(encoded_len <= 128);
    }

    /// Test that Trade event round-trips with maker and taker order IDs.
    #[test]
    fn event_codec_trade_round_trips_maker_taker_price_quantity() {
        // Given: a Trade event with all fields
        let event = EngineEvent::Trade {
            sequence: Sequence::new(5),
            timestamp_nanos: TimestampNanos::new(5000),
            maker_order_id: OrderId::new(10),
            taker_order_id: OrderId::new(20),
            instrument_id: InstrumentId::new(2),
            price: engine_types::Price::new(100),
            quantity: engine_types::Quantity::new(5),
        };

        // When: we encode and decode
        let mut buf = [0u8; 128];
        let encoded_len = encode_event(&event, &mut buf);

        let (decoded_evt, consumed) = decode_event(&buf).expect("decode should succeed");

        // Then: round trip preserves event
        assert_eq!(event, decoded_evt);
        assert_eq!(encoded_len, consumed);

        // Verify size is within limit (128 bytes max per SPECS)
        assert!(encoded_len <= 128);

        // Verify Trade-specific fields
        match decoded_evt {
            EngineEvent::Trade {
                maker_order_id,
                taker_order_id,
                price,
                quantity,
                ..
            } => {
                assert_eq!(maker_order_id.inner(), 10);
                assert_eq!(taker_order_id.inner(), 20);
                assert_eq!(price.inner(), 100);
                assert_eq!(quantity.inner(), 5);
            }
            _ => panic!("Expected Trade variant"),
        }
    }

    /// Test that Rejected event with Other reason round-trips.
    #[test]
    fn event_codec_rejected_round_trips_reason_other() {
        // Given: a Rejected event with Other reason (wire value 255)
        let event = EngineEvent::Rejected {
            sequence: Sequence::new(2),
            timestamp_nanos: TimestampNanos::new(2000),
            client_order_id: ClientOrderId::new(7),
            reason: EngineEventRejectReason::Other,
        };

        // When: we encode and decode
        let mut buf = [0u8; 128];
        let encoded_len = encode_event(&event, &mut buf);

        let (decoded_evt, consumed) = decode_event(&buf).expect("decode should succeed");

        // Then: round trip preserves event
        assert_eq!(event, decoded_evt);
        assert_eq!(encoded_len, consumed);

        // Verify reason is Other
        match decoded_evt {
            EngineEvent::Rejected { reason, .. } => {
                assert_eq!(reason, EngineEventRejectReason::Other);
            }
            _ => panic!("Expected Rejected variant"),
        }

        // Verify size is within limit
        assert!(encoded_len <= 128);
    }

    /// Test that all rejection reasons round-trip correctly.
    #[test]
    fn event_codec_rejected_round_trips_all_reasons() {
        let reasons = vec![
            EngineEventRejectReason::InvalidQuantity,
            EngineEventRejectReason::UnknownInstrument,
            EngineEventRejectReason::OrderNotFound,
            EngineEventRejectReason::InvalidPrice,
            EngineEventRejectReason::Other,
        ];

        for reason in reasons {
            let event = EngineEvent::Rejected {
                sequence: Sequence::new(1),
                timestamp_nanos: TimestampNanos::new(1000),
                client_order_id: ClientOrderId::new(7),
                reason,
            };

            let mut buf = [0u8; 128];
            let encoded_len = encode_event(&event, &mut buf);

            let (decoded_evt, _) = decode_event(&buf).expect("decode should succeed");

            match decoded_evt {
                EngineEvent::Rejected {
                    reason: decoded_reason,
                    ..
                } => {
                    assert_eq!(reason, decoded_reason);
                }
                _ => panic!("Expected Rejected variant"),
            }

            assert!(encoded_len <= 128);
        }
    }

    /// Test that unknown kind returns decode error.
    #[test]
    fn event_codec_unknown_kind_returns_decode_error() {
        // Given: a buffer with unknown kind (0xFF)
        let mut buf = [0u8; 10];
        // frame_len = 2 (kind + 1 byte payload)
        buf[0..2].copy_from_slice(&2u16.to_le_bytes());
        // Unknown kind
        buf[2] = 0xFF;
        // Payload byte
        buf[3] = 0x00;

        // When: we try to decode
        let result = decode_event(&buf);

        // Then: it returns an error about unknown kind
        assert!(matches!(result, Err(DecodeError::UnknownKind(0xFF))));
    }

    /// Test that truncated payload returns error.
    #[test]
    fn event_codec_truncated_payload_returns_error() {
        // Given: a buffer with Accepted frame but truncated payload
        let mut buf = [0u8; 30]; // claims more than it has
                                 // frame_len = 45 (claiming 45 bytes after header)
        buf[0..2].copy_from_slice(&45u16.to_le_bytes());
        // Valid kind (Accepted = 0x81)
        buf[2] = FrameKind::ACCEPTED.encode();

        // When: we try to decode
        let result = decode_event(&buf);

        // Then: it returns an error about payload too short (wrapped in Frame)
        assert!(matches!(
            result,
            Err(DecodeError::Frame(FrameDecodeError::PayloadTooShort { .. }))
        ));
    }

    /// Test that decode validates rejection reason values.
    #[test]
    fn event_codec_rejected_decode_validates_reason_value() {
        // Given: a Rejected payload with invalid reason value (100)
        let mut buf = [0u8; 30];
        // frame_len = 27 (kind + payload)
        buf[0..2].copy_from_slice(&27u16.to_le_bytes());
        // Valid kind (Rejected = 0x82)
        buf[2] = FrameKind::REJECTED.encode();

        // Write valid values for first 24 bytes
        buf[3..11].copy_from_slice(&1u64.to_le_bytes()); // sequence
        buf[11..19].copy_from_slice(&1000u64.to_le_bytes()); // timestamp_nanos
        buf[19..27].copy_from_slice(&7u64.to_le_bytes()); // client_order_id

        // Invalid reason value (100)
        buf[27] = 100;

        // When: we try to decode
        let result = decode_event(&buf);

        // Then: it returns an error about invalid rejection reason
        assert!(matches!(
            result,
            Err(DecodeError::InvalidRejectionReason(_))
        ));
    }

    /// Test that encode produces expected byte sizes for each event type.
    #[test]
    fn event_codec_expected_sizes() {
        // Accepted: 3 (header) + 40 (payload) = 43 bytes
        let accepted_evt = EngineEvent::Accepted {
            sequence: Sequence::new(1),
            timestamp_nanos: TimestampNanos::new(1000),
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(7),
            account_id: AccountId::new(1),
        };
        let mut buf = [0u8; 128];
        assert_eq!(encode_event(&accepted_evt, &mut buf), 43);

        // Rejected: 3 (header) + 25 (payload) = 28 bytes
        let rejected_evt = EngineEvent::Rejected {
            sequence: Sequence::new(2),
            timestamp_nanos: TimestampNanos::new(2000),
            client_order_id: ClientOrderId::new(7),
            reason: EngineEventRejectReason::InvalidQuantity,
        };
        assert_eq!(encode_event(&rejected_evt, &mut buf), 28);

        // Replaced: 3 (header) + 32 (payload) = 35 bytes
        let replaced_evt = EngineEvent::Replaced {
            sequence: Sequence::new(3),
            timestamp_nanos: TimestampNanos::new(3000),
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(99),
        };
        assert_eq!(encode_event(&replaced_evt, &mut buf), 35);

        // Canceled: 3 (header) + 24 (payload) = 27 bytes
        let canceled_evt = EngineEvent::Canceled {
            sequence: Sequence::new(4),
            timestamp_nanos: TimestampNanos::new(4000),
            order_id: OrderId::new(42),
        };
        assert_eq!(encode_event(&canceled_evt, &mut buf), 27);

        // Trade: 3 (header) + 64 (payload) = 67 bytes
        let trade_evt = EngineEvent::Trade {
            sequence: Sequence::new(5),
            timestamp_nanos: TimestampNanos::new(5000),
            maker_order_id: OrderId::new(10),
            taker_order_id: OrderId::new(20),
            instrument_id: InstrumentId::new(2),
            price: engine_types::Price::new(100),
            quantity: engine_types::Quantity::new(5),
        };
        assert_eq!(encode_event(&trade_evt, &mut buf), 67);
    }

    /// Test that Replaced event round-trips correctly.
    #[test]
    fn event_codec_replaced_round_trips() {
        // Given: a Replaced event
        let event = EngineEvent::Replaced {
            sequence: Sequence::new(3),
            timestamp_nanos: TimestampNanos::new(3000),
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(99),
        };

        // When: we encode and decode
        let mut buf = [0u8; 128];
        let encoded_len = encode_event(&event, &mut buf);

        let (decoded_evt, consumed) = decode_event(&buf).expect("decode should succeed");

        // Then: round trip preserves event
        assert_eq!(event, decoded_evt);
        assert_eq!(encoded_len, consumed);

        // Verify size is within limit
        assert!(encoded_len <= 128);
    }

    /// Test that Canceled event round-trips correctly.
    #[test]
    fn event_codec_canceled_round_trips() {
        // Given: a Canceled event
        let event = EngineEvent::Canceled {
            sequence: Sequence::new(4),
            timestamp_nanos: TimestampNanos::new(4000),
            order_id: OrderId::new(42),
        };

        // When: we encode and decode
        let mut buf = [0u8; 128];
        let encoded_len = encode_event(&event, &mut buf);

        let (decoded_evt, consumed) = decode_event(&buf).expect("decode should succeed");

        // Then: round trip preserves event
        assert_eq!(event, decoded_evt);
        assert_eq!(encoded_len, consumed);

        // Verify size is within limit
        assert!(encoded_len <= 128);
    }

    /// Test that multiple events can be encoded and decoded sequentially.
    #[test]
    fn event_codec_multiple_events_round_trip() {
        // Given: multiple events
        #[allow(clippy::useless_vec)]
        let events = vec![
            EngineEvent::Accepted {
                sequence: Sequence::new(1),
                timestamp_nanos: TimestampNanos::new(1000),
                order_id: OrderId::new(42),
                client_order_id: ClientOrderId::new(7),
                account_id: AccountId::new(1),
            },
            EngineEvent::Trade {
                sequence: Sequence::new(2),
                timestamp_nanos: TimestampNanos::new(2000),
                maker_order_id: OrderId::new(42),
                taker_order_id: OrderId::new(99),
                instrument_id: InstrumentId::new(2),
                price: engine_types::Price::new(100),
                quantity: engine_types::Quantity::new(5),
            },
            EngineEvent::Rejected {
                sequence: Sequence::new(3),
                timestamp_nanos: TimestampNanos::new(3000),
                client_order_id: ClientOrderId::new(7),
                reason: EngineEventRejectReason::InvalidQuantity,
            },
        ];

        // When: we encode each event into separate buffers
        let mut bufs = vec![[0u8; 128]; events.len()];
        for (i, evt) in events.iter().enumerate() {
            let encoded_len = encode_event(evt, &mut bufs[i]);
            let (decoded_evt, consumed) = decode_event(&bufs[i]).expect("decode should succeed");
            assert_eq!(*evt, decoded_evt);
            assert_eq!(encoded_len, consumed);
            assert!(encoded_len <= 128);
        }
    }
}
