//! Length-prefixed OFL1 stream envelope for one journal record.
//!
//! When a caller writes a stream journal record, it uses this module so kind,
//! JournalSequence, and payload sit behind a length prefix.

use displaydoc::Display;
use engine_types::JournalSequence;
use thiserror::Error;

/// Byte count of the stream frame header: frame_len, kind, and JournalSequence.
pub const FRAME_HEADER_SIZE: usize = 11;

/// Failure of frame decode.
/// Truncated headers, short payloads, too-small frame_len, and unknown kinds are
/// distinct.
#[derive(Display, Debug, Error, PartialEq, Eq)]
pub enum DecodeError {
    /// Buffer too short for frame header: expected at least {expected} bytes, got {actual}
    HeaderTooShort { expected: usize, actual: usize },
    /// Frame payload too short: frame_len is {frame_len}, but only {available} bytes available
    PayloadTooShort { frame_len: u16, available: usize },
    /// Frame length must cover kind and journal_sequence
    FrameLengthTooSmall,
    /// Unknown frame kind: {0:#04x}
    UnknownKind(u8),
}

/// Failure of frame encode.
/// A short buffer and a payload that overflows u16 are distinct.
#[derive(Display, Debug, Error, PartialEq, Eq)]
pub enum EncodeError {
    /// Buffer too short: expected at least {expected} bytes, got {actual}
    BufferTooShort { expected: usize, actual: usize },
    /// Payload exceeds the u16 frame length limit ({0} bytes)
    PayloadTooLarge(usize),
}

/// FrameKind is the opcode of one OFL1 command, event, or control frame.
/// When encode or decode names a kind, it uses this type so the set is closed:
/// reserved RFQ and iceberg opcodes stay unknown.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FrameKind {
    /// Control frame that repeats the last JournalSequence.
    Heartbeat = 0x00,
    /// CLOB command to open a limit order.
    NewLimit = 0x01,
    /// CLOB command to open a market order.
    NewMarket = 0x02,
    /// CLOB command to cancel by engine OrderId.
    CancelByOrder = 0x03,
    /// CLOB command to cancel by opening New identifiers.
    CancelByClient = 0x04,
    /// CLOB command to amend price or quantity.
    Replace = 0x05,
    /// CLOB event for an accepted NewLimit or NewMarket.
    Accepted = 0x81,
    /// CLOB event for a rejected command.
    Rejected = 0x82,
    /// CLOB event for an amended resting order.
    Replaced = 0x83,
    /// CLOB event for a removed resting order.
    Canceled = 0x84,
    /// CLOB event for a maker and taker fill.
    Trade = 0x85,
}

impl FrameKind {
    /// Answers whether this kind is a client command (Heartbeat and events are other kinds).
    pub fn is_command(&self) -> bool {
        matches!(
            self,
            FrameKind::NewLimit
                | FrameKind::NewMarket
                | FrameKind::CancelByOrder
                | FrameKind::CancelByClient
                | FrameKind::Replace
        )
    }

    /// Answers whether this kind is an emitted event (Heartbeat and commands are other kinds).
    pub fn is_event(&self) -> bool {
        matches!(
            self,
            FrameKind::Accepted
                | FrameKind::Rejected
                | FrameKind::Replaced
                | FrameKind::Canceled
                | FrameKind::Trade
        )
    }

    /// Answers whether this kind is a control frame (Heartbeat).
    pub fn is_control(&self) -> bool {
        matches!(self, FrameKind::Heartbeat)
    }

    /// Answers the journal opcode of this kind.
    pub fn encode(self) -> u8 {
        self as u8
    }

    /// Answers the FrameKind of a journal opcode.
    /// Reserved RFQ and iceberg bytes stay unknown. The Err value is the opcode.
    pub fn decode(value: u8) -> Result<Self, u8> {
        match value {
            0x00 => Ok(FrameKind::Heartbeat),
            0x01 => Ok(FrameKind::NewLimit),
            0x02 => Ok(FrameKind::NewMarket),
            0x03 => Ok(FrameKind::CancelByOrder),
            0x04 => Ok(FrameKind::CancelByClient),
            0x05 => Ok(FrameKind::Replace),
            0x81 => Ok(FrameKind::Accepted),
            0x82 => Ok(FrameKind::Rejected),
            0x83 => Ok(FrameKind::Replaced),
            0x84 => Ok(FrameKind::Canceled),
            0x85 => Ok(FrameKind::Trade),
            unknown => Err(unknown),
        }
    }
}

/// Frame is one length-prefixed kind, JournalSequence, and payload.
/// When a caller wraps a command or event for the stream journal, it uses this
/// type so the payload stays opaque bytes.
#[derive(Debug)]
pub struct Frame<'a> {
    kind: FrameKind,
    journal_sequence: JournalSequence,
    payload: &'a [u8],
}

impl<'a> Frame<'a> {
    /// Answers a Frame from kind, JournalSequence, and payload.
    pub fn new(kind: FrameKind, journal_sequence: JournalSequence, payload: &'a [u8]) -> Self {
        Frame {
            kind,
            journal_sequence,
            payload,
        }
    }

    /// Answers the opcode of this frame.
    pub fn kind(&self) -> FrameKind {
        self.kind
    }

    /// Answers the OFL1 frame counter of this frame.
    pub fn journal_sequence(&self) -> JournalSequence {
        self.journal_sequence
    }

    /// Answers the bytes after kind and JournalSequence.
    pub fn payload(&self) -> &[u8] {
        self.payload
    }

    /// Answers the byte count of a frame with this payload length.
    pub fn encoded_size(payload_len: usize) -> usize {
        FRAME_HEADER_SIZE + payload_len
    }

    /// Answers a packed stream frame of a kind, JournalSequence, and payload.
    pub fn encode(
        kind: FrameKind,
        journal_sequence: JournalSequence,
        payload: &[u8],
        buffer: &mut [u8],
    ) -> Result<usize, EncodeError> {
        let total_len = FRAME_HEADER_SIZE + payload.len();
        // frame_len is the content after the length prefix, not the full encoded size.
        let content_len = 1usize.saturating_add(8).saturating_add(payload.len());
        if content_len > u16::MAX as usize {
            return Err(EncodeError::PayloadTooLarge(payload.len()));
        }
        if buffer.len() < total_len {
            return Err(EncodeError::BufferTooShort {
                expected: total_len,
                actual: buffer.len(),
            });
        }

        let frame_len = content_len as u16;
        buffer[0..2].copy_from_slice(&frame_len.to_le_bytes());
        buffer[2] = kind.encode();
        buffer[3..11].copy_from_slice(&journal_sequence.inner().to_le_bytes());
        if !payload.is_empty() {
            buffer[11..total_len].copy_from_slice(payload);
        }
        Ok(total_len)
    }

    /// Answers the kind, JournalSequence, and payload of a packed stream frame.
    pub fn decode(buffer: &[u8]) -> Result<(FrameKind, JournalSequence, &[u8]), DecodeError> {
        if buffer.len() < FRAME_HEADER_SIZE {
            return Err(DecodeError::HeaderTooShort {
                expected: FRAME_HEADER_SIZE,
                actual: buffer.len(),
            });
        }

        let frame_len = u16::from_le_bytes([buffer[0], buffer[1]]) as usize;

        // Reject a length that cannot hold kind and JournalSequence.
        if frame_len < 9 {
            return Err(DecodeError::FrameLengthTooSmall);
        }

        let expected_payload_len = frame_len - 9;
        let available_payload = buffer.len() - FRAME_HEADER_SIZE;

        if available_payload < expected_payload_len {
            return Err(DecodeError::PayloadTooShort {
                frame_len: frame_len as u16,
                available: available_payload,
            });
        }

        let kind = FrameKind::decode(buffer[2]).map_err(DecodeError::UnknownKind)?;
        let journal_sequence = JournalSequence::new(u64::from_le_bytes([
            buffer[3], buffer[4], buffer[5], buffer[6], buffer[7], buffer[8], buffer[9], buffer[10],
        ]));
        let payload = &buffer[11..11 + expected_payload_len];

        Ok((kind, journal_sequence, payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_rejects_unknown_kind() {
        // Given a buffer with unknown kind (0xFF)
        let mut buf = [0u8; FRAME_HEADER_SIZE + 1];
        buf[0..2].copy_from_slice(&10u16.to_le_bytes());
        buf[2] = 0xFF;
        buf[3..11].copy_from_slice(&0u64.to_le_bytes());
        buf[11] = 0x00;

        // When we try to decode
        let result = Frame::decode(&buf);

        // Then it returns an error about unknown kind
        assert!(matches!(result, Err(DecodeError::UnknownKind(0xFF))));
    }

    #[test]
    fn decode_rejects_reserved_clob_iceberg_kind() {
        // Given a buffer with reserved NewIceberg kind (0x06)
        let mut buf = [0u8; FRAME_HEADER_SIZE];
        buf[0..2].copy_from_slice(&9u16.to_le_bytes());
        buf[2] = 0x06;
        buf[3..11].copy_from_slice(&0u64.to_le_bytes());

        // When we try to decode
        let result = Frame::decode(&buf);

        // Then it fails closed
        assert!(matches!(result, Err(DecodeError::UnknownKind(0x06))));
    }

    #[test]
    fn decode_rejects_reserved_rfq_command_kind() {
        // Given a buffer with reserved RfqRequest kind (0x40)
        let mut buf = [0u8; FRAME_HEADER_SIZE];
        buf[0..2].copy_from_slice(&9u16.to_le_bytes());
        buf[2] = 0x40;
        buf[3..11].copy_from_slice(&0u64.to_le_bytes());

        // When we try to decode
        let result = Frame::decode(&buf);

        // Then it fails closed
        assert!(matches!(result, Err(DecodeError::UnknownKind(0x40))));
    }

    #[test]
    fn decode_rejects_reserved_rfq_event_kind() {
        // Given a buffer with reserved RfqQuoted kind (0xA0)
        let mut buf = [0u8; FRAME_HEADER_SIZE];
        buf[0..2].copy_from_slice(&9u16.to_le_bytes());
        buf[2] = 0xA0;
        buf[3..11].copy_from_slice(&0u64.to_le_bytes());

        // When we try to decode
        let result = Frame::decode(&buf);

        // Then it fails closed
        assert!(matches!(result, Err(DecodeError::UnknownKind(0xA0))));
    }

    #[test]
    fn decode_rejects_header_too_short() {
        // Given a buffer shorter than the stream header
        let short_buf = [0u8; 10];

        // When we try to decode
        let result = Frame::decode(&short_buf);

        // Then it returns a header too short error
        assert!(matches!(result, Err(DecodeError::HeaderTooShort { .. })));
    }

    #[test]
    fn decode_rejects_payload_too_short() {
        // Given a buffer where frame_len claims more payload than available
        let mut buf = [0u8; FRAME_HEADER_SIZE + 2];
        buf[0..2].copy_from_slice(&20u16.to_le_bytes());
        buf[2] = FrameKind::NewLimit.encode();
        buf[3..11].copy_from_slice(&1u64.to_le_bytes());

        // When we try to decode
        let result = Frame::decode(&buf);

        // Then it returns a payload too short error
        assert!(matches!(result, Err(DecodeError::PayloadTooShort { .. })));
    }

    #[test]
    fn new_limit_frame_encode_decode_round_trips() {
        // Given a NewLimit frame with payload and JournalSequence
        let kind = FrameKind::NewLimit;
        let journal_sequence = JournalSequence::new(7);
        let payload = [1u8, 2u8, 3u8];

        // When we encode and then decode
        let mut buf = [0u8; FRAME_HEADER_SIZE + 3];
        Frame::encode(kind, journal_sequence, &payload, &mut buf).expect("encode");

        let (decoded_kind, decoded_sequence, decoded_payload) =
            Frame::decode(&buf).expect("decode");

        // Then round trip preserves kind, JournalSequence, and payload
        assert_eq!(decoded_kind, FrameKind::NewLimit);
        assert_eq!(decoded_sequence.inner(), 7);
        assert_eq!(decoded_payload, payload.as_ref());
    }

    #[test]
    fn heartbeat_frame_with_empty_payload_round_trips() {
        // Given a Heartbeat with empty payload
        let kind = FrameKind::Heartbeat;
        let journal_sequence = JournalSequence::new(42);

        // When we encode and decode
        let mut buf = [0u8; FRAME_HEADER_SIZE];
        Frame::encode(kind, journal_sequence, &[], &mut buf).expect("encode");
        let (decoded_kind, decoded_sequence, decoded_payload) =
            Frame::decode(&buf).expect("decode");

        // Then Heartbeat round-trips with empty payload
        assert_eq!(decoded_kind, FrameKind::Heartbeat);
        assert_eq!(decoded_sequence.inner(), 42);
        assert!(decoded_payload.is_empty());
    }

    #[test]
    fn frame_kind_classification() {
        // Given the closed set of FrameKind values
        // When we classify them
        // Then commands, events, and control are distinct
        assert!(FrameKind::NewLimit.is_command());
        assert!(FrameKind::NewMarket.is_command());
        assert!(FrameKind::CancelByOrder.is_command());
        assert!(FrameKind::CancelByClient.is_command());
        assert!(FrameKind::Replace.is_command());
        assert!(FrameKind::Accepted.is_event());
        assert!(FrameKind::Trade.is_event());
        assert!(FrameKind::Heartbeat.is_control());
        assert!(!FrameKind::NewLimit.is_event());
        assert!(!FrameKind::Accepted.is_command());
        assert!(!FrameKind::Heartbeat.is_command());
    }

    #[test]
    fn decode_rejects_frame_length_too_small() {
        // Given a header whose frame_len omits journal_sequence
        let mut buf = [0u8; FRAME_HEADER_SIZE];
        buf[0..2].copy_from_slice(&1u16.to_le_bytes());
        buf[2] = FrameKind::NewLimit.encode();

        // When the frame is decoded
        let result = Frame::decode(&buf);

        // Then decode fails closed
        assert!(matches!(result, Err(DecodeError::FrameLengthTooSmall)));
    }
}
