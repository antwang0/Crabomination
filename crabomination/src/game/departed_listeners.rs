//! CR 603.2 — an ability triggers the moment its event happens. Triggers are
//! collected after a resolution, from the permanents still on the
//! battlefield, so a listener the same resolution removed never saw the
//! events that happened while it was still there: Soul Warden under a Martial
//! Coup (X ≥ 5) gained nothing for the five Soldiers that entered before the
//! sweep. This pass supplies those triggers from the departed listener's
//! snapshot, for the entries the batch records between its own entry (if
//! any) and its departure.
//!
//! Scope: the kinds [`looks_back`] names — ones that only the dispatch walk
//! fires, so a departed listener can't also have fired from a direct hook —
//! and only departures with an in-order event — the synthesized
//! `PermanentDied` is appended after the batch, so it can't date a departure.
//! A once-each-turn / "one or more" listener fires at most once, and a
//! once-each-turn one only if the turn's budget (`triggered_once_per_turn_used`)
//! is unspent; the caller records what it spends.

use super::GameState;
use super::types::{GameEvent, TriggerCandidate};
use crate::card::CardId;
use crate::effect::{EventKind, EventScope};
use crate::game::effects::{EffectContext, events};

/// The listener kinds this pass serves: "whenever [a permanent] enters",
/// "whenever a creature dies" (one bounced or exiled; one that died looks back
/// in `simultaneous_deaths`), "whenever you gain / an opponent loses life",
/// "whenever a player draws / discards a card" and "whenever a counter is put
/// on". Printed listeners of
/// each fire only from the walk (`fire_life_gained_watchers` serves delayed
/// ones).
fn looks_back(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::EntersBattlefield
            | EventKind::CreatureDied
            | EventKind::LifeGained
            | EventKind::LifeLost
            | EventKind::CardDrawn
            | EventKind::CardDiscarded
            | EventKind::CounterAdded(_)
            | EventKind::AnyCounterAdded
    )
}

/// Whether `ev` is an event a [`looks_back`] listener can see.
fn looked_back_on(ev: &GameEvent) -> bool {
    matches!(
        ev,
        GameEvent::PermanentEntered { .. }
            | GameEvent::CreatureDied { .. }
            | GameEvent::LifeGained { .. }
            | GameEvent::LifeLost { .. }
            | GameEvent::CardDrawn { .. }
            | GameEvent::CardDiscarded { .. }
            | GameEvent::CounterAdded { .. }
            | GameEvent::KeywordCounterAdded { .. }
    )
}

/// The card an in-order departure event moves off the battlefield. A
/// noncreature put into a graveyard reports only `CardPutIntoGraveyard`,
/// which a discard or a resolving spell reports too: it is a departure only
/// for a card with a death snapshot this dispatch.
fn departure_of(g: &GameState, ev: &GameEvent) -> Option<CardId> {
    match *ev {
        GameEvent::CreatureDied { card_id }
        | GameEvent::PermanentLeftBattlefield { card_id, .. }
        | GameEvent::PermanentExiled { card_id }
        | GameEvent::PermanentReturnedToHand { card_id, .. } => Some(card_id),
        GameEvent::CardPutIntoGraveyard { card_id, .. } if g.died_card_snapshots.get(&card_id).is_some() => {
            Some(card_id)
        }
        _ => None,
    }
}

impl GameState {
    /// One candidate per (departed listener, event it saw before it left).
    /// Empty unless the batch holds both a looked-back event and a departure.
    /// Pushes the once-each-turn keys it fires onto `once_spent`.
    pub(crate) fn departed_listener_candidates(
        &self,
        events: &[GameEvent],
        once_spent: &mut Vec<(CardId, usize)>,
    ) -> Vec<TriggerCandidate> {
        let mut out: Vec<TriggerCandidate> = Vec::new();
        if !events.iter().any(looked_back_on)
            || !events.iter().any(|e| departure_of(self, e).is_some())
        {
            return out;
        }
        let mut seen: Vec<CardId> = Vec::new();
        for (dep, ev) in events.iter().enumerate() {
            let Some(listener) = departure_of(self, ev) else { continue };
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
            let died = !matches!(
                ev,
                GameEvent::PermanentLeftBattlefield { .. }
                    | GameEvent::PermanentExiled { .. }
                    | GameEvent::PermanentReturnedToHand { .. }
            );
            let listens = |ta: &&crate::card::TriggeredAbility| {
                looks_back(&ta.event.kind)
                    && (!died || ta.event.kind != EventKind::CreatureDied)
                    && !ta.event.zone.command_zone_only()
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
            for (idx, ta) in snap.definition.triggered_abilities.iter().enumerate() {
                if !listens(&ta) {
                    continue;
                }
                // CR 603.3d — "only once each turn": not if the turn's one fire
                // is spent; "one or more" / once each turn: one fire here.
                let once_key = (listener, idx);
                if ta.event.once_per_turn && self.triggered_once_per_turn_used.contains(&once_key) {
                    continue;
                }
                let single = ta.event.once_per_turn || ta.event.once_per_batch;
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
                        effect: self.trigger_effect_for(&ta.effect, &ta.event, snap, ev, events),
                        controller: snap.controller,
                        filter: ta.event.filter.clone(),
                        subject,
                        event_amount,
                        triggered_by_etb: matches!(ev, GameEvent::PermanentEntered { .. }),
                        triggered_by_death: matches!(ev, GameEvent::CreatureDied { .. }),
                        triggered_by_attack: false,
                        triggered_by_land_entry: false,
                        triggered_by_face_up: false,
                        triggered_by_draw: matches!(ev, GameEvent::CardDrawn { .. }),
                        damaged_creature_controller: None,
                        from_mana_ability: false,
                        actor: events::event_actor(self, ev),
                    });
                    if single {
                        if ta.event.once_per_turn {
                            once_spent.push(once_key);
                        }
                        break;
                    }
                }
            }
        }
        out
    }
}
