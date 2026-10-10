//! CR 113.6 / 603.6 — triggered abilities that function from exile however
//! the card got there: `TriggerZone::WhileExiled`, "if this card is exiled"
//! (Senu, Keen-Eyed Protector). The suspend and self-exile families keep
//! their own walk; this one sits behind the pile's `has_exile_trigger` lane,
//! so a dispatch with no such card in exile pays one word load.

use super::{GameEvent, GameState, TriggerCandidate};
use crate::effect::TriggerZone;

impl GameState {
    /// Push one candidate per matching event for each exiled card's
    /// `WhileExiled` trigger, controlled by the card's owner (CR 108.4a: a
    /// card in exile has no controller; its owner controls the trigger).
    pub(super) fn gather_while_exiled_triggers(
        &self,
        events: &[GameEvent],
        candidates: &mut Vec<TriggerCandidate>,
    ) {
        if !self.exile.has_exile_trigger() {
            return;
        }
        for card in self.exile.iter() {
            for ta in &card.definition.triggered_abilities {
                if ta.event.zone != TriggerZone::WhileExiled {
                    continue;
                }
                let mut seen: Vec<crate::game::effects::EntityRef> = Vec::new();
                for ev in events {
                    if !crate::game::effects::event_matches_spec(self, ev, &ta.event, card) {
                        continue;
                    }
                    let subject = crate::game::effects::event_subject(ev, &ta.event.kind);
                    if let Some(s) = subject {
                        if seen.contains(&s) {
                            continue;
                        }
                        seen.push(s);
                    }
                    candidates.push(TriggerCandidate {
                        actor: None,
                        source: card.id,
                        effect: self.trigger_effect_for(&ta.effect, &ta.event, card, ev, events),
                        controller: card.owner,
                        filter: ta.event.filter.clone(),
                        subject,
                        event_amount: self.event_amount_in(ev, events),
                        triggered_by_etb: false,
                        triggered_by_death: false,
                        triggered_by_attack: false,
                        triggered_by_land_entry: false,
                        triggered_by_face_up: false,
                        triggered_by_draw: false,
                        damaged_creature_controller: None,
                        from_mana_ability: false,
                    });
                    if !crate::game::effects::events::event_kind_fans_out(&ta.event.kind)
                        || ta.event.once_per_batch
                    {
                        break;
                    }
                }
            }
        }
    }
}
