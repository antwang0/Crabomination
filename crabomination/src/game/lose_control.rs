//! "When you lose control of [it], …" delayed triggers (Ray of Command): one
//! watcher per (permanent, player), fired by the first `ControlChanged` that
//! moves the permanent away from that player — a duration ending in cleanup
//! (CR 514.2) and a later steal alike.

use super::GameState;
use super::types::{DelayedKind, GameEvent, TriggerPush};
use crate::game::effects::EntityRef;

impl GameState {
    /// Fire and drop every `WhenPlayerLosesControlOf` watcher a control change
    /// in `events` satisfies. The permanent is the trigger subject and target.
    pub(crate) fn fire_lose_control_delayed(&mut self, events: &[GameEvent]) {
        if !events.iter().any(|e| matches!(e, GameEvent::ControlChanged { .. })) {
            return;
        }
        for ev in events {
            let GameEvent::ControlChanged { card_id, from, .. } = *ev else { continue };
            let lost = DelayedKind::WhenPlayerLosesControlOf { card: card_id, player: from };
            let mut i = 0;
            while i < self.delayed_triggers.len() {
                if self.delayed_triggers[i].kind != lost {
                    i += 1;
                    continue;
                }
                let dt = self.delayed_triggers.remove(i);
                self.push_stack(
                    TriggerPush::new(dt.source, dt.controller, dt.effect)
                        .target(dt.target)
                        .trigger_source(Some(EntityRef::Permanent(card_id)))
                        .build(),
                );
            }
        }
    }
}
