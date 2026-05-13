//! Helpers for matching game `Event`s against trigger specs and extracting
//! the actor / subject of an event.

use super::EntityRef;
use crate::card::{CardId, CardInstance};
use crate::effect::{EventKind, EventScope, EventSpec};
use crate::game::{GameEvent, GameState};

/// Returns true if `event` matches the `EventSpec` on `source` (a permanent
/// on the battlefield). Used by `fire_triggers_for_event` to decide whether a
/// triggered ability should be pushed onto the stack.
pub(crate) fn event_matches_spec(
    state: &GameState,
    event: &GameEvent,
    spec: &EventSpec,
    source: &CardInstance,
) -> bool {
    let kind_ok = match (&spec.kind, event) {
        (EventKind::EntersBattlefield, GameEvent::PermanentEntered { .. }) => true,
        (EventKind::CreatureDied, GameEvent::CreatureDied { .. }) => true,
        (EventKind::PermanentLeavesBattlefield, GameEvent::CreatureDied { .. }) => true,
        (EventKind::CardDrawn, GameEvent::CardDrawn { .. }) => true,
        (EventKind::CardDiscarded, GameEvent::CardDiscarded { .. }) => true,
        (EventKind::LandPlayed, GameEvent::LandPlayed { .. }) => true,
        (EventKind::SpellCast, GameEvent::SpellCast { .. }) => true,
        (EventKind::Attacks, GameEvent::AttackerDeclared(_)) => true,
        // `Blocks` fires from the blocker's side ("whenever this creature
        // blocks"). `BecomesBlocked` fires from the attacker's side
        // ("whenever this creature becomes blocked"). Both come off the
        // same BlockerDeclared event but are filtered by `event_card`
        // below — see `event_card` matching on `Blocks` vs
        // `BecomesBlocked`.
        (EventKind::Blocks, GameEvent::BlockerDeclared { .. }) => true,
        (EventKind::BecomesBlocked, GameEvent::BlockerDeclared { .. }) => true,
        (EventKind::LifeGained, GameEvent::LifeGained { .. }) => true,
        (EventKind::LifeLost, GameEvent::LifeLost { .. }) => true,
        (EventKind::StepBegins(s), GameEvent::StepChanged(got)) => s == got,
        (EventKind::TurnBegins, GameEvent::TurnStarted { .. }) => true,
        (EventKind::CounterAdded(k), GameEvent::CounterAdded { counter_type, .. }) => counter_type == k,
        (EventKind::AbilityActivated, GameEvent::AbilityActivated { .. }) => true,
        (EventKind::CardLeftGraveyard, GameEvent::CardLeftGraveyard { .. }) => true,
        _ => false,
    };
    if !kind_ok {
        return false;
    }

    let scope_ok = match spec.scope {
        EventScope::SelfSource => matches!(
            event,
            GameEvent::PermanentEntered { card_id } if *card_id == source.id
        ) || matches!(
            event,
            GameEvent::AttackerDeclared(id) if *id == source.id
        ) || matches!(
            event,
            GameEvent::CreatureDied { card_id } if *card_id == source.id
        ) || (
            // `Blocks` vs `BecomesBlocked` look at different sides of
            // the same BlockerDeclared event:
            //   - `EventKind::Blocks` → the blocker side ("when this
            //     creature blocks"). Source must equal `blocker`.
            //   - `EventKind::BecomesBlocked` → the attacker side
            //     ("when this becomes blocked"). Source must equal
            //     `attacker`.
            matches!(event, GameEvent::BlockerDeclared { blocker, .. }
                if matches!(spec.kind, EventKind::Blocks) && *blocker == source.id)
            || matches!(event, GameEvent::BlockerDeclared { attacker, .. }
                if matches!(spec.kind, EventKind::BecomesBlocked) && *attacker == source.id)
        ) || matches!(
            event,
            GameEvent::CounterAdded { card_id, .. } if *card_id == source.id
        ),
        EventScope::YourControl => event_actor(state, event)
            .is_some_and(|p| p == source.controller),
        EventScope::OpponentControl => event_actor(state, event)
            .is_some_and(|p| p != source.controller),
        EventScope::AnyPlayer | EventScope::ActivePlayer => true,
        EventScope::AnotherOfYours => {
            // ETB / die / attack triggers for "another creature/permanent
            // you control". Two checks:
            // (1) the subject card isn't the trigger source itself; and
            // (2) the subject card's controller (or graveyard-owner for
            //     a CreatureDied subject) matches the trigger source's
            //     controller. Without (2) the scope would silently fire
            //     for opponent-side subjects too — Felisa would mint an
            //     Inkling on an opp's counter-bearing creature dying,
            //     Sparring Regimen would pump an opp attacker, etc.
            let Some(target) = event_card(event) else { return false; };
            if target == source.id { return false; }
            let subject_controller = state
                .battlefield_find(target)
                .map(|c| c.controller)
                .or_else(|| {
                    state.players.iter().position(|p|
                        p.graveyard.iter().any(|c| c.id == target))
                });
            subject_controller == Some(source.controller)
        }
        EventScope::FromYourGraveyard => event_actor(state, event)
            .is_some_and(|p| p == source.owner),
    };

    if !scope_ok {
        return false;
    }

    // Filter predicate evaluation is deferred to when the trigger actually
    // resolves; at this stage we just ensure the shape matches.
    true
}

/// The "actor" of an event for `EventScope::YourControl` /
/// `OpponentControl` checks: the player whose action / permanent the
/// event hangs off. For player-keyed events (CardDrawn, LifeGained, etc.)
/// this is the event's `player` field; for permanent-keyed events
/// (PermanentEntered, AttackerDeclared, CreatureDied) this is the
/// permanent's controller, looked up on the battlefield. CreatureDied
/// fires after the card has left the battlefield, so we fall back to the
/// graveyard owner — close enough for "your creature died" triggers.
pub(crate) fn event_actor(state: &GameState, event: &GameEvent) -> Option<usize> {
    if let Some(p) = event_player(event) {
        return Some(p);
    }
    let cid = event_card(event)?;
    if let Some(c) = state.battlefield_find(cid) {
        return Some(c.controller);
    }
    state
        .players
        .iter()
        .position(|p| p.graveyard.iter().any(|c| c.id == cid))
}

fn event_player(event: &GameEvent) -> Option<usize> {
    match event {
        GameEvent::CardDrawn { player, .. }
        | GameEvent::CardDiscarded { player, .. }
        | GameEvent::LandPlayed { player, .. }
        | GameEvent::SpellCast { player, .. }
        | GameEvent::LifeGained { player, .. }
        | GameEvent::LifeLost { player, .. }
        | GameEvent::PoisonAdded { player, .. }
        | GameEvent::CardMilled { player, .. }
        | GameEvent::ManaAdded { player, .. }
        | GameEvent::ColorlessManaAdded { player }
        | GameEvent::CardLeftGraveyard { player, .. }
        | GameEvent::TurnStarted { player, .. } => Some(*player),
        _ => None,
    }
}

/// Extract the "subject" of an event as an `EntityRef` — the entity the
/// trigger's filter predicate should treat as `Selector::TriggerSource`.
/// For card-subject events (cast spell, ETB permanent, attacker, etc.)
/// this is the card; for player-subject events (life gain/loss, draw,
/// discard) it's the player. Used by `dispatch_triggers_for_events` so
/// filters like `Predicate::ValueAtLeast(ManaValueOf(TriggerSource), 5)`
/// can pin-point the cast spell on the stack.
pub(crate) fn event_subject(event: &GameEvent) -> Option<EntityRef> {
    match event {
        GameEvent::SpellCast { card_id, .. } => Some(EntityRef::Card(*card_id)),
        GameEvent::PermanentEntered { card_id } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::CreatureDied { card_id } => Some(EntityRef::Card(*card_id)),
        GameEvent::AttackerDeclared(card_id) => Some(EntityRef::Permanent(*card_id)),
        GameEvent::BlockerDeclared { blocker, .. } => Some(EntityRef::Permanent(*blocker)),
        GameEvent::LandPlayed { card_id, .. } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::PermanentTapped { card_id } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::PermanentUntapped { card_id } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::TokenCreated { card_id } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::CardDrawn { player, .. }
        | GameEvent::CardDiscarded { player, .. }
        | GameEvent::CardMilled { player, .. }
        | GameEvent::LifeGained { player, .. }
        | GameEvent::LifeLost { player, .. }
        | GameEvent::ManaAdded { player, .. }
        | GameEvent::ColorlessManaAdded { player } => Some(EntityRef::Player(*player)),
        GameEvent::CardLeftGraveyard { card_id, .. } => Some(EntityRef::Card(*card_id)),
        _ => None,
    }
}

fn event_card(event: &GameEvent) -> Option<CardId> {
    match event {
        GameEvent::PermanentEntered { card_id }
        | GameEvent::PermanentExiled { card_id }
        | GameEvent::CreatureDied { card_id }
        | GameEvent::PermanentTapped { card_id }
        | GameEvent::PermanentUntapped { card_id }
        | GameEvent::TokenCreated { card_id }
        | GameEvent::CounterAdded { card_id, .. }
        | GameEvent::AttackerDeclared(card_id) => Some(*card_id),
        GameEvent::BlockerDeclared { blocker, .. } => Some(*blocker),
        _ => None,
    }
}
