//! Criterion groups for OFL1 encode, decode, journal walk, and datagram ingress.
//!
//! When a caller measures codec cost, it uses this bench so a later change
//! compares against a same-machine baseline. Single-frame groups keep Criterion's
//! byte throughput and also print how many of that command or event fit in one second.

use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, Bencher, BenchmarkGroup, Criterion,
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
    let journal_sequence = JournalSequence::new(0);
    let session_id = fixtures::session_id();

    for (kind_name, sequenced_command) in fixtures::command_kinds_for_timing_bench() {
        let mut size_buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
        let encoded_len = encode_command(&sequenced_command, journal_sequence, &mut size_buffer)
            .expect("stream command encode");
        bench_one_frame(
            &mut group,
            "encode_frame",
            "stream",
            kind_name,
            encoded_len,
            |bencher| {
                let mut buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
                bencher.iter(|| {
                    let encoded_len = encode_command(
                        black_box(&sequenced_command),
                        black_box(journal_sequence),
                        &mut buffer,
                    )
                    .expect("stream command encode");
                    black_box(encoded_len)
                });
            },
        );

        let encoded_len = encode_command_datagram(
            &sequenced_command,
            session_id,
            journal_sequence,
            &mut size_buffer,
        )
        .expect("datagram command encode");
        bench_one_frame(
            &mut group,
            "encode_frame",
            "datagram",
            kind_name,
            encoded_len,
            |bencher| {
                let mut buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
                bencher.iter(|| {
                    let encoded_len = encode_command_datagram(
                        black_box(&sequenced_command),
                        black_box(session_id),
                        black_box(journal_sequence),
                        &mut buffer,
                    )
                    .expect("datagram command encode");
                    black_box(encoded_len)
                });
            },
        );
    }

    for (kind_name, engine_event) in fixtures::event_kinds_for_timing_bench() {
        let mut size_buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
        let encoded_len = encode_event(&engine_event, journal_sequence, &mut size_buffer)
            .expect("stream event encode");
        bench_one_frame(
            &mut group,
            "encode_frame",
            "stream",
            kind_name,
            encoded_len,
            |bencher| {
                let mut buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
                bencher.iter(|| {
                    let encoded_len = encode_event(
                        black_box(&engine_event),
                        black_box(journal_sequence),
                        &mut buffer,
                    )
                    .expect("stream event encode");
                    black_box(encoded_len)
                });
            },
        );

        let encoded_len = encode_event_datagram(
            &engine_event,
            session_id,
            journal_sequence,
            &mut size_buffer,
        )
        .expect("datagram event encode");
        bench_one_frame(
            &mut group,
            "encode_frame",
            "datagram",
            kind_name,
            encoded_len,
            |bencher| {
                let mut buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
                bencher.iter(|| {
                    let encoded_len = encode_event_datagram(
                        black_box(&engine_event),
                        black_box(session_id),
                        black_box(journal_sequence),
                        &mut buffer,
                    )
                    .expect("datagram event encode");
                    black_box(encoded_len)
                });
            },
        );
    }

    group.finish();
}

fn decode_frame(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("decode_frame");
    let journal_sequence = JournalSequence::new(0);
    let session_id = fixtures::session_id();

    for (kind_name, sequenced_command) in fixtures::command_kinds_for_timing_bench() {
        let mut stream_buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
        let stream_len = encode_command(&sequenced_command, journal_sequence, &mut stream_buffer)
            .expect("stream command encode");
        bench_one_frame(
            &mut group,
            "decode_frame",
            "stream",
            kind_name,
            stream_len,
            |bencher| {
                bencher.iter(|| {
                    let decoded = decode_command(black_box(&stream_buffer[..stream_len]))
                        .expect("stream decode");
                    black_box(decoded)
                });
            },
        );

        let mut datagram_buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
        let datagram_len = encode_command_datagram(
            &sequenced_command,
            session_id,
            journal_sequence,
            &mut datagram_buffer,
        )
        .expect("datagram command encode");
        bench_one_frame(
            &mut group,
            "decode_frame",
            "datagram",
            kind_name,
            datagram_len,
            |bencher| {
                bencher.iter(|| {
                    let decoded =
                        decode_command_datagram(black_box(&datagram_buffer[..datagram_len]))
                            .expect("datagram decode");
                    black_box(decoded)
                });
            },
        );
    }

    for (kind_name, engine_event) in fixtures::event_kinds_for_timing_bench() {
        let mut stream_buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
        let stream_len = encode_event(&engine_event, journal_sequence, &mut stream_buffer)
            .expect("stream event encode");
        bench_one_frame(
            &mut group,
            "decode_frame",
            "stream",
            kind_name,
            stream_len,
            |bencher| {
                bencher.iter(|| {
                    let decoded = decode_event(black_box(&stream_buffer[..stream_len]))
                        .expect("stream decode");
                    black_box(decoded)
                });
            },
        );

        let mut datagram_buffer = [0u8; fixtures::MAX_ENCODED_STREAM_FRAME];
        let datagram_len = encode_event_datagram(
            &engine_event,
            session_id,
            journal_sequence,
            &mut datagram_buffer,
        )
        .expect("datagram event encode");
        bench_one_frame(
            &mut group,
            "decode_frame",
            "datagram",
            kind_name,
            datagram_len,
            |bencher| {
                bencher.iter(|| {
                    let decoded =
                        decode_event_datagram(black_box(&datagram_buffer[..datagram_len]))
                            .expect("datagram decode");
                    black_box(decoded)
                });
            },
        );
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

fn bench_journal_walk(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    frame_count: usize,
) {
    let journal = fixtures::encode_stream_journal(frame_count);
    group.throughput(Throughput::Bytes(journal.len() as u64));
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
            let sequenced_command = decode_command_payload(kind, payload).expect("command payload");
            checksum = checksum
                .wrapping_add(sequenced_command.command_sequence().inner())
                .wrapping_add(journal_sequence.inner());
        } else {
            let engine_event = decode_event_payload(kind, payload).expect("event payload");
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
    let total_bytes: usize = datagrams.iter().map(|datagram| datagram.len()).sum();
    group.throughput(Throughput::Bytes(total_bytes as u64));
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
            let sequenced_command = decode_command_payload(kind, payload).expect("command payload");
            checksum = checksum.wrapping_add(sequenced_command.command_sequence().inner());
        } else {
            let engine_event = decode_event_payload(kind, payload).expect("event payload");
            checksum = checksum.wrapping_add(event_sequence_inner(&engine_event));
        }
    }
    checksum
}

/// Prints how many of this frame fit in one second.
///
/// One iteration encodes or decodes one frame of this type. The slope Criterion
/// stores for that id is the same interval as the time line, so the count matches
/// the byte throughput already printed for this benchmark.
fn bench_one_frame(
    group: &mut BenchmarkGroup<'_, WallTime>,
    group_name: &str,
    envelope: &str,
    kind_name: &str,
    encoded_len: usize,
    mut measure: impl FnMut(&mut Bencher<'_, WallTime>),
) {
    let estimates_path = frame_estimates_path(group_name, envelope, kind_name);
    let stamp_before = estimates_stamp(&estimates_path);
    group.throughput(Throughput::Bytes(encoded_len as u64));
    group.bench_function(
        criterion::BenchmarkId::new(envelope, kind_name),
        |bencher| {
            measure(bencher);
        },
    );
    let stamp_after = estimates_stamp(&estimates_path);
    let ran = match (stamp_before, stamp_after) {
        (None, Some(_)) => true,
        (Some(before), Some(after)) => after > before,
        _ => false,
    };
    if !ran {
        return;
    }
    let Ok(estimates_json) = fs::read_to_string(&estimates_path) else {
        return;
    };
    if let Some(slope) = slope_nanoseconds(&estimates_json) {
        println!(
            "{group_name}/{envelope}/{kind_name}\n{}frames: {}",
            " ".repeat(24),
            format_frames_per_second(slope)
        );
    }
}

fn frame_estimates_path(group_name: &str, envelope: &str, kind_name: &str) -> PathBuf {
    criterion_home().join(format!(
        "{group_name}/{envelope}/{kind_name}/new/estimates.json"
    ))
}

fn criterion_home() -> PathBuf {
    if let Some(value) = std::env::var_os("CRITERION_HOME") {
        return PathBuf::from(value);
    }
    if let Some(value) = std::env::var_os("CARGO_TARGET_DIR") {
        return PathBuf::from(value).join("criterion");
    }
    // target/{profile}/deps/<bench> sits beside target/criterion, including a custom target dir.
    if let Ok(executable) = std::env::current_exe() {
        if let Some(target_directory) = executable
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
        {
            return target_directory.join("criterion");
        }
    }
    PathBuf::from("target/criterion")
}

fn estimates_stamp(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
}

/// Nanosecond slope Criterion prints as the time line: lower bound, point, upper bound.
struct SlopeNanoseconds {
    lower: f64,
    point: f64,
    upper: f64,
}

fn slope_nanoseconds(estimates_json: &str) -> Option<SlopeNanoseconds> {
    let slope_json = estimates_json.split("\"slope\":").nth(1)?;
    Some(SlopeNanoseconds {
        lower: json_f64_after(slope_json, "\"lower_bound\":")?,
        point: json_f64_after(slope_json, "\"point_estimate\":")?,
        upper: json_f64_after(slope_json, "\"upper_bound\":")?,
    })
}

fn json_f64_after(json: &str, key: &str) -> Option<f64> {
    let rest = json.split(key).nth(1)?;
    let number: String = rest
        .chars()
        .take_while(|character| {
            character.is_ascii_digit()
                || *character == '.'
                || *character == '-'
                || *character == '+'
                || *character == 'e'
                || *character == 'E'
        })
        .collect();
    number.parse().ok()
}

fn format_frames_per_second(slope: SlopeNanoseconds) -> String {
    let low = 1e9 / slope.upper;
    let mid = 1e9 / slope.point;
    let high = 1e9 / slope.lower;
    let (scale, unit) = if mid >= 1e9 {
        (1e9, "G/s")
    } else if mid >= 1e6 {
        (1e6, "M/s")
    } else if mid >= 1e3 {
        (1e3, "K/s")
    } else {
        (1.0, "/s")
    };
    format!(
        "[{:.3} {unit} {:.3} {unit} {:.3} {unit}]",
        low / scale,
        mid / scale,
        high / scale,
    )
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
