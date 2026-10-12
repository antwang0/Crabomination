//! "As long as [this] hasn't dealt damage yet" (Ratonhnhaké꞉ton): a permanent
//! that asks (`CardDefinition::tracks_own_damage`) is stamped
//! `has_dealt_damage` as soon as a `DamageDealt` event names it, combat or
//! not. Read at trigger dispatch, which every damage path reaches.

use super::GameState;
use super::types::GameEvent;

impl GameState {
    pub(crate) fn note_damage_dealers(&mut self, events: &[GameEvent]) {
        for ev in events {
            let GameEvent::DamageDealt { amount, from_card: Some(src), .. } = ev else { continue };
            if *amount == 0 {
                continue;
            }
            // Immutable probe first: the mutable find unshares the battlefield.
            if self
                .battlefield_find(*src)
                .is_some_and(|c| !c.has_dealt_damage && c.definition.tracks_own_damage())
                && let Some(c) = self.battlefield_find_mut(*src)
            {
                c.has_dealt_damage = true;
            }
        }
    }
}
