//! Packed encode and decode of SequencedCommand.
//!
//! When a caller writes a command into a stream or datagram buffer, it uses
//! this module so both envelopes share one payload layout.
//!
//! ```
//! use engine_types::{
//!     AccountId, ClientOrderId, CommandSequence, EngineCommand, InstrumentId, JournalSequence,
//!     Price, Quantity, SequencedCommand, SessionId, Side,
//! };
//! use protocol::command_codec;
//!
//! let sequenced = SequencedCommand::new(
//!     CommandSequence::new(0),
//!     EngineCommand::NewLimit {
//!         account_id: AccountId::new(1),
//!         client_order_id: ClientOrderId::new(7),
//!         instrument_id: InstrumentId::new(2),
//!         side: Side::Buy,
//!         price: Price::new(100),
//!         quantity: Quantity::new(10),
//!     },
//! );
//! let mut buffer = [0u8; 128];
//! let encoded_len = command_codec::encode_command(
//!     &sequenced,
//!     JournalSequence::new(0),
//!     &mut buffer,
//! )
//! .expect("encode");
//! let (decoded, journal_sequence, _consumed) =
//!     command_codec::decode_command(&buffer).expect("decode");
//! assert_eq!(sequenced, decoded);
//! assert_eq!(journal_sequence.inner(), 0);
//! assert!(encoded_len > 0);
//! ```

use displaydoc::Display;
use engine_types::{
    AccountId, ClientOrderId, CommandSequence, EngineCommand, InstrumentId, JournalSequence,
    OrderId, Price, Quantity, SequencedCommand, SessionId, Side,
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

/// Failure of command frame decode.
/// Frame errors, datagram errors, unknown kinds, and payload mismatches stay
/// distinct.
#[derive(Display, Debug, Error, PartialEq, Eq)]
pub enum DecodeError {
    /// Frame decode failed: {0}
    Frame(#[source] FrameDecodeError),
    /// Datagram decode failed: {0}
    Datagram(#[source] DatagramDecodeError),
    /// Unknown command kind: {0:#04x}
    UnknownKind(u8),
    /// Payload length is {actual} bytes, expected {expected} for kind {kind:#04x}
    PayloadLengthMismatch {
        kind: u8,
        expected: usize,
        actual: usize,
    },
    /// Failed to decode {field} from payload
    FieldDecode { field: &'static str },
}

/// Failure of command frame encode.
/// Frame and datagram packing failures stay distinct.
#[derive(Display, Debug, Error, PartialEq, Eq)]
pub enum EncodeError {
    /// Frame encode failed: {0}
    Frame(#[source] FrameEncodeError),
    /// Datagram encode failed: {0}
    Datagram(#[source] DatagramEncodeError),
}

/// Byte count of a NewLimit payload.
/// Fields: command sequence, account, client, instrument, side, price, quantity.
const NEW_LIMIT_PAYLOAD_LEN: usize = 57;
/// Byte count of a NewMarket payload.
/// Fields: command sequence, account, client, instrument, side, quantity.
const NEW_MARKET_PAYLOAD_LEN: usize = 49;
/// Byte count of a CancelByOrder payload: command_sequence and order_id.
const CANCEL_BY_ORDER_PAYLOAD_LEN: usize = 16;
/// Byte count of a CancelByClient payload: command_sequence, account, and client.
const CANCEL_BY_CLIENT_PAYLOAD_LEN: usize = 24;
/// Byte count of a Replace payload: command_sequence, order_id, client, price, quantity.
const REPLACE_PAYLOAD_LEN: usize = 48;

/// Answers the payload length of an EngineCommand variant.
fn command_payload_len(command: &EngineCommand) -> usize {
    match command {
        EngineCommand::NewLimit { .. } => NEW_LIMIT_PAYLOAD_LEN,
        EngineCommand::NewMarket { .. } => NEW_MARKET_PAYLOAD_LEN,
        EngineCommand::CancelByOrder { .. } => CANCEL_BY_ORDER_PAYLOAD_LEN,
        EngineCommand::CancelByClient { .. } => CANCEL_BY_CLIENT_PAYLOAD_LEN,
        EngineCommand::Replace { .. } => REPLACE_PAYLOAD_LEN,
    }
}

/// Writes the shared packed payload of a SequencedCommand into `payload`.
///
/// When encode has already reserved the exact payload region, it uses this
/// function so stream and datagram frames share one field layout.
fn write_command_payload(sequenced: &SequencedCommand, payload: &mut [u8]) {
    write_u64(payload, 0, sequenced.command_sequence().inner());
    match sequenced.command() {
        EngineCommand::NewLimit {
            account_id,
            client_order_id,
            instrument_id,
            side,
            price,
            quantity,
        } => {
            write_u64(payload, 8, account_id.inner());
            write_u64(payload, 16, client_order_id.inner());
            write_u64(payload, 24, instrument_id.inner());
            payload[32] = encode_side(side);
            write_i64(payload, 33, price.inner());
            write_u128(payload, 41, quantity.inner());
        }
        EngineCommand::NewMarket {
            account_id,
            client_order_id,
            instrument_id,
            side,
            quantity,
        } => {
            write_u64(payload, 8, account_id.inner());
            write_u64(payload, 16, client_order_id.inner());
            write_u64(payload, 24, instrument_id.inner());
            payload[32] = encode_side(side);
            write_u128(payload, 33, quantity.inner());
        }
        EngineCommand::CancelByOrder { order_id } => {
            write_u64(payload, 8, order_id.inner());
        }
        EngineCommand::CancelByClient {
            account_id,
            client_order_id,
        } => {
            write_u64(payload, 8, account_id.inner());
            write_u64(payload, 16, client_order_id.inner());
        }
        EngineCommand::Replace {
            order_id,
            client_order_id,
            price,
            quantity,
        } => {
            write_u64(payload, 8, order_id.inner());
            write_u64(payload, 16, client_order_id.inner());
            write_i64(payload, 24, price.inner());
            write_u128(payload, 32, quantity.inner());
        }
    }
}

/// Answers an owned buffer of the SequencedCommand payload.
/// Building the buffer allocates. The bytes match the payload written into a caller buffer.
pub fn encode_command_payload(sequenced: &SequencedCommand) -> Vec<u8> {
    let payload_len = command_payload_len(&sequenced.command());
    let mut payload = vec![0u8; payload_len];
    write_command_payload(sequenced, &mut payload);
    payload
}

/// Answers the wire byte for a Side (0 = Buy, 1 = Sell).
fn encode_side(side: Side) -> u8 {
    match side {
        Side::Buy => 0u8,
        Side::Sell => 1u8,
    }
}

/// Answers the Side of a wire byte (0 = Buy, 1 = Sell); unknown bytes are a field decode error.
fn decode_side(side_byte: u8) -> Result<Side, DecodeError> {
    match side_byte {
        0 => Ok(Side::Buy),
        1 => Ok(Side::Sell),
        _ => Err(DecodeError::FieldDecode { field: "side" }),
    }
}

/// Answers the FrameKind of an EngineCommand variant.
fn frame_kind_for_command(command: &EngineCommand) -> FrameKind {
    match command {
        EngineCommand::NewLimit { .. } => FrameKind::NewLimit,
        EngineCommand::NewMarket { .. } => FrameKind::NewMarket,
        EngineCommand::CancelByOrder { .. } => FrameKind::CancelByOrder,
        EngineCommand::CancelByClient { .. } => FrameKind::CancelByClient,
        EngineCommand::Replace { .. } => FrameKind::Replace,
    }
}

/// Writes the packed stream frame of a SequencedCommand into `buffer`.
/// Answers the number of bytes written, or an encode error when `buffer` cannot hold the frame.
pub fn encode_command(
    sequenced: &SequencedCommand,
    journal_sequence: JournalSequence,
    buffer: &mut [u8],
) -> Result<usize, EncodeError> {
    let kind = frame_kind_for_command(&sequenced.command());
    let payload_len = command_payload_len(&sequenced.command());
    let payload_region = Frame::encode_header(kind, journal_sequence, payload_len, buffer)
        .map_err(EncodeError::Frame)?;
    write_command_payload(sequenced, payload_region);
    Ok(FRAME_HEADER_SIZE + payload_len)
}

/// Answers a SequencedCommand of a shared command payload.
pub fn decode_command_payload(
    kind: FrameKind,
    payload: &[u8],
) -> Result<SequencedCommand, DecodeError> {
    match kind {
        FrameKind::NewLimit => decode_new_limit(payload),
        FrameKind::NewMarket => decode_new_market(payload),
        FrameKind::CancelByOrder => decode_cancel_by_order(payload),
        FrameKind::CancelByClient => decode_cancel_by_client(payload),
        FrameKind::Replace => decode_replace(payload),
        other => Err(DecodeError::UnknownKind(other.encode())),
    }
}

/// Answers a SequencedCommand of a packed stream frame.
pub fn decode_command(
    buffer: &[u8],
) -> Result<(SequencedCommand, JournalSequence, usize), DecodeError> {
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
    let sequenced = decode_command_payload(kind, payload)?;
    Ok((sequenced, journal_sequence, consumed))
}

/// Writes the packed datagram of a SequencedCommand into `buffer`.
/// Answers the number of bytes written, or an encode error when `buffer` cannot hold the datagram.
pub fn encode_command_datagram(
    sequenced: &SequencedCommand,
    session_id: SessionId,
    journal_sequence: JournalSequence,
    buffer: &mut [u8],
) -> Result<usize, EncodeError> {
    let kind = frame_kind_for_command(&sequenced.command());
    let payload_len = command_payload_len(&sequenced.command());
    let payload_region =
        datagram::encode_header(session_id, journal_sequence, kind, payload_len, buffer)
            .map_err(EncodeError::Datagram)?;
    write_command_payload(sequenced, payload_region);
    Ok(DATAGRAM_HEADER_SIZE + payload_len)
}

/// Answers a SequencedCommand of a packed datagram.
pub fn decode_command_datagram(
    buffer: &[u8],
) -> Result<(SequencedCommand, SessionId, JournalSequence), DecodeError> {
    let (session_id, journal_sequence, kind, payload) = match datagram::decode(buffer) {
        Ok(decoded) => decoded,
        Err(DatagramDecodeError::UnknownKind(unknown_kind)) => {
            return Err(DecodeError::UnknownKind(unknown_kind));
        }
        Err(datagram_error) => {
            return Err(DecodeError::Datagram(datagram_error));
        }
    };
    let sequenced = decode_command_payload(kind, payload)?;
    Ok((sequenced, session_id, journal_sequence))
}

/// Answers a SequencedCommand of a NewLimit payload, or a length mismatch when
/// the payload is not exactly 57 bytes (NEW_LIMIT_PAYLOAD_LEN).
fn decode_new_limit(payload: &[u8]) -> Result<SequencedCommand, DecodeError> {
    if payload.len() != NEW_LIMIT_PAYLOAD_LEN {
        return Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::NewLimit.encode(),
            expected: NEW_LIMIT_PAYLOAD_LEN,
            actual: payload.len(),
        });
    }
    let command_sequence = CommandSequence::new(read_u64(payload, 0));
    let side = decode_side(payload[32])?;
    Ok(SequencedCommand::new(
        command_sequence,
        EngineCommand::NewLimit {
            account_id: AccountId::new(read_u64(payload, 8)),
            client_order_id: ClientOrderId::new(read_u64(payload, 16)),
            instrument_id: InstrumentId::new(read_u64(payload, 24)),
            side,
            price: Price::from_i64(read_i64(payload, 33)),
            quantity: Quantity::from_u128(read_u128(payload, 41)),
        },
    ))
}

/// Answers a SequencedCommand of a NewMarket payload, or a length mismatch when
/// the payload is not exactly 49 bytes.
fn decode_new_market(payload: &[u8]) -> Result<SequencedCommand, DecodeError> {
    if payload.len() != NEW_MARKET_PAYLOAD_LEN {
        return Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::NewMarket.encode(),
            expected: NEW_MARKET_PAYLOAD_LEN,
            actual: payload.len(),
        });
    }
    let command_sequence = CommandSequence::new(read_u64(payload, 0));
    let side = decode_side(payload[32])?;
    Ok(SequencedCommand::new(
        command_sequence,
        EngineCommand::NewMarket {
            account_id: AccountId::new(read_u64(payload, 8)),
            client_order_id: ClientOrderId::new(read_u64(payload, 16)),
            instrument_id: InstrumentId::new(read_u64(payload, 24)),
            side,
            quantity: Quantity::from_u128(read_u128(payload, 33)),
        },
    ))
}

/// Answers a SequencedCommand of a CancelByOrder payload, or a length mismatch
/// when the payload is not exactly 16 bytes.
fn decode_cancel_by_order(payload: &[u8]) -> Result<SequencedCommand, DecodeError> {
    if payload.len() != CANCEL_BY_ORDER_PAYLOAD_LEN {
        return Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::CancelByOrder.encode(),
            expected: CANCEL_BY_ORDER_PAYLOAD_LEN,
            actual: payload.len(),
        });
    }
    Ok(SequencedCommand::new(
        CommandSequence::new(read_u64(payload, 0)),
        EngineCommand::CancelByOrder {
            order_id: OrderId::new(read_u64(payload, 8)),
        },
    ))
}

/// Answers a SequencedCommand of a CancelByClient payload, or a length mismatch
/// when the payload is not exactly 24 bytes.
fn decode_cancel_by_client(payload: &[u8]) -> Result<SequencedCommand, DecodeError> {
    if payload.len() != CANCEL_BY_CLIENT_PAYLOAD_LEN {
        return Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::CancelByClient.encode(),
            expected: CANCEL_BY_CLIENT_PAYLOAD_LEN,
            actual: payload.len(),
        });
    }
    Ok(SequencedCommand::new(
        CommandSequence::new(read_u64(payload, 0)),
        EngineCommand::CancelByClient {
            account_id: AccountId::new(read_u64(payload, 8)),
            client_order_id: ClientOrderId::new(read_u64(payload, 16)),
        },
    ))
}

/// Answers a SequencedCommand of a Replace payload, or a length mismatch when
/// the payload is not exactly 48 bytes.
fn decode_replace(payload: &[u8]) -> Result<SequencedCommand, DecodeError> {
    if payload.len() != REPLACE_PAYLOAD_LEN {
        return Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::Replace.encode(),
            expected: REPLACE_PAYLOAD_LEN,
            actual: payload.len(),
        });
    }
    Ok(SequencedCommand::new(
        CommandSequence::new(read_u64(payload, 0)),
        EngineCommand::Replace {
            order_id: OrderId::new(read_u64(payload, 8)),
            client_order_id: ClientOrderId::new(read_u64(payload, 16)),
            price: Price::from_i64(read_i64(payload, 24)),
            quantity: Quantity::from_u128(read_u128(payload, 32)),
        },
    ))
}

#[cfg(test)]
#[path = "command_codec_test.rs"]
mod tests;
