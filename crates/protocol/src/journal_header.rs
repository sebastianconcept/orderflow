//! 15-byte session prefix of a packed journal.
//!
//! When a caller names the session and clock of a journal, it uses this module
//! so magic, schema version, SessionId, and ClockKind sit before the first frame.
//!
//! # Examples
//!
//! ```
//! use engine_types::SessionId;
//! use protocol::journal_header::{ClockKind, JournalHeader};
//!
//! let header = JournalHeader::new(SessionId::new(42), ClockKind::Logical);
//! let mut buf = [0u8; 15];
//! header.encode(&mut buf).expect("header buffer is HEADER_SIZE");
//! let decoded = JournalHeader::decode(&buf).expect("round-trip");
//! assert_eq!(decoded.session_id().inner(), 42);
//! assert_eq!(decoded.clock_kind(), ClockKind::Logical);
//! ```

use displaydoc::Display;
use engine_types::SessionId;
use thiserror::Error;

/// Four-byte OFL1 identifier of a valid journal.
const MAGIC: [u8; 4] = [0x4F, 0x46, 0x4C, 0x31]; // "OFL1"

/// Current journal schema version.
const SCHEMA_VERSION: u16 = 1;

/// Byte count of the journal session prefix.
pub const HEADER_SIZE: usize = 15;

/// Failure of journal header decode.
/// Wrong magic, unsupported schema version, unknown clock kind, and a short
/// buffer are distinct.
#[derive(Display, Debug, Error, PartialEq, Eq)]
pub enum DecodeError {
    /// Invalid magic bytes: expected OFL1
    WrongMagic,
    /// Unsupported schema version: {0} (expected 1)
    UnsupportedSchemaVersion(u16),
    /// Unsupported clock kind: {0}
    UnsupportedClockKind(u8),
    /// Buffer too short: expected at least {expected} bytes, got {actual}
    BufferTooShort { expected: usize, actual: usize },
}

/// Failure of journal header encode.
/// The buffer must hold HEADER_SIZE bytes.
#[derive(Display, Debug, Error, PartialEq, Eq)]
pub enum EncodeError {
    /// Buffer too short: expected at least {expected} bytes, got {actual}
    BufferTooShort { expected: usize, actual: usize },
}

/// ClockKind is how timestamps in this journal are produced.
/// When a header names the clock, it uses this type so Logical is a session
/// clock and Wall is wall time.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClockKind {
    /// Monotonically increasing session clock.
    Logical = 0,
    /// Real-time nanosecond clock.
    Wall = 1,
}

impl ClockKind {
    /// Answers the journal byte of this clock kind.
    pub fn encode(self) -> u8 {
        self as u8
    }

    /// Answers the ClockKind of a journal byte.
    /// An unknown byte is an unsupported clock kind, not a schema error.
    pub fn decode(value: u8) -> Result<Self, DecodeError> {
        match value {
            0 => Ok(ClockKind::Logical),
            1 => Ok(ClockKind::Wall),
            unknown => Err(DecodeError::UnsupportedClockKind(unknown)),
        }
    }
}

/// JournalHeader is the 15-byte session prefix of a packed journal.
/// When a caller names the session and clock before the first frame, it uses
/// this type so magic, schema version, SessionId, and ClockKind are one prefix.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct JournalHeader {
    session_id: SessionId,
    clock_kind: ClockKind,
}

impl JournalHeader {
    /// Answers a JournalHeader from a SessionId and a ClockKind.
    pub fn new(session_id: SessionId, clock_kind: ClockKind) -> Self {
        JournalHeader {
            session_id,
            clock_kind,
        }
    }

    /// Answers a JournalHeader that uses a logical clock.
    pub fn new_logical(session_id: SessionId) -> Self {
        JournalHeader::new(session_id, ClockKind::Logical)
    }

    /// Answers a JournalHeader that uses a wall clock.
    pub fn new_wall(session_id: SessionId) -> Self {
        JournalHeader::new(session_id, ClockKind::Wall)
    }

    /// Answers the matching session that owns this journal.
    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    /// Answers how timestamps in this journal are produced.
    pub fn clock_kind(&self) -> ClockKind {
        self.clock_kind
    }

    /// Answers the 15-byte prefix of this header.
    pub fn encode(&self, buffer: &mut [u8]) -> Result<usize, EncodeError> {
        if buffer.len() < HEADER_SIZE {
            return Err(EncodeError::BufferTooShort {
                expected: HEADER_SIZE,
                actual: buffer.len(),
            });
        }

        buffer[0..4].copy_from_slice(&MAGIC);
        buffer[4..6].copy_from_slice(&SCHEMA_VERSION.to_le_bytes());
        buffer[6..14].copy_from_slice(&self.session_id.inner().to_le_bytes());
        buffer[14] = self.clock_kind.encode();
        Ok(HEADER_SIZE)
    }

    /// Answers a JournalHeader of a 15-byte prefix.
    /// An unknown clock kind is UnsupportedClockKind, not a schema error.
    pub fn decode(buffer: &[u8]) -> Result<Self, DecodeError> {
        if buffer.len() < HEADER_SIZE {
            return Err(DecodeError::BufferTooShort {
                expected: HEADER_SIZE,
                actual: buffer.len(),
            });
        }

        let magic = [buffer[0], buffer[1], buffer[2], buffer[3]];
        if magic != MAGIC {
            return Err(DecodeError::WrongMagic);
        }

        let schema_version = u16::from_le_bytes([buffer[4], buffer[5]]);
        if schema_version != SCHEMA_VERSION {
            return Err(DecodeError::UnsupportedSchemaVersion(schema_version));
        }

        let session_id = SessionId::new(u64::from_le_bytes([
            buffer[6], buffer[7], buffer[8], buffer[9], buffer[10], buffer[11], buffer[12],
            buffer[13],
        ]));
        let clock_kind = ClockKind::decode(buffer[14])?;

        Ok(JournalHeader {
            session_id,
            clock_kind,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_rejects_wrong_magic() {
        // Given a buffer with wrong magic bytes
        let mut buf = [0u8; HEADER_SIZE];
        // Set wrong magic "XXXX"
        buf[0..4].copy_from_slice(b"XXXX");
        // Valid schema version
        buf[4..6].copy_from_slice(&SCHEMA_VERSION.to_le_bytes());
        // Valid session id
        buf[6..14].copy_from_slice(&42u64.to_le_bytes());
        // Valid clock kind
        buf[14] = ClockKind::Logical.encode();

        // When we try to decode
        let result = JournalHeader::decode(&buf);

        // Then it returns an error about wrong magic
        assert!(matches!(result, Err(DecodeError::WrongMagic)));
    }

    #[test]
    fn decode_rejects_schema_version_not_one() {
        // Given a buffer with unsupported schema version
        let mut buf = [0u8; HEADER_SIZE];
        // Valid magic
        buf[0..4].copy_from_slice(&MAGIC);
        // Unsupported schema version (2)
        buf[4..6].copy_from_slice(&2u16.to_le_bytes());
        // Valid session id
        buf[6..14].copy_from_slice(&42u64.to_le_bytes());
        // Valid clock kind
        buf[14] = ClockKind::Logical.encode();

        // When we try to decode
        let result = JournalHeader::decode(&buf);

        // Then it returns an error about unsupported schema version
        assert!(matches!(
            result,
            Err(DecodeError::UnsupportedSchemaVersion(2))
        ));
    }

    #[test]
    fn encode_decode_round_trips_session_id_and_clock_kind() {
        // Given a header with specific values
        let original_header = JournalHeader::new(SessionId::new(42), ClockKind::Logical);

        // When we encode and then decode
        let mut buf = [0u8; HEADER_SIZE];
        original_header
            .encode(&mut buf)
            .expect("encode should succeed");
        let decoded_header = JournalHeader::decode(&buf).expect("decode should succeed");

        // Then the decoded header matches the original
        assert_eq!(decoded_header.session_id().inner(), 42);
        assert_eq!(decoded_header.clock_kind(), ClockKind::Logical);

        // Test with wall clock too
        let original_wall = JournalHeader::new(SessionId::new(999), ClockKind::Wall);
        let mut wall_buf = [0u8; HEADER_SIZE];
        original_wall
            .encode(&mut wall_buf)
            .expect("encode should succeed");
        let decoded_wall = JournalHeader::decode(&wall_buf).expect("decode should succeed");

        assert_eq!(decoded_wall.session_id().inner(), 999);
        assert_eq!(decoded_wall.clock_kind(), ClockKind::Wall);
    }

    #[test]
    fn clock_kind_decode_rejects_unknown_value() {
        // Given an unknown clock kind value (2)
        let result = ClockKind::decode(2);

        // Then it returns an error
        assert!(matches!(result, Err(DecodeError::UnsupportedClockKind(2))));
    }

    #[test]
    fn decode_rejects_unknown_clock_kind_on_full_header() {
        // Given a valid OFL1 header whose clock byte is not 0 or 1
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&MAGIC);
        buf[4..6].copy_from_slice(&SCHEMA_VERSION.to_le_bytes());
        buf[6..14].copy_from_slice(&42u64.to_le_bytes());
        buf[14] = 2;

        // When the header is decoded
        let result = JournalHeader::decode(&buf);

        // Then the error names the clock kind, not the schema version
        assert!(matches!(result, Err(DecodeError::UnsupportedClockKind(2))));
    }

    #[test]
    fn decode_rejects_buffer_too_short() {
        // Given a buffer with only 10 bytes
        let short_buf = [0u8; 10];

        // When we try to decode
        let result = JournalHeader::decode(&short_buf);

        // Then it returns a buffer too short error
        assert!(matches!(result, Err(DecodeError::BufferTooShort { .. })));
    }
}
