//! Assignment of JournalSequence for OFL1 frames.
//!
//! When a caller stamps stream or datagram frames, it uses this module so
//! positions are contiguous in one session. CommandSequence and EventSequence
//! are other clocks.

use engine_types::JournalSequence;

/// JournalCursor is the owned counter that stamps OFL1 frames with JournalSequence.
/// When a journal writer assigns frame positions in one session, it uses this
/// type so each engine instance owns its own journal clock.
#[derive(Debug, Clone)]
pub struct JournalCursor {
    next: u64,
}

impl JournalCursor {
    /// Answers a JournalCursor starting at zero.
    pub fn new() -> Self {
        JournalCursor { next: 0 }
    }

    /// Answers the next JournalSequence of this cursor, then advances.
    /// After u64::MAX the clock stays at u64::MAX, so later frames can share a sequence.
    pub fn next_journal_sequence(&mut self) -> JournalSequence {
        let journal_sequence = JournalSequence::new(self.next);
        self.next = self.next.saturating_add(1);
        journal_sequence
    }

    /// Answers the JournalSequence this cursor would assign next, without advancing.
    pub fn peek(&self) -> JournalSequence {
        JournalSequence::new(self.next)
    }
}

impl Default for JournalCursor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_cursor_assigns_contiguous_sequences() {
        // Given a JournalCursor
        let mut cursor = JournalCursor::new();

        // When we take three JournalSequence values
        let first = cursor.next_journal_sequence();
        let second = cursor.next_journal_sequence();
        let third = cursor.next_journal_sequence();

        // Then the values are contiguous 0, 1, and 2
        assert_eq!(first.inner(), 0);
        assert_eq!(second.inner(), 1);
        assert_eq!(third.inner(), 2);
    }

    #[test]
    fn journal_cursor_instances_are_independent() {
        // Given two JournalCursor instances
        let mut first = JournalCursor::new();
        let mut second = JournalCursor::new();

        // When each advances once
        let first_sequence = first.next_journal_sequence();
        let second_sequence = second.next_journal_sequence();

        // Then each starts at zero
        assert_eq!(first_sequence.inner(), 0);
        assert_eq!(second_sequence.inner(), 0);
    }
}
