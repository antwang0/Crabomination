//! Rod of Absorption / River Song's Diary: "whenever a player casts an
//! instant or sorcery spell [from their hand], exile it instead of putting it
//! into a graveyard as it resolves." The cast fixes which permanent takes the
//! spell, so one cast before the permanent arrived is not taken, and one cast
//! while it was there is exiled with it even if it has since left.

use super::GameState;
use super::types::{GameEvent, StackItem};
use crate::card::CardId;

impl GameState {
    /// The first permanent with an absorbing static that takes a spell cast
    /// `from_hand` or not.
    fn spell_absorber(&self, from_hand: bool) -> Option<CardId> {
        use crate::effect::StaticEffect as SE;
        self.battlefield
            .iter()
            .find(|c| {
                c.definition.static_abilities.iter().any(|sa| match sa.effect {
                    SE::ExileResolvingInstantsAndSorceries => true,
                    SE::ExileResolvingHandCastInstantsAndSorceries => from_hand,
                    _ => false,
                })
            })
            .map(|c| c.id)
    }

    /// Stamp each instant or sorcery `events` cast with its absorber (or clear
    /// a stale stamp from an earlier cast of the same card).
    pub(crate) fn stamp_absorbed_spells(&mut self, events: &[GameEvent]) {
        for ev in events {
            let GameEvent::SpellCast { card_id, .. } = *ev else { continue };
            let Some((from_hand, stale)) = self.stack.iter().find_map(|it| match it {
                StackItem::Spell { card, .. }
                    if card.id == card_id
                        && !card.is_token
                        && (card.definition.is_instant() || card.definition.is_sorcery()) =>
                {
                    Some((card.cast_from_hand, card.cold_any(|k| k.absorbed_by.is_some())))
                }
                _ => None,
            }) else {
                continue;
            };
            let absorber = self.spell_absorber(from_hand);
            if absorber.is_none() && !stale {
                continue;
            }
            for it in self.stack.iter_mut() {
                if let StackItem::Spell { card, .. } = it
                    && card.id == card_id
                {
                    card.absorbed_by = absorber;
                }
            }
        }
    }
}
