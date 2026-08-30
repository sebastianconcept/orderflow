//! Journal header for protocol-encoded event logs.
//!
//! The journal header provides session-scoped metadata that precedes all
//! frame-encoded commands and events in a replayable event log.
//!
//! # Header Layout (15 bytes, little-endian)
//!
//! | Offset | Size | Field |
//! |--------|------|-------|
//! | 0 | 4 | magic `OFL1` (`0x4F 0x46 0x4C 0x31`) |
//! | 4 | 2 | `schema_version` = `1` |
//! | 6 | 8 | `SessionId` (`u64`) |
//! | 14 | 1 | `clock_kind`: `0` logical, `1` wall |
//!
//! # Versioning
//!
//! Unknown `schema_version` values cause decode to fail with an error.
//! This ensures forward compatibility without silent misinterpretation.
//!
//! # Examples
//!
//! ```ignore
//! use protocol::{JournalHeader, SessionId};
//!
//! // Create a header for session 42 with logical clock
//! let header = JournalHeader::new(SessionId::new(42), ClockKind::Logical);
//!
//! // Encode to bytes
//! let mut buf = vec![0; 15];
//! header.encode(&mut buf);
//!
//! // Decode from bytes
//! let decoded = JournalHeader::decode(&buf)?;
//! assert_eq!(decoded.session_id().inner(), 42);
//! assert_eq!(decoded.clock_kind(), ClockKind::Logical);
//! ```

/// Magic bytes that identify a valid protocol journal.
const MAGIC: [u8; 4] = [0x4F, 0x46, 0x4C, 0x31]; // "OFL1"

/// Current schema version for the journal format.
const SCHEMA_VERSION: u16 = 1;

/// The size of the journal header in bytes.
pub const HEADER_SIZE: usize = 15;

/// Error types for journal header decoding.
#[derive(thiserror::Error, Debug)]
pub enum DecodeError {
    /// The magic bytes do not match the expected "OFL1" marker.
    #[error("invalid magic bytes: expected OFL1")]
    WrongMagic,

    /// The schema version is not supported (only version 1 is accepted).
    #[error("unsupported schema version: {0} (expected 1)")]
    UnsupportedSchemaVersion(u16),

    /// The input buffer is too short to contain a valid header.
    #[error("buffer too short: expected at least {expected} bytes, got {actual}")]
    BufferTooShort { expected: usize, actual: usize },
}

/// Clock kind indicating how timestamps are generated.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClockKind {
    /// Logical clock - monotonically increasing sequence numbers.
    Logical = 0,
    /// Wall clock - real-time timestamps in nanoseconds.
    Wall = 1,
}

impl ClockKind {
    /// Encode the clock kind as a u8 for wire format.
    pub fn encode(self) -> u8 {
        self as u8
    }

    /// Decode a clock kind from a u8 value.
    ///
    /// # Arguments
    ///
    /// * `value` - The encoded u8 value (0 for logical, 1 for wall).
    ///
    /// # Returns
    ///
    /// * `Ok(ClockKind)` - The decoded clock kind.
    /// * `Err(DecodeError)` - If the value is not 0 or 1.
    pub fn decode(value: u8) -> Result<Self, DecodeError> {
        match value {
            0 => Ok(ClockKind::Logical),
            1 => Ok(ClockKind::Wall),
            _ => Err(DecodeError::UnsupportedSchemaVersion(value as u16)),
        }
    }
}

/// Journal header containing session metadata.
///
/// The header precedes all frame-encoded data in a journal file and
/// provides the necessary context to interpret the session's events.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct JournalHeader {
    session_id: u64,
    clock_kind: ClockKind,
}

impl JournalHeader {
    /// Create a new journal header with the given session ID and clock kind.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the matching session.
    /// * `clock_kind` - The clock source used for timestamps in this session.
    pub fn new(session_id: u64, clock_kind: ClockKind) -> Self {
        JournalHeader {
            session_id,
            clock_kind,
        }
    }

    /// Create a new journal header with logical clock.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the matching session.
    pub fn new_logical(session_id: u64) -> Self {
        JournalHeader::new(session_id, ClockKind::Logical)
    }

    /// Create a new journal header with wall clock.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the matching session.
    pub fn new_wall(session_id: u64) -> Self {
        JournalHeader::new(session_id, ClockKind::Wall)
    }

    /// Get the session identifier.
    pub fn session_id(&self) -> u64 {
        self.session_id
    }

    /// Get the clock kind.
    pub fn clock_kind(&self) -> ClockKind {
        self.clock_kind
    }

    /// Encode the header to a byte buffer.
    ///
    /// # Arguments
    ///
    /// * `buf` - A mutable byte buffer of at least [`HEADER_SIZE`] bytes.
    pub fn encode(&self, buf: &mut [u8]) {
        debug_assert!(buf.len() >= HEADER_SIZE);

        // Magic bytes
        buf[0..4].copy_from_slice(&MAGIC);
        // Schema version (little-endian)
        buf[4..6].copy_from_slice(&SCHEMA_VERSION.to_le_bytes());
        // SessionId (little-endian)
        buf[6..14].copy_from_slice(&self.session_id.to_le_bytes());
        // Clock kind
        buf[14] = self.clock_kind.encode();
    }

    /// Decode a journal header from a byte slice.
    ///
    /// # Arguments
    ///
    /// * `buf` - A byte slice containing a valid journal header.
    ///
    /// # Returns
    ///
    /// * `Ok(JournalHeader)` - The decoded header.
    /// * `Err(DecodeError)` - If decoding fails (wrong magic, unsupported version,
    ///   buffer too short).
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        if buf.len() < HEADER_SIZE {
            return Err(DecodeError::BufferTooShort {
                expected: HEADER_SIZE,
                actual: buf.len(),
            });
        }

        // Check magic bytes
        let magic = [buf[0], buf[1], buf[2], buf[3]];
        if magic != MAGIC {
            return Err(DecodeError::WrongMagic);
        }

        // Check schema version
        let schema_version = u16::from_le_bytes([buf[4], buf[5]]);
        if schema_version != SCHEMA_VERSION {
            return Err(DecodeError::UnsupportedSchemaVersion(schema_version));
        }

        // Decode session id and clock kind
        let session_id = u64::from_le_bytes([
            buf[6], buf[7], buf[8], buf[9], buf[10], buf[11], buf[12], buf[13],
        ]);
        let clock_kind = ClockKind::decode(buf[14])?;

        Ok(JournalHeader {
            session_id,
            clock_kind,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that decoding rejects wrong magic bytes.
    #[test]
    fn decode_rejects_wrong_magic() {
        // Given: a buffer with wrong magic bytes
        let mut buf = [0u8; HEADER_SIZE];
        // Set wrong magic "XXXX"
        buf[0..4].copy_from_slice(b"XXXX");
        // Valid schema version
        buf[4..6].copy_from_slice(&SCHEMA_VERSION.to_le_bytes());
        // Valid session id
        buf[6..14].copy_from_slice(&42u64.to_le_bytes());
        // Valid clock kind
        buf[14] = ClockKind::Logical.encode();

        // When: we try to decode
        let result = JournalHeader::decode(&buf);

        // Then: it returns an error about wrong magic
        assert!(matches!(result, Err(DecodeError::WrongMagic)));
    }

    /// Test that decoding rejects unsupported schema versions.
    #[test]
    fn decode_rejects_schema_version_not_one() {
        // Given: a buffer with unsupported schema version
        let mut buf = [0u8; HEADER_SIZE];
        // Valid magic
        buf[0..4].copy_from_slice(&MAGIC);
        // Unsupported schema version (2)
        buf[4..6].copy_from_slice(&2u16.to_le_bytes());
        // Valid session id
        buf[6..14].copy_from_slice(&42u64.to_le_bytes());
        // Valid clock kind
        buf[14] = ClockKind::Logical.encode();

        // When: we try to decode
        let result = JournalHeader::decode(&buf);

        // Then: it returns an error about unsupported schema version
        assert!(matches!(
            result,
            Err(DecodeError::UnsupportedSchemaVersion(2))
        ));
    }

    /// Test that encode/decode round trips session id and clock kind.
    #[test]
    fn encode_decode_round_trips_session_id_and_clock_kind() {
        // Given: a header with specific values
        let original_header = JournalHeader::new(42, ClockKind::Logical);

        // When: we encode and then decode
        let mut buf = [0u8; HEADER_SIZE];
        original_header.encode(&mut buf);
        let decoded_header = JournalHeader::decode(&buf).expect("decode should succeed");

        // Then: the decoded header matches the original
        assert_eq!(decoded_header.session_id(), 42);
        assert_eq!(decoded_header.clock_kind(), ClockKind::Logical);

        // Test with wall clock too
        let original_wall = JournalHeader::new(999, ClockKind::Wall);
        let mut wall_buf = [0u8; HEADER_SIZE];
        original_wall.encode(&mut wall_buf);
        let decoded_wall = JournalHeader::decode(&wall_buf).expect("decode should succeed");

        assert_eq!(decoded_wall.session_id(), 999);
        assert_eq!(decoded_wall.clock_kind(), ClockKind::Wall);
    }

    /// Test that clock kind decode rejects unknown values.
    #[test]
    fn clock_kind_decode_rejects_unknown_value() {
        // Given: an unknown clock kind value (2)
        let result = ClockKind::decode(2);

        // Then: it returns an error
        assert!(result.is_err());
    }

    /// Test that buffer too short is rejected.
    #[test]
    fn decode_rejects_buffer_too_short() {
        // Given: a buffer with only 10 bytes
        let short_buf = [0u8; 10];

        // When: we try to decode
        let result = JournalHeader::decode(&short_buf);

        // Then: it returns a buffer too short error
        assert!(matches!(result, Err(DecodeError::BufferTooShort { .. })));
    }
}
