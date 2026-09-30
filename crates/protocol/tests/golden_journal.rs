//! Golden round-trip of mixed OFL1 command and event frames.
//!
//! When a caller checks stream and datagram envelopes, it uses this module so
//! both share one payload.

use engine_types::{
    AccountId, ClientOrderId, CommandSequence, EngineCommand, EngineEvent, EventSequence,
    InstrumentId, JournalSequence, OrderId, SequencedCommand, SessionId, Side, TimestampNanos,
};
use protocol::{
    command_codec::{
        decode_command, decode_command_datagram, encode_command, encode_command_datagram,
    },
    datagram::{encode_heartbeat, DATAGRAM_HEADER_SIZE},
    event_codec::{decode_event, decode_event_datagram, encode_event, encode_event_datagram},
    journal_cursor::JournalCursor,
    journal_header::{ClockKind, JournalHeader},
    FrameKind, FRAME_HEADER_SIZE,
};

fn sample_new_limit(command_sequence: u64) -> SequencedCommand {
    SequencedCommand::new(
        CommandSequence::new(command_sequence),
        EngineCommand::NewLimit {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            price: engine_types::Price::new(100),
            quantity: engine_types::Quantity::new(10),
        },
    )
}

#[test]
fn golden_journal_mixed_commands_and_events_round_trip() {
    // Given a session header and a mixed command/event journal
    let session_id = SessionId::new(42);
    let mut header_bytes = [0u8; 15];
    JournalHeader::new(session_id, ClockKind::Logical)
        .encode(&mut header_bytes)
        .expect("header");
    let mut cursor = JournalCursor::new();

    let new_limit = sample_new_limit(0);
    let accepted = EngineEvent::Accepted {
        event_sequence: EventSequence::new(0),
        command_sequence: CommandSequence::new(0),
        timestamp_nanos: TimestampNanos::new(1000),
        order_id: OrderId::new(100),
        client_order_id: ClientOrderId::new(7),
        account_id: AccountId::new(1),
    };
    let new_market = SequencedCommand::new(
        CommandSequence::new(1),
        EngineCommand::NewMarket {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(8),
            instrument_id: InstrumentId::new(3),
            side: Side::Sell,
            quantity: engine_types::Quantity::new(5),
        },
    );
    let trade = EngineEvent::Trade {
        event_sequence: EventSequence::new(1),
        command_sequence: CommandSequence::new(1),
        timestamp_nanos: TimestampNanos::new(2000),
        maker_order_id: OrderId::new(100),
        taker_order_id: OrderId::new(101),
        instrument_id: InstrumentId::new(2),
        price: engine_types::Price::new(100),
        quantity: engine_types::Quantity::new(3),
    };
    let cancel = SequencedCommand::new(
        CommandSequence::new(2),
        EngineCommand::CancelByOrder {
            order_id: OrderId::new(100),
        },
    );
    let canceled = EngineEvent::Canceled {
        event_sequence: EventSequence::new(2),
        command_sequence: CommandSequence::new(2),
        timestamp_nanos: TimestampNanos::new(3000),
        order_id: OrderId::new(100),
    };

    // When we encode with contiguous JournalSequence values
    let mut journal = Vec::new();
    journal.extend_from_slice(&header_bytes);

    let mut frame_buf = [0u8; 128];

    let journal_0 = cursor.next_journal_sequence();
    let len = encode_command(&new_limit, journal_0, &mut frame_buf).expect("new");
    journal.extend_from_slice(&frame_buf[..len]);

    let journal_1 = cursor.next_journal_sequence();
    let len = encode_event(&accepted, journal_1, &mut frame_buf).expect("accepted");
    journal.extend_from_slice(&frame_buf[..len]);

    let journal_2 = cursor.next_journal_sequence();
    let len = encode_command(&new_market, journal_2, &mut frame_buf).expect("market");
    journal.extend_from_slice(&frame_buf[..len]);

    let journal_3 = cursor.next_journal_sequence();
    let len = encode_event(&trade, journal_3, &mut frame_buf).expect("trade");
    journal.extend_from_slice(&frame_buf[..len]);

    let journal_4 = cursor.next_journal_sequence();
    let len = encode_command(&cancel, journal_4, &mut frame_buf).expect("cancel");
    journal.extend_from_slice(&frame_buf[..len]);

    let journal_5 = cursor.next_journal_sequence();
    let len = encode_event(&canceled, journal_5, &mut frame_buf).expect("canceled");
    journal.extend_from_slice(&frame_buf[..len]);

    // Then the header and frames round-trip with contiguous JournalSequence 0..5
    let decoded_header = JournalHeader::decode(&journal[..15]).expect("header decode");
    assert_eq!(decoded_header.session_id(), session_id);

    let mut offset = 15usize;
    let (decoded_new, js0, consumed) = decode_command(&journal[offset..]).expect("new");
    assert_eq!(decoded_new, new_limit);
    assert_eq!(js0.inner(), 0);
    offset += consumed;

    let (decoded_accepted, js1, consumed) = decode_event(&journal[offset..]).expect("accepted");
    assert_eq!(decoded_accepted, accepted);
    assert_eq!(js1.inner(), 1);
    offset += consumed;

    let (decoded_market, js2, consumed) = decode_command(&journal[offset..]).expect("market");
    assert_eq!(decoded_market, new_market);
    assert_eq!(js2.inner(), 2);
    offset += consumed;

    let (decoded_trade, js3, consumed) = decode_event(&journal[offset..]).expect("trade");
    assert_eq!(decoded_trade, trade);
    assert_eq!(js3.inner(), 3);
    offset += consumed;

    let (decoded_cancel, js4, consumed) = decode_command(&journal[offset..]).expect("cancel");
    assert_eq!(decoded_cancel, cancel);
    assert_eq!(js4.inner(), 4);
    offset += consumed;

    let (decoded_canceled, js5, consumed) = decode_event(&journal[offset..]).expect("canceled");
    assert_eq!(decoded_canceled, canceled);
    assert_eq!(js5.inner(), 5);
    offset += consumed;

    assert_eq!(offset, journal.len());
}

#[test]
fn new_limit_and_trade_frames_stay_within_size_caps() {
    // Given a NewLimit command and a Trade event
    let sequenced = sample_new_limit(0);
    let trade = EngineEvent::Trade {
        event_sequence: EventSequence::new(1),
        command_sequence: CommandSequence::new(0),
        timestamp_nanos: TimestampNanos::new(2000),
        maker_order_id: OrderId::new(100),
        taker_order_id: OrderId::new(101),
        instrument_id: InstrumentId::new(2),
        price: engine_types::Price::new(100),
        quantity: engine_types::Quantity::new(3),
    };
    let mut stream_buf = [0u8; 128];
    let mut datagram_buf = [0u8; 512];

    // When we encode stream and datagram forms
    let stream_new =
        encode_command(&sequenced, JournalSequence::new(0), &mut stream_buf).expect("stream new");
    let stream_trade =
        encode_event(&trade, JournalSequence::new(1), &mut stream_buf).expect("stream trade");
    let datagram_new = encode_command_datagram(
        &sequenced,
        SessionId::new(1),
        JournalSequence::new(0),
        &mut datagram_buf,
    )
    .expect("datagram new");
    let datagram_trade = encode_event_datagram(
        &trade,
        SessionId::new(1),
        JournalSequence::new(1),
        &mut datagram_buf,
    )
    .expect("datagram trade");

    // Then stream frames fit in 128 and datagrams stay under 512
    assert!(stream_new <= 128);
    assert!(stream_trade <= 128);
    assert!(datagram_new <= 512);
    assert!(datagram_trade <= 512);
    assert_eq!(datagram_new, DATAGRAM_HEADER_SIZE + 57);
    assert_eq!(datagram_trade, DATAGRAM_HEADER_SIZE + 72);
}

#[test]
fn duplicate_new_limit_with_different_command_sequence_differs_on_wire() {
    // Given two NewLimit commands that share AccountId and ClientOrderId
    let first = sample_new_limit(1);
    let second = sample_new_limit(2);
    let mut buf1 = [0u8; 128];
    let mut buf2 = [0u8; 128];

    // When we encode both
    let len1 = encode_command(&first, JournalSequence::new(0), &mut buf1).expect("first");
    let len2 = encode_command(&second, JournalSequence::new(1), &mut buf2).expect("second");

    // Then encodings differ because CommandSequence differs
    assert_ne!(&buf1[..len1], &buf2[..len2]);
    let (decoded1, _, _) = decode_command(&buf1).expect("decode1");
    let (decoded2, _, _) = decode_command(&buf2).expect("decode2");
    assert_eq!(decoded1.command(), decoded2.command());
    assert_ne!(decoded1.command_sequence(), decoded2.command_sequence());
}

#[test]
fn stream_and_datagram_share_payload_bytes() {
    // Given a NewLimit command
    let sequenced = sample_new_limit(4);
    let session_id = SessionId::new(42);
    let journal_sequence = JournalSequence::new(11);
    let mut stream_buf = [0u8; 128];
    let mut datagram_buf = [0u8; 128];

    // When we encode both envelopes
    encode_command(&sequenced, journal_sequence, &mut stream_buf).expect("stream");
    let datagram_len =
        encode_command_datagram(&sequenced, session_id, journal_sequence, &mut datagram_buf)
            .expect("datagram");
    let (stream_decoded, stream_journal, _) = decode_command(&stream_buf).expect("stream decode");
    let (datagram_decoded, datagram_session, datagram_journal) =
        decode_command_datagram(&datagram_buf[..datagram_len]).expect("datagram decode");

    // Then both envelopes yield the same SequencedCommand
    assert_eq!(stream_decoded, sequenced);
    assert_eq!(datagram_decoded, sequenced);
    assert_eq!(stream_journal, journal_sequence);
    assert_eq!(datagram_journal, journal_sequence);
    assert_eq!(datagram_session, session_id);
}

#[test]
fn heartbeat_datagram_is_kind_zero_with_empty_payload() {
    // Given a heartbeat for the last JournalSequence
    let mut buffer = [0u8; DATAGRAM_HEADER_SIZE];
    let encoded_len =
        encode_heartbeat(SessionId::new(1), JournalSequence::new(99), &mut buffer).expect("encode");

    // When we decode through the datagram envelope
    let (_session, journal, kind, payload) =
        protocol::datagram::decode(&buffer[..encoded_len]).expect("decode");

    // Then kind is Heartbeat and payload is empty
    assert_eq!(kind, FrameKind::Heartbeat);
    assert_eq!(journal.inner(), 99);
    assert!(payload.is_empty());
}

#[test]
fn reserved_opcodes_fail_closed() {
    // Given reserved iceberg and RFQ kinds on the stream envelope
    for kind in [0x06u8, 0x40u8, 0xA0u8] {
        let mut buf = [0u8; FRAME_HEADER_SIZE];
        buf[0..2].copy_from_slice(&9u16.to_le_bytes());
        buf[2] = kind;
        buf[3..11].copy_from_slice(&0u64.to_le_bytes());

        // When we decode as a command
        let result = decode_command(&buf);

        // Then decode fails closed
        assert!(result.is_err(), "kind {kind:#04x} should fail closed");
    }
}

#[test]
fn reserved_event_opcodes_fail_closed_on_stream() {
    // Given reserved iceberg and RFQ kinds on the stream envelope
    for kind in [0x06u8, 0x40u8, 0xA0u8] {
        let mut buf = [0u8; FRAME_HEADER_SIZE];
        buf[0..2].copy_from_slice(&9u16.to_le_bytes());
        buf[2] = kind;
        buf[3..11].copy_from_slice(&0u64.to_le_bytes());

        // When we decode as an event
        let result = decode_event(&buf);

        // Then decode fails closed
        assert!(result.is_err(), "kind {kind:#04x} should fail closed");
    }
}

#[test]
fn reserved_opcodes_fail_closed_on_datagram() {
    // Given reserved iceberg and RFQ kinds on a datagram
    for kind in [0x06u8, 0x40u8, 0xA0u8] {
        let mut buffer = [0u8; DATAGRAM_HEADER_SIZE];
        buffer[16] = kind;

        // When we decode as a command datagram
        let command_result = decode_command_datagram(&buffer);
        // And when we decode as an event datagram
        let event_result = decode_event_datagram(&buffer);

        // Then both fail closed
        assert!(
            command_result.is_err(),
            "command kind {kind:#04x} should fail closed"
        );
        assert!(
            event_result.is_err(),
            "event kind {kind:#04x} should fail closed"
        );
    }
}
