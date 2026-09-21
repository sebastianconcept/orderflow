//! Packed datagram envelope for one OFL1 frame.
//!
//! When a caller writes a UDP-shaped buffer, it uses this module so SessionId,
//! JournalSequence, kind, and payload sit in one datagram. The datagram is the
//! whole envelope.

use displaydoc::Display;
use engine_types::{JournalSequence, SessionId};
use thiserror::Error;

use crate::frame::FrameKind;

/// Byte count of the datagram header: SessionId, JournalSequence, and kind.
pub const DATAGRAM_HEADER_SIZE: usize = 17;

/// Failure of datagram decode.
/// A truncated header and an unknown kind are distinct.
#[derive(Display, Debug, Error, PartialEq, Eq)]
pub enum DecodeError {
    /// Buffer too short for datagram header: expected at least {expected} bytes, got {actual}
    HeaderTooShort { expected: usize, actual: usize },
    /// Unknown frame kind: {0:#04x}
    UnknownKind(u8),
}

/// Failure of datagram encode.
/// The buffer must hold the header and payload.
#[derive(Display, Debug, Error, PartialEq, Eq)]
pub enum EncodeError {
    /// Buffer too short: expected at least {expected} bytes, got {actual}
    BufferTooShort { expected: usize, actual: usize },
}

/// Answers a packed datagram of SessionId, JournalSequence, kind, and payload.
pub fn encode(
    session_id: SessionId,
    journal_sequence: JournalSequence,
    kind: FrameKind,
    payload: &[u8],
    buffer: &mut [u8],
) -> Result<usize, EncodeError> {
    let total_len = DATAGRAM_HEADER_SIZE + payload.len();
    if buffer.len() < total_len {
        return Err(EncodeError::BufferTooShort {
            expected: total_len,
            actual: buffer.len(),
        });
    }
    buffer[0..8].copy_from_slice(&session_id.inner().to_le_bytes());
    buffer[8..16].copy_from_slice(&journal_sequence.inner().to_le_bytes());
    buffer[16] = kind.encode();
    if !payload.is_empty() {
        buffer[17..total_len].copy_from_slice(payload);
    }
    Ok(total_len)
}

/// Answers the SessionId, JournalSequence, kind, and payload of a packed datagram.
/// The payload stays opaque bytes.
pub fn decode(
    buffer: &[u8],
) -> Result<(SessionId, JournalSequence, FrameKind, &[u8]), DecodeError> {
    if buffer.len() < DATAGRAM_HEADER_SIZE {
        return Err(DecodeError::HeaderTooShort {
            expected: DATAGRAM_HEADER_SIZE,
            actual: buffer.len(),
        });
    }
    let session_id = SessionId::new(u64::from_le_bytes([
        buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
    ]));
    let journal_sequence = JournalSequence::new(u64::from_le_bytes([
        buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14],
        buffer[15],
    ]));
    let kind = FrameKind::decode(buffer[16]).map_err(DecodeError::UnknownKind)?;
    let payload = &buffer[DATAGRAM_HEADER_SIZE..];
    Ok((session_id, journal_sequence, kind, payload))
}

/// Answers a packed datagram of a Heartbeat control frame with empty payload.
pub fn encode_heartbeat(
    session_id: SessionId,
    journal_sequence: JournalSequence,
    buffer: &mut [u8],
) -> Result<usize, EncodeError> {
    encode(
        session_id,
        journal_sequence,
        FrameKind::Heartbeat,
        &[],
        buffer,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datagram_encode_decode_round_trips() {
        // Given a datagram with SessionId, JournalSequence, kind, and payload
        let session_id = SessionId::new(42);
        let journal_sequence = JournalSequence::new(7);
        let payload = [1u8, 2u8, 3u8];
        let mut buffer = [0u8; 64];

        // When we encode and decode
        let encoded_len = encode(
            session_id,
            journal_sequence,
            FrameKind::NewLimit,
            &payload,
            &mut buffer,
        )
        .expect("encode");
        let (decoded_session, decoded_journal, decoded_kind, decoded_payload) =
            decode(&buffer[..encoded_len]).expect("decode");

        // Then round trip preserves all fields
        assert_eq!(decoded_session, session_id);
        assert_eq!(decoded_journal, journal_sequence);
        assert_eq!(decoded_kind, FrameKind::NewLimit);
        assert_eq!(decoded_payload, payload.as_ref());
        assert_eq!(encoded_len, DATAGRAM_HEADER_SIZE + 3);
    }

    #[test]
    fn decode_rejects_header_too_short() {
        // Given a buffer shorter than the datagram header
        let short_buf = [0u8; 16];

        // When we decode
        let result = decode(&short_buf);

        // Then it fails closed
        assert!(matches!(result, Err(DecodeError::HeaderTooShort { .. })));
    }

    #[test]
    fn decode_rejects_unknown_kind() {
        // Given a datagram with unknown kind
        let mut buffer = [0u8; DATAGRAM_HEADER_SIZE];
        buffer[16] = 0xFF;

        // When we decode
        let result = decode(&buffer);

        // Then it fails closed
        assert!(matches!(result, Err(DecodeError::UnknownKind(0xFF))));
    }

    #[test]
    fn heartbeat_encodes_as_kind_zero_with_empty_payload() {
        // Given a heartbeat for the last JournalSequence
        let mut buffer = [0u8; DATAGRAM_HEADER_SIZE];
        let encoded_len =
            encode_heartbeat(SessionId::new(1), JournalSequence::new(99), &mut buffer)
                .expect("encode");

        // When we decode
        let (_session, journal, kind, payload) = decode(&buffer[..encoded_len]).expect("decode");

        // Then kind is Heartbeat and payload is empty
        assert_eq!(kind, FrameKind::Heartbeat);
        assert_eq!(journal.inner(), 99);
        assert!(payload.is_empty());
        assert_eq!(encoded_len, DATAGRAM_HEADER_SIZE);
    }

    #[test]
    fn decode_rejects_reserved_kinds() {
        // Given datagrams whose kind is reserved iceberg or RFQ
        for kind in [0x06u8, 0x40u8, 0xA0u8] {
            let mut buffer = [0u8; DATAGRAM_HEADER_SIZE];
            buffer[16] = kind;

            // When we decode
            let result = decode(&buffer);

            // Then decode fails closed with that opcode
            assert_eq!(result, Err(DecodeError::UnknownKind(kind)));
        }
    }
}
