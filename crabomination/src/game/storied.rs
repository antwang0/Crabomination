//! Storied — "If you control three or more artifacts, legendaries, and/or
//! Sagas, you have an enduring story for the rest of the game." Like ascend
//! (`ascend.rs`), the count can only grow as a permanent enters or changes
//! control, so that funnel asks here; seats with no storied permanent pay one
//! flag read.

use super::GameState;
use crate::card::{CardId, CardType, EnchantmentSubtype, Supertype};
use crate::effect::StaticEffect;

impl GameState {
    /// A permanent entered under (or moved to) some player's control: arm its
    /// controller if it is storied, then latch the enduring story once an
    /// armed seat controls three artifacts, legendaries and/or Sagas.
    pub(super) fn apply_permanent_storied(&mut self, entered: CardId) {
        let Some(c) = self.battlefield_find(entered) else { return };
        let p = c.controller;
        if self.players[p].enduring_story {
            return;
        }
        let storied = |c: &crate::card::CardInstance| {
            c.definition.static_abilities.iter().any(|sa| matches!(sa.effect, StaticEffect::Storied))
        };
        if !self.players[p].storied_armed {
            if !storied(c) {
                return;
            }
            self.players[p].storied_armed = true;
        }
        let mut count = 0;
        let mut any_storied = false;
        for c in self.battlefield.iter().filter(|c| c.controller == p) {
            any_storied |= storied(c);
            let d = &c.definition;
            if self.computed_has_card_type(c, CardType::Artifact)
                || d.supertypes.contains(&Supertype::Legendary)
                || d.subtypes.enchantment_subtypes.contains(&EnchantmentSubtype::Saga)
            {
                count += 1;
            }
        }
        if any_storied && count >= 3 {
            self.players[p].enduring_story = true;
        }
    }
}
