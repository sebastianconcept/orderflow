//! Frame envelope for command and event encoding.
//!
//! The frame envelope provides a length-prefixed format for individual
//! commands and events in the protocol journal. Each frame contains:
//!
//! - `frame_len`: Length of the frame body (kind + payload)
//! - `kind`: Opcode identifying the message type
//! - `payload`: Variable-length data specific to the kind
//!
//! # Frame Layout
//!
//! | Field | Size | Notes |
//! |-------|------|-------|
//! | frame_len | 2 bytes (u16, little-endian) | Bytes following this field |
//! | kind | 1 byte (u8) | Opcode for the message type |
//! | payload | variable | Kind-specific data |
//!
//! # Opcodes
//!
//! ## Commands (replay injects these)
//!
//! | Kind | Name |
//! |------|------|
//! | 0x01 | New |
//! | 0x02 | CancelByOrder |
//! | 0x03 | CancelByClient |
//! | 0x04 | Replace |
//!
//! ## Events (golden output only)
//!
//! | Kind | Name |
//! |------|------|
//! | 0x81 | Accepted |
//! | 0x82 | Rejected |
//! | 0x83 | Replaced |
//! | 0x84 | Canceled |
//! | 0x85 | Trade |
//!
//! # Errors
//!
//! Unknown `kind` values cause [`Frame::decode`] to return a [`DecodeError`]
//! with variant [`DecodeErrorKind::UnknownKind`].
//!
//! # Examples
//!
//! ```ignore
//! use protocol::{Frame, FrameKind};
//!
//! // Encode a command frame
//! let mut buf = vec![0; 10];
//! Frame::encode(FrameKind::New, &[1, 2, 3], &mut buf)?;
//!
//! // Decode a frame
//! let (kind, payload) = Frame::decode(&buf)?;
//! assert_eq!(kind, FrameKind::New);
//! ```

/// Size of the frame header (frame_len + kind).
pub const FRAME_HEADER_SIZE: usize = 3;

/// Error types for frame decoding.
#[derive(thiserror::Error, Debug)]
pub enum DecodeError {
    /// The input buffer is too short to contain a valid frame header.
    #[error("buffer too short for frame header: expected at least {expected} bytes, got {actual}")]
    HeaderTooShort { expected: usize, actual: usize },

    /// The frame length indicates more data than is available in the buffer.
    #[error(
        "frame payload too short: frame_len is {frame_len}, but only {available} bytes available"
    )]
    PayloadTooShort { frame_len: u16, available: usize },

    /// The frame kind is not recognized.
    #[error("unknown frame kind: {0:#04x}")]
    UnknownKind(u8),
}

/// Frame kind representing the type of message (command or event).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FrameKind(u8);

impl FrameKind {
    /// Command opcodes.
    pub const NEW: Self = FrameKind(0x01);
    pub const CANCEL_BY_ORDER: Self = FrameKind(0x02);
    pub const CANCEL_BY_CLIENT: Self = FrameKind(0x03);
    pub const REPLACE: Self = FrameKind(0x04);

    /// Event opcodes.
    pub const ACCEPTED: Self = FrameKind(0x81);
    pub const REJECTED: Self = FrameKind(0x82);
    pub const REPLACED: Self = FrameKind(0x83);
    pub const CANCELED: Self = FrameKind(0x84);
    pub const TRADE: Self = FrameKind(0x85);

    /// Create a new FrameKind from a raw u8 value.
    pub fn new(kind: u8) -> Self {
        FrameKind(kind)
    }

    /// Get the raw u8 value.
    pub fn inner(&self) -> u8 {
        self.0
    }

    /// Check if this is a command kind.
    pub fn is_command(&self) -> bool {
        matches!(self.0, 0x01..=0x04)
    }

    /// Check if this is an event kind.
    pub fn is_event(&self) -> bool {
        matches!(self.0, 0x81..=0x85)
    }

    /// Encode the frame kind to a u8.
    pub fn encode(self) -> u8 {
        self.0
    }

    /// Decode a frame kind from a u8 value.
    ///
    /// # Arguments
    ///
    /// * `value` - The raw u8 opcode.
    ///
    /// # Returns
    ///
    /// * `Ok(FrameKind)` - The decoded kind.
    /// * `Err(DecodeError)` - If the kind is unknown.
    pub fn decode(value: u8) -> Result<Self, DecodeError> {
        match value {
            0x01 | 0x02 | 0x03 | 0x04 | 0x81 | 0x82 | 0x83 | 0x84 | 0x85 => Ok(FrameKind(value)),
            _ => Err(DecodeError::UnknownKind(value)),
        }
    }
}

/// A single frame containing a kind and payload.
///
/// Frames are the fundamental unit of protocol encoding, wrapping each
/// command or event with a length prefix and opcode.
#[derive(Debug)]
pub struct Frame<'a> {
    kind: FrameKind,
    payload: &'a [u8],
}

impl<'a> Frame<'a> {
    /// Create a new frame with the given kind and payload.
    pub fn new(kind: FrameKind, payload: &'a [u8]) -> Self {
        Frame { kind, payload }
    }

    /// Get the frame kind.
    pub fn kind(&self) -> FrameKind {
        self.kind
    }

    /// Get the payload.
    pub fn payload(&self) -> &[u8] {
        self.payload
    }

    /// Calculate the total frame size (header + payload).
    pub fn total_size() -> usize {
        FRAME_HEADER_SIZE
    }

    /// Encode the frame to a byte buffer.
    ///
    /// The buffer must have at least [`Frame::total_size()` + payload.len()] bytes.
    ///
    /// # Arguments
    ///
    /// * `kind` - The frame kind (opcode).
    /// * `payload` - The payload bytes.
    /// * `buf` - Mutable buffer to write the encoded frame.
    pub fn encode(kind: FrameKind, payload: &[u8], buf: &mut [u8]) {
        let total_len = FRAME_HEADER_SIZE + payload.len();
        debug_assert!(buf.len() >= total_len);

        // frame_len (little-endian) = kind + payload length
        let frame_len = (1 + payload.len()) as u16;
        buf[0..2].copy_from_slice(&frame_len.to_le_bytes());
        // kind
        buf[2] = kind.encode();
        // payload
        if !payload.is_empty() {
            buf[3..total_len].copy_from_slice(payload);
        }
    }

    /// Decode a frame from a byte slice.
    ///
    /// # Arguments
    ///
    /// * `buf` - The byte slice containing a frame.
    ///
    /// # Returns
    ///
    /// * `Ok((FrameKind, &[u8]))` - The decoded kind and payload slice.
    /// * `Err(DecodeError)` - If decoding fails (header too short, payload
    ///   truncated, or unknown kind).
    pub fn decode(buf: &[u8]) -> Result<(FrameKind, &[u8]), DecodeError> {
        if buf.len() < FRAME_HEADER_SIZE {
            return Err(DecodeError::HeaderTooShort {
                expected: FRAME_HEADER_SIZE,
                actual: buf.len(),
            });
        }

        // Read frame_len
        let frame_len = u16::from_le_bytes([buf[0], buf[1]]) as usize;

        // frame_len = kind (1 byte) + payload
        let expected_payload_len = frame_len - 1;
        let available_payload = buf.len() - FRAME_HEADER_SIZE;

        if frame_len < 1 {
            return Err(DecodeError::HeaderTooShort {
                expected: FRAME_HEADER_SIZE,
                actual: buf.len(),
            });
        }

        if available_payload < expected_payload_len {
            return Err(DecodeError::PayloadTooShort {
                frame_len: frame_len as u16,
                available: available_payload,
            });
        }

        // Read kind
        let kind = FrameKind::decode(buf[2])?;

        // Extract payload slice
        let payload = &buf[3..3 + expected_payload_len];

        Ok((kind, payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that decode rejects unknown frame kinds.
    #[test]
    fn decode_rejects_unknown_kind() {
        // Given: a buffer with unknown kind (0xFF)
        let mut buf = [0u8; 5];
        // frame_len = 2 (kind + 1 byte payload)
        buf[0..2].copy_from_slice(&2u16.to_le_bytes());
        // Unknown kind
        buf[2] = 0xFF;
        // Payload byte
        buf[3] = 0x00;

        // When: we try to decode
        let result = Frame::decode(&buf);

        // Then: it returns an error about unknown kind
        assert!(matches!(result, Err(DecodeError::UnknownKind(0xFF))));
    }

    /// Test that decode rejects buffer too short.
    #[test]
    fn decode_rejects_header_too_short() {
        // Given: a buffer with only 2 bytes
        let short_buf = [0u8; 2];

        // When: we try to decode
        let result = Frame::decode(&short_buf);

        // Then: it returns a header too short error
        assert!(matches!(result, Err(DecodeError::HeaderTooShort { .. })));
    }

    /// Test that decode rejects truncated payload.
    #[test]
    fn decode_rejects_payload_too_short() {
        // Given: a buffer where frame_len claims 10 bytes but only 2 are available
        let mut buf = [0u8; 5];
        // frame_len = 10 (claiming 10 bytes after header)
        buf[0..2].copy_from_slice(&10u16.to_le_bytes());
        // Valid kind (New = 0x01)
        buf[2] = FrameKind::NEW.encode();
        // Only 2 payload bytes available (but frame claims 9)
        buf[3] = 0x01;
        buf[4] = 0x02;

        // When: we try to decode
        let result = Frame::decode(&buf);

        // Then: it returns a payload too short error
        assert!(matches!(result, Err(DecodeError::PayloadTooShort { .. })));
    }

    /// Test encode/decode round trip for New frame.
    #[test]
    fn new_frame_encode_decode_round_trips() {
        // Given: a New frame with payload
        let kind = FrameKind::NEW;
        let payload = [1u8, 2u8, 3u8]; // account_id u64 + client_order_id u64 + instrument_id u64

        // When: we encode and then decode
        let mut buf = [0u8; 6]; // header (3) + payload (3)
        Frame::encode(kind, &payload, &mut buf);

        let (decoded_kind, decoded_payload) = Frame::decode(&buf).expect("decode should succeed");

        // Then: round trip preserves kind and payload
        assert_eq!(decoded_kind.inner(), FrameKind::NEW.encode());
        assert_eq!(decoded_payload, payload.as_ref());
    }

    /// Test encode/decode round trip for Trade frame.
    #[test]
    fn trade_frame_encode_decode_round_trips() {
        // Given: a Trade frame with longer payload
        let kind = FrameKind::TRADE;
        // sequence u64 + timestamp_nanos u64 + maker_order_id u64 + taker_order_id u64
        // + instrument_id u64 + price i64 + quantity u128 = 57 bytes
        let payload = [0u8; 57];

        // When: we encode and then decode
        let mut buf = vec![0u8; FRAME_HEADER_SIZE + payload.len()];
        Frame::encode(kind, &payload, &mut buf);

        let (decoded_kind, decoded_payload) = Frame::decode(&buf).expect("decode should succeed");

        // Then: round trip preserves kind and payload
        assert_eq!(decoded_kind.inner(), FrameKind::TRADE.encode());
        assert_eq!(decoded_payload.len(), payload.len());
    }

    /// Test that empty payload is handled correctly.
    #[test]
    fn frame_with_empty_payload() {
        // Given: a frame with empty payload
        let kind = FrameKind::CANCEL_BY_ORDER;

        // When: we encode with empty payload
        let mut buf = [0u8; FRAME_HEADER_SIZE];
        Frame::encode(kind, &[], &mut buf);

        // Then: we can decode it
        let (decoded_kind, decoded_payload) = Frame::decode(&buf).expect("decode should succeed");

        assert_eq!(decoded_kind.inner(), FrameKind::CANCEL_BY_ORDER.encode());
        assert!(decoded_payload.is_empty());
    }

    /// Test that frame kind is_command and is_event work correctly.
    #[test]
    fn frame_kind_classification() {
        // Command kinds
        assert!(FrameKind::NEW.is_command());
        assert!(FrameKind::CANCEL_BY_ORDER.is_command());
        assert!(FrameKind::CANCEL_BY_CLIENT.is_command());
        assert!(FrameKind::REPLACE.is_command());

        // Event kinds
        assert!(FrameKind::ACCEPTED.is_event());
        assert!(FrameKind::REJECTED.is_event());
        assert!(FrameKind::REPLACED.is_event());
        assert!(FrameKind::CANCELED.is_event());
        assert!(FrameKind::TRADE.is_event());

        // Commands are not events
        assert!(!FrameKind::NEW.is_event());
        assert!(!FrameKind::CANCEL_BY_ORDER.is_event());

        // Events are not commands
        assert!(!FrameKind::ACCEPTED.is_command());
        assert!(!FrameKind::TRADE.is_command());
    }

    /// Test that frame kind decode rejects unknown values.
    #[test]
    fn frame_kind_decode_rejects_unknown() {
        assert!(matches!(
            FrameKind::decode(0x55),
            Err(DecodeError::UnknownKind(_))
        ));
    }
}
