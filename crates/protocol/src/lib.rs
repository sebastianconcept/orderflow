//! Packed little-endian journal codec for engine commands and events.
//!
//! When a caller encodes or decodes a stream or datagram envelope, it uses this
//! crate so both share one payload layout. Main types: [Frame], [FrameKind],
//! [JournalHeader], [JournalCursor].

pub mod command_codec;
pub mod datagram;
pub mod event_codec;
pub mod frame;
pub mod journal_cursor;
pub mod journal_header;
mod packed_le;

pub use command_codec::{
    decode_command, decode_command_datagram, encode_command, encode_command_datagram,
    DecodeError as CommandDecodeError, EncodeError as CommandEncodeError,
};
pub use datagram::{
    encode_heartbeat, DecodeError as DatagramDecodeError, EncodeError as DatagramEncodeError,
    DATAGRAM_HEADER_SIZE,
};
pub use event_codec::{
    decode_event, decode_event_datagram, encode_event, encode_event_datagram,
    DecodeError as EventDecodeError, EncodeError as EventEncodeError,
};
pub use frame::{Frame, FrameKind, FRAME_HEADER_SIZE};
pub use journal_cursor::JournalCursor;
pub use journal_header::{ClockKind, DecodeError as JournalHeaderDecodeError, JournalHeader};
