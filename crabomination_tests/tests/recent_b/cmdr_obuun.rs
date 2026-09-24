//! Commander: the Land's Wrath precon (ZNC, Obuun, `decks::cmdr_obuun`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}


fn declare(g: &mut GameState, attacker: CardId, defender: usize) -> Result<(), String> {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(defender),
    }]))
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn counters(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0)
}

#[test]
fn armorcraft_judge_draws_per_countered_creature() {
    let mut g = main_phase(2);
    for _ in 0..2 {
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(b).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        g.add_card_to_library(0, catalog::plains());
    }
    let j = g.add_card_to_hand(0, catalog::armorcraft_judge());
    cast(&mut g, j, &[]);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Choose one or both (CR 700.2): both modes exile.
#[test]
fn crush_contraband_exiles_an_artifact_and_an_enchantment() {
    let mut g = main_phase(2);
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    let ench = g.add_card_to_battlefield(1, catalog::grave_peril());
    let c = g.add_card_to_hand(0, catalog::crush_contraband());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: c,
        spree_modes: vec![0, 1],
        target: Some(Target::Permanent(art)),
        additional_targets: vec![Target::Permanent(ench)],
        x_value: None,
    })
    .expect("both modes");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none() && g.battlefield_find(ench).is_none());
}

#[test]
fn elite_scaleguard_bolsters_and_taps_a_blocker() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::elite_scaleguard());
    cast(&mut g, s, &[]);
    assert_eq!(counters(&g, bear), 2, "bolster picks the least toughness");
    let blocker = g.add_card_to_battlefield(1, catalog::celestial_force());
    declare(&mut g, bear, 1).expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(blocker).unwrap().tapped);
}

#[test]
fn elvish_rejuvenator_puts_a_land_from_the_top_five() {
    let mut g = main_phase(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.add_card_to_library(0, catalog::forest());
    let e = g.add_card_to_hand(0, catalog::elvish_rejuvenator());
    cast(&mut g, e, &[]);
    let land = g.battlefield.iter().find(|c| c.definition.name == "Forest" && c.controller == 0).expect("the forest");
    assert!(land.tapped);
}

/// With ten nonland permanents out it costs {3} less.
#[test]
fn hour_of_revelation_gets_cheap_and_wipes_nonlands() {
    let mut g = main_phase(2);
    let land = g.add_card_to_battlefield(0, catalog::plains());
    for _ in 0..10 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    let h = g.add_card_to_hand(0, catalog::hour_of_revelation());
    g.players[0].mana_pool.add(Color::White, 3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: h, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{W}{W}{W} is enough");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.definition.is_land()));
    assert!(g.battlefield_find(land).is_some());
}

#[test]
fn keeper_of_fables_draws_when_a_non_human_connects() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::keeper_of_fables());
    g.add_card_to_library(0, catalog::plains());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    connect(&mut g, bear, 1);
    assert_eq!(g.players[0].hand.len(), 1);
}

#[test]
fn murasa_rootgrazer_drops_and_returns_basics() {
    let mut g = main_phase(2);
    let m = g.add_card_to_battlefield(0, catalog::murasa_rootgrazer());
    g.clear_sickness(m);
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![forest])]));
    activate(&mut g, m, 0, None).expect("drop a land");
    assert!(g.battlefield_find(forest).is_some());
    g.battlefield_find_mut(m).unwrap().tapped = false;
    activate(&mut g, m, 1, Some(Target::Permanent(forest))).expect("pick it back up");
    assert!(g.players[0].hand.iter().any(|c| c.id == forest));
}

#[test]
fn naya_panorama_fetches_a_basic_tapped() {
    let mut g = main_phase(2);
    let p = g.add_card_to_battlefield(0, catalog::naya_panorama());
    g.add_card_to_library(0, catalog::mountain());
    activate(&mut g, p, 1, None).expect("fetch");
    let m = g.battlefield.iter().find(|c| c.definition.name == "Mountain").expect("mountain");
    assert!(m.tapped);
}

/// Beginning of combat: a land becomes an X/X hasty trampler; landfall adds a
/// counter.
#[test]
fn obuun_animates_a_land_and_grows_on_landfall() {
    let mut g = main_phase(2);
    let o = g.add_card_to_battlefield(0, catalog::obuun_mul_daya_ancestor());
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(forest)).expect("play");
    drain_stack(&mut g);
    assert_eq!(counters(&g, o), 1, "landfall picks Obuun, the only creature");
    advance_to(&mut g, TurnStep::DeclareAttackers);
    assert_eq!(pt(&g, forest), (4, 4));
    assert!(g.permanent_has_keyword(forest, &Keyword::Haste));
    assert!(g.permanent_has_keyword(forest, &Keyword::Trample));
}

#[test]
fn scaretiller_drops_a_land_when_it_taps() {
    let mut g = main_phase(2);
    let s = g.add_card_to_battlefield(0, catalog::scaretiller());
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![forest])]));
    declare(&mut g, s, 1).expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(forest).is_some_and(|c| c.tapped));
}

#[test]
fn struggle_hits_for_lands_and_survive_shuffles_graveyards() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let force = g.add_card_to_battlefield(1, catalog::celestial_force());
    let s = g.add_card_to_hand(0, catalog::struggle_survive());
    cast(&mut g, s, &[Target::Permanent(force)]);
    assert_eq!(g.battlefield_find(force).unwrap().damage, 3);
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastAftermath {
        card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Survive from the graveyard (aftermath)");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.is_empty());
}

#[test]
fn sylvan_reclamation_exiles_two() {
    let mut g = main_phase(2);
    let a = g.add_card_to_battlefield(1, catalog::sol_ring());
    let b = g.add_card_to_battlefield(1, catalog::grave_peril());
    let s = g.add_card_to_hand(0, catalog::sylvan_reclamation());
    cast(&mut g, s, &[Target::Permanent(a), Target::Permanent(b)]);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Chapter I: mill two, then a creature card back to hand.
#[test]
fn the_mending_of_dominaria_mills_and_regrows() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::plains());
    let m = g.add_card_to_hand(0, catalog::the_mending_of_dominaria());
    cast(&mut g, m, &[]);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Support 2, then the {1} makes a countered creature come home when it dies.
#[test]
fn together_forever_supports_and_saves_a_countered_creature() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::together_forever());
    cast(&mut g, t, &[Target::Permanent(bear)]);
    assert_eq!(counters(&g, bear), 1);
    activate(&mut g, t, 0, Some(Target::Permanent(bear))).expect("watch it");
    let kill = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, kill, &[Target::Permanent(bear)]);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Landfall exiles a cheap permanent card; the Warden leaving returns it.
#[test]
fn trove_warden_stores_and_releases_permanent_cards() {
    let mut g = main_phase(2);
    let w = g.add_card_to_battlefield(0, catalog::trove_warden());
    let stored = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(forest)).expect("play");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == stored));
    let kill = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, kill, &[Target::Permanent(w)]);
    assert!(g.battlefield_find(stored).is_some());
}

#[test]
fn waker_of_the_wilds_awakens_a_land() {
    let mut g = main_phase(2);
    let w = g.add_card_to_battlefield(0, catalog::waker_of_the_wilds());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: w, ability_index: 0, target: Some(Target::Permanent(land)), additional_targets: vec![], x_value: Some(3), mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(pt(&g, land), (3, 3));
    assert!(g.permanent_has_keyword(land, &Keyword::Haste));
}

fn connect(g: &mut GameState, attacker: CardId, defender: usize) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(defender) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = defender;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(g);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}
