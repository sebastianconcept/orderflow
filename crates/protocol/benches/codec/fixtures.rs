//! Packed journals and datagram vectors for codec benches.
//!
//! When Criterion builds a journal or datagram batch, it uses this module so
//! the timed loop receives bytes built outside the measurement window.

// The allocation test and this bench both call every kind in this module.
#[path = "../../support/command_event_samples.rs"]
mod command_event_samples;

#[path = "../../support/mixed_journal_slot.rs"]
mod mixed_journal_slot;

pub use command_event_samples::{
    all_command_kinds, all_event_kinds, session_id, MAX_ENCODED_STREAM_FRAME,
};

use command_event_samples::{
    cancel_by_order_sequenced_command, new_limit_sequenced_command, rejected_engine_event,
    replace_sequenced_command, trade_engine_event,
};
use engine_types::{
    CommandSequence, EngineEvent, EventSequence, JournalSequence, SequencedCommand,
};
use mixed_journal_slot::{mixed_journal_slot, MixedJournalSlot};
use protocol::{
    command_codec::{encode_command, encode_command_datagram},
    event_codec::{encode_event, encode_event_datagram},
    journal_cursor::JournalCursor,
    journal_header::{ClockKind, JournalHeader, HEADER_SIZE},
};

/// Frame count of the in-cache journal walk and datagram ingress benches.
pub const JOURNAL_FRAME_COUNT_10K: usize = 10_000;

/// Frame count of the out-of-cache journal walk bench.
pub const JOURNAL_FRAME_COUNT_1M: usize = 1_000_000;

/// Rough bytes per mixed stream frame for journal `Vec` capacity.
const ESTIMATED_STREAM_FRAME_BYTES: usize = 72;

/// JournalRecord is one command or event slot in the mixed replay journal.
enum JournalRecord {
    Command(SequencedCommand),
    Event(EngineEvent),
}

/// Answers the mixed-journal record at `index`.
fn mix_journal_record(index: u64) -> JournalRecord {
    let command_sequence = CommandSequence::new(index);
    let event_sequence = EventSequence::new(index);
    match mixed_journal_slot(index) {
        MixedJournalSlot::NewLimit => {
            JournalRecord::Command(new_limit_sequenced_command(command_sequence))
        }
        MixedJournalSlot::CancelByOrder => {
            JournalRecord::Command(cancel_by_order_sequenced_command(command_sequence))
        }
        MixedJournalSlot::Replace => {
            JournalRecord::Command(replace_sequenced_command(command_sequence))
        }
        MixedJournalSlot::Trade => {
            JournalRecord::Event(trade_engine_event(event_sequence, command_sequence))
        }
        MixedJournalSlot::Rejected => {
            JournalRecord::Event(rejected_engine_event(event_sequence, command_sequence))
        }
    }
}

/// Answers a packed stream journal: session header plus `frame_count` mixed frames.
pub fn encode_stream_journal(frame_count: usize) -> Vec<u8> {
    let mut header_bytes = [0u8; HEADER_SIZE];
    JournalHeader::new(session_id(), ClockKind::Logical)
        .encode(&mut header_bytes)
        .expect("journal header fits HEADER_SIZE");

    let estimated_bytes = HEADER_SIZE + frame_count * ESTIMATED_STREAM_FRAME_BYTES;
    let mut journal = Vec::with_capacity(estimated_bytes);
    journal.extend_from_slice(&header_bytes);

    let mut cursor = JournalCursor::new();
    let mut frame_buffer = [0u8; MAX_ENCODED_STREAM_FRAME];
    for index in 0..frame_count as u64 {
        let journal_sequence = cursor.next_journal_sequence();
        let encoded_len = encode_journal_record_stream(
            mix_journal_record(index),
            journal_sequence,
            &mut frame_buffer,
        );
        journal.extend_from_slice(&frame_buffer[..encoded_len]);
    }
    journal
}

/// Answers `frame_count` packed datagrams, one mixed record each.
pub fn encode_datagrams(frame_count: usize) -> Vec<Vec<u8>> {
    let mut cursor = JournalCursor::new();
    let mut datagrams = Vec::with_capacity(frame_count);
    let mut frame_buffer = [0u8; MAX_ENCODED_STREAM_FRAME];
    for index in 0..frame_count as u64 {
        let journal_sequence = cursor.next_journal_sequence();
        let encoded_len = encode_journal_record_datagram(
            mix_journal_record(index),
            journal_sequence,
            &mut frame_buffer,
        );
        datagrams.push(frame_buffer[..encoded_len].to_vec());
    }
    datagrams
}

fn encode_journal_record_stream(
    record: JournalRecord,
    journal_sequence: JournalSequence,
    frame_buffer: &mut [u8],
) -> usize {
    match record {
        JournalRecord::Command(sequenced_command) => {
            encode_command(&sequenced_command, journal_sequence, frame_buffer)
                .expect("stream command fits MAX_ENCODED_STREAM_FRAME")
        }
        JournalRecord::Event(engine_event) => {
            encode_event(&engine_event, journal_sequence, frame_buffer)
                .expect("stream event fits MAX_ENCODED_STREAM_FRAME")
        }
    }
}

fn encode_journal_record_datagram(
    record: JournalRecord,
    journal_sequence: JournalSequence,
    frame_buffer: &mut [u8],
) -> usize {
    match record {
        JournalRecord::Command(sequenced_command) => encode_command_datagram(
            &sequenced_command,
            session_id(),
            journal_sequence,
            frame_buffer,
        )
        .expect("datagram command fits MAX_ENCODED_STREAM_FRAME"),
        JournalRecord::Event(engine_event) => {
            encode_event_datagram(&engine_event, session_id(), journal_sequence, frame_buffer)
                .expect("datagram event fits MAX_ENCODED_STREAM_FRAME")
        }
    }
}
