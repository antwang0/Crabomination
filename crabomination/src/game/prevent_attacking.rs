//! CR 615 — Rescue Retriever's "prevent all damage that would be dealt to
//! other attacking Soldiers you control": both damage funnels ask it through
//! `permanent_prevents_all_damage_to_self`, and only an attacking permanent
//! pays for the battlefield walk.

use super::GameState;
use crate::card::CardId;
use crate::effect::StaticEffect;

impl GameState {
    /// `tgt` is attacking and some `PreventAllDamageToAttackingMatching`
    /// static on the battlefield covers it.
    pub(crate) fn attacking_damage_shielded(&self, tgt: CardId) -> bool {
        if !self.attacking.iter().any(|a| a.attacker == tgt) {
            return false;
        }
        let Some(card) = self.battlefield_find(tgt) else { return false };
        self.battlefield.iter().any(|s| {
            s.definition.static_abilities.iter().any(|sa| match &sa.effect {
                StaticEffect::PreventAllDamageToAttackingMatching { filter } => {
                    self.evaluate_requirement_static_on(filter, card, s.controller, Some(s.id))
                }
                _ => false,
            })
        })
    }
}
