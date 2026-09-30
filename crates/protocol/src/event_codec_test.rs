//! Checks for EngineEvent encode and decode.
//!
//! When a payload layout changes, these checks use the stream and datagram
//! envelopes so both still answer the same event.

use engine_types::{Price, Quantity};

use super::*;

fn sample_accepted() -> EngineEvent {
    EngineEvent::Accepted {
        event_sequence: EventSequence::new(1),
        command_sequence: CommandSequence::new(10),
        timestamp_nanos: TimestampNanos::new(1000),
        order_id: OrderId::new(42),
        client_order_id: ClientOrderId::new(7),
        account_id: AccountId::new(1),
    }
}

#[test]
fn event_codec_accepted_round_trips() {
    // Given an Accepted event
    let event = sample_accepted();
    let mut buffer = [0u8; 128];

    // When we encode and decode on the stream envelope
    let encoded_len = encode_event(&event, JournalSequence::new(5), &mut buffer).expect("encode");
    let (decoded, journal_sequence, consumed) = decode_event(&buffer).expect("decode");

    // Then round trip preserves the event and JournalSequence
    assert_eq!(decoded, event);
    assert_eq!(journal_sequence.inner(), 5);
    assert_eq!(encoded_len, consumed);
    assert!(encoded_len <= 128);
}

#[test]
fn event_sequence_independent_of_command_sequence_on_wire() {
    // Given an Accepted whose clocks differ
    let event = sample_accepted();
    let payload = encode_event_payload(&event);

    // When we read the first two u64 fields
    let event_sequence = crate::packed_le::read_u64(&payload, 0);
    let command_sequence = crate::packed_le::read_u64(&payload, 8);

    // Then they remain distinct on the wire
    assert_eq!(event_sequence, 1);
    assert_eq!(command_sequence, 10);
}

#[test]
fn trade_payload_stays_within_128_bytes() {
    // Given a Trade event
    let event = EngineEvent::Trade {
        event_sequence: EventSequence::new(5),
        command_sequence: CommandSequence::new(14),
        timestamp_nanos: TimestampNanos::new(5000),
        maker_order_id: OrderId::new(10),
        taker_order_id: OrderId::new(20),
        instrument_id: InstrumentId::new(2),
        price: Price::new(100),
        quantity: Quantity::new(5),
    };
    let payload = encode_event_payload(&event);
    let mut datagram = [0u8; 512];
    let encoded_len = encode_event_datagram(
        &event,
        SessionId::new(1),
        JournalSequence::new(0),
        &mut datagram,
    )
    .expect("encode");

    // When we measure sizes
    // Then payload is at most 128 and the datagram is under 512
    assert!(payload.len() <= 128);
    assert!(encoded_len <= 512);
    assert_eq!(payload.len(), 72);
}

#[test]
fn stream_and_datagram_round_trip_same_payload() {
    // Given an Accepted event
    let event = sample_accepted();
    let session_id = SessionId::new(42);
    let journal_sequence = JournalSequence::new(11);
    let mut stream_buf = [0u8; 128];
    let mut datagram_buf = [0u8; 128];

    // When we encode both envelopes
    encode_event(&event, journal_sequence, &mut stream_buf).expect("stream");
    let datagram_len =
        encode_event_datagram(&event, session_id, journal_sequence, &mut datagram_buf)
            .expect("datagram");
    let (stream_decoded, _, _) = decode_event(&stream_buf).expect("stream decode");
    let (datagram_decoded, _, _) =
        decode_event_datagram(&datagram_buf[..datagram_len]).expect("datagram");

    // Then both envelopes yield the same event payload
    assert_eq!(stream_decoded, event);
    assert_eq!(datagram_decoded, event);
    assert_eq!(
        encode_event_payload(&stream_decoded),
        encode_event_payload(&datagram_decoded)
    );
}

#[test]
fn decode_rejects_invalid_rejection_reason() {
    // Given a Rejected payload whose reason byte is not in the closed set
    let event = EngineEvent::Rejected {
        event_sequence: EventSequence::new(2),
        command_sequence: CommandSequence::new(11),
        timestamp_nanos: TimestampNanos::new(2000),
        client_order_id: ClientOrderId::new(7),
        reason: EngineEventRejectReason::InvalidQuantity,
    };
    let mut payload = encode_event_payload(&event);
    payload[32] = 4;

    // When we decode the payload
    let result = decode_event_payload(FrameKind::Rejected, &payload);

    // Then decode names the invalid reason byte
    assert_eq!(result, Err(DecodeError::InvalidRejectionReason(4)));
}

#[test]
fn decode_rejects_accepted_payload_length_mismatch() {
    // Given an Accepted payload that is one byte short
    let payload = encode_event_payload(&sample_accepted());
    let short_payload = &payload[..payload.len() - 1];

    // When we decode the payload
    let result = decode_event_payload(FrameKind::Accepted, short_payload);

    // Then decode names the expected Accepted length
    assert_eq!(
        result,
        Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::Accepted.encode(),
            expected: 48,
            actual: 47,
        })
    );
}
