//! Criterion groups for OFL1 encode, decode, journal walk, and datagram ingress.
//!
//! When a caller measures codec cost, it uses this bench so a later change
//! compares against a same-machine baseline. `thrpt` is how many of that
//! command or event are written or parsed per second.

use std::hint::black_box;
use std::time::Duration;

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, BenchmarkId, Criterion,
    Throughput,
};
use engine_types::{EngineEvent, JournalSequence};
use protocol::{
    command_codec::{
        decode_command, decode_command_datagram, decode_command_payload, encode_command,
        encode_command_datagram,
    },
    datagram,
    event_codec::{
        decode_event, decode_event_datagram, decode_event_payload, encode_event,
        encode_event_datagram,
    },
    journal_header::{JournalHeader, HEADER_SIZE},
    Frame, FRAME_HEADER_SIZE,
};

mod fixtures;

fn encode_frame(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("encode_frame");
    group.throughput(Throughput::Elements(1));
    let journal_sequence = JournalSequence::new(0);
    let session_id = fixtures::session_id();

    for (kind_name, sequenced_command) in fixtures::all_command_kinds() {
        group.bench_function(BenchmarkId::new("stream", kind_name), |bencher| {
            let mut buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
            bencher.iter(|| {
                let encoded_len = encode_command(
                    black_box(&sequenced_command),
                    black_box(journal_sequence),
                    &mut buffer,
                )
                .expect("stream command encode");
                black_box(&buffer[..encoded_len]);
            });
        });

        group.bench_function(BenchmarkId::new("datagram", kind_name), |bencher| {
            let mut buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
            bencher.iter(|| {
                let encoded_len = encode_command_datagram(
                    black_box(&sequenced_command),
                    black_box(session_id),
                    black_box(journal_sequence),
                    &mut buffer,
                )
                .expect("datagram command encode");
                black_box(&buffer[..encoded_len]);
            });
        });
    }

    for (kind_name, engine_event) in fixtures::all_event_kinds() {
        group.bench_function(BenchmarkId::new("stream", kind_name), |bencher| {
            let mut buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
            bencher.iter(|| {
                let encoded_len = encode_event(
                    black_box(&engine_event),
                    black_box(journal_sequence),
                    &mut buffer,
                )
                .expect("stream event encode");
                black_box(&buffer[..encoded_len]);
            });
        });

        group.bench_function(BenchmarkId::new("datagram", kind_name), |bencher| {
            let mut buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
            bencher.iter(|| {
                let encoded_len = encode_event_datagram(
                    black_box(&engine_event),
                    black_box(session_id),
                    black_box(journal_sequence),
                    &mut buffer,
                )
                .expect("datagram event encode");
                black_box(&buffer[..encoded_len]);
            });
        });
    }

    group.finish();
}

fn decode_frame(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("decode_frame");
    group.throughput(Throughput::Elements(1));
    let journal_sequence = JournalSequence::new(0);
    let session_id = fixtures::session_id();

    for (kind_name, sequenced_command) in fixtures::all_command_kinds() {
        let mut stream_buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
        let stream_len = encode_command(&sequenced_command, journal_sequence, &mut stream_buffer)
            .expect("stream command encode");
        group.bench_function(BenchmarkId::new("stream", kind_name), |bencher| {
            bencher.iter(|| {
                let decoded =
                    decode_command(black_box(&stream_buffer[..stream_len])).expect("stream decode");
                black_box(decoded)
            });
        });

        let mut datagram_buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
        let datagram_len = encode_command_datagram(
            &sequenced_command,
            session_id,
            journal_sequence,
            &mut datagram_buffer,
        )
        .expect("datagram command encode");
        group.bench_function(BenchmarkId::new("datagram", kind_name), |bencher| {
            bencher.iter(|| {
                let decoded = decode_command_datagram(black_box(&datagram_buffer[..datagram_len]))
                    .expect("datagram decode");
                black_box(decoded)
            });
        });
    }

    for (kind_name, engine_event) in fixtures::all_event_kinds() {
        let mut stream_buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
        let stream_len = encode_event(&engine_event, journal_sequence, &mut stream_buffer)
            .expect("stream event encode");
        group.bench_function(BenchmarkId::new("stream", kind_name), |bencher| {
            bencher.iter(|| {
                let decoded =
                    decode_event(black_box(&stream_buffer[..stream_len])).expect("stream decode");
                black_box(decoded)
            });
        });

        let mut datagram_buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
        let datagram_len = encode_event_datagram(
            &engine_event,
            session_id,
            journal_sequence,
            &mut datagram_buffer,
        )
        .expect("datagram event encode");
        group.bench_function(BenchmarkId::new("datagram", kind_name), |bencher| {
            bencher.iter(|| {
                let decoded = decode_event_datagram(black_box(&datagram_buffer[..datagram_len]))
                    .expect("datagram decode");
                black_box(decoded)
            });
        });
    }

    group.finish();
}

fn journal_walk_10k(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("journal_walk_10k");
    bench_journal_walk(&mut group, "mixed", fixtures::JOURNAL_FRAME_COUNT_10K);
    group.finish();
}

fn journal_walk_1m(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("journal_walk_1m");
    group.warm_up_time(Duration::from_secs(1));
    group.sample_size(10);
    bench_journal_walk(&mut group, "mixed", fixtures::JOURNAL_FRAME_COUNT_1M);
    group.finish();
}

fn bench_journal_walk(group: &mut BenchmarkGroup<'_, WallTime>, name: &str, frame_count: usize) {
    let journal = fixtures::encode_stream_journal(frame_count);
    group.throughput(Throughput::Elements(frame_count as u64));
    group.bench_function(name, |bencher| {
        bencher.iter(|| black_box(walk_stream_journal(black_box(&journal))));
    });
}

fn walk_stream_journal(journal: &[u8]) -> u64 {
    let header = JournalHeader::decode(journal).expect("journal header");
    let mut checksum = header.session_id().inner();
    let mut offset = HEADER_SIZE;
    while offset < journal.len() {
        let (kind, journal_sequence, payload) = Frame::decode(&journal[offset..]).expect("frame");
        if kind.is_command() {
            let sequenced_command =
                black_box(decode_command_payload(kind, payload).expect("command payload"));
            checksum = checksum
                .wrapping_add(sequenced_command.command_sequence().inner())
                .wrapping_add(journal_sequence.inner());
        } else {
            let engine_event =
                black_box(decode_event_payload(kind, payload).expect("event payload"));
            checksum = checksum
                .wrapping_add(event_sequence_inner(&engine_event))
                .wrapping_add(journal_sequence.inner());
        }
        offset += FRAME_HEADER_SIZE + payload.len();
    }
    checksum
}

fn datagram_ingress(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("datagram_ingress");
    let datagrams = fixtures::encode_datagrams(fixtures::JOURNAL_FRAME_COUNT_10K);
    group.throughput(Throughput::Elements(datagrams.len() as u64));
    group.bench_function("10k", |bencher| {
        bencher.iter(|| black_box(walk_datagrams(black_box(&datagrams))));
    });
    group.finish();
}

fn walk_datagrams(datagrams: &[Vec<u8>]) -> u64 {
    let mut checksum = 0u64;
    for datagram in datagrams {
        let (session_id, journal_sequence, kind, payload) =
            datagram::decode(datagram).expect("datagram");
        checksum = checksum
            .wrapping_add(session_id.inner())
            .wrapping_add(journal_sequence.inner());
        if kind.is_command() {
            let sequenced_command =
                black_box(decode_command_payload(kind, payload).expect("command payload"));
            checksum = checksum.wrapping_add(sequenced_command.command_sequence().inner());
        } else {
            let engine_event =
                black_box(decode_event_payload(kind, payload).expect("event payload"));
            checksum = checksum.wrapping_add(event_sequence_inner(&engine_event));
        }
    }
    checksum
}

fn event_sequence_inner(engine_event: &EngineEvent) -> u64 {
    match engine_event {
        EngineEvent::Accepted { event_sequence, .. }
        | EngineEvent::Rejected { event_sequence, .. }
        | EngineEvent::Replaced { event_sequence, .. }
        | EngineEvent::Canceled { event_sequence, .. }
        | EngineEvent::Trade { event_sequence, .. } => event_sequence.inner(),
    }
}

criterion_group!(
    benches,
    encode_frame,
    decode_frame,
    journal_walk_10k,
    journal_walk_1m,
    datagram_ingress
);
criterion_main!(benches);
