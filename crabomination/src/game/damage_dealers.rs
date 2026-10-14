//! Two per-permanent rule reads. "As long as [this] hasn't dealt damage yet"
//! (Ratonhnhaké꞉ton): a permanent that asks
//! (`CardDefinition::tracks_own_damage`) is stamped `has_dealt_damage` as soon
//! as a `DamageDealt` event names it, combat or not (combat only for one that
//! asks `tracks_own_combat_damage` — Ruric Thar, Magecrusher) — read at trigger
//! dispatch, which every damage path reaches. And CR 701.27's "can't
//! transform" statics (Immerwolf).

use super::GameState;
use super::types::GameEvent;

impl GameState {
    pub(crate) fn note_damage_dealers(&mut self, events: &[GameEvent]) {
        for ev in events {
            let GameEvent::DamageDealt { amount, from_card: Some(src), combat, .. } = ev else { continue };
            if *amount == 0 {
                continue;
            }
            // Immutable probe first: the mutable find unshares the battlefield.
            if self
                .battlefield_find(*src)
                .is_some_and(|c| {
                    !c.has_dealt_damage
                        && (c.definition.tracks_own_damage() || *combat && c.definition.tracks_own_combat_damage())
                })
                && let Some(c) = self.battlefield_find_mut(*src)
            {
                c.has_dealt_damage = true;
            }
        }
    }
}

impl GameState {
    /// CR 701.27 — a `StaticEffect::CantTransform` on the battlefield names
    /// `id` (Immerwolf).
    pub(crate) fn transform_prohibited(&self, id: crate::card::CardId) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|src| {
            src.definition.static_abilities.iter().any(|sa| match &sa.effect {
                StaticEffect::CantTransform { filter } => self.evaluate_requirement_static(
                    filter,
                    &crate::game::types::Target::Permanent(id),
                    src.controller,
                    Some(src.id),
                ),
                _ => false,
            })
        })
    }
}
