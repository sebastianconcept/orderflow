//! Invariants of the frozen mixed-journal slot cycle.

#[path = "../support/mixed_journal_slot.rs"]
mod mixed_journal_slot;

use mixed_journal_slot::{mixed_journal_slot, MixedJournalSlot};

#[test]
fn mixed_journal_repeats_frozen_slot_ratio_every_twenty_frames() {
    // Given the first twenty indexes in one mixed-journal cycle
    let mut new_limit = 0usize;
    let mut cancel_by_order = 0usize;
    let mut replace = 0usize;
    let mut trade = 0usize;
    let mut rejected = 0usize;

    // When we classify each slot
    for index in 0..20u64 {
        match mixed_journal_slot(index) {
            MixedJournalSlot::NewLimit => new_limit += 1,
            MixedJournalSlot::CancelByOrder => cancel_by_order += 1,
            MixedJournalSlot::Replace => replace += 1,
            MixedJournalSlot::Trade => trade += 1,
            MixedJournalSlot::Rejected => rejected += 1,
        }
    }

    // Then the ratio matches 50/20/10/15/5 percent of twenty frames
    assert_eq!(new_limit, 10);
    assert_eq!(cancel_by_order, 4);
    assert_eq!(replace, 2);
    assert_eq!(trade, 3);
    assert_eq!(rejected, 1);
}
