//! Twenty-frame cycle of the mixed OFL1 replay journal.
//!
//! When a journal builder or mix test picks the shape at an index, it uses this
//! module so the cycle stays 50% NewLimit, 20% CancelByOrder, 10% Replace,
//! 15% Trade, and 5% Rejected. Heartbeat is absent; control frames are covered
//! separately.

/// MixedJournalSlot names which shape appears at a position in the 20-frame cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixedJournalSlot {
    NewLimit,
    CancelByOrder,
    Replace,
    Trade,
    Rejected,
}

/// One 20-frame cycle. Index `n` uses the entry at `n % 20`.
const MIXED_JOURNAL_CYCLE: [MixedJournalSlot; 20] = [
    MixedJournalSlot::NewLimit,
    MixedJournalSlot::NewLimit,
    MixedJournalSlot::NewLimit,
    MixedJournalSlot::NewLimit,
    MixedJournalSlot::NewLimit,
    MixedJournalSlot::NewLimit,
    MixedJournalSlot::NewLimit,
    MixedJournalSlot::NewLimit,
    MixedJournalSlot::NewLimit,
    MixedJournalSlot::NewLimit,
    MixedJournalSlot::CancelByOrder,
    MixedJournalSlot::CancelByOrder,
    MixedJournalSlot::CancelByOrder,
    MixedJournalSlot::CancelByOrder,
    MixedJournalSlot::Replace,
    MixedJournalSlot::Replace,
    MixedJournalSlot::Trade,
    MixedJournalSlot::Trade,
    MixedJournalSlot::Trade,
    MixedJournalSlot::Rejected,
];

/// Answers which mixed-journal shape occupies index `index` in the 20-frame cycle.
pub fn mixed_journal_slot(index: u64) -> MixedJournalSlot {
    MIXED_JOURNAL_CYCLE[(index % 20) as usize]
}
