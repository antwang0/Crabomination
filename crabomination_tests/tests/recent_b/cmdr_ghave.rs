//! Commander: the Counterpunch precon (CMD, Ghave, `decks::cmdr_ghave`).

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

fn count_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
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

fn counters(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0)
}

/// "That permanent's controller or that player" gets the Squirrel.
#[test]
fn acorn_catapult_pays_its_victim_a_squirrel() {
    let mut g = main_phase(2);
    let cat = g.add_card_to_battlefield(0, catalog::acorn_catapult());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, cat, 0, Some(Target::Permanent(bear))).expect("shoot the bear");
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1);
    assert_eq!(count_named(&g, 1, "Squirrel"), 1);
    g.battlefield_find_mut(cat).unwrap().tapped = false;
    let life = g.players[1].life;
    activate(&mut g, cat, 0, Some(Target::Player(1))).expect("shoot the player");
    assert_eq!(g.players[1].life, life - 1);
    assert_eq!(count_named(&g, 1, "Squirrel"), 2);
}

/// CR 207.2c — join forces sums every seat's payment.
#[test]
fn alliance_of_arms_gives_everyone_the_total_paid() {
    let mut g = main_phase(2);
    g.players[1].mana_pool.add(Color::White, 2);
    let a = g.add_card_to_hand(0, catalog::alliance_of_arms());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2), DecisionAnswer::Amount(1)]));
    cast(&mut g, a, &[]);
    assert_eq!(count_named(&g, 0, "Soldier"), 3);
    assert_eq!(count_named(&g, 1, "Soldier"), 3);
}

#[test]
fn awakening_zone_offers_a_spawn_each_upkeep() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::awakening_zone());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Draw {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(count_named(&g, 0, "Eldrazi Spawn"), 1);
}

/// "Each upkeep": the opponent's upkeep feeds it too.
#[test]
fn celestial_force_gains_three_on_every_upkeep() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::celestial_force());
    let life = g.players[0].life;
    g.active_player_idx = 1;
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Draw {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, life + 3);
}

/// The greatest pick ends on top, so the draw takes it.
#[test]
fn footbottom_feast_stacks_creatures_and_draws_the_best() {
    let mut g = main_phase(2);
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::celestial_force());
    let f = g.add_card_to_hand(0, catalog::footbottom_feast());
    cast(&mut g, f, &[]);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Celestial Force"));
}

#[test]
fn ghave_enters_with_five_and_turns_counters_into_saprolings() {
    let mut g = main_phase(2);
    let ghave = g.add_card_to_hand(0, catalog::ghave_guru_of_spores());
    cast(&mut g, ghave, &[]);
    assert_eq!(counters(&g, ghave), 5);
    activate(&mut g, ghave, 0, None).expect("remove a counter");
    assert_eq!(counters(&g, ghave), 4);
    assert_eq!(count_named(&g, 0, "Saproling"), 1);
    let sap = g.battlefield.iter().find(|c| c.definition.name == "Saproling").unwrap().id;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, ghave, 1, Some(Target::Permanent(bear))).expect("sacrifice for a counter");
    assert!(g.battlefield_find(sap).is_none(), "the Saproling is the cheapest fodder");
    assert_eq!(counters(&g, bear), 1);
}

/// Karador costs {1} less per creature card in your graveyard.
#[test]
fn karador_gets_cheaper_with_every_dead_creature() {
    let mut g = main_phase(2);
    for _ in 0..5 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let k = g.add_card_to_hand(0, catalog::karador_ghost_chieftain());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell { card_id: k, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("five creature cards pay the generic five");
    drain_stack(&mut g);
    assert!(g.battlefield_find(k).is_some());
}

/// Once during each of your turns, a creature spell from the graveyard.
#[test]
fn karador_casts_one_creature_a_turn_from_the_graveyard() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::karador_ghost_chieftain());
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: a, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("the first graveyard cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_some());
    flood(&mut g, 0);
    let second = g.perform_action(GameAction::CastSpell { card_id: b, target: None, additional_targets: vec![], mode: None, x_value: None });
    assert!(second.is_err(), "only once a turn");
}

#[test]
fn necrogenesis_eats_a_graveyard_creature_for_a_saproling() {
    let mut g = main_phase(2);
    let n = g.add_card_to_battlefield(0, catalog::necrogenesis());
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    activate(&mut g, n, 0, Some(Target::Permanent(dead))).expect("activate");
    assert!(g.players[1].graveyard.is_empty());
    assert_eq!(count_named(&g, 0, "Saproling"), 1);
}

/// Unpaid, the Spire is sacrificed.
#[test]
fn rupture_spire_needs_its_toll() {
    let mut g = main_phase(2);
    let land = g.add_card_to_hand(0, catalog::rupture_spire());
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "no mana to pay");
}

/// CR 603.4 — only a 1/1 arrival gets the counters.
#[test]
fn sigil_captain_grows_one_one_arrivals() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::sigil_captain());
    let n = g.add_card_to_battlefield(0, catalog::necrogenesis());
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    activate(&mut g, n, 0, Some(Target::Permanent(dead))).expect("activate");
    let sap = g.battlefield.iter().find(|c| c.definition.name == "Saproling").unwrap().id;
    assert_eq!(pt(&g, sap), (3, 3));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, &[]);
    assert_eq!(pt(&g, bear), (2, 2));
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

#[test]
fn spawnwrithe_copies_itself_when_it_connects() {
    let mut g = main_phase(2);
    let s = g.add_card_to_battlefield(0, catalog::spawnwrithe());
    connect(&mut g, s, 1);
    assert_eq!(count_named(&g, 0, "Spawnwrithe"), 2);
}

#[test]
fn teneb_may_pay_to_reanimate_from_any_graveyard() {
    let mut g = main_phase(2);
    let t = g.add_card_to_battlefield(0, catalog::teneb_the_harvester());
    let dead = g.add_card_to_graveyard(1, catalog::celestial_force());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    connect(&mut g, t, 1);
    assert!(g.battlefield_find(dead).is_some_and(|c| c.controller == 0));
}

#[test]
fn vish_kal_eats_power_then_spends_it_as_minus_counters() {
    let mut g = main_phase(2);
    let v = g.add_card_to_battlefield(0, catalog::vish_kal_blood_arbiter());
    g.add_card_to_battlefield(0, catalog::celestial_force());
    activate(&mut g, v, 0, None).expect("sacrifice");
    assert_eq!(counters(&g, v), 7);
    let victim = g.add_card_to_battlefield(1, catalog::celestial_force());
    activate(&mut g, v, 1, Some(Target::Permanent(victim))).expect("remove all");
    assert!(g.battlefield_find(victim).is_none(), "-7/-7 kills a 7/7");
    assert_eq!(counters(&g, v), 0);
}

/// CR 508.1a — the Vow keeps the creature off its Aura's controller.
#[test]
fn vow_of_wildness_keeps_the_creature_off_you() {
    let mut g = main_phase(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let v = g.add_card_to_hand(0, catalog::vow_of_wildness());
    cast(&mut g, v, &[Target::Permanent(bear)]);
    assert_eq!(pt(&g, bear), (5, 5));
    assert!(g.permanent_has_keyword(bear, &Keyword::Trample));
    g.active_player_idx = 1;
    assert!(declare(&mut g, bear, 0).is_err());
    declare(&mut g, bear, 2).expect("another opponent is fine");
}
