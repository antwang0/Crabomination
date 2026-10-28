//! Dr. McCoy, Surly Surgeon — CR 614.8: a destruction replacement that is
//! not regeneration. "If another creature you control with a +1/+1 counter
//! on it would be destroyed, instead remove all +1/+1 counters from it and
//! all damage already dealt to it is healed." The creature stays untapped
//! and in combat.

use crate::card::{CardId, CounterType};
use crate::effect::StaticEffect;
use crate::game::{GameState, KeywordSlice};

impl GameState {
    /// The simultaneous-destruction pass (Wrath of God, lethal damage in one
    /// SBA check): replace every victim's destruction before any victim —
    /// McCoy among them — leaves. An indestructible victim isn't destroyed,
    /// so nothing is replaced.
    pub(crate) fn shed_before_simultaneous_destroy(&mut self, cid: CardId) -> bool {
        let indestructible = self
            .computed_permanent(cid)
            .is_some_and(|cp| cp.keywords().has_kw(&crate::card::Keyword::Indestructible));
        !indestructible && self.shed_counters_instead_of_destroyed(cid)
    }

    /// Apply the replacement to `cid` if a source of its controller's offers
    /// it; true when the destruction was replaced.
    pub(crate) fn shed_counters_instead_of_destroyed(&mut self, cid: CardId) -> bool {
        let Some(c) = self.battlefield_find(cid) else { return false };
        if c.counter_count(CounterType::PlusOnePlusOne) == 0 || !self.computed_is_creature(c) {
            return false;
        }
        let ctrl = c.controller;
        let offered = self.battlefield.iter().any(|s| {
            s.id != cid
                && s.controller == ctrl
                && s.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, StaticEffect::OtherCounteredCreaturesShedCountersInsteadOfDestroyed)
                })
        });
        if !offered {
            return false;
        }
        if let Some(c) = self.battlefield_find_mut(cid) {
            c.counters.remove(&CounterType::PlusOnePlusOne);
            c.damage = 0;
        }
        true
    }
}
