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
        (EventKind::CreatureSacrificed, GameEvent::CreatureSacrificed { .. }) => true,
        (EventKind::PermanentSacrificed, GameEvent::PermanentSacrificed { .. }) => true,
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
        (EventKind::AttacksAndIsntBlocked, GameEvent::AttackerWentUnblocked { .. }) => true,
        // Enrage (CR 702.130): keyed on the damaged permanent, so we only
        // match `DamageDealt` events that hit a card (not a player).
        (EventKind::DealtDamage, GameEvent::DamageDealt { to_card: Some(_), .. }) => true,
        (EventKind::LifeGained, GameEvent::LifeGained { .. }) => true,
        (EventKind::LifeLost, GameEvent::LifeLost { .. }) => true,
        (EventKind::StepBegins(s), GameEvent::StepChanged(got)) => s == got,
        (EventKind::TurnBegins, GameEvent::TurnStarted { .. }) => true,
        (EventKind::CounterAdded(k), GameEvent::CounterAdded { counter_type, .. }) => counter_type == k,
        (EventKind::AbilityActivated, GameEvent::AbilityActivated { .. }) => true,
        (EventKind::CardLeftGraveyard, GameEvent::CardLeftGraveyard { .. }) => true,
        (EventKind::BecameTarget, GameEvent::BecameTarget { .. }) => true,
        (EventKind::CardCycled, GameEvent::CardCycled { .. }) => true,
        (EventKind::BecomesUntapped, GameEvent::PermanentUntapped { .. }) => true,
        (EventKind::Tapped, GameEvent::PermanentTapped { .. }) => true,
        (EventKind::Explored, GameEvent::Explored { .. }) => true,
        (EventKind::BecameMonstrous, GameEvent::BecameMonstrous { .. }) => true,
        (EventKind::EnergyGained, GameEvent::EnergyGained { .. }) => true,
        (EventKind::WonCoinFlip, GameEvent::CoinFlipWon { .. }) => true,
        _ => false,
    };
    if !kind_ok {
        return false;
    }

    // BecameTarget has an implicit "the trigger source is the targeted
    // permanent" check — the trigger fires for the targeted permanent
    // by design (CR 603.x — "this permanent becomes the target …"
    // triggers always reference the source as the target). The scope
    // check below then refines on the caster.
    if let (EventKind::BecameTarget, GameEvent::BecameTarget { target, .. }) =
        (&spec.kind, event)
        && *target != source.id
        // The "any permanent you control" scope intentionally fires for
        // subjects other than the source, so skip the implicit check there.
        && spec.scope != EventScope::YourPermanentTargetedByOpponent
    {
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
        ) || matches!(
            event,
            GameEvent::CreatureSacrificed { card_id, .. } if *card_id == source.id
        ) || matches!(
            event,
            GameEvent::PermanentSacrificed { card_id, .. } if *card_id == source.id
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
            || matches!(event, GameEvent::AttackerWentUnblocked { attacker }
                if matches!(spec.kind, EventKind::AttacksAndIsntBlocked) && *attacker == source.id)
        ) || matches!(
            event,
            GameEvent::CounterAdded { card_id, .. } if *card_id == source.id
        ) || matches!(
            // Enrage: "Whenever this creature is dealt damage." Source
            // must equal the damaged card.
            event,
            GameEvent::DamageDealt { to_card: Some(cid), .. } if *cid == source.id
        ) || matches!(
            // CR 702.29c — "When you cycle this card" triggers fire
            // with the cycled card as the trigger source. The card is
            // in the graveyard by the time triggers dispatch (per
            // CR 702.29c, "These abilities trigger from whatever zone
            // the card winds up in after it's cycled"), so SelfSource
            // here intentionally identifies the source by id; the
            // dispatcher will walk hand → graveyard to find the
            // card's printed abilities.
            event,
            GameEvent::CardCycled { card_id, .. } if *card_id == source.id
        ) || matches!(
            // CR 702.108 Inspired — "Whenever this becomes untapped."
            event,
            GameEvent::PermanentUntapped { card_id } if *card_id == source.id
        ) || matches!(
            // CR 701.40 — "Whenever this creature explores." Source must
            // equal the exploring permanent.
            event,
            GameEvent::Explored { card_id, .. } if *card_id == source.id
        ) || matches!(
            // CR 701.31 — "When this becomes monstrous." Source must equal
            // the permanent that became monstrous.
            event,
            GameEvent::BecameMonstrous { card_id } if *card_id == source.id
        ) || matches!(
            // "When this becomes the target of a spell or ability." The
            // implicit target==source.id check above already constrained it;
            // accept the event here so the SelfSource trigger fires (Goldspan
            // Dragon's Treasure, Phantasmal Image's sacrifice rider).
            event,
            GameEvent::BecameTarget { target, .. } if *target == source.id
        ),
        // CR 810.8 — in Two-Headed Giant, "you" effects fan out to
        // teammates: a "whenever you gain life" trigger on team A
        // fires regardless of which team-A member's life event
        // produced it. `same_team` collapses to `a == b` for solo
        // teams (singleton FFA / 1v1 / Commander) so 1v1 behavior
        // is unchanged. Symmetric treatment for OpponentControl —
        // teammate actions aren't "opponent" actions, so the
        // implicit "not me" widens to "not on my team."
        EventScope::YourControl => event_actor(state, event)
            .is_some_and(|p| state.same_team(p, source.controller)),
        EventScope::OpponentControl => event_actor(state, event)
            .is_some_and(|p| !state.same_team(p, source.controller)),
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
                })
                // For token deaths the SBA "ceases to exist" rule
                // (CR 111.7c) removes the token from every zone
                // in the same SBA sweep as the death event emission, so by
                // the time the trigger dispatcher walks this lookup the
                // token isn't anywhere. The `died_card_snapshots` cache
                // is populated at die-time in `check_state_based_actions`
                // so AnotherOfYours-scope triggers (Witherbloom Pestmaster,
                // Felisa, Fang of Silverquill) still fire on token death.
                .or_else(|| state.died_card_snapshots.get(&target).map(|c| c.controller));
            subject_controller == Some(source.controller)
        }
        EventScope::FromYourGraveyard => event_actor(state, event)
            .is_some_and(|p| p == source.owner),
        EventScope::YourPermanentTargetedByOpponent => {
            // The targeted permanent must be controlled by the trigger's
            // controller, and the caster must be an opponent. Battle Mammoth.
            if let GameEvent::BecameTarget { target, caster } = event {
                let target_ctrl = state.battlefield_find(*target).map(|c| c.controller);
                target_ctrl == Some(source.controller)
                    && !state.same_team(*caster, source.controller)
            } else {
                false
            }
        }
        // Dispatched manually in `declare_attackers` (the defending player's
        // listeners get the attacker's controller bound into the target
        // slot), so the unified dispatcher must not also fire it.
        EventScope::ControllerAttackedByOpponent => false,
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
        // Token deaths: zone walk fails because the token ceases to exist
        // in the same SBA pass. Fall back to the snapshot cache populated
        // at die-time in `check_state_based_actions`.
        .or_else(|| state.died_card_snapshots.get(&cid).map(|c| c.controller))
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
        | GameEvent::CardCycled { player, .. }
        | GameEvent::EnergyGained { player, .. }
        | GameEvent::CoinFlipWon { player }
        | GameEvent::TurnStarted { player, .. } => Some(*player),
        // For BecameTarget the "actor" is the caster of the spell or
        // ability that picked the target. This drives YourControl /
        // OpponentControl scope checks (Tenured Concocter wants
        // OpponentControl → caster is opponent of source's controller).
        GameEvent::BecameTarget { caster, .. } => Some(*caster),
        // Sacrifice is performed by the controller of the sacrificed
        // permanent ("a player sacrifices a creature"). This routes the
        // YourControl / OpponentControl scope check correctly: a
        // Mortician Beetle-style "Whenever a player sacrifices a creature"
        // would use AnyPlayer scope; a "whenever an opponent sacrifices"
        // would use OpponentControl.
        GameEvent::CreatureSacrificed { who, .. } => Some(*who),
        GameEvent::PermanentSacrificed { who, .. } => Some(*who),
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
///
/// `kind` is the matching trigger's `EventKind`. For `BlockerDeclared`
/// the underlying event is shared between two trigger kinds — `Blocks`
/// fires on the blocker side (subject = blocker), while `BecomesBlocked`
/// fires on the attacker side (subject = attacker). The kind disambiguates.
pub(crate) fn event_subject(event: &GameEvent, kind: &EventKind) -> Option<EntityRef> {
    match event {
        GameEvent::SpellCast { card_id, .. } => Some(EntityRef::Card(*card_id)),
        GameEvent::PermanentEntered { card_id } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::CreatureDied { card_id } => Some(EntityRef::Card(*card_id)),
        GameEvent::CreatureSacrificed { card_id, .. } => Some(EntityRef::Card(*card_id)),
        GameEvent::PermanentSacrificed { card_id, .. } => Some(EntityRef::Card(*card_id)),
        GameEvent::AttackerDeclared(card_id) => Some(EntityRef::Permanent(*card_id)),
        GameEvent::BlockerDeclared { blocker, attacker } => Some(EntityRef::Permanent(
            if matches!(kind, EventKind::BecomesBlocked) { *attacker } else { *blocker },
        )),
        GameEvent::AttackerWentUnblocked { attacker } => Some(EntityRef::Permanent(*attacker)),
        GameEvent::LandPlayed { card_id, .. } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::PermanentTapped { card_id } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::PermanentUntapped { card_id } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::Explored { card_id, .. } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::BecameMonstrous { card_id } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::TokenCreated { card_id } => Some(EntityRef::Permanent(*card_id)),
        // Enrage: the subject is the damaged permanent, so trigger bodies
        // referencing `Selector::TriggerSource` (and the implicit
        // SelfSource scope) bind to the creature that took the damage.
        GameEvent::DamageDealt { to_card: Some(card_id), .. } => Some(EntityRef::Permanent(*card_id)),
        // CardDrawn / CardDiscarded carry a card_id — bind
        // `Selector::TriggerSource` to the *card* (not the player) so
        // filters like `Predicate::EntityMatches { what: TriggerSource,
        // filter: IS }` can introspect the drawn/discarded card.
        // Lorehold the Historian's miracle grant relies on this.
        GameEvent::CardDrawn { card_id, .. } => Some(EntityRef::Card(*card_id)),
        GameEvent::CardDiscarded { card_id, .. } => Some(EntityRef::Card(*card_id)),
        GameEvent::CardMilled { player, .. }
        | GameEvent::LifeGained { player, .. }
        | GameEvent::LifeLost { player, .. }
        | GameEvent::ManaAdded { player, .. }
        | GameEvent::EnergyGained { player, .. }
        | GameEvent::CoinFlipWon { player }
        | GameEvent::ColorlessManaAdded { player } => Some(EntityRef::Player(*player)),
        GameEvent::CardLeftGraveyard { card_id, .. } => Some(EntityRef::Card(*card_id)),
        // The "subject" of a BecameTarget event is the permanent that
        // became the target — what `Selector::TriggerSource` should bind
        // to for any filter predicate.
        GameEvent::BecameTarget { target, .. } => Some(EntityRef::Permanent(*target)),
        // CardCycled binds TriggerSource to the cycled card (which is
        // now in the controller's graveyard). Used by future "Whenever
        // a player cycles a card" payoffs that introspect the cycled
        // card's printed attributes.
        GameEvent::CardCycled { card_id, .. } => Some(EntityRef::Card(*card_id)),
        _ => None,
    }
}

/// Like `event_matches_spec`, but for a player-owned emblem (CR 114) that
/// has no backing permanent. Handles the player-keyed event kinds emblem
/// triggers actually use; scope is resolved against `controller` (the
/// emblem's owner). Step-keyed kinds are dispatched separately in
/// `fire_step_triggers`, so they're rejected here.
pub(crate) fn emblem_event_matches(
    state: &GameState,
    event: &GameEvent,
    spec: &EventSpec,
    controller: usize,
) -> bool {
    let kind_ok = matches!(
        (&spec.kind, event),
        (EventKind::LifeGained, GameEvent::LifeGained { .. })
            | (EventKind::LifeLost, GameEvent::LifeLost { .. })
            | (EventKind::CardDrawn, GameEvent::CardDrawn { .. })
            | (EventKind::CardDiscarded, GameEvent::CardDiscarded { .. })
            | (EventKind::SpellCast, GameEvent::SpellCast { .. })
            | (EventKind::LandPlayed, GameEvent::LandPlayed { .. })
            | (EventKind::CreatureDied, GameEvent::CreatureDied { .. })
            | (EventKind::Attacks, GameEvent::AttackerDeclared(_))
    );
    if !kind_ok {
        return false;
    }
    match spec.scope {
        EventScope::YourControl | EventScope::SelfSource => {
            event_actor(state, event).is_some_and(|p| state.same_team(p, controller))
        }
        EventScope::OpponentControl => {
            event_actor(state, event).is_some_and(|p| !state.same_team(p, controller))
        }
        EventScope::AnyPlayer | EventScope::ActivePlayer | EventScope::AnotherOfYours => true,
        EventScope::FromYourGraveyard
        | EventScope::YourPermanentTargetedByOpponent
        | EventScope::ControllerAttackedByOpponent => false,
    }
}

fn event_card(event: &GameEvent) -> Option<CardId> {
    match event {
        GameEvent::PermanentEntered { card_id }
        | GameEvent::PermanentExiled { card_id }
        | GameEvent::CreatureDied { card_id }
        | GameEvent::CreatureSacrificed { card_id, .. }
        | GameEvent::CardCycled { card_id, .. }
        | GameEvent::PermanentSacrificed { card_id, .. }
        | GameEvent::PermanentTapped { card_id }
        | GameEvent::PermanentUntapped { card_id }
        | GameEvent::Explored { card_id, .. }
        | GameEvent::BecameMonstrous { card_id }
        | GameEvent::TokenCreated { card_id }
        | GameEvent::CounterAdded { card_id, .. }
        | GameEvent::AttackerDeclared(card_id) => Some(*card_id),
        GameEvent::BlockerDeclared { blocker, .. } => Some(*blocker),
        GameEvent::AttackerWentUnblocked { attacker } => Some(*attacker),
        // BecameTarget's card is the targeted permanent (used by
        // AnotherOfYours scope checks if any card uses it).
        GameEvent::BecameTarget { target, .. } => Some(*target),
        // Enrage: the damaged permanent (drives AnotherOfYours /
        // YourControl scope checks for "a creature you control is dealt
        // damage" payoffs).
        GameEvent::DamageDealt { to_card: Some(card_id), .. } => Some(*card_id),
        _ => None,
    }
}
