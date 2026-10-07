//! CR 603.10a — "leaves-the-battlefield abilities" look back in time. When
//! creatures die at the same time (a Wrath, one state-based sweep), each
//! dying creature's "whenever another creature [you control] dies" trigger
//! sees every other creature that died with it — Midnight Reaper and a Bear
//! under Wrath of God draw two. A noncreature listener destroyed in the same
//! batch looks back too (Bastion of Remembrance under Planar Cleansing). The
//! battlefield walk in `dispatch_triggers_for_events` can't find these
//! observers (they are in the graveyard by dispatch time), and the self-death
//! funnel only fires a creature's triggers for its own death, so this pass
//! supplies the rest from the death snapshots. Deaths a later resolution step
//! caused (`GameEvent::DeathStepEnded` between them) happened after it left.

use super::GameState;
use super::types::{GameEvent, TriggerCandidate};
use crate::card::CardId;
use crate::effect::{EventKind, EventScope};
use crate::game::effects::{EffectContext, events};

impl GameState {
    /// One candidate per (dying observer, other permanent that died in the
    /// same batch) its printed death trigger matches. Empty unless two or
    /// more permanents died in `events`.
    pub(crate) fn simultaneous_death_observer_candidates(
        &self,
        events: &[GameEvent],
        dies_suppressed: bool,
    ) -> Vec<TriggerCandidate> {
        let mut out: Vec<TriggerCandidate> = Vec::new();
        if dies_suppressed {
            return out;
        }
        // `PermanentDied` is synthesized for every death (a noncreature emits
        // no `CreatureDied`), so it names the noncreature observers.
        let mut died: Vec<CardId> = Vec::new();
        for ev in events {
            if let GameEvent::CreatureDied { card_id } | GameEvent::PermanentDied { card_id, .. } = ev
                && !died.contains(card_id)
                && !self.death_was_replaced(*card_id)
            {
                died.push(*card_id);
            }
        }
        if died.len() < 2 {
            return out;
        }
        // CR 608.2c — the resolution step each creature died in; a death in a
        // later step than the observer's happened after it left.
        let step_of = |id: CardId| -> Option<usize> {
            let mut step = 0usize;
            for ev in events {
                match ev {
                    GameEvent::DeathStepEnded => step += 1,
                    GameEvent::CreatureDied { card_id } if *card_id == id => return Some(step),
                    _ => {}
                }
            }
            None
        };
        let stepped = events.iter().any(|e| matches!(e, GameEvent::DeathStepEnded));
        for &observer in &died {
            // Only a creature that is gone: one still on the battlefield
            // (returned by a replacement) is found by the battlefield walk.
            if self.battlefield_find(observer).is_some() {
                continue;
            }
            let Some(snap) = self.died_card_snapshots.get(&observer) else { continue };
            let observer_step = if stepped { step_of(observer) } else { None };
            // An attachment whose host died went to the graveyard after it,
            // by a later sweep (CR 704.5m/n): it didn't die with the host.
            if snap.attached_to.is_some_and(|h| h != observer && died.contains(&h)) {
                continue;
            }
            for ta in &snap.definition.triggered_abilities {
                // Battlefield abilities only: a graveyard-functioning trigger
                // (Nether Traitor) wasn't in the graveyard when the others
                // died, and a command-zone one isn't a permanent's.
                if !matches!(
                    ta.event.kind,
                    EventKind::CreatureDied | EventKind::CreatureOrArtifactDied | EventKind::PermanentDied
                ) || ta.event.zone.command_zone_only()
                    || matches!(
                        ta.event.scope,
                        EventScope::SelfSource
                            | EventScope::EnchantedBySource
                            | EventScope::FromYourGraveyard
                            | EventScope::FromYourGraveyardAnyPlayer
                    )
                {
                    continue;
                }
                let fanout = events::event_kind_fans_out(&ta.event.kind)
                    && !ta.event.once_per_turn
                    && !ta.event.once_per_batch;
                for ev in events {
                    let (GameEvent::CreatureDied { card_id } | GameEvent::PermanentDied { card_id, .. }) = ev else {
                        continue;
                    };
                    // Its own death is the self-death funnel's.
                    if *card_id == observer || !died.contains(card_id) {
                        continue;
                    }
                    if let (Some(o), Some(d)) = (observer_step, step_of(*card_id))
                        && d > o
                    {
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

/// CR 608.2c / 603.10a — after a resolution step that killed something,
/// with another step still to come, mark the boundary so the death
/// look-back reads the deaths on either side as sequential.
pub(crate) fn mark_death_step(events: &mut Vec<GameEvent>, before: usize) {
    if events[before.min(events.len())..]
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. } | GameEvent::PermanentDied { .. }))
    {
        events.push(GameEvent::DeathStepEnded);
    }
}
