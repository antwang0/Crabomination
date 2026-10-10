//! CR 702.153a — Ob Nixilis, the Adversary's "Casualty X. The copy isn't
//! legendary and has starting loyalty X": the casualty copy, just put on the
//! stack, is rewritten from the sacrificed creature's power.

use super::GameState;
use crate::card::{CardDefinition, CardId, Supertype};
use crate::effect::StaticEffect;
use crate::game::types::StackItem;

/// Does `def` print the casualty-X copy rewrite?
pub(crate) fn rewrites_casualty_copy(def: &CardDefinition) -> bool {
    def.static_abilities.iter().any(|s| matches!(s.effect, StaticEffect::CasualtyCopyLoyaltyFromSacrifice))
}

impl GameState {
    /// The topmost spell copy of `original` (a different object with the
    /// same name) loses Legendary and starts with `power` loyalty.
    pub(crate) fn rewrite_casualty_copy(&mut self, original: CardId, power: u32) {
        let Some(name) = self.stack.iter().find_map(|s| match s {
            StackItem::Spell { card, .. } if card.id == original => Some(card.definition.name),
            _ => None,
        }) else {
            return;
        };
        let Some(StackItem::Spell { card, .. }) = self.stack.iter_mut().rev().find(|s| {
            matches!(s, StackItem::Spell { card, .. } if card.id != original && card.definition.name == name)
        }) else {
            return;
        };
        let def = card.definition_make_mut();
        def.supertypes.retain(|t| *t != Supertype::Legendary);
        def.base_loyalty = power;
    }
}
