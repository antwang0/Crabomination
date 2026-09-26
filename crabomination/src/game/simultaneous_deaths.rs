//! CR 603.10a — "leaves-the-battlefield abilities" look back in time. When
//! creatures die at the same time (a Wrath, one state-based sweep), each
//! dying creature's "whenever another creature [you control] dies" trigger
//! sees every other creature that died with it — Midnight Reaper and a Bear
//! under Wrath of God draw two. The battlefield walk in
//! `dispatch_triggers_for_events` can't find these observers (they are in
//! the graveyard by dispatch time), and the self-death funnel only fires a
//! creature's triggers for its own death, so this pass supplies the rest from
//! the death snapshots.

use super::GameState;
use super::types::{GameEvent, TriggerCandidate};
use crate::card::CardId;
use crate::effect::{EventKind, EventScope};
use crate::game::effects::{EffectContext, events};

impl GameState {
    /// One candidate per (dying observer, other creature that died in the
    /// same batch) its printed death trigger matches. Empty unless two or
    /// more creatures died in `events`.
    pub(crate) fn simultaneous_death_observer_candidates(
        &self,
        events: &[GameEvent],
        dies_suppressed: bool,
    ) -> Vec<TriggerCandidate> {
        let mut out: Vec<TriggerCandidate> = Vec::new();
        if dies_suppressed {
            return out;
        }
        let mut died: Vec<CardId> = Vec::new();
        for ev in events {
            if let GameEvent::CreatureDied { card_id } = ev
                && !died.contains(card_id)
                && !self.death_was_replaced(*card_id)
            {
                died.push(*card_id);
            }
        }
        if died.len() < 2 {
            return out;
        }
        for &observer in &died {
            // Only a creature that is gone: one still on the battlefield
            // (returned by a replacement) is found by the battlefield walk.
            if self.battlefield_find(observer).is_some() {
                continue;
            }
            let Some(snap) = self.died_card_snapshots.get(&observer) else { continue };
            for ta in &snap.definition.triggered_abilities {
                if ta.event.kind != EventKind::CreatureDied || matches!(ta.event.scope, EventScope::SelfSource) {
                    continue;
                }
                let fanout = events::event_kind_fans_out(&ta.event.kind)
                    && !ta.event.once_per_turn
                    && !ta.event.once_per_batch;
                for ev in events {
                    let GameEvent::CreatureDied { card_id } = ev else { continue };
                    // Its own death is the self-death funnel's.
                    if *card_id == observer || !died.contains(card_id) {
                        continue;
                    }
                    if !events::event_matches_spec(self, ev, &ta.event, snap) {
                        continue;
                    }
                    let subject = events::event_subject(ev, &ta.event.kind);
                    let event_amount = self.event_amount_for(ev);
                    if let Some(filter) = &ta.event.filter {
                        let ctx = EffectContext::for_intervening_filter(snap.controller, observer, subject, event_amount);
                        if !self.evaluate_predicate(filter, &ctx) {
                            continue;
                        }
                    }
                    out.push(TriggerCandidate {
                        source: observer,
                        effect: ta.effect.clone(),
                        controller: snap.controller,
                        filter: ta.event.filter.clone(),
                        subject,
                        event_amount,
                        triggered_by_etb: false,
                        triggered_by_death: true,
                        triggered_by_attack: false,
                        triggered_by_land_entry: false,
                        triggered_by_face_up: false,
                        triggered_by_draw: false,
                        damaged_creature_controller: None,
                        from_mana_ability: false,
                        actor: events::event_actor(self, ev),
                    });
                    if !fanout {
                        break;
                    }
                }
            }
        }
        out
    }
}
