//! Identifier newtypes for engine types.
//!
//! This module provides strongly-typed identifiers that wrap `u64` values.
//! All types implement `Copy`, `Debug`, `PartialEq`, and `Eq` for efficient
//! use in hot paths.

/// Order identifier - engine-assigned unique identifier for an order.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OrderId(u64);

impl OrderId {
    /// Create a new OrderId from a raw u64 value.
    pub fn new(id: u64) -> Self {
        OrderId(id)
    }

    /// Get the inner u64 value.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// Client order identifier - client-assigned unique identifier for a request.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClientOrderId(u64);

impl ClientOrderId {
    /// Create a new ClientOrderId from a raw u64 value.
    pub fn new(id: u64) -> Self {
        ClientOrderId(id)
    }

    /// Get the inner u64 value.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// Instrument identifier - unique identifier for a tradable instrument.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InstrumentId(u64);

impl InstrumentId {
    /// Create a new InstrumentId from a raw u64 value.
    pub fn new(id: u64) -> Self {
        InstrumentId(id)
    }

    /// Get the inner u64 value.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// Account identifier - unique identifier for an account.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AccountId(u64);

impl AccountId {
    /// Create a new AccountId from a raw u64 value.
    pub fn new(id: u64) -> Self {
        AccountId(id)
    }

    /// Get the inner u64 value.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// Session identifier - unique identifier for a matching session.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SessionId(u64);

impl SessionId {
    /// Create a new SessionId from a raw u64 value.
    pub fn new(id: u64) -> Self {
        SessionId(id)
    }

    /// Get the inner u64 value.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// Sequence number - monotonically increasing counter within a session.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Sequence(u64);

impl Sequence {
    /// Create a new Sequence from a raw u64 value.
    pub fn new(id: u64) -> Self {
        Sequence(id)
    }

    /// Get the inner u64 value.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// Timestamp in nanoseconds.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TimestampNanos(u64);

impl TimestampNanos {
    /// Create a new TimestampNanos from a raw u64 value.
    pub fn new(nanos: u64) -> Self {
        TimestampNanos(nanos)
    }

    /// Get the inner u64 value.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_id_new_round_trips_inner_u64() {
        // Given: a u64 value representing an order identifier
        // Given a u64 value
        let original_value: u64 = 12345;

        // When we create an OrderId from it and extract the inner value
        let order_id = OrderId::new(original_value);
        let extracted_value = order_id.inner();

        // Then the round-trip preserves the value
        assert_eq!(original_value, extracted_value);
    }

    #[test]
    fn session_id_is_copy() {
        // Given a SessionId
        let session_id = SessionId::new(999);

        // When we assign it to another variable
        let copy = session_id;

        // Then both the original and copy can be used (Copy trait)
        assert_eq!(session_id.inner(), 999);
        assert_eq!(copy.inner(), 999);
    }

    #[test]
    fn sequence_and_timestamp_nanos_are_distinct_types() {
        // Given a Sequence and TimestampNanos with the same underlying value
        let sequence = Sequence::new(42);
        let timestamp = TimestampNanos::new(42);

        // Verify they are both Copy
        let _seq_copy = sequence;
        let _ts_copy = timestamp;

        // When we compare their inner values
        assert_eq!(sequence.inner(), timestamp.inner());

        // Verify both can be used separately (compile-time distinct types)
        assert_eq!(sequence.inner(), 42);
        assert_eq!(timestamp.inner(), 42);
    }
}
