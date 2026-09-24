//! CR 205.4e — "A player may cast a legendary instant or sorcery spell only if
//! they control a legendary creature or a legendary planeswalker." The check
//! is a cast-time legality gate, not a cost: nothing is paid or tapped for it.

use super::GameState;
use crate::card::{CardDefinition, CardType, Supertype};

impl GameState {
    /// Whether `seat` may cast `def` as far as CR 205.4e is concerned: always,
    /// unless it is a legendary instant or sorcery and `seat` controls no
    /// legendary creature or planeswalker.
    pub(crate) fn legendary_spell_castable(&self, seat: usize, def: &CardDefinition) -> bool {
        let legendary_spell = def.supertypes.contains(&Supertype::Legendary)
            && (def.card_types.contains(&CardType::Instant) || def.card_types.contains(&CardType::Sorcery));
        !legendary_spell
            || self.battlefield.iter().any(|c| {
                c.controller == seat
                    && self.computed_permanent(c.id).is_some_and(|cp| {
                        cp.supertypes().contains(&Supertype::Legendary)
                            && (cp.card_types().contains(&CardType::Creature)
                                || cp.card_types().contains(&CardType::Planeswalker))
                    })
            })
    }
}
