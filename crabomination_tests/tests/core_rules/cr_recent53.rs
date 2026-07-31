//! CR conformance for this run's engine work:
//! - CR 113.3 — any-player activated abilities (Damping Engine).
//! - CR 120.4a — excess damage redirected before the damage event.
//! - CR 208.3a — a P/T-modifying effect on a noncreature permanent is created
//!   anyway and applies once that permanent becomes a creature.
//! - CR 611.2c — "for as long as this remains tapped" continuous effects.

use crabomination::card::CardType;
use crabomination::catalog;
use crabomination::effect::{Duration, Effect, Selector, Value};
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;

fn activate(g: &mut GameState, id: CardId, idx: usize) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

// ── CR 113.3 — abilities ────────────────────────────────────────────────────

/// CR 113.3 — an ability flagged "any player may activate this" is usable by a
/// player who doesn't control the source; ordinary abilities are not.
#[test]
fn cr_113_3_any_player_may_activate_a_flagged_ability() {
    let mut g = two_player_game();
    let engine = g.add_card_to_battlefield(0, catalog::damping_engine());
    let jar = g.add_card_to_battlefield(0, catalog::memory_jar());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: jar,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "an ordinary ability stays with its controller"
    );
    activate(&mut g, engine, 0);
    assert!(g.players[1].statics_ignored_this_turn.contains(&engine));
}

// ── CR 120.4a — excess damage ───────────────────────────────────────────────

/// CR 120.4a — the redirect happens before the damage is dealt, so the creature
/// is marked for exactly lethal and only the rest lands on its controller.
#[test]
fn cr_120_4a_excess_is_split_off_before_the_damage_event() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    let ctx = crabomination::game::effects::EffectContext::for_spell(
        0,
        Some(Target::Permanent(bear)),
        0,
        0,
    );
    g.resolve_effect(
        &Effect::DealDamageExcessToController {
            to: Selector::Target(0),
            amount: Value::Const(6),
        },
        &ctx,
    )
    .expect("resolve");
    assert_eq!(g.players[1].life, life - 4, "6 minus the lethal 2 spilled over");
    assert!(g.battlefield_find(bear).is_none());
}

/// CR 120.4a — a deathtouch source makes everything past 1 damage excess.
#[test]
fn cr_120_4a_deathtouch_makes_one_damage_lethal() {
    let mut g = two_player_game();
    let rats = g.add_card_to_battlefield(0, catalog::typhoid_rats());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        rats,
        0,
        Some(Target::Permanent(bear)),
    );
    g.resolve_effect(
        &Effect::DealDamageExcessToController {
            to: Selector::Target(0),
            amount: Value::Const(6),
        },
        &ctx,
    )
    .expect("resolve");
    assert_eq!(g.players[1].life, life - 5, "only 1 was needed on the body");
}

// ── CR 208.3a — P/T effects on noncreature permanents ───────────────────────

/// CR 208.3a — a +N/+N effect created against a noncreature permanent exists
/// and starts applying the moment that permanent becomes a creature.
#[test]
fn cr_208_3a_pump_on_a_noncreature_applies_once_it_animates() {
    let mut g = two_player_game();
    let jar = g.add_card_to_battlefield(0, catalog::memory_jar());
    let ctx = crabomination::game::effects::EffectContext::for_spell(
        0,
        Some(Target::Permanent(jar)),
        0,
        0,
    );
    g.resolve_effect(
        &Effect::PumpPT {
            what: Selector::Target(0),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        },
        &ctx,
    )
    .expect("pump");
    assert!(!g.computed_permanent(jar).unwrap().card_types.contains(&CardType::Creature));
    g.resolve_effect(
        &Effect::BecomeCreature {
            what: Selector::Target(0),
            power: Value::Const(3),
            toughness: Value::Const(3),
            creature_types: vec![],
            keywords: vec![],
            duration: Duration::EndOfTurn,
        },
        &ctx,
    )
    .expect("animate");
    let cp = g.computed_permanent(jar).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "the banked pump is live now");
}

// ── CR 611.2c — "for as long as" durations ──────────────────────────────────

/// CR 611.2c — the affected set is locked in when the effect starts, and the
/// effect ends when its stated condition (the source staying tapped) fails.
#[test]
fn cr_611_2c_while_source_tapped_locks_its_set_and_expires_on_untap() {
    let mut g = two_player_game();
    let weaponry = g.add_card_to_battlefield(0, catalog::thran_weaponry());
    let early = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, weaponry, 0);
    let late = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(early).unwrap().power, 4);
    assert_eq!(g.computed_permanent(late).unwrap().power, 2, "joined after the lock-in");
    g.battlefield_find_mut(weaponry).unwrap().tapped = false;
    g.check_state_based_actions();
    assert_eq!(g.computed_permanent(early).unwrap().power, 2);
}
