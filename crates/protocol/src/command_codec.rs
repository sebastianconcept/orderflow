//! Command encoding and decoding for the protocol.
//!
//! This module provides serialization and deserialization of [`EngineCommand`]
//! values into the compact binary format used in replayable event logs.
//!
//! # Wire Format
//!
//! Each command is encoded as a frame:
//!
//! - `frame_len` (u16, little-endian): Length of kind + payload
//! - `kind` (u8): Opcode identifying the command type
//! - `payload` (variable): Command-specific data
//!
//! # Opcodes
//!
//! | Kind | Name           |
//! |------|----------------|
//! | 0x01 | New            |
//! | 0x02 | CancelByOrder  |
//! | 0x03 | CancelByClient |
//! | 0x04 | Replace        |
//!
//! # Examples
//!
//! ```
//! use engine_types::{AccountId, ClientOrderId, EngineCommand, InstrumentId, OrderType, Price, Quantity, Side};
//! use protocol::command_codec;
//!
//! // Create a New command
//! let command = EngineCommand::New {
//!     account_id: AccountId::new(1),
//!     client_order_id: ClientOrderId::new(7),
//!     instrument_id: InstrumentId::new(2),
//!     side: Side::Buy,
//!     order_type: OrderType::Limit,
//!     price: Price::new(100),
//!     quantity: Quantity::new(10),
//! };
//!
//! // Encode to bytes
//! let mut buf = [0u8; 128];
//! let encoded_len = command_codec::encode_command(&command, &mut buf);
//!
//! // Decode back
//! let (decoded_cmd, consumed) = command_codec::decode_command(&buf).expect("should decode successfully");
//! assert_eq!(command, decoded_cmd);
//! ```
//!
//! # Error Handling
//!
//! Unknown opcodes return [`DecodeError::UnknownKind`]. Malformed data
//! returns appropriate errors describing the failure.

use engine_types::{
    AccountId, ClientOrderId, EngineCommand, InstrumentId, OrderId, OrderType, Price, Quantity,
    Side,
};

use crate::frame::{DecodeError as FrameDecodeError, Frame, FrameKind};
use thiserror::Error;

/// Error types for command decoding.
#[derive(Error, Debug)]
pub enum DecodeError {
    /// The input buffer is too short to contain a valid frame.
    #[error(transparent)]
    Frame(#[from] FrameDecodeError),

    /// The command kind is not recognized.
    #[error("unknown command kind: {0:#04x}")]
    UnknownKind(u8),

    /// The payload is too short for the expected command type.
    #[error("payload too short for command kind {kind:#04x}: expected at least {expected} bytes, got {actual}")]
    PayloadTooShort {
        kind: u8,
        expected: usize,
        actual: usize,
    },

    /// Failed to decode a required field from the payload.
    #[error("failed to decode {field} from payload")]
    FieldDecode { field: &'static str },
}

/// Encode an EngineCommand to a byte buffer.
///
/// # Arguments
///
/// * `command` - The command to encode.
/// * `buf` - Mutable buffer to write the encoded frame.
///
/// # Returns
///
/// The number of bytes written (frame header + payload).
pub fn encode_command(command: &EngineCommand, buf: &mut [u8]) -> usize {
    let kind = match command {
        EngineCommand::New { .. } => FrameKind::NEW,
        EngineCommand::CancelByOrder { .. } => FrameKind::CANCEL_BY_ORDER,
        EngineCommand::CancelByClient { .. } => FrameKind::CANCEL_BY_CLIENT,
        EngineCommand::Replace { .. } => FrameKind::REPLACE,
    };

    let payload = encode_command_payload(command);
    Frame::encode(kind, &payload, buf);

    FRAME_HEADER_SIZE + payload.len()
}

/// Encode a command's payload into bytes.
fn encode_command_payload(command: &EngineCommand) -> Vec<u8> {
    let mut payload = Vec::new();
    match command {
        EngineCommand::New {
            account_id,
            client_order_id,
            instrument_id,
            side,
            order_type,
            price,
            quantity,
        } => {
            // New payload: account_id u64, client_order_id u64, instrument_id u64,
            // side u8 (0 buy, 1 sell), order_type u8 (0 limit, 1 market),
            // price i64, quantity u128

            payload.extend_from_slice(&account_id.inner().to_le_bytes());
            payload.extend_from_slice(&client_order_id.inner().to_le_bytes());
            payload.extend_from_slice(&instrument_id.inner().to_le_bytes());

            // side: 0 = Buy, 1 = Sell
            let side_val = match side {
                Side::Buy => 0u8,
                Side::Sell => 1u8,
            };
            payload.push(side_val);

            // order_type: 0 = Limit, 1 = Market
            let order_type_val = match order_type {
                OrderType::Limit => 0u8,
                OrderType::Market => 1u8,
            };
            payload.push(order_type_val);

            payload.extend_from_slice(&price.inner().to_le_bytes());
            payload.extend_from_slice(&quantity.inner().to_le_bytes());
        }
        EngineCommand::CancelByOrder { order_id } => {
            // CancelByOrder payload: order_id u64
            payload.extend_from_slice(&order_id.inner().to_le_bytes());
        }
        EngineCommand::CancelByClient {
            account_id,
            client_order_id,
        } => {
            // CancelByClient payload: account_id u64, client_order_id u64
            payload.extend_from_slice(&account_id.inner().to_le_bytes());
            payload.extend_from_slice(&client_order_id.inner().to_le_bytes());
        }
        EngineCommand::Replace {
            order_id,
            client_order_id,
            price,
            quantity,
        } => {
            // Replace payload: order_id u64, client_order_id u64, price i64, quantity u128
            payload.extend_from_slice(&order_id.inner().to_le_bytes());
            payload.extend_from_slice(&client_order_id.inner().to_le_bytes());
            payload.extend_from_slice(&price.inner().to_le_bytes());
            payload.extend_from_slice(&quantity.inner().to_le_bytes());
        }
    }

    payload
}

/// Decode an EngineCommand from a byte slice.
///
/// # Arguments
///
/// * `buf` - The byte slice containing an encoded command frame.
///
/// # Returns
///
/// * `Ok((EngineCommand, consumed))` - The decoded command and number of bytes consumed.
/// * `Err(DecodeError)` - If decoding fails (unknown kind, truncated payload, etc.).
pub fn decode_command(buf: &[u8]) -> Result<(EngineCommand, usize), DecodeError> {
    // First decode the frame to get kind and payload
    let result = Frame::decode(buf);

    // Handle the frame decode result - re-raise UnknownKind directly
    let (kind, payload) = match result {
        Ok((k, p)) => (k, p),
        Err(FrameDecodeError::UnknownKind(kind_val)) => {
            // Re-raise UnknownKind as our own error
            return Err(DecodeError::UnknownKind(kind_val));
        }
        Err(e) => {
            // Wrap other errors
            return Err(DecodeError::Frame(e));
        }
    };

    // Extract kind value
    let kind_val = kind.inner();

    // Decode the command based on kind
    match kind_val {
        0x01 => decode_new_command(payload).map(|cmd| (cmd, FRAME_HEADER_SIZE + payload.len())),
        0x02 => decode_cancel_by_order(payload).map(|cmd| (cmd, FRAME_HEADER_SIZE + payload.len())),
        0x03 => {
            decode_cancel_by_client(payload).map(|cmd| (cmd, FRAME_HEADER_SIZE + payload.len()))
        }
        0x04 => decode_replace_command(payload).map(|cmd| (cmd, FRAME_HEADER_SIZE + payload.len())),
        _ => Err(DecodeError::UnknownKind(kind_val)),
    }
}

/// Decode a New command from its payload.
fn decode_new_command(payload: &[u8]) -> Result<EngineCommand, DecodeError> {
    const EXPECTED_LEN: usize = 50; // account_id(8) + client_order_id(8) + instrument_id(8) +
                                    // side(1) + order_type(1) + price(8) + quantity(16)

    if payload.len() < EXPECTED_LEN {
        return Err(DecodeError::PayloadTooShort {
            kind: 0x01,
            expected: EXPECTED_LEN,
            actual: payload.len(),
        });
    }

    let account_id = u64::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3], payload[4], payload[5], payload[6],
        payload[7],
    ]);

    let client_order_id = u64::from_le_bytes([
        payload[8],
        payload[9],
        payload[10],
        payload[11],
        payload[12],
        payload[13],
        payload[14],
        payload[15],
    ]);

    let instrument_id = u64::from_le_bytes([
        payload[16],
        payload[17],
        payload[18],
        payload[19],
        payload[20],
        payload[21],
        payload[22],
        payload[23],
    ]);

    let side_val = payload[24];
    let side = match side_val {
        0 => Side::Buy,
        1 => Side::Sell,
        _ => return Err(DecodeError::FieldDecode { field: "side" }),
    };

    let order_type_val = payload[25];
    let order_type = match order_type_val {
        0 => OrderType::Limit,
        1 => OrderType::Market,
        _ => {
            return Err(DecodeError::FieldDecode {
                field: "order_type",
            })
        }
    };

    let price = i64::from_le_bytes([
        payload[26],
        payload[27],
        payload[28],
        payload[29],
        payload[30],
        payload[31],
        payload[32],
        payload[33],
    ]);

    let quantity_bytes = [
        payload[34],
        payload[35],
        payload[36],
        payload[37],
        payload[38],
        payload[39],
        payload[40],
        payload[41],
        payload[42],
        payload[43],
        payload[44],
        payload[45],
        payload[46],
        payload[47],
        payload[48],
        payload[49],
    ];
    let quantity = u128::from_le_bytes(quantity_bytes);

    Ok(EngineCommand::New {
        account_id: AccountId::new(account_id),
        client_order_id: ClientOrderId::new(client_order_id),
        instrument_id: InstrumentId::new(instrument_id),
        side,
        order_type,
        price: Price::from_i64(price),
        quantity: Quantity::from_u128(quantity),
    })
}

/// Decode a CancelByOrder command from its payload.
fn decode_cancel_by_order(payload: &[u8]) -> Result<EngineCommand, DecodeError> {
    const EXPECTED_LEN: usize = 8; // order_id u64

    if payload.len() < EXPECTED_LEN {
        return Err(DecodeError::PayloadTooShort {
            kind: 0x02,
            expected: EXPECTED_LEN,
            actual: payload.len(),
        });
    }

    let order_id = u64::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3], payload[4], payload[5], payload[6],
        payload[7],
    ]);

    Ok(EngineCommand::CancelByOrder {
        order_id: OrderId::new(order_id),
    })
}

/// Decode a CancelByClient command from its payload.
fn decode_cancel_by_client(payload: &[u8]) -> Result<EngineCommand, DecodeError> {
    const EXPECTED_LEN: usize = 16; // account_id u64 + client_order_id u64

    if payload.len() < EXPECTED_LEN {
        return Err(DecodeError::PayloadTooShort {
            kind: 0x03,
            expected: EXPECTED_LEN,
            actual: payload.len(),
        });
    }

    let account_id = u64::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3], payload[4], payload[5], payload[6],
        payload[7],
    ]);

    let client_order_id = u64::from_le_bytes([
        payload[8],
        payload[9],
        payload[10],
        payload[11],
        payload[12],
        payload[13],
        payload[14],
        payload[15],
    ]);

    Ok(EngineCommand::CancelByClient {
        account_id: AccountId::new(account_id),
        client_order_id: ClientOrderId::new(client_order_id),
    })
}

/// Decode a Replace command from its payload.
fn decode_replace_command(payload: &[u8]) -> Result<EngineCommand, DecodeError> {
    const EXPECTED_LEN: usize = 40; // order_id(8) + client_order_id(8) + price(8) + quantity(16)

    if payload.len() < EXPECTED_LEN {
        return Err(DecodeError::PayloadTooShort {
            kind: 0x04,
            expected: EXPECTED_LEN,
            actual: payload.len(),
        });
    }

    let order_id = u64::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3], payload[4], payload[5], payload[6],
        payload[7],
    ]);

    let client_order_id = u64::from_le_bytes([
        payload[8],
        payload[9],
        payload[10],
        payload[11],
        payload[12],
        payload[13],
        payload[14],
        payload[15],
    ]);

    let price = i64::from_le_bytes([
        payload[16],
        payload[17],
        payload[18],
        payload[19],
        payload[20],
        payload[21],
        payload[22],
        payload[23],
    ]);

    let quantity_bytes = [
        payload[24],
        payload[25],
        payload[26],
        payload[27],
        payload[28],
        payload[29],
        payload[30],
        payload[31],
        payload[32],
        payload[33],
        payload[34],
        payload[35],
        payload[36],
        payload[37],
        payload[38],
        payload[39],
    ];
    let quantity = u128::from_le_bytes(quantity_bytes);

    Ok(EngineCommand::Replace {
        order_id: OrderId::new(order_id),
        client_order_id: ClientOrderId::new(client_order_id),
        price: Price::from_i64(price),
        quantity: Quantity::from_u128(quantity),
    })
}

/// Size of the frame header (frame_len + kind).
pub const FRAME_HEADER_SIZE: usize = 3;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that New limit order round-trips through encode/decode.
    #[test]
    fn command_codec_new_limit_round_trips() {
        // Given: a New limit buy order
        let command = EngineCommand::New {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            order_type: OrderType::Limit,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };

        // When: we encode and decode
        let mut buf = [0u8; 128];
        let encoded_len = encode_command(&command, &mut buf);

        let (decoded_cmd, consumed) = decode_command(&buf).expect("decode should succeed");

        // Then: round trip preserves command
        assert_eq!(command, decoded_cmd);
        assert_eq!(encoded_len, consumed);

        // Verify size is within limit (128 bytes max per SPECS)
        assert!(encoded_len <= 128);
    }

    /// Test that CancelByClient round-trips through encode/decode.
    #[test]
    fn command_codec_cancel_by_client_round_trips() {
        // Given: a CancelByClient command
        let command = EngineCommand::CancelByClient {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
        };

        // When: we encode and decode
        let mut buf = [0u8; 128];
        let encoded_len = encode_command(&command, &mut buf);

        let (decoded_cmd, consumed) = decode_command(&buf).expect("decode should succeed");

        // Then: round trip preserves command
        assert_eq!(command, decoded_cmd);
        assert_eq!(encoded_len, consumed);

        // Verify size is within limit (128 bytes max per SPECS)
        assert!(encoded_len <= 128);
    }

    /// Test that CancelByOrder round-trips through encode/decode.
    #[test]
    fn command_codec_cancel_by_order_round_trips() {
        // Given: a CancelByOrder command
        let command = EngineCommand::CancelByOrder {
            order_id: OrderId::new(42),
        };

        // When: we encode and decode
        let mut buf = [0u8; 128];
        let encoded_len = encode_command(&command, &mut buf);

        let (decoded_cmd, consumed) = decode_command(&buf).expect("decode should succeed");

        // Then: round trip preserves command
        assert_eq!(command, decoded_cmd);
        assert_eq!(encoded_len, consumed);

        // Verify size is within limit
        assert!(encoded_len <= 128);
    }

    /// Test that Replace round-trips through encode/decode.
    #[test]
    fn command_codec_replace_round_trips() {
        // Given: a Replace command
        let command = EngineCommand::Replace {
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(99),
            price: Price::new(105),
            quantity: Quantity::new(12),
        };

        // When: we encode and decode
        let mut buf = [0u8; 128];
        let encoded_len = encode_command(&command, &mut buf);

        let (decoded_cmd, consumed) = decode_command(&buf).expect("decode should succeed");

        // Then: round trip preserves command
        assert_eq!(command, decoded_cmd);
        assert_eq!(encoded_len, consumed);

        // Verify size is within limit
        assert!(encoded_len <= 128);
    }

    /// Test that unknown kind returns decode error.
    #[test]
    fn command_codec_unknown_kind_returns_decode_error() {
        // Given: a buffer with unknown kind (0xFF)
        let mut buf = [0u8; 10];
        // frame_len = 2 (kind + 1 byte payload)
        buf[0..2].copy_from_slice(&2u16.to_le_bytes());
        // Unknown kind
        buf[2] = 0xFF;
        // Payload byte
        buf[3] = 0x00;

        // When: we try to decode
        let result = decode_command(&buf);

        // Then: it returns an error about unknown kind
        assert!(matches!(result, Err(DecodeError::UnknownKind(0xFF))));
    }

    /// Test that truncated payload returns error.
    #[test]
    fn command_codec_truncated_payload_returns_error() {
        // Given: a buffer with New frame but truncated payload
        let mut buf = [0u8; 20]; // claims more than it has
                                 // frame_len = 35 (claiming 35 bytes after header)
        buf[0..2].copy_from_slice(&35u16.to_le_bytes());
        // Valid kind (New = 0x01)
        buf[2] = FrameKind::NEW.encode();

        // When: we try to decode
        let result = decode_command(&buf);

        // Then: it returns an error about payload too short (wrapped in Frame)
        assert!(matches!(
            result,
            Err(DecodeError::Frame(FrameDecodeError::PayloadTooShort { .. }))
        ));
    }

    /// Test that side and order_type decode validate values.
    #[test]
    fn command_codec_side_and_order_type_decode_validates_values() {
        // Given: a New payload with invalid side value (2)
        let mut buf = [0u8; 54];
        // frame_len = 51 (kind + payload)
        buf[0..2].copy_from_slice(&51u16.to_le_bytes());
        // Valid kind (New = 0x01)
        buf[2] = FrameKind::NEW.encode();

        // Write valid values for first 25 bytes
        buf[3..11].copy_from_slice(&1u64.to_le_bytes()); // account_id
        buf[11..19].copy_from_slice(&7u64.to_le_bytes()); // client_order_id
        buf[19..27].copy_from_slice(&2u64.to_le_bytes()); // instrument_id

        // Invalid side value (2)
        buf[27] = 2;

        // Valid order_type (0)
        buf[28] = 0;

        // Valid price and quantity
        buf[29..37].copy_from_slice(&100i64.to_le_bytes());
        buf[37..53].copy_from_slice(&10u128.to_le_bytes());

        // When: we try to decode
        let result = decode_command(&buf);

        // Then: it returns an error about side value
        assert!(matches!(
            result,
            Err(DecodeError::FieldDecode { field: "side", .. })
        ));
    }

    /// Test that encode produces expected byte sizes for each command type.
    #[test]
    fn command_codec_expected_sizes() {
        // New: 3 (header) + 50 (payload) = 53 bytes
        let new_cmd = EngineCommand::New {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            order_type: OrderType::Limit,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };
        let mut buf = [0u8; 128];
        assert_eq!(encode_command(&new_cmd, &mut buf), 53);

        // CancelByOrder: 3 (header) + 8 (payload) = 11 bytes
        let cancel_order_cmd = EngineCommand::CancelByOrder {
            order_id: OrderId::new(42),
        };
        assert_eq!(encode_command(&cancel_order_cmd, &mut buf), 11);

        // CancelByClient: 3 (header) + 16 (payload) = 19 bytes
        let cancel_client_cmd = EngineCommand::CancelByClient {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
        };
        assert_eq!(encode_command(&cancel_client_cmd, &mut buf), 19);

        // Replace: 3 (header) + 40 (payload) = 43 bytes
        let replace_cmd = EngineCommand::Replace {
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(99),
            price: Price::new(105),
            quantity: Quantity::new(12),
        };
        assert_eq!(encode_command(&replace_cmd, &mut buf), 43);
    }

    /// Test that duplicate New with different sequence round-trips correctly.
    #[test]
    fn command_codec_duplicate_new_different_sequence() {
        // Given: two New commands (different client_order_ids would be different requests)
        let cmd1 = EngineCommand::New {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            order_type: OrderType::Limit,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };

        // When: we encode both
        let mut buf1 = [0u8; 128];
        let len1 = encode_command(&cmd1, &mut buf1);

        // Then: round trip preserves both
        let (decoded1, _) = decode_command(&buf1).expect("decode should succeed");
        assert_eq!(cmd1, decoded1);

        // Encode another command with different values
        let cmd2 = EngineCommand::New {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(8), // different
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            order_type: OrderType::Limit,
            price: Price::new(101),
            quantity: Quantity::new(11),
        };

        let mut buf2 = [0u8; 128];
        let len2 = encode_command(&cmd2, &mut buf2);

        // Then: different commands have different encodings
        assert_ne!(buf1[..len1], buf2[..len2]);

        // And round-trip still works
        let (decoded2, _) = decode_command(&buf2).expect("decode should succeed");
        assert_eq!(cmd2, decoded2);
    }
}
