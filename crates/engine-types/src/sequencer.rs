//! Assignment of CommandSequence and EventSequence counters.
//!
//! When a parent engine stamps admission or event order, it uses this module so
//! each clock is an owned counter. JournalSequence is a different clock.

use crate::engine_command::EngineCommand;
use crate::identity::{CommandSequence, EventSequence};
use crate::sequenced_command::SequencedCommand;

/// CommandSequencer is the owned counter that stamps EngineCommand with CommandSequence.
/// When a parent engine assigns admission order before matching, it uses this type
/// so each engine instance owns its own command clock.
#[derive(Debug, Clone)]
pub struct CommandSequencer {
    next: u64,
}

impl CommandSequencer {
    /// Answers a CommandSequencer starting at zero.
    pub fn new() -> Self {
        CommandSequencer { next: 0 }
    }

    /// Answers a SequencedCommand stamped with the next CommandSequence.
    /// Admission order is CommandSequence, not EventSequence or JournalSequence.
    /// After u64::MAX the clock stays at u64::MAX, so later stamps can share a sequence.
    pub fn stamp(&mut self, command: EngineCommand) -> SequencedCommand {
        let command_sequence = CommandSequence::new(self.next);
        self.next = self.next.saturating_add(1);
        SequencedCommand::new(command_sequence, command)
    }

    /// Answers the CommandSequence this sequencer would assign next, without advancing.
    pub fn next_command_sequence(&self) -> CommandSequence {
        CommandSequence::new(self.next)
    }
}

impl Default for CommandSequencer {
    fn default() -> Self {
        Self::new()
    }
}

/// EventSequencer is the owned counter that stamps events with EventSequence.
/// When a parent engine assigns event order while appending EngineEvent, it uses
/// this type so each engine instance owns its own event clock.
#[derive(Debug, Clone)]
pub struct EventSequencer {
    next: u64,
}

impl EventSequencer {
    /// Answers an EventSequencer starting at zero.
    pub fn new() -> Self {
        EventSequencer { next: 0 }
    }

    /// Answers the next EventSequence of this sequencer, then advances.
    /// After u64::MAX the clock stays at u64::MAX, so later allocates can share a sequence.
    pub fn allocate(&mut self) -> EventSequence {
        let event_sequence = EventSequence::new(self.next);
        self.next = self.next.saturating_add(1);
        event_sequence
    }

    /// Answers the EventSequence this sequencer would assign next, without advancing.
    pub fn next_event_sequence(&self) -> EventSequence {
        EventSequence::new(self.next)
    }
}

impl Default for EventSequencer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{AccountId, ClientOrderId, InstrumentId};
    use crate::order::Side;
    use crate::price::Price;
    use crate::quantity::Quantity;

    fn sample_new_limit() -> EngineCommand {
        EngineCommand::NewLimit {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            price: Price::new(100),
            quantity: Quantity::new(10),
        }
    }

    #[test]
    fn command_sequencer_stamps_monotonic_sequence() {
        // Given a CommandSequencer
        let mut sequencer = CommandSequencer::new();

        // When we stamp three commands
        let first = sequencer.stamp(sample_new_limit());
        let second = sequencer.stamp(sample_new_limit());
        let third = sequencer.stamp(sample_new_limit());

        // Then the CommandSequence values are 0, 1, and 2
        assert_eq!(first.command_sequence().inner(), 0);
        assert_eq!(second.command_sequence().inner(), 1);
        assert_eq!(third.command_sequence().inner(), 2);
    }

    #[test]
    fn command_sequencer_instances_are_independent() {
        // Given two CommandSequencer instances
        let mut first_sequencer = CommandSequencer::new();
        let mut second_sequencer = CommandSequencer::new();

        // When each stamps one command
        let first = first_sequencer.stamp(sample_new_limit());
        let second = second_sequencer.stamp(sample_new_limit());

        // Then each starts at zero
        assert_eq!(first.command_sequence().inner(), 0);
        assert_eq!(second.command_sequence().inner(), 0);
    }

    #[test]
    fn duplicate_client_order_id_gets_distinct_command_sequence() {
        // Given two NewLimit commands that share AccountId and ClientOrderId
        let mut sequencer = CommandSequencer::new();
        let first_command = sample_new_limit();
        let second_command = sample_new_limit();

        // When we stamp both
        let first = sequencer.stamp(first_command);
        let second = sequencer.stamp(second_command);

        // Then they receive different CommandSequence values
        assert_eq!(first.command(), second.command());
        assert_ne!(first.command_sequence(), second.command_sequence());
    }

    #[test]
    fn event_sequencer_stamps_monotonic_sequence() {
        // Given an EventSequencer
        let mut sequencer = EventSequencer::new();

        // When we take three event sequences
        let first = sequencer.allocate();
        let second = sequencer.allocate();
        let third = sequencer.allocate();

        // Then the EventSequence values are 0, 1, and 2
        assert_eq!(first.inner(), 0);
        assert_eq!(second.inner(), 1);
        assert_eq!(third.inner(), 2);
    }
}
