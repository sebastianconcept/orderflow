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
};
use crate::frame::{
    DecodeError as FrameDecodeError, EncodeError as FrameEncodeError, Frame, FrameKind,
    FRAME_HEADER_SIZE,
};
use crate::packed_le::{read_i64, read_u128, read_u64};

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

/// Answers the shared packed payload of a SequencedCommand.
/// Stream and datagram envelopes wrap these bytes.
pub fn encode_command_payload(sequenced: &SequencedCommand) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&sequenced.command_sequence().inner().to_le_bytes());
    match sequenced.command() {
        EngineCommand::NewLimit {
            account_id,
            client_order_id,
            instrument_id,
            side,
            price,
            quantity,
        } => {
            payload.extend_from_slice(&account_id.inner().to_le_bytes());
            payload.extend_from_slice(&client_order_id.inner().to_le_bytes());
            payload.extend_from_slice(&instrument_id.inner().to_le_bytes());
            payload.push(encode_side(side));
            payload.extend_from_slice(&price.inner().to_le_bytes());
            payload.extend_from_slice(&quantity.inner().to_le_bytes());
        }
        EngineCommand::NewMarket {
            account_id,
            client_order_id,
            instrument_id,
            side,
            quantity,
        } => {
            payload.extend_from_slice(&account_id.inner().to_le_bytes());
            payload.extend_from_slice(&client_order_id.inner().to_le_bytes());
            payload.extend_from_slice(&instrument_id.inner().to_le_bytes());
            payload.push(encode_side(side));
            payload.extend_from_slice(&quantity.inner().to_le_bytes());
        }
        EngineCommand::CancelByOrder { order_id } => {
            payload.extend_from_slice(&order_id.inner().to_le_bytes());
        }
        EngineCommand::CancelByClient {
            account_id,
            client_order_id,
        } => {
            payload.extend_from_slice(&account_id.inner().to_le_bytes());
            payload.extend_from_slice(&client_order_id.inner().to_le_bytes());
        }
        EngineCommand::Replace {
            order_id,
            client_order_id,
            price,
            quantity,
        } => {
            payload.extend_from_slice(&order_id.inner().to_le_bytes());
            payload.extend_from_slice(&client_order_id.inner().to_le_bytes());
            payload.extend_from_slice(&price.inner().to_le_bytes());
            payload.extend_from_slice(&quantity.inner().to_le_bytes());
        }
    }
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

/// Answers a packed stream frame of a SequencedCommand.
pub fn encode_command(
    sequenced: &SequencedCommand,
    journal_sequence: JournalSequence,
    buffer: &mut [u8],
) -> Result<usize, EncodeError> {
    let kind = frame_kind_for_command(&sequenced.command());
    let payload = encode_command_payload(sequenced);
    Frame::encode(kind, journal_sequence, &payload, buffer).map_err(EncodeError::Frame)
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

/// Answers a packed datagram of a SequencedCommand.
pub fn encode_command_datagram(
    sequenced: &SequencedCommand,
    session_id: SessionId,
    journal_sequence: JournalSequence,
    buffer: &mut [u8],
) -> Result<usize, EncodeError> {
    let kind = frame_kind_for_command(&sequenced.command());
    let payload = encode_command_payload(sequenced);
    datagram::encode(session_id, journal_sequence, kind, &payload, buffer)
        .map_err(EncodeError::Datagram)
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
/// the payload is not exactly 57 bytes.
fn decode_new_limit(payload: &[u8]) -> Result<SequencedCommand, DecodeError> {
    // command_sequence(8) + account(8) + client(8) + instrument(8) + side(1) + price(8) + quantity(16)
    const EXPECTED_LEN: usize = 57;
    if payload.len() != EXPECTED_LEN {
        return Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::NewLimit.encode(),
            expected: EXPECTED_LEN,
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
    // command_sequence(8) + account(8) + client(8) + instrument(8) + side(1) + quantity(16)
    const EXPECTED_LEN: usize = 49;
    if payload.len() != EXPECTED_LEN {
        return Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::NewMarket.encode(),
            expected: EXPECTED_LEN,
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
    const EXPECTED_LEN: usize = 16;
    if payload.len() != EXPECTED_LEN {
        return Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::CancelByOrder.encode(),
            expected: EXPECTED_LEN,
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
    const EXPECTED_LEN: usize = 24;
    if payload.len() != EXPECTED_LEN {
        return Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::CancelByClient.encode(),
            expected: EXPECTED_LEN,
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
    const EXPECTED_LEN: usize = 48;
    if payload.len() != EXPECTED_LEN {
        return Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::Replace.encode(),
            expected: EXPECTED_LEN,
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
mod tests {
    use super::*;

    fn sample_new_limit(command_sequence: u64) -> SequencedCommand {
        SequencedCommand::new(
            CommandSequence::new(command_sequence),
            EngineCommand::NewLimit {
                account_id: AccountId::new(1),
                client_order_id: ClientOrderId::new(7),
                instrument_id: InstrumentId::new(2),
                side: Side::Buy,
                price: Price::new(100),
                quantity: Quantity::new(10),
            },
        )
    }

    #[test]
    fn command_codec_new_limit_round_trips() {
        // Given a sequenced NewLimit command
        let sequenced = sample_new_limit(3);
        let mut buffer = [0u8; 128];

        // When we encode and decode on the stream envelope
        let encoded_len =
            encode_command(&sequenced, JournalSequence::new(9), &mut buffer).expect("encode");
        let (decoded, journal_sequence, consumed) = decode_command(&buffer).expect("decode");

        // Then round trip preserves the command and JournalSequence
        assert_eq!(decoded, sequenced);
        assert_eq!(journal_sequence.inner(), 9);
        assert_eq!(encoded_len, consumed);
        assert!(encoded_len <= 128);
    }

    #[test]
    fn command_codec_new_market_has_no_price_bytes() {
        // Given a sequenced NewMarket command
        let sequenced = SequencedCommand::new(
            CommandSequence::new(1),
            EngineCommand::NewMarket {
                account_id: AccountId::new(1),
                client_order_id: ClientOrderId::new(8),
                instrument_id: InstrumentId::new(2),
                side: Side::Sell,
                quantity: Quantity::new(5),
            },
        );
        let payload = encode_command_payload(&sequenced);
        let new_limit_payload = encode_command_payload(&sample_new_limit(1));

        // When we compare payload sizes
        // Then NewMarket is shorter than NewLimit (no price)
        assert_eq!(payload.len(), 49);
        assert_eq!(new_limit_payload.len(), 57);
        assert!(payload.len() < new_limit_payload.len());
    }

    #[test]
    fn duplicate_client_order_id_gets_different_payload_bytes() {
        // Given two NewLimit commands that share AccountId and ClientOrderId
        let first = sample_new_limit(0);
        let second = sample_new_limit(1);

        // When we encode both payloads
        let first_payload = encode_command_payload(&first);
        let second_payload = encode_command_payload(&second);

        // Then the payloads differ because CommandSequence differs
        assert_ne!(first_payload, second_payload);
    }

    #[test]
    fn stream_and_datagram_round_trip_same_payload() {
        // Given a sequenced NewLimit command
        let sequenced = sample_new_limit(4);
        let session_id = SessionId::new(42);
        let journal_sequence = JournalSequence::new(11);
        let mut stream_buf = [0u8; 128];
        let mut datagram_buf = [0u8; 128];

        // When we encode both envelopes and decode
        encode_command(&sequenced, journal_sequence, &mut stream_buf).expect("stream encode");
        let datagram_len =
            encode_command_datagram(&sequenced, session_id, journal_sequence, &mut datagram_buf)
                .expect("datagram encode");
        let (stream_decoded, stream_journal, _) = decode_command(&stream_buf).expect("stream");
        let (datagram_decoded, datagram_session, datagram_journal) =
            decode_command_datagram(&datagram_buf[..datagram_len]).expect("datagram");

        // Then both envelopes yield the same SequencedCommand and JournalSequence
        assert_eq!(stream_decoded, sequenced);
        assert_eq!(datagram_decoded, sequenced);
        assert_eq!(stream_journal, journal_sequence);
        assert_eq!(datagram_journal, journal_sequence);
        assert_eq!(datagram_session, session_id);
        assert_eq!(
            encode_command_payload(&stream_decoded),
            encode_command_payload(&datagram_decoded)
        );
    }

    #[test]
    fn decode_rejects_unknown_command_kind() {
        // Given a stream frame with unknown kind
        let mut buf = [0u8; FRAME_HEADER_SIZE];
        buf[0..2].copy_from_slice(&9u16.to_le_bytes());
        buf[2] = 0xFF;
        buf[3..11].copy_from_slice(&0u64.to_le_bytes());

        // When we decode
        let result = decode_command(&buf);

        // Then it fails closed
        assert!(matches!(result, Err(DecodeError::UnknownKind(0xFF))));
    }

    #[test]
    fn decode_rejects_invalid_side() {
        // Given a NewLimit payload whose side byte is not Buy or Sell
        let sequenced = sample_new_limit(0);
        let mut payload = encode_command_payload(&sequenced);
        payload[32] = 0x02;

        // When we decode the payload
        let result = decode_command_payload(FrameKind::NewLimit, &payload);

        // Then decode names the side field
        assert!(matches!(
            result,
            Err(DecodeError::FieldDecode { field: "side" })
        ));
    }

    #[test]
    fn decode_rejects_new_limit_payload_length_mismatch() {
        // Given a NewLimit payload that is one byte short
        let sequenced = sample_new_limit(0);
        let payload = encode_command_payload(&sequenced);
        let short_payload = &payload[..payload.len() - 1];

        // When we decode the payload
        let result = decode_command_payload(FrameKind::NewLimit, short_payload);

        // Then decode names the expected NewLimit length
        assert_eq!(
            result,
            Err(DecodeError::PayloadLengthMismatch {
                kind: FrameKind::NewLimit.encode(),
                expected: 57,
                actual: 56,
            })
        );
    }
}
