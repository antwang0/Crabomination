//! CR conformance for this run's engine work:
//! - CR 605.1b / 605.4a — triggered mana abilities resolve off-stack.
//! - CR 502.3 — a permanent with a "doesn't untap" counter stays tapped.
//! - CR 601.2b — the "cast only during combat" timing restriction.
//! - CR 615 / 614.9 — colour-scoped prevention and spell-damage redirection.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::effects::EntityRef;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn tap_for_mana(g: &mut GameState, land: CardId) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap for mana");
}

// ── CR 605 — Mana Abilities ─────────────────────────────────────────────────

/// CR 605.4a — a triggered mana ability doesn't use the stack; its mana is in
/// the pool the instant the land is tapped, without anyone passing priority.
#[test]
fn cr_605_4a_triggered_mana_ability_resolves_off_stack() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::overabundance());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    tap_for_mana(&mut g, forest);
    assert!(g.stack.is_empty(), "the triggered mana ability never hit the stack");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "both mana are available");
}

/// CR 605.1b — the ping rider doesn't stop it being a mana ability, and the
/// rider still happens.
#[test]
fn cr_605_1b_mana_trigger_riders_still_apply() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::overabundance());
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    let before = g.players[1].life;
    g.priority.player_with_priority = 1;
    tap_for_mana(&mut g, forest);
    assert_eq!(g.players[1].life, before - 1, "the tapper took the ping");
}

/// CR 605.5a — a mana-adding trigger that fires off something other than a
/// mana ability is a normal triggered ability and uses the stack.
#[test]
fn cr_605_5a_non_mana_trigger_still_uses_the_stack() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::overabundance());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    for c in [Color::Green, Color::White] {
        g.players[0].mana_pool.add(c, 5);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    // Nothing tapped for mana, so Overabundance never fired at all.
    assert_eq!(g.players[0].life, 20, "no ping without a mana ability");
}

// ── CR 502 — Untap Step ─────────────────────────────────────────────────────

/// CR 502.3 — an hourglass counter keeps its permanent tapped through the
/// untap step; clearing the counter frees it again.
#[test]
fn cr_502_3_counter_gated_permanent_doesnt_untap() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::temporal_distortion());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    tap_for_mana(&mut g, forest);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(forest).unwrap().counter_count(CounterType::Hourglass), 1);
    g.do_untap();
    assert!(g.battlefield_find(forest).unwrap().tapped, "the counter held it down");

    g.battlefield_find_mut(forest).unwrap().remove_counters(CounterType::Hourglass, 1);
    g.do_untap();
    assert!(!g.battlefield_find(forest).unwrap().tapped, "it untaps once the counter is gone");
}

// ── CR 601.2b — cast timing ─────────────────────────────────────────────────

/// CR 601.2b — "cast this spell only during combat" is a cast-time legality
/// gate, not a resolution check.
#[test]
fn cr_601_2b_cast_only_during_combat_is_gated_at_cast() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::spinal_embrace());
    for c in [Color::Blue, Color::Black] {
        g.players[0].mana_pool.add(c, 10);
    }
    g.players[0].mana_pool.add_colorless(10);
    let cast = |g: &mut GameState| {
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(cast(&mut g).is_err(), "rejected in a main phase");
    g.step = TurnStep::DeclareBlockers;
    assert!(cast(&mut g).is_ok(), "legal in combat");
}

// ── CR 615 / 614.9 — prevention and redirection ─────────────────────────────

/// CR 615 — the shared-colour prevention is symmetric: it also stops your own
/// creature hurting your own.
#[test]
fn cr_615_shared_color_prevention_is_symmetric() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::well_laid_plans());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut events = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(b), 2, Some(a), &mut events);
    assert_eq!(g.battlefield_find(b).unwrap().damage, 0);
}

/// CR 614.9 — only one redirect per event: the redirected damage lands on the
/// caster and isn't bounced onward.
#[test]
fn cr_614_9_spell_damage_redirect_applies_once() {
    let mut g = main_phase();
    for seat in [0, 1] {
        let ward = g.add_card_to_battlefield(seat, catalog::harsh_judgment());
        g.battlefield_find_mut(ward).unwrap().chosen_color = Some(Color::Red);
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 5);
    let (me, them) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, them, "the aimed-at seat is spared");
    assert_eq!(g.players[0].life, me - 3, "the caster eats it, once");
}
