//! CR 603.2 — an ability triggers the moment its event happens. Triggers are
//! collected after a resolution, from the permanents still on the
//! battlefield, so a listener the same resolution removed never saw the
//! events that happened while it was still there: Soul Warden under a Martial
//! Coup (X ≥ 5) gained nothing for the five Soldiers that entered before the
//! sweep. This pass supplies those triggers from the departed listener's
//! snapshot, for the entries the batch records between its own entry (if
//! any) and its departure.
//!
//! Scope: the kinds in [`LOOKS_BACK`] — ones that only the dispatch walk
//! fires, so a departed listener can't also have fired from a direct hook —
//! and only departures with an in-order event — the synthesized
//! `PermanentDied` is appended after the batch, so it can't date a departure.
//! Once-per-turn / once-per-batch listeners are left to the walk's budget
//! bookkeeping and skipped here.

use super::GameState;
use super::types::{GameEvent, TriggerCandidate};
use crate::card::CardId;
use crate::effect::{EventKind, EventScope};
use crate::game::effects::{EffectContext, events};

/// The listener kinds this pass serves: "whenever [a permanent] enters" and
/// "whenever you gain life" (printed `LifeGained` listeners fire only from
/// the walk; `fire_life_gained_watchers` serves delayed ones).
const LOOKS_BACK: [EventKind; 2] = [EventKind::EntersBattlefield, EventKind::LifeGained];

/// Whether `ev` is an event a [`LOOKS_BACK`] listener can see.
fn looked_back_on(ev: &GameEvent) -> bool {
    matches!(ev, GameEvent::PermanentEntered { .. } | GameEvent::LifeGained { .. })
}

/// The card an in-order departure event moves off the battlefield.
fn departure_of(ev: &GameEvent) -> Option<CardId> {
    match *ev {
        GameEvent::CreatureDied { card_id }
        | GameEvent::PermanentLeftBattlefield { card_id, .. }
        | GameEvent::PermanentExiled { card_id }
        | GameEvent::PermanentReturnedToHand { card_id, .. } => Some(card_id),
        _ => None,
    }
}

impl GameState {
    /// One candidate per (departed listener, event it saw before it left).
    /// Empty unless the batch holds both a looked-back event and a departure.
    pub(crate) fn departed_listener_candidates(&self, events: &[GameEvent]) -> Vec<TriggerCandidate> {
        let mut out: Vec<TriggerCandidate> = Vec::new();
        if !events.iter().any(looked_back_on)
            || !events.iter().any(|e| departure_of(e).is_some())
        {
            return out;
        }
        let mut seen: Vec<CardId> = Vec::new();
        for (dep, ev) in events.iter().enumerate() {
            let Some(listener) = departure_of(ev) else { continue };
            // First departure only; one still (or back) on the battlefield is
            // the walk's.
            if seen.contains(&listener) || self.battlefield_find(listener).is_some() {
                continue;
            }
            seen.push(listener);
            let Some(snap) = self.died_card_snapshots.get(&listener).or_else(|| self.leaves_bf_lki.get(&listener))
            else {
                continue;
            };
            let listens = |ta: &&crate::card::TriggeredAbility| {
                LOOKS_BACK.contains(&ta.event.kind)
                    && !ta.event.zone.command_zone_only()
                    && !ta.event.once_per_turn
                    && !ta.event.once_per_batch
                    && !matches!(
                        ta.event.scope,
                        EventScope::SelfSource | EventScope::FromYourGraveyard | EventScope::FromYourGraveyardAnyPlayer
                    )
            };
            if !snap.definition.triggered_abilities.iter().any(|ta| listens(&ta)) {
                continue;
            }
            // Entries before the listener's own (re-)entry in this batch
            // happened before it was there.
            let start = events[..dep]
                .iter()
                .rposition(|e| matches!(e, GameEvent::PermanentEntered { card_id } if *card_id == listener))
                .map_or(0, |i| i + 1);
            for ta in snap.definition.triggered_abilities.iter().filter(listens) {
                for ev in &events[start..dep] {
                    if !looked_back_on(ev)
                        || matches!(ev, GameEvent::PermanentEntered { card_id } if *card_id == listener)
                        || !events::event_matches_spec(self, ev, &ta.event, snap)
                    {
                        continue;
                    }
                    let subject = events::event_subject(ev, &ta.event.kind);
                    let event_amount = self.event_amount_for(ev);
                    if let Some(filter) = &ta.event.filter {
                        let ctx = EffectContext::for_intervening_filter(snap.controller, listener, subject, event_amount);
                        if !self.evaluate_predicate(filter, &ctx) {
                            continue;
                        }
                    }
                    out.push(TriggerCandidate {
                        source: listener,
                        effect: ta.effect.clone(),
                        controller: snap.controller,
                        filter: ta.event.filter.clone(),
                        subject,
                        event_amount,
                        triggered_by_etb: matches!(ev, GameEvent::PermanentEntered { .. }),
                        triggered_by_death: false,
                        triggered_by_attack: false,
                        triggered_by_land_entry: false,
                        triggered_by_face_up: false,
                        triggered_by_draw: false,
                        damaged_creature_controller: None,
                        from_mana_ability: false,
                        actor: events::event_actor(self, ev),
                    });
                }
            }
        }
        out
    }
}
