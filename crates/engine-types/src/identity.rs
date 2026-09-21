//! u64 identity of orders, accounts, instruments, sessions, sequences, and timestamps.
//!
//! When a command or event names a record, it uses this module so each kind is a
//! distinct type.

/// OrderId is the engine-assigned identity of a resting or filled order.
/// When the engine names an order after an accept, it uses this type so the key is
/// an engine order identifier, not a client request identifier.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OrderId(u64);

impl OrderId {
    /// Answers an OrderId from a u64.
    pub fn new(id: u64) -> Self {
        OrderId(id)
    }

    /// Answers the u64 inside this OrderId.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// ClientOrderId is the client-assigned identity of a request.
/// When a matcher treats two New commands as one request, it uses this type so the
/// key is a client request identifier, not an engine OrderId.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClientOrderId(u64);

impl ClientOrderId {
    /// Answers a ClientOrderId from a u64.
    pub fn new(id: u64) -> Self {
        ClientOrderId(id)
    }

    /// Answers the u64 inside this ClientOrderId.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// InstrumentId is the identity of a tradable instrument.
/// When a command names the book it targets, it uses this type so the key is an
/// instrument identifier, not an account identifier.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InstrumentId(u64);

impl InstrumentId {
    /// Answers an InstrumentId from a u64.
    pub fn new(id: u64) -> Self {
        InstrumentId(id)
    }

    /// Answers the u64 inside this InstrumentId.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// AccountId is the identity of a trading account.
/// When CancelByClient names the opening New, it uses this type so the key is an
/// account identifier, not a session identifier.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AccountId(u64);

impl AccountId {
    /// Answers an AccountId from a u64.
    pub fn new(id: u64) -> Self {
        AccountId(id)
    }

    /// Answers the u64 inside this AccountId.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// SessionId is the identity of a matching session.
/// When a journal header names the session that owns its frames, it uses this type
/// so the key is session identity, not an account identifier.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SessionId(u64);

impl SessionId {
    /// Answers a SessionId from a u64.
    pub fn new(id: u64) -> Self {
        SessionId(id)
    }

    /// Answers the u64 inside this SessionId.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// CommandSequence is the monotonically increasing counter of commands in a session.
/// When a sequencer stamps admission order, it uses this type so matching and
/// command replay share one counter, distinct from EventSequence and JournalSequence.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommandSequence(u64);

impl CommandSequence {
    /// Answers a CommandSequence from a u64.
    pub fn new(id: u64) -> Self {
        CommandSequence(id)
    }

    /// Answers the u64 inside this CommandSequence.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// EventSequence is the monotonically increasing counter of events in a session.
/// When a consumer records a watermark, it uses this type so event order is
/// EventSequence, not CommandSequence or JournalSequence.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventSequence(u64);

impl EventSequence {
    /// Answers an EventSequence from a u64.
    pub fn new(id: u64) -> Self {
        EventSequence(id)
    }

    /// Answers the u64 inside this EventSequence.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// JournalSequence is the monotonically increasing counter of OFL1 frames in a session.
/// When a journal names a frame position, it uses this type so the index is
/// JournalSequence, not CommandSequence or EventSequence.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JournalSequence(u64);

impl JournalSequence {
    /// Answers a JournalSequence from a u64.
    pub fn new(id: u64) -> Self {
        JournalSequence(id)
    }

    /// Answers the u64 inside this JournalSequence.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// TimestampNanos is a clock reading in nanoseconds.
/// When an event records when it occurred, it uses this type so the value is a
/// timestamp, not a sequence number.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimestampNanos(u64);

impl TimestampNanos {
    /// Answers a TimestampNanos from a u64 nanosecond reading.
    pub fn new(nanos: u64) -> Self {
        TimestampNanos(nanos)
    }

    /// Answers the u64 inside this TimestampNanos.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_copy<T: Copy>() {}

    #[test]
    fn order_id_new_round_trips_inner_u64() {
        // Given a u64 value representing an order identifier
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

        // Then both the original and copy can be used
        assert_eq!(session_id.inner(), 999);
        assert_eq!(copy.inner(), 999);
    }

    #[test]
    fn three_clocks_and_timestamp_nanos_are_distinct_types() {
        // Given the three clocks and a TimestampNanos with the same underlying value
        let command_sequence = CommandSequence::new(42);
        let event_sequence = EventSequence::new(42);
        let journal_sequence = JournalSequence::new(42);
        let timestamp = TimestampNanos::new(42);

        let _command_copy = command_sequence;
        let _event_copy = event_sequence;
        let _journal_copy = journal_sequence;
        let _timestamp_copy = timestamp;

        // When we compare their inner values
        assert_eq!(command_sequence.inner(), event_sequence.inner());
        assert_eq!(event_sequence.inner(), journal_sequence.inner());
        assert_eq!(journal_sequence.inner(), timestamp.inner());

        // Then all remain usable as separate types
        assert_eq!(command_sequence.inner(), 42);
        assert_eq!(event_sequence.inner(), 42);
        assert_eq!(journal_sequence.inner(), 42);
        assert_eq!(timestamp.inner(), 42);
    }

    #[test]
    fn identifier_types_implement_copy() {
        // Given the identifier newtypes
        // When the compiler checks the Copy bound
        assert_copy::<OrderId>();
        assert_copy::<ClientOrderId>();
        assert_copy::<InstrumentId>();
        assert_copy::<AccountId>();
        assert_copy::<SessionId>();
        assert_copy::<CommandSequence>();
        assert_copy::<EventSequence>();
        assert_copy::<JournalSequence>();
        assert_copy::<TimestampNanos>();

        // Then each identifier type satisfies Copy
    }
}
