//! Checks for SequencedCommand encode and decode.
//!
//! When a payload layout changes, these checks use the stream and datagram
//! envelopes so both still answer the same command.

use super::*;

fn sample_new_limit(command_sequence: u64) -> SequencedCommand {
    SequencedCommand::new(
        CommandSequence::new(command_sequence),
        EngineCommand::NewLimit {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            price: Price::new(100),
            quantity: Quantity::new(10),
        },
    )
}

#[test]
fn command_codec_new_limit_round_trips() {
    // Given a sequenced NewLimit command
    let sequenced = sample_new_limit(3);
    let mut buffer = [0u8; 128];

    // When we encode and decode on the stream envelope
    let encoded_len =
        encode_command(&sequenced, JournalSequence::new(9), &mut buffer).expect("encode");
    let (decoded, journal_sequence, consumed) = decode_command(&buffer).expect("decode");

    // Then round trip preserves the command and JournalSequence
    assert_eq!(decoded, sequenced);
    assert_eq!(journal_sequence.inner(), 9);
    assert_eq!(encoded_len, consumed);
    assert!(encoded_len <= 128);
}

#[test]
fn command_codec_new_market_has_no_price_bytes() {
    // Given a sequenced NewMarket command
    let sequenced = SequencedCommand::new(
        CommandSequence::new(1),
        EngineCommand::NewMarket {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(8),
            instrument_id: InstrumentId::new(2),
            side: Side::Sell,
            quantity: Quantity::new(5),
        },
    );
    let payload = encode_command_payload(&sequenced);
    let new_limit_payload = encode_command_payload(&sample_new_limit(1));

    // When we compare payload sizes
    // Then NewMarket is shorter than NewLimit (no price)
    assert_eq!(payload.len(), 49);
    assert_eq!(new_limit_payload.len(), 57);
    assert!(payload.len() < new_limit_payload.len());
}

#[test]
fn duplicate_client_order_id_gets_different_payload_bytes() {
    // Given two NewLimit commands that share AccountId and ClientOrderId
    let first = sample_new_limit(0);
    let second = sample_new_limit(1);

    // When we encode both payloads
    let first_payload = encode_command_payload(&first);
    let second_payload = encode_command_payload(&second);

    // Then the payloads differ because CommandSequence differs
    assert_ne!(first_payload, second_payload);
}

#[test]
fn stream_and_datagram_round_trip_same_payload() {
    // Given a sequenced NewLimit command
    let sequenced = sample_new_limit(4);
    let session_id = SessionId::new(42);
    let journal_sequence = JournalSequence::new(11);
    let mut stream_buf = [0u8; 128];
    let mut datagram_buf = [0u8; 128];

    // When we encode both envelopes and decode
    encode_command(&sequenced, journal_sequence, &mut stream_buf).expect("stream encode");
    let datagram_len =
        encode_command_datagram(&sequenced, session_id, journal_sequence, &mut datagram_buf)
            .expect("datagram encode");
    let (stream_decoded, stream_journal, _) = decode_command(&stream_buf).expect("stream");
    let (datagram_decoded, datagram_session, datagram_journal) =
        decode_command_datagram(&datagram_buf[..datagram_len]).expect("datagram");

    // Then both envelopes yield the same SequencedCommand and JournalSequence
    assert_eq!(stream_decoded, sequenced);
    assert_eq!(datagram_decoded, sequenced);
    assert_eq!(stream_journal, journal_sequence);
    assert_eq!(datagram_journal, journal_sequence);
    assert_eq!(datagram_session, session_id);
    assert_eq!(
        encode_command_payload(&stream_decoded),
        encode_command_payload(&datagram_decoded)
    );
}

#[test]
fn decode_rejects_unknown_command_kind() {
    // Given a stream frame with unknown kind
    let mut buf = [0u8; FRAME_HEADER_SIZE];
    buf[0..2].copy_from_slice(&9u16.to_le_bytes());
    buf[2] = 0xFF;
    buf[3..11].copy_from_slice(&0u64.to_le_bytes());

    // When we decode
    let result = decode_command(&buf);

    // Then it fails closed
    assert!(matches!(result, Err(DecodeError::UnknownKind(0xFF))));
}

#[test]
fn decode_rejects_invalid_side() {
    // Given a NewLimit payload whose side byte is not Buy or Sell
    let sequenced = sample_new_limit(0);
    let mut payload = encode_command_payload(&sequenced);
    payload[32] = 0x02;

    // When we decode the payload
    let result = decode_command_payload(FrameKind::NewLimit, &payload);

    // Then decode names the side field
    assert!(matches!(
        result,
        Err(DecodeError::FieldDecode { field: "side" })
    ));
}

#[test]
fn decode_rejects_new_limit_payload_length_mismatch() {
    // Given a NewLimit payload that is one byte short
    let sequenced = sample_new_limit(0);
    let payload = encode_command_payload(&sequenced);
    let short_payload = &payload[..payload.len() - 1];

    // When we decode the payload
    let result = decode_command_payload(FrameKind::NewLimit, short_payload);

    // Then decode names the expected NewLimit length
    assert_eq!(
        result,
        Err(DecodeError::PayloadLengthMismatch {
            kind: FrameKind::NewLimit.encode(),
            expected: 57,
            actual: 56,
        })
    );
}
