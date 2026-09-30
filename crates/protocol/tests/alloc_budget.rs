//! Allocation budget of one OFL1 encode or decode.
//!
//! When a codec change adds a heap allocation, this test fails so the
//! budget is an exact count, not a timing comparison.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use engine_types::JournalSequence;
use protocol::{
    command_codec::{
        decode_command, decode_command_datagram, encode_command, encode_command_datagram,
    },
    datagram::{self, encode_heartbeat},
    event_codec::{decode_event, decode_event_datagram, encode_event, encode_event_datagram},
    Frame, FrameKind,
};

#[path = "../support/command_event_samples.rs"]
mod command_event_samples;

struct CountingAllocator;

static ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);
static ALLOCATION_BYTES: AtomicUsize = AtomicUsize::new(0);
static MEASURE_LOCK: Mutex<()> = Mutex::new(());

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
}

fn record_allocation(size: usize) {
    if COUNTING.with(|counting| counting.get()) {
        ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        ALLOCATION_BYTES.fetch_add(size, Ordering::Relaxed);
    }
}

// SAFETY: every method forwards to System with the same pointer and layout.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation(layout.size());
        // SAFETY: layout is the allocation request the caller made of the global allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation(layout.size());
        // SAFETY: layout is the allocation request the caller made of the global allocator.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record_allocation(new_size);
        // SAFETY: pointer and layout came from a prior alloc on this allocator.
        unsafe { System.realloc(pointer, layout, new_size) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: pointer and layout came from a prior alloc on this allocator.
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

/// Observed heap use during one encode or decode call.
///
/// Encode and decode write or read the caller buffer with no heap. A non-zero
/// snapshot means a codec change added an allocation on that path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllocationSnapshot {
    allocation_count: usize,
    allocation_bytes: usize,
}

/// Locked heap use for one encode or decode: no allocations.
const ZERO_ALLOCATION: AllocationSnapshot = AllocationSnapshot {
    allocation_count: 0,
    allocation_bytes: 0,
};

struct CountingGuard;

impl Drop for CountingGuard {
    fn drop(&mut self) {
        COUNTING.with(|counting| counting.set(false));
    }
}

fn with_measure_lock(body: impl FnOnce()) {
    let _lock = MEASURE_LOCK.lock().expect("measure lock");
    body();
}

fn measure(body: impl FnOnce()) -> AllocationSnapshot {
    ALLOCATION_COUNT.store(0, Ordering::Relaxed);
    ALLOCATION_BYTES.store(0, Ordering::Relaxed);
    COUNTING.with(|counting| counting.set(true));
    let _counting_guard = CountingGuard;
    body();
    AllocationSnapshot {
        allocation_count: ALLOCATION_COUNT.load(Ordering::Relaxed),
        allocation_bytes: ALLOCATION_BYTES.load(Ordering::Relaxed),
    }
}

fn journal_sequence() -> JournalSequence {
    JournalSequence::new(0)
}

fn assert_zero_allocations(snapshot: AllocationSnapshot, label: &str) {
    assert_eq!(snapshot.allocation_count, 0, "{label} allocation count");
    assert_eq!(snapshot.allocation_bytes, 0, "{label} allocation bytes");
}

#[test]
fn decode_allocates_nothing_for_commands_events_and_heartbeat() {
    with_measure_lock(|| {
        // Given packed stream and datagram frames of every command and event kind
        let mut stream_buffer = [0u8; command_event_samples::MAX_ENCODED_STREAM_FRAME];
        let mut datagram_buffer = [0u8; command_event_samples::MAX_ENCODED_STREAM_FRAME];
        let session_id = command_event_samples::session_id();

        for (kind_name, sequenced_command) in command_event_samples::all_command_kinds() {
            let stream_len =
                encode_command(&sequenced_command, journal_sequence(), &mut stream_buffer)
                    .expect("stream command encode");

            // When we decode the stream frame under the counting allocator
            let snapshot = measure(|| {
                decode_command(&stream_buffer[..stream_len]).expect("stream command decode");
            });
            // Then decode allocates no heap
            assert_zero_allocations(snapshot, &format!("{kind_name} stream decode"));

            let datagram_len = encode_command_datagram(
                &sequenced_command,
                session_id,
                journal_sequence(),
                &mut datagram_buffer,
            )
            .expect("datagram command encode");
            let snapshot = measure(|| {
                decode_command_datagram(&datagram_buffer[..datagram_len])
                    .expect("datagram command decode");
            });
            assert_zero_allocations(snapshot, &format!("{kind_name} datagram decode"));
        }

        for (kind_name, engine_event) in command_event_samples::all_event_kinds() {
            let stream_len = encode_event(&engine_event, journal_sequence(), &mut stream_buffer)
                .expect("stream event encode");
            let snapshot = measure(|| {
                decode_event(&stream_buffer[..stream_len]).expect("stream event decode");
            });
            assert_zero_allocations(snapshot, &format!("{kind_name} stream decode"));

            let datagram_len = encode_event_datagram(
                &engine_event,
                session_id,
                journal_sequence(),
                &mut datagram_buffer,
            )
            .expect("datagram event encode");
            let snapshot = measure(|| {
                decode_event_datagram(&datagram_buffer[..datagram_len])
                    .expect("datagram event decode");
            });
            assert_zero_allocations(snapshot, &format!("{kind_name} datagram decode"));
        }

        // Given a stream Heartbeat and a datagram Heartbeat
        let stream_len = Frame::encode(
            FrameKind::Heartbeat,
            journal_sequence(),
            &[],
            &mut stream_buffer,
        )
        .expect("stream heartbeat encode");
        let datagram_len = encode_heartbeat(session_id, journal_sequence(), &mut datagram_buffer)
            .expect("datagram heartbeat encode");

        // When we decode each Heartbeat under the counting allocator
        let snapshot = measure(|| {
            Frame::decode(&stream_buffer[..stream_len]).expect("stream heartbeat decode");
        });
        // Then decode allocates no heap
        assert_zero_allocations(snapshot, "Heartbeat stream decode");

        let snapshot = measure(|| {
            datagram::decode(&datagram_buffer[..datagram_len]).expect("datagram heartbeat decode");
        });
        assert_zero_allocations(snapshot, "Heartbeat datagram decode");
    });
}

#[test]
fn encode_matches_locked_budget_for_every_kind() {
    with_measure_lock(|| {
        // Given every command and event kind and Heartbeat envelopes
        let session_id = command_event_samples::session_id();
        let command_budgets: [AllocationSnapshot; 5] = [ZERO_ALLOCATION; 5];
        let event_budgets: [AllocationSnapshot; 5] = [ZERO_ALLOCATION; 5];

        // When we encode once under the counting allocator
        // Then allocation count and bytes match the locked budget
        for ((kind_name, sequenced_command), budget) in command_event_samples::all_command_kinds()
            .iter()
            .zip(command_budgets.iter())
        {
            let budget = *budget;
            let snapshot = measure(|| {
                let mut buffer = [0u8; command_event_samples::MAX_ENCODED_STREAM_FRAME];
                encode_command(sequenced_command, journal_sequence(), &mut buffer)
                    .expect("stream command encode");
            });
            assert_eq!(
                snapshot, budget,
                "{kind_name} stream encode allocation budget"
            );

            let snapshot = measure(|| {
                let mut buffer = [0u8; command_event_samples::MAX_ENCODED_STREAM_FRAME];
                encode_command_datagram(
                    sequenced_command,
                    session_id,
                    journal_sequence(),
                    &mut buffer,
                )
                .expect("datagram command encode");
            });
            assert_eq!(
                snapshot, budget,
                "{kind_name} datagram encode allocation budget"
            );
        }

        for ((kind_name, engine_event), budget) in command_event_samples::all_event_kinds()
            .iter()
            .zip(event_budgets.iter())
        {
            let budget = *budget;
            let snapshot = measure(|| {
                let mut buffer = [0u8; command_event_samples::MAX_ENCODED_STREAM_FRAME];
                encode_event(engine_event, journal_sequence(), &mut buffer)
                    .expect("stream event encode");
            });
            assert_eq!(
                snapshot, budget,
                "{kind_name} stream encode allocation budget"
            );

            let snapshot = measure(|| {
                let mut buffer = [0u8; command_event_samples::MAX_ENCODED_STREAM_FRAME];
                encode_event_datagram(engine_event, session_id, journal_sequence(), &mut buffer)
                    .expect("datagram event encode");
            });
            assert_eq!(
                snapshot, budget,
                "{kind_name} datagram encode allocation budget"
            );
        }

        let snapshot = measure(|| {
            let mut buffer = [0u8; command_event_samples::MAX_ENCODED_STREAM_FRAME];
            Frame::encode(FrameKind::Heartbeat, journal_sequence(), &[], &mut buffer)
                .expect("stream heartbeat encode");
        });
        assert_eq!(
            snapshot, ZERO_ALLOCATION,
            "Heartbeat stream encode allocation budget"
        );

        let snapshot = measure(|| {
            let mut buffer = [0u8; command_event_samples::MAX_ENCODED_STREAM_FRAME];
            encode_heartbeat(session_id, journal_sequence(), &mut buffer)
                .expect("datagram heartbeat encode");
        });
        assert_eq!(
            snapshot, ZERO_ALLOCATION,
            "Heartbeat datagram encode allocation budget"
        );
    });
}
