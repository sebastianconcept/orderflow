//! Packed encode and decode of EngineEvent.
//!
//! When a caller writes an event into a stream or datagram buffer, it uses this
//! module so both envelopes share one payload layout.
//!
//! ```
//! use engine_types::{
//!     AccountId, ClientOrderId, CommandSequence, EngineEvent, EventSequence, JournalSequence,
//!     OrderId, TimestampNanos,
//! };
//! use protocol::event_codec;
//!
//! let event = EngineEvent::Accepted {
//!     event_sequence: EventSequence::new(1),
//!     command_sequence: CommandSequence::new(0),
//!     timestamp_nanos: TimestampNanos::new(1000),
//!     order_id: OrderId::new(42),
//!     client_order_id: ClientOrderId::new(7),
//!     account_id: AccountId::new(1),
//! };
//! let mut buffer = [0u8; 128];
//! let encoded_len =
//!     event_codec::encode_event(&event, JournalSequence::new(0), &mut buffer).expect("encode");
//! let (decoded_event, journal_sequence, _consumed) =
//!     event_codec::decode_event(&buffer).expect("decode");
//! assert_eq!(event, decoded_event);
//! assert_eq!(journal_sequence.inner(), 0);
//! assert!(encoded_len > 0);
//! ```

use displaydoc::Display;
use engine_types::{
    AccountId, ClientOrderId, CommandSequence, EngineEvent, EngineEventRejectReason, EventSequence,
    InstrumentId, JournalSequence, OrderId, SessionId, TimestampNanos,
};
use thiserror::Error;

use crate::datagram::{
    self, DecodeError as DatagramDecodeError, EncodeError as DatagramEncodeError,
    DATAGRAM_HEADER_SIZE,
};
use crate::frame::{
    DecodeError as FrameDecodeError, EncodeError as FrameEncodeError, Frame, FrameKind,
    FRAME_HEADER_SIZE,
};
use crate::packed_le::{read_i64, read_u128, read_u64, write_i64, write_u128, write_u64};

/// Failure of event frame decode.
/// Frame errors, datagram errors, unknown kinds, and payload mismatches stay
/// distinct.
#[derive(Display, Debug, Error, PartialEq, Eq)]
pub enum DecodeError {
    /// Frame decode failed: {0}
    Frame(#[source] FrameDecodeError),
    /// Datagram decode failed: {0}
    Datagram(#[source] DatagramDecodeError),
    /// Unknown event kind: {0:#04x}
    UnknownKind(u8),
    /// Payload length is {actual} bytes, expected {expected} for kind {kind:#04x}
    PayloadLengthMismatch {
        kind: u8,
        expected: usize,
        actual: usize,
    },
    /// Invalid rejection reason value: {0}
    InvalidRejectionReason(u8),
}

/// Failure of event frame encode.
/// Frame and datagram packing failures stay distinct.
#[derive(Display, Debug, Error, PartialEq, Eq)]
pub enum EncodeError {
    /// Frame encode failed: {0}
    Frame(#[source] FrameEncodeError),
    /// Datagram encode failed: {0}
    Datagram(#[source] DatagramEncodeError),
}

/// Byte count of an Accepted payload: event, command, timestamp, order, client, account.
const ACCEPTED_PAYLOAD_LEN: usize = 48;
/// Byte count of a Rejected payload: event, command, timestamp, client, reason.
const REJECTED_PAYLOAD_LEN: usize = 33;
/// Byte count of a Replaced payload: event, command, timestamp, order, client.
const REPLACED_PAYLOAD_LEN: usize = 40;
/// Byte count of a Canceled payload: event, command, timestamp, order.
const CANCELED_PAYLOAD_LEN: usize = 32;
/// Byte count of a Trade payload.
/// Fields: event, command, timestamp, maker, taker, instrument, price, quantity.
const TRADE_PAYLOAD_LEN: usize = 72;

/// Answers the FrameKind of an EngineEvent variant.
fn frame_kind_for_event(event: &EngineEvent) -> FrameKind {
    match event {
        EngineEvent::Accepted { .. } => FrameKind::Accepted,
        EngineEvent::Rejected { .. } => FrameKind::Rejected,
        EngineEvent::Replaced { .. } => FrameKind::Replaced,
        EngineEvent::Canceled { .. } => FrameKind::Canceled,
        EngineEvent::Trade { .. } => FrameKind::Trade,
    }
}

/// Answers the payload length of an EngineEvent variant.
fn event_payload_len(event: &EngineEvent) -> usize {
    match event {
        EngineEvent::Accepted { .. } => ACCEPTED_PAYLOAD_LEN,
        EngineEvent::Rejected { .. } => REJECTED_PAYLOAD_LEN,
        EngineEvent::Replaced { .. } => REPLACED_PAYLOAD_LEN,
        EngineEvent::Canceled { .. } => CANCELED_PAYLOAD_LEN,
        EngineEvent::Trade { .. } => TRADE_PAYLOAD_LEN,
    }
}

/// Writes the shared packed payload of an EngineEvent into `payload`.
///
/// When encode has already reserved the exact payload region, it uses this
/// function so stream and datagram frames share one field layout.
fn write_event_payload(event: &EngineEvent, payload: &mut [u8]) {
    match event {
        EngineEvent::Accepted {
            event_sequence,
            command_sequence,
            timestamp_nanos,
            order_id,
            client_order_id,
            account_id,
        } => {
            write_u64(payload, 0, event_sequence.inner());
            write_u64(payload, 8, command_sequence.inner());
            write_u64(payload, 16, timestamp_nanos.inner());
            write_u64(payload, 24, order_id.inner());
            write_u64(payload, 32, client_order_id.inner());
            write_u64(payload, 40, account_id.inner());
        }
        EngineEvent::Rejected {
            event_sequence,
            command_sequence,
            timestamp_nanos,
            client_order_id,
            reason,
        } => {
            write_u64(payload, 0, event_sequence.inner());
            write_u64(payload, 8, command_sequence.inner());
            write_u64(payload, 16, timestamp_nanos.inner());
            write_u64(payload, 24, client_order_id.inner());
            payload[32] = encode_rejection_reason(reason);
        }
        EngineEvent::Replaced {
            event_sequence,
            command_sequence,
            timestamp_nanos,
            order_id,
            client_order_id,
        } => {
            write_u64(payload, 0, event_sequence.inner());
            write_u64(payload, 8, command_sequence.inner());
            write_u64(payload, 16, timestamp_nanos.inner());
            write_u64(payload, 24, order_id.inner());
            write_u64(payload, 32, client_order_id.inner());
        }
        EngineEvent::Canceled {
            event_sequence,
            command_sequence,
            timestamp_nanos,
            order_id,
        } => {
            write_u64(payload, 0, event_sequence.inner());
            write_u64(payload, 8, command_sequence.inner());
            write_u64(payload, 16, timestamp_nanos.inner());
            write_u64(payload, 24, order_id.inner());
        }
        EngineEvent::Trade {
            event_sequence,
            command_sequence,
            timestamp_nanos,
            maker_order_id,
            taker_order_id,
            instrument_id,
            price,
            quantity,
        } => {
            write_u64(payload, 0, event_sequence.inner());
            write_u64(payload, 8, command_sequence.inner());
            write_u64(payload, 16, timestamp_nanos.inner());
            write_u64(payload, 24, maker_order_id.inner());
            write_u64(payload, 32, taker_order_id.inner());
            write_u64(payload, 40, instrument_id.inner());
            write_i64(payload, 48, price.inner());
            write_u128(payload, 56, quantity.inner());
        }
    }
}

/// Answers an owned buffer of the EngineEvent payload.
/// Building the buffer allocates. The bytes match the payload written into a caller buffer.
pub fn encode_event_payload(event: &EngineEvent) -> Vec<u8> {
    let payload_len = event_payload_len(event);
    let mut payload = vec![0u8; payload_len];
    write_event_payload(event, &mut payload);
    payload
}

/// Answers the wire byte for an EngineEventRejectReason (0–3 named, 255 = Other).
fn encode_rejection_reason(reason: &EngineEventRejectReason) -> u8 {
    match reason {
        EngineEventRejectReason::InvalidQuantity => 0,
        EngineEventRejectReason::UnknownInstrument => 1,
        EngineEventRejectReason::OrderNotFound => 2,
        EngineEventRejectReason::InvalidPrice => 3,
        EngineEventRejectReason::Other => 255,
    }
}

/// Answers the EngineEventRejectReason of a wire byte; unrecognised values are
/// InvalidRejectionReason.
fn decode_rejection_reason(value: u8) -> Result<EngineEventRejectReason, DecodeError> {
    match value {
        0 => Ok(EngineEventRejectReason::InvalidQuantity),
        1 => Ok(EngineEventRejectReason::UnknownInstrument),
        2 => Ok(EngineEventRejectReason::OrderNotFound),
        3 => Ok(EngineEventRejectReason::InvalidPrice),
        255 => Ok(EngineEventRejectReason::Other),
        other => Err(DecodeError::InvalidRejectionReason(other)),
    }
}

/// Writes the packed stream frame of an EngineEvent into `buffer`.
/// Answers the number of bytes written, or an encode error when `buffer` cannot hold the frame.
pub fn encode_event(
    event: &EngineEvent,
    journal_sequence: JournalSequence,
    buffer: &mut [u8],
) -> Result<usize, EncodeError> {
    let kind = frame_kind_for_event(event);
    let payload_len = event_payload_len(event);
    let payload_region = Frame::encode_header(kind, journal_sequence, payload_len, buffer)
        .map_err(EncodeError::Frame)?;
    write_event_payload(event, payload_region);
    Ok(FRAME_HEADER_SIZE + payload_len)
}

/// Answers an EngineEvent of a shared event payload.
pub fn decode_event_payload(kind: FrameKind, payload: &[u8]) -> Result<EngineEvent, DecodeError> {
    match kind {
        FrameKind::Accepted => decode_accepted(payload),
        FrameKind::Rejected => decode_rejected(payload),
        FrameKind::Replaced => decode_replaced(payload),
        FrameKind::Canceled => decode_canceled(payload),
        FrameKind::Trade => decode_trade(payload),
        other => Err(DecodeError::UnknownKind(other.encode())),
    }
}

/// Answers an EngineEvent of a packed stream frame.
pub fn decode_event(buffer: &[u8]) -> Result<(EngineEvent, JournalSequence, usize), DecodeError> {
    let (kind, journal_sequence, payload) = match Frame::decode(buffer) {
        Ok(decoded) => decoded,
        Err(FrameDecodeError::UnknownKind(unknown_kind)) => {
            return Err(DecodeError::UnknownKind(unknown_kind));
        }
        Err(frame_error) => {
            return Err(DecodeError::Frame(frame_error));
        }
    };
    let consumed = FRAME_HEADER_SIZE + payload.len();
    let event = decode_event_payload(kind, payload)?;
    Ok((event, journal_sequence, consumed))
}

/// Writes the packed datagram of an EngineEvent into `buffer`.
/// Answers the number of bytes written, or an encode error when `buffer` cannot hold the datagram.
pub fn encode_event_datagram(
    event: &EngineEvent,
    session_id: SessionId,
    journal_sequence: JournalSequence,
    buffer: &mut [u8],
) -> Result<usize, EncodeError> {
    let kind = frame_kind_for_event(event);
    let payload_len = event_payload_len(event);
    let payload_region =
        datagram::encode_header(session_id, journal_sequence, kind, payload_len, buffer)
            .map_err(EncodeError::Datagram)?;
    write_event_payload(event, payload_region);
    Ok(DATAGRAM_HEADER_SIZE + payload_len)
}

/// Answers an EngineEvent of a packed datagram.
pub fn decode_event_datagram(
    buffer: &[u8],
) -> Result<(EngineEvent, SessionId, JournalSequence), DecodeError> {
    let (session_id, journal_sequence, kind, payload) = match datagram::decode(buffer) {
        Ok(decoded) => decoded,
        Err(DatagramDecodeError::UnknownKind(unknown_kind)) => {
            return Err(DecodeError::UnknownKind(unknown_kind));
        }
        Err(datagram_error) => {
            return Err(DecodeError::Datagram(datagram_error));
        }
    };
    let event = decode_event_payload(kind, payload)?;
    Ok((event, session_id, journal_sequence))
}

/// Answers an EngineEvent::Accepted of a payload, or a length mismatch when the
/// payload is not exactly 48 bytes.
fn decode_accepted(payload: &[u8]) -> Result<EngineEvent, DecodeError> {
    if payload.len() != ACCEPTED_PAYLOAD_LEN {
        return Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::Accepted.encode(),
            expected: ACCEPTED_PAYLOAD_LEN,
            actual: payload.len(),
        });
    }
    Ok(EngineEvent::Accepted {
        event_sequence: EventSequence::new(read_u64(payload, 0)),
        command_sequence: CommandSequence::new(read_u64(payload, 8)),
        timestamp_nanos: TimestampNanos::new(read_u64(payload, 16)),
        order_id: OrderId::new(read_u64(payload, 24)),
        client_order_id: ClientOrderId::new(read_u64(payload, 32)),
        account_id: AccountId::new(read_u64(payload, 40)),
    })
}

/// Answers an EngineEvent::Rejected of a payload, or a length mismatch when the
/// payload is not exactly 33 bytes.
fn decode_rejected(payload: &[u8]) -> Result<EngineEvent, DecodeError> {
    if payload.len() != REJECTED_PAYLOAD_LEN {
        return Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::Rejected.encode(),
            expected: REJECTED_PAYLOAD_LEN,
            actual: payload.len(),
        });
    }
    Ok(EngineEvent::Rejected {
        event_sequence: EventSequence::new(read_u64(payload, 0)),
        command_sequence: CommandSequence::new(read_u64(payload, 8)),
        timestamp_nanos: TimestampNanos::new(read_u64(payload, 16)),
        client_order_id: ClientOrderId::new(read_u64(payload, 24)),
        reason: decode_rejection_reason(payload[32])?,
    })
}

/// Answers an EngineEvent::Replaced of a payload, or a length mismatch when the
/// payload is not exactly 40 bytes.
fn decode_replaced(payload: &[u8]) -> Result<EngineEvent, DecodeError> {
    if payload.len() != REPLACED_PAYLOAD_LEN {
        return Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::Replaced.encode(),
            expected: REPLACED_PAYLOAD_LEN,
            actual: payload.len(),
        });
    }
    Ok(EngineEvent::Replaced {
        event_sequence: EventSequence::new(read_u64(payload, 0)),
        command_sequence: CommandSequence::new(read_u64(payload, 8)),
        timestamp_nanos: TimestampNanos::new(read_u64(payload, 16)),
        order_id: OrderId::new(read_u64(payload, 24)),
        client_order_id: ClientOrderId::new(read_u64(payload, 32)),
    })
}

/// Answers an EngineEvent::Canceled of a payload, or a length mismatch when the
/// payload is not exactly 32 bytes.
fn decode_canceled(payload: &[u8]) -> Result<EngineEvent, DecodeError> {
    if payload.len() != CANCELED_PAYLOAD_LEN {
        return Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::Canceled.encode(),
            expected: CANCELED_PAYLOAD_LEN,
            actual: payload.len(),
        });
    }
    Ok(EngineEvent::Canceled {
        event_sequence: EventSequence::new(read_u64(payload, 0)),
        command_sequence: CommandSequence::new(read_u64(payload, 8)),
        timestamp_nanos: TimestampNanos::new(read_u64(payload, 16)),
        order_id: OrderId::new(read_u64(payload, 24)),
    })
}

/// Answers an EngineEvent::Trade of a payload, or a length mismatch when the
/// payload is not exactly 72 bytes.
fn decode_trade(payload: &[u8]) -> Result<EngineEvent, DecodeError> {
    if payload.len() != TRADE_PAYLOAD_LEN {
        return Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::Trade.encode(),
            expected: TRADE_PAYLOAD_LEN,
            actual: payload.len(),
        });
    }
    Ok(EngineEvent::Trade {
        event_sequence: EventSequence::new(read_u64(payload, 0)),
        command_sequence: CommandSequence::new(read_u64(payload, 8)),
        timestamp_nanos: TimestampNanos::new(read_u64(payload, 16)),
        maker_order_id: OrderId::new(read_u64(payload, 24)),
        taker_order_id: OrderId::new(read_u64(payload, 32)),
        instrument_id: InstrumentId::new(read_u64(payload, 40)),
        price: engine_types::Price::from_i64(read_i64(payload, 48)),
        quantity: engine_types::Quantity::from_u128(read_u128(payload, 56)),
    })
}

#[cfg(test)]
#[path = "event_codec_test.rs"]
mod tests;
