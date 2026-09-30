//! Assignment of sequence numbers before matching.
//!
//! When a pipeline stage stamps EngineCommand, it uses this module so admission
//! order is CommandSequence. Commands are stamped, not Order values.

use engine_types::{CommandSequencer, EngineCommand, SequencedCommand};

/// Sequencer is the pipeline facade that stamps EngineCommand with CommandSequence.
/// When a pipeline stage admits commands into the match stream, it uses this
/// type so each engine instance owns its own command clock.
pub struct Sequencer {
    inner: CommandSequencer,
}

impl Sequencer {
    /// Answers a Sequencer.
    pub fn new() -> Self {
        Self {
            inner: CommandSequencer::new(),
        }
    }

    /// Answers a SequencedCommand stamped with the next CommandSequence.
    /// Admission order is CommandSequence, not EventSequence or JournalSequence.
    pub fn stamp(&mut self, command: EngineCommand) -> SequencedCommand {
        self.inner.stamp(command)
    }

    /// Answers the CommandSequence this sequencer would assign next, without advancing.
    pub fn next_command_sequence(&self) -> engine_types::CommandSequence {
        self.inner.next_command_sequence()
    }
}

impl Default for Sequencer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_types::{AccountId, ClientOrderId, InstrumentId, Price, Quantity, Side};

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
    fn stamp_assigns_monotonic_command_sequence() {
        // Given a sequencer and a NewLimit command
        let mut sequencer = Sequencer::new();
        let command = sample_new_limit();

        // When we stamp three times
        let first = sequencer.stamp(command);
        let second = sequencer.stamp(command);
        let third = sequencer.stamp(command);

        // Then the CommandSequence values are 0, 1, and 2
        assert_eq!(first.command_sequence().inner(), 0);
        assert_eq!(second.command_sequence().inner(), 1);
        assert_eq!(third.command_sequence().inner(), 2);
        assert_eq!(first.command(), command);
    }

    #[test]
    fn next_command_sequence_starts_at_zero() {
        // Given a sequencer
        let sequencer = Sequencer::new();

        // When we read the next CommandSequence
        let sequence = sequencer.next_command_sequence();

        // Then the next sequence is zero
        assert_eq!(sequence.inner(), 0);
    }

    #[test]
    fn sequencer_instances_are_independent() {
        // Given two sequencers
        let mut first = Sequencer::new();
        let mut second = Sequencer::new();

        // When each stamps one command
        let first_stamped = first.stamp(sample_new_limit());
        let second_stamped = second.stamp(sample_new_limit());

        // Then each starts at zero
        assert_eq!(first_stamped.command_sequence().inner(), 0);
        assert_eq!(second_stamped.command_sequence().inner(), 0);
    }
}
