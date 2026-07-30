//! CR conformance for the modern_decks Darksteel/Fifth Dawn pass:
//! - CR 702.43 — Modular N (enters-with, the death hand-off, 702.43b's
//!   "each instance works separately").
//! - CR 702.44 — Sunburst (+1/+1 on a creature, charge otherwise; a colorless
//!   cast lands nothing; it's a replacement, so Solemnity blanks it).
//! - CR 704.8 — a permanent that leaves via a state-based action reports LKI
//!   from the state *before* any of that sweep's actions ran.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// Cast `def` from seat 0's hand paying with exactly `colors` (plus
/// `colorless` generic) — Sunburst counts the colors actually spent, so the
/// pool has to be exact.
fn cast_with(
    g: &mut GameState,
    def: crabomination::card::CardDefinition,
    colors: &[Color],
    colorless: u32,
) -> CardId {
    let id = g.add_card_to_hand(0, def);
    for c in colors {
        g.players[0].mana_pool.add(*c, 1);
    }
    if colorless > 0 {
        g.players[0].mana_pool.add_colorless(colorless);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(g);
    id
}

// ── CR 702.43 — Modular ──

/// 702.43a — a modular permanent enters with N +1/+1 counters.
#[test]
fn cr_702_43a_modular_enters_with_its_counters() {
    let mut g = main_phase();
    let bruiser = cast_with(&mut g, catalog::arcbound_bruiser(), &[], 5);
    assert_eq!(g.battlefield_find(bruiser).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    let cp = g.computed_permanent(bruiser).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "0/0 body plus three counters");
}

/// 702.43a — the death half moves every +1/+1 counter to a target artifact
/// creature, not just the printed N.
#[test]
fn cr_702_43a_modular_death_moves_all_its_counters() {
    let mut g = main_phase();
    let worker = g.add_card_to_battlefield_with_counters(0, catalog::arcbound_worker());
    g.battlefield_find_mut(worker).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    let heir = g.add_card_to_battlefield(0, catalog::coretapper());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let mut events = vec![];
    g.destroy_permanent(worker, false, &mut events);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(heir).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

/// 702.43b — a "creature with modular" filter matches regardless of N, so
/// Modular 1 and Modular 6 are both seen by Arcbound Overseer.
#[test]
fn cr_702_43b_modular_filter_is_value_agnostic() {
    let mut g = main_phase();
    let one = g.add_card_to_battlefield_with_counters(0, catalog::arcbound_worker());
    let six = g.add_card_to_battlefield_with_counters(0, catalog::arcbound_overseer());
    for id in [one, six] {
        assert!(g.computed_permanent(id).unwrap().keywords.iter().any(|k| matches!(k, Keyword::Modular(_))));
    }
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(one).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.battlefield_find(six).unwrap().counter_count(CounterType::PlusOnePlusOne), 7);
}

// ── CR 702.44 — Sunburst ──

/// 702.44a — a Sunburst creature enters with one +1/+1 counter per color of
/// mana spent.
#[test]
fn cr_702_44a_sunburst_creature_counts_colors_spent() {
    let mut g = main_phase();
    let myr = cast_with(&mut g, catalog::suntouched_myr(), &[Color::White, Color::Blue, Color::Black], 0);
    let cp = g.computed_permanent(myr).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "three colors spent");
}

/// 702.44a — a Sunburst noncreature gets charge counters instead.
#[test]
fn cr_702_44a_sunburst_noncreature_gets_charge_counters() {
    let mut g = main_phase();
    let prism = cast_with(&mut g, catalog::pentad_prism(), &[Color::Red, Color::Green], 0);
    assert_eq!(g.battlefield_find(prism).unwrap().counter_count(CounterType::Charge), 2);
    assert_eq!(g.battlefield_find(prism).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// 702.44b — no colored mana spent means no counters at all.
#[test]
fn cr_702_44b_sunburst_lands_nothing_on_a_colorless_cast() {
    let mut g = main_phase();
    let prism = cast_with(&mut g, catalog::pentad_prism(), &[], 2);
    assert_eq!(g.battlefield_find(prism).unwrap().counter_count(CounterType::Charge), 0);
}

/// 702.44a is a CR 614.12 replacement, so Solemnity's counter lock blanks it
/// (it isn't a trigger that can be responded to).
#[test]
fn cr_702_44a_sunburst_is_blanked_by_a_counter_lock() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::solemnity());
    let prism = cast_with(&mut g, catalog::pentad_prism(), &[Color::Red, Color::Green], 0);
    assert_eq!(g.battlefield_find(prism).unwrap().counter_count(CounterType::Charge), 0);
}

/// 702.44c — "Modular—Sunburst" takes its counter count from Sunburst and
/// still hands the pile off on death.
#[test]
fn cr_702_44c_modular_sunburst_hands_off_its_sunburst_counters() {
    let mut g = main_phase();
    let wanderer = cast_with(
        &mut g,
        catalog::arcbound_wanderer(),
        &[Color::White, Color::Blue, Color::Black, Color::Red, Color::Green],
        1,
    );
    assert_eq!(g.battlefield_find(wanderer).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
    let heir = g.add_card_to_battlefield(0, catalog::coretapper());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let mut events = vec![];
    g.destroy_permanent(wanderer, false, &mut events);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(heir).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
}

// ── CR 704.8 — LKI across a single SBA sweep ──

/// 704.8 — Young Wolf has one +1/+1 counter and takes three -1/-1 counters.
/// The 704.5q annihilation and the 704.5f death happen in the same sweep, so
/// undying reads the pre-sweep state (a +1/+1 counter was on it) and declines.
#[test]
fn cr_704_8_undying_reads_pre_sweep_counters() {
    let mut g = main_phase();
    let wolf = g.add_card_to_battlefield(0, catalog::young_wolf());
    {
        let c = g.battlefield_find_mut(wolf).unwrap();
        c.add_counters(CounterType::PlusOnePlusOne, 1);
        c.add_counters(CounterType::MinusOneMinusOne, 3);
    }
    g.check_state_based_actions();
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(
        !g.battlefield.iter().any(|c| c.definition.name == "Young Wolf"),
        "undying doesn't return it — it had a +1/+1 counter before the sweep",
    );
    assert!(g.players[0].graveyard.iter().any(|c| c.id == wolf));
}

/// The control case: no +1/+1 counter before the sweep, so undying returns it.
#[test]
fn cr_704_8_undying_returns_a_counterless_creature() {
    let mut g = main_phase();
    let wolf = g.add_card_to_battlefield(0, catalog::young_wolf());
    g.battlefield_find_mut(wolf).unwrap().damage = 5;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Young Wolf"),
        "undying brings it back with a counter",
    );
}

/// Test of Faith's shield converts prevented damage into counters even when
/// the damage would have been lethal (CR 615.7 ordering: prevention runs
/// before the SBA sweep sees the damage).
#[test]
fn cr_615_7_prevention_with_counters_beats_the_sba_sweep() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let test = g.add_card_to_hand(0, catalog::test_of_faith());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: test, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast Test of Faith");
    drain_stack(&mut g);
    let mut events = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(bear),
        3,
        None,
        &mut events,
    );
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_some(), "all 3 prevented");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// CR 615.7 — Avacyn's chosen source color is a real prompt for a UI seat
/// (it used to be auto-picked off the resolving decider).
#[test]
fn cr_615_7_chosen_color_prevention_prompts_a_ui_seat() {
    use crabomination::decision::{Decision, DecisionAnswer};
    let mut g = main_phase();
    g.players[0].wants_ui = true;
    let avacyn = g.add_card_to_battlefield(0, catalog::avacyn_guardian_angel());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: avacyn, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    })
    .expect("activate the creature-prevention half");
    drain_stack(&mut g);
    let pending = g.pending_decision.as_ref().expect("a ChooseColor prompt is raised");
    assert!(matches!(pending.decision, Decision::ChooseColor { .. }));
    assert!(g.prevention_shields.is_empty(), "nothing lands until the color is picked");
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Color(Color::Red)))
        .expect("pick red");
    drain_stack(&mut g);
    assert_eq!(
        g.prevention_shields.iter().filter(|s| s.source_color == Some(Color::Red)).count(),
        1,
        "the shield lands scoped to the chosen color",
    );
}
