//! Commander: the Quandrix Unlimited precon (SOC, Zimone, Infinite Analyst,
//! `decks::cmdr_zimone_ia`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn counters(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

/// Zimone — the first {X} spell grows her by two and is discounted by her
/// counters; the second isn't.
#[test]
fn zimone_discounts_and_grows_on_the_first_x_spell() {
    let mut g = pod(2);
    let z = g.add_card_to_battlefield(0, catalog::zimone_infinite_analyst());
    g.battlefield_find_mut(z).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    let hydra = g.add_card_to_hand(0, catalog::primordial_hydra());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    // {4}{G}{G} for X = 4, less three: {1}{G}{G}.
    g.perform_action(GameAction::CastSpell { card_id: hydra, target: None, additional_targets: vec![], mode: None, x_value: Some(4) })
        .expect("discounted");
    drain_stack(&mut g);
    assert_eq!(counters(&g, z), 5);
    assert_eq!(counters(&g, hydra), 4);
    let second = g.add_card_to_hand(0, catalog::steelbane_hydra());
    cast_x(&mut g, 0, second, None, Some(1)).expect("second");
    assert_eq!(counters(&g, z), 5, "only the first X spell");
}

/// Alchemist's Refuge — sorceries at instant speed for the turn.
#[test]
fn alchemists_refuge_grants_flash() {
    let mut g = pod(2);
    let ar = g.add_card_to_battlefield(0, catalog::alchemists_refuge());
    activate(&mut g, 0, ar, 1, None).expect("refuge");
    g.step = TurnStep::BeginCombat;
    let ea = g.add_card_to_hand(0, catalog::expansion_algorithm());
    cast_x(&mut g, 0, ea, None, Some(1)).expect("a sorcery in combat");
}

/// Altered Ego — enters as a copy with X additional counters.
#[test]
fn altered_ego_copies_with_counters() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::serra_angel());
    let ae = g.add_card_to_hand(0, catalog::altered_ego());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast_x(&mut g, 0, ae, None, Some(2)).expect("ego");
    let c = g.battlefield_find(ae).unwrap();
    assert_eq!(c.definition.name, "Serra Angel");
    assert_eq!(counters(&g, ae), 2);
}

/// Benevolent Hydra — +1/+1 counters on your other creatures get one more;
/// its own don't.
#[test]
fn benevolent_hydra_adds_to_others() {
    let mut g = pod(2);
    let bh = g.add_card_to_hand(0, catalog::benevolent_hydra());
    cast_x(&mut g, 0, bh, None, Some(2)).expect("hydra");
    assert_eq!(counters(&g, bh), 2, "no bonus on itself");
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bh);
    activate(&mut g, 0, bh, 0, Some(Target::Permanent(bears))).expect("move");
    assert_eq!(counters(&g, bh), 1);
    assert_eq!(counters(&g, bears), 2, "one plus one");
}

/// Brass Infiniscope — the next {X} spell draws and gains half X.
#[test]
fn brass_infiniscope_rewards_the_next_x_spell() {
    let mut g = pod(2);
    let bi = g.add_card_to_battlefield(0, catalog::brass_infiniscope());
    g.add_card_to_library(0, catalog::island());
    activate(&mut g, 0, bi, 0, None).expect("tap");
    let hydra = g.add_card_to_hand(0, catalog::primordial_hydra());
    let hand = g.players[0].hand.len();
    cast_x(&mut g, 0, hydra, None, Some(5)).expect("hydra");
    assert_eq!(g.players[0].hand.len(), hand, "−1 cast, +1 drawn");
    assert_eq!(g.players[0].life, 22, "half of 5, rounded down");
}

/// Entrancing Melody — takes a creature with mana value exactly X.
#[test]
fn entrancing_melody_steals_mv_x() {
    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let em = g.add_card_to_hand(0, catalog::entrancing_melody());
    cast_x(&mut g, 0, em, Some(Target::Permanent(angel)), Some(5)).expect("melody");
    assert_eq!(g.battlefield_find(angel).unwrap().controller, 0);
}

/// Expansion Algorithm — proliferates X times.
#[test]
fn expansion_algorithm_proliferates_x_times() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let ea = g.add_card_to_hand(0, catalog::expansion_algorithm());
    cast_x(&mut g, 0, ea, None, Some(3)).expect("algorithm");
    assert_eq!(counters(&g, bears), 4);
}

/// Kinetic Ooze — at X 5 it destroys a small artifact and draws.
#[test]
fn kinetic_ooze_breaks_and_draws() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.add_card_to_library(0, catalog::island());
    let ko = g.add_card_to_hand(0, catalog::kinetic_ooze());
    let hand = g.players[0].hand.len();
    cast_x(&mut g, 0, ko, None, Some(5)).expect("ooze");
    assert_eq!(counters(&g, ko), 5);
    assert!(g.battlefield_find(ring).is_none());
    assert_eq!(g.players[0].hand.len(), hand, "−1 cast, +1 drawn");
}

/// Lattice Library — a Fractal on entering and on the first {X} spell, sized
/// by its study counters.
#[test]
fn lattice_library_makes_fractals() {
    let mut g = pod(2);
    let ll = g.add_card_to_hand(0, catalog::lattice_library());
    cast_x(&mut g, 0, ll, None, Some(3)).expect("library");
    let f = named(&g, 0, "Fractal");
    assert_eq!(f.len(), 1);
    assert_eq!(pt(&g, f[0]), (3, 3));
    // The Library itself was this turn's first X spell; next turn's first
    // one makes another Fractal.
    g.players[0].spell_ids_cast_this_turn.clear();
    let ea = g.add_card_to_hand(0, catalog::expansion_algorithm());
    cast_x(&mut g, 0, ea, None, Some(0)).expect("algorithm");
    assert_eq!(named(&g, 0, "Fractal").len(), 2);
}

/// Nev — trample for countered creatures; the first {X} spell puts X
/// counters on Nev.
#[test]
fn nev_grows_by_x() {
    let mut g = pod(2);
    let nev = g.add_card_to_battlefield(0, catalog::nev_the_practical_dean());
    let sh = g.add_card_to_hand(0, catalog::steelbane_hydra());
    cast_x(&mut g, 0, sh, None, Some(3)).expect("hydra");
    assert_eq!(counters(&g, nev), 3);
    assert!(g.computed_permanent(nev).unwrap().keywords().contains(&Keyword::Trample));
}

/// Nexus Mentality — with a commander, move counters and cash them in.
#[test]
fn nexus_mentality_removes_counters_to_draw() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let nm = g.add_card_to_hand(0, catalog::nexus_mentality());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    let hand = g.players[0].hand.len();
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: nm,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("mentality");
    drain_stack(&mut g);
    assert_eq!(counters(&g, bears), 0);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3);
}

/// Owlin Spiralmancer — copies the first {X} spell.
#[test]
fn owlin_spiralmancer_copies_the_first_x_spell() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::owlin_spiralmancer());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ea = g.add_card_to_hand(0, catalog::expansion_algorithm());
    cast_x(&mut g, 0, ea, None, Some(1)).expect("algorithm");
    assert_eq!(counters(&g, bears), 3, "the copy proliferated too");
}

/// Ozolith — +1/+1 counters on an artifact get one more.
#[test]
fn ozolith_boosts_artifacts_too() {
    let mut g = pod(2);
    let oz = g.add_card_to_battlefield(0, catalog::ozolith_the_shattered_spire());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    activate(&mut g, 0, oz, 0, Some(Target::Permanent(ring))).expect("ozolith");
    assert_eq!(counters(&g, ring), 2);
}

/// Primo — enters with twice X counters; a base-power-0 attacker connecting
/// makes a Fractal the size of its damage.
#[test]
fn primo_makes_fractals_from_damage() {
    let mut g = pod(2);
    let p = g.add_card_to_hand(0, catalog::primo_the_unbounded());
    cast_x(&mut g, 0, p, None, Some(2)).expect("primo");
    assert_eq!(counters(&g, p), 4);
    g.clear_sickness(p);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: p, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    let f = named(&g, 0, "Fractal");
    assert_eq!(f.len(), 1);
    assert_eq!(pt(&g, f[0]), (4, 4));
}

/// Primordial Hydra — doubles each upkeep and tramples at ten.
#[test]
fn primordial_hydra_doubles() {
    let mut g = pod(2);
    let ph = g.add_card_to_hand(0, catalog::primordial_hydra());
    cast_x(&mut g, 0, ph, None, Some(5)).expect("hydra");
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(counters(&g, ph), 10);
    assert!(g.computed_permanent(ph).unwrap().keywords().contains(&Keyword::Trample));
}

/// Steelbane Hydra — a counter buys an artifact kill.
#[test]
fn steelbane_hydra_spends_counters() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let sh = g.add_card_to_hand(0, catalog::steelbane_hydra());
    cast_x(&mut g, 0, sh, None, Some(2)).expect("hydra");
    activate(&mut g, 0, sh, 0, Some(Target::Permanent(ring))).expect("destroy");
    assert!(g.battlefield_find(ring).is_none());
    assert_eq!(counters(&g, sh), 1);
}

/// The lands — Rain-Slicked Copse enters tapped; Turbulent Wilderness only
/// untapped against eight opposing lands.
#[test]
fn quandrix_lands_enter_as_printed() {
    for (def, tapped) in [(catalog::rain_slicked_copse(), true), (catalog::turbulent_wilderness(), true)] {
        let mut g = pod(2);
        let id = g.add_card_to_hand(0, def);
        g.perform_action(GameAction::PlayLand(id)).expect("land");
        assert_eq!(g.battlefield_find(id).unwrap().tapped, tapped);
    }
    let mut g = pod(2);
    for _ in 0..8 {
        g.add_card_to_battlefield(1, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::turbulent_wilderness());
    g.perform_action(GameAction::PlayLand(id)).expect("land");
    assert!(!g.battlefield_find(id).unwrap().tapped);
}

/// Striding Shotcaller — connecting prepares Run the Play: counters, flying
/// and a card.
#[test]
fn striding_shotcaller_prepares_run_the_play() {
    let mut g = pod(2);
    let ss = g.add_card_to_battlefield(0, catalog::striding_shotcaller());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bears);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ss).unwrap().counter_count(CounterType::Prepared), 1);
    g.step = TurnStep::PostCombatMain;
    g.add_card_to_library(0, catalog::island());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: ss,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![Target::Permanent(ss)],
        mode: None,
        x_value: Some(2),
    })
    .expect("Run the Play");
    drain_stack(&mut g);
    assert_eq!(counters(&g, bears), 1);
    assert_eq!(counters(&g, ss), 1);
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::Flying));
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Unbound Flourishing — doubles a permanent spell's X and copies an {X}
/// sorcery.
#[test]
fn unbound_flourishing_doubles_and_copies() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::unbound_flourishing());
    let ph = g.add_card_to_hand(0, catalog::primordial_hydra());
    cast_x(&mut g, 0, ph, None, Some(3)).expect("hydra");
    assert_eq!(counters(&g, ph), 6, "X doubled");
    let ea = g.add_card_to_hand(0, catalog::expansion_algorithm());
    cast_x(&mut g, 0, ea, None, Some(1)).expect("algorithm");
    assert_eq!(counters(&g, ph), 8, "the copy proliferated too");
}

/// Yavimaya Bloomsage — its end step grows a creature and prepares Channel
/// once that creature reaches power 7.
#[test]
fn yavimaya_bloomsage_prepares_at_seven_power() {
    let mut g = pod(2);
    let yb = g.add_card_to_battlefield(0, catalog::yavimaya_bloomsage());
    let big = g.add_card_to_battlefield(0, catalog::craw_wurm());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(counters(&g, big), 1);
    assert_eq!(g.battlefield_find(yb).unwrap().counter_count(CounterType::Prepared), 1, "a 7/5 Craw Wurm");
}
