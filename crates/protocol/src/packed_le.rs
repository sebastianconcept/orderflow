//! Little-endian integer fields of an OFL1 payload.
//!
//! When a command or event codec reads or writes a packed integer, it uses this
//! module so stream and datagram payloads share one load and one store.

/// Answers the little-endian u64 at a byte offset in a payload slice.
pub(crate) fn read_u64(payload: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        payload[offset],
        payload[offset + 1],
        payload[offset + 2],
        payload[offset + 3],
        payload[offset + 4],
        payload[offset + 5],
        payload[offset + 6],
        payload[offset + 7],
    ])
}

/// Answers the little-endian i64 at a byte offset in a payload slice.
pub(crate) fn read_i64(payload: &[u8], offset: usize) -> i64 {
    i64::from_le_bytes([
        payload[offset],
        payload[offset + 1],
        payload[offset + 2],
        payload[offset + 3],
        payload[offset + 4],
        payload[offset + 5],
        payload[offset + 6],
        payload[offset + 7],
    ])
}

/// Answers the little-endian u128 at a byte offset in a payload slice.
pub(crate) fn read_u128(payload: &[u8], offset: usize) -> u128 {
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&payload[offset..offset + 16]);
    u128::from_le_bytes(bytes)
}

/// Writes a little-endian u64 at a byte offset in a payload slice.
pub(crate) fn write_u64(payload: &mut [u8], offset: usize, value: u64) {
    payload[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

/// Writes a little-endian i64 at a byte offset in a payload slice.
pub(crate) fn write_i64(payload: &mut [u8], offset: usize, value: i64) {
    payload[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

/// Writes a little-endian u128 at a byte offset in a payload slice.
pub(crate) fn write_u128(payload: &mut [u8], offset: usize, value: u128) {
    payload[offset..offset + 16].copy_from_slice(&value.to_le_bytes());
}
