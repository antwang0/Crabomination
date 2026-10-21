//! Per-object "can't" locks (CR 614.17): "[it] can't have counters put on
//! it" (`StaticEffect::CantHaveCountersPutOn`, Melira's Keepers, Blossombind)
//! and "[it] can't become untapped" (`StaticEffect::CantBecomeUntapped`,
//! Frozen in Ice, Blossombind). Each reads `This` on the object itself,
//! `AttachedTo(This)` on an Aura / Equipment attached to it, or
//! `ChosenPermanentOfSource` for the permanent its source remembered.

use crate::card::CardId;
use crate::effect::{Selector, StaticEffect};
use crate::game::GameState;

/// Whether a lock's `applies_to` on source `src` reaches `cid`.
fn reaches(applies_to: &Selector, src: &crate::card::CardInstance, cid: CardId) -> bool {
    match applies_to {
        Selector::This => src.id == cid,
        Selector::AttachedTo(inner) => matches!(**inner, Selector::This) && src.attached_to == Some(cid),
        // Suncleanser's "for as long as this creature remains": its remembered target.
        Selector::ChosenPermanentOfSource => src.chosen_permanent == Some(cid),
        _ => false,
    }
}

impl GameState {
    /// CR 122.1 / 614.17 — counters can't be put on permanent `cid`: Solemnity
    /// or a per-object lock. Every counter placement on a known permanent goes
    /// through `scaled_counter_count_on`, which answers 0 for a locked one.
    pub fn counters_locked_on(&self, cid: CardId) -> bool {
        self.counters_locked() || self.counters_locked_on_object(cid)
    }

    /// The per-object half of [`Self::counters_locked_on`].
    pub(crate) fn counters_locked_on_object(&self, cid: CardId) -> bool {
        self.battlefield.iter().any(|c| {
            (c.id == cid || c.attached_to == Some(cid) || c.chosen_permanent == Some(cid))
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(self.active_static(&sa.effect, c),
                        Some(StaticEffect::CantHaveCountersPutOn { applies_to }) if reaches(applies_to, c, cid))
                })
        })
    }

    /// CR 702.26 — "permanents can't phase in" is in force.
    pub fn phase_in_locked(&self) -> bool {
        self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(self.active_static(&sa.effect, c), Some(StaticEffect::PermanentsCantPhaseIn)))
        })
    }

    /// CR 701.26b / 614.17 — permanent `cid` can't become untapped, by the
    /// untap step or by an effect.
    pub fn cant_become_untapped(&self, cid: CardId) -> bool {
        self.battlefield.iter().any(|c| {
            (c.id == cid || c.attached_to == Some(cid) || c.chosen_permanent == Some(cid))
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(self.active_static(&sa.effect, c),
                        Some(StaticEffect::CantBecomeUntapped { applies_to }) if reaches(applies_to, c, cid))
                })
        })
    }
}
