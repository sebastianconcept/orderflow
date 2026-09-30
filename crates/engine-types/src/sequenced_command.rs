//! A client command stamped with a CommandSequence.
//!
//! When a matching engine receives work, it uses this module so admission order
//! travels with the intent. The stamp is CommandSequence, not EventSequence or
//! JournalSequence.

use crate::engine_command::EngineCommand;
use crate::identity::CommandSequence;

/// SequencedCommand is a client intent stamped with a CommandSequence.
/// When a matching engine processes work, it uses this type so admission order
/// travels with the intent. The stamp is CommandSequence, not EventSequence or
/// JournalSequence.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SequencedCommand {
    command_sequence: CommandSequence,
    command: EngineCommand,
}

impl SequencedCommand {
    /// Answers a SequencedCommand from a CommandSequence and an EngineCommand.
    pub fn new(command_sequence: CommandSequence, command: EngineCommand) -> Self {
        SequencedCommand {
            command_sequence,
            command,
        }
    }

    /// Answers the admission CommandSequence of this command.
    pub fn command_sequence(&self) -> CommandSequence {
        self.command_sequence
    }

    /// Answers the client intent of this SequencedCommand.
    pub fn command(&self) -> EngineCommand {
        self.command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{AccountId, ClientOrderId, InstrumentId};
    use crate::order::Side;
    use crate::price::Price;
    use crate::quantity::Quantity;

    #[test]
    fn sequenced_command_holds_command_sequence_and_command() {
        // Given a NewLimit command and a CommandSequence
        let command = EngineCommand::NewLimit {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };
        let command_sequence = CommandSequence::new(3);

        // When we wrap them in a SequencedCommand
        let sequenced = SequencedCommand::new(command_sequence, command);

        // Then both fields are preserved
        assert_eq!(sequenced.command_sequence().inner(), 3);
        assert_eq!(sequenced.command(), command);
    }

    #[test]
    fn sequenced_command_implements_copy() {
        // Given a SequencedCommand
        let sequenced = SequencedCommand::new(
            CommandSequence::new(1),
            EngineCommand::CancelByOrder {
                order_id: crate::identity::OrderId::new(42),
            },
        );

        // When we copy it
        let copy = sequenced;

        // Then the original and the copy compare equal
        assert_eq!(sequenced, copy);
    }
}
