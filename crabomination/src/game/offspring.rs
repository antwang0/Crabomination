//! CR 702.175a — Offspring: "When this permanent enters, if its offspring
//! cost was paid, create a token that's a copy of it, except it's 1/1." The
//! trigger is the creature's own, whether the keyword is printed or a static
//! granted it as it was cast (Zinnia, Valley's Voice) — so it fires even if
//! the granting permanent has left by then.

use super::GameState;
use super::types::{GameEvent, TriggerPush};
use crate::effect::{Effect, PlayerRef, Selector, Value};

impl GameState {
    pub(crate) fn fire_offspring_triggers(&mut self, events: &[GameEvent]) {
        for ev in events {
            let GameEvent::PermanentEntered { card_id } = *ev else { continue };
            let Some(c) = self.battlefield_find(card_id) else { continue };
            // A granted offspring rides the `kicked` stamp, so a multikicked
            // or non-creature permanent (Marshal's Anthem) is not one.
            let granted = c.paid_granted_offspring()
                && self.computed_is_creature(c)
                && c.definition.has_multikicker().is_none();
            if !c.kicked {
                continue;
            }
            // A printed and a granted offspring both paid are two triggers.
            let n = usize::from(c.definition.has_offspring().is_some()) + usize::from(granted);
            let controller = c.controller;
            for _ in 0..n {
                self.push_stack(
                    TriggerPush::new(
                        card_id,
                        controller,
                        Effect::CreateTokenCopyOf {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            source: Selector::This,
                            extra_creature_types: vec![],
                            extra_card_types: vec![],
                            override_pt: Some((1, 1)),
                            override_colors: None,
                            enters_tapped: false,
                            non_legendary: false,
                            legendary: false,
                            extra_keywords: vec![],
                            no_mana_cost: false,
                            enters_with_counters: None,
                        },
                    )
                    .build(),
                );
            }
        }
    }
}
