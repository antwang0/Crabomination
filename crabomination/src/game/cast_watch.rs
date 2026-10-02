//! "When you cast [that card] this turn, …" delayed triggers (Havengul Lich):
//! one watcher per (card, player), fired by that player's `SpellCast` of the
//! card. A card that leaves its zone any other way is a new object (CR 400.7)
//! and drops the watcher; cleanup lapses it (`expires_after_turn`).

use super::GameState;
use super::types::{DelayedKind, GameEvent, TriggerPush};
use crate::game::effects::EntityRef;

impl GameState {
    /// Fire or drop every `WhenPlayerCastsCard` watcher `events` reach. The
    /// cast spell is the trigger subject.
    pub(crate) fn fire_cast_watch_delayed(&mut self, events: &[GameEvent]) {
        if !self.delayed_triggers.iter().any(|dt| matches!(dt.kind, DelayedKind::WhenPlayerCastsCard { .. })) {
            return;
        }
        for ev in events {
            let (card, caster) = match *ev {
                GameEvent::SpellCast { player, card_id, .. } => (card_id, Some(player)),
                GameEvent::CardLeftGraveyard { card_id, .. } => (card_id, None),
                _ => continue,
            };
            let mut i = 0;
            while i < self.delayed_triggers.len() {
                let DelayedKind::WhenPlayerCastsCard { card: c, player } = self.delayed_triggers[i].kind else {
                    i += 1;
                    continue;
                };
                // Leaving the graveyard to be cast is the cast itself.
                let cast_now = |p: usize| {
                    events.iter().any(|e| matches!(*e, GameEvent::SpellCast { player, card_id, .. } if card_id == c && player == p))
                };
                if c != card || (caster.is_none() && cast_now(player)) {
                    i += 1;
                    continue;
                }
                let dt = self.delayed_triggers.remove(i);
                if caster == Some(player) {
                    self.stack.push(
                        TriggerPush::new(dt.source, dt.controller, dt.effect)
                            .target(dt.target)
                            .trigger_source(Some(EntityRef::Card(card)))
                            .build(),
                    );
                }
            }
        }
    }
}
