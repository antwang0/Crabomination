//! Commander: the Eternal Might precon (DRC, Hashaton, `decks::cmdr_hashaton`).

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

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

#[test]
fn binding_mummy_taps_something_when_a_zombie_arrives() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::binding_mummy());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let z = g.add_card_to_hand(0, catalog::wayward_servant());
    cast(&mut g, z, &[]);
    assert!(g.battlefield_find(victim).unwrap().tapped);
}

/// Corpse counters survive the sacrifice cost (CR 608.2h last-known info).
#[test]
fn crowded_crypt_counts_corpses_into_decayed_zombies() {
    let mut g = main_phase(2);
    let crypt = g.add_card_to_battlefield(0, catalog::crowded_crypt());
    for _ in 0..2 {
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let kill = g.add_card_to_hand(0, catalog::murder());
        cast(&mut g, kill, &[Target::Permanent(bear)]);
    }
    assert_eq!(g.battlefield_find(crypt).unwrap().counter_count(CounterType::Corpse), 2);
    activate(&mut g, crypt, 1, None).expect("sacrifice");
    assert_eq!(count_named(&g, 0, "Zombie"), 2);
    let z = g.battlefield.iter().find(|c| c.definition.name == "Zombie").unwrap().id;
    assert!(g.permanent_has_keyword(z, &Keyword::Decayed));
}

#[test]
fn a_cycling_desert_enters_tapped_and_cycles() {
    let mut g = main_phase(2);
    let land = g.add_card_to_hand(0, catalog::desert_of_the_true());
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped);
    g.add_card_to_library(0, catalog::plains());
    let other = g.add_card_to_hand(0, catalog::desert_of_the_glorified());
    flood(&mut g, 0);
    g.perform_action(GameAction::Cycle { card_id: other, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == other));
}

/// A nontoken creature dying gains 1 and loots; with six creature cards in
/// the graveyard the Gate fetches God-Pharaoh's Gift from the library.
#[test]
fn gate_to_the_afterlife_loots_and_fetches_the_gift() {
    let mut g = main_phase(2);
    let gate = g.add_card_to_battlefield(0, catalog::gate_to_the_afterlife());
    g.add_card_to_library(0, catalog::plains());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let kill = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, kill, &[Target::Permanent(bear)]);
    assert_eq!(g.players[0].life, life + 1);
    assert!(activate(&mut g, gate, 0, None).is_err(), "needs six creature cards");
    for _ in 0..5 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.add_card_to_library(0, catalog::god_pharaohs_gift());
    activate(&mut g, gate, 0, None).expect("fetch");
    assert_eq!(count_named(&g, 0, "God-Pharaoh's Gift"), 1);
}

/// At the beginning of combat: exile the best creature card for a hasty
/// 4/4 black Zombie copy.
#[test]
fn god_pharaohs_gift_makes_a_hasty_four_four_copy() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::god_pharaohs_gift());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    advance_to(&mut g, TurnStep::DeclareAttackers);
    assert!(g.exile.iter().any(|c| c.id == angel), "the angel (greatest power) is exiled");
    let copy = g.battlefield.iter().find(|c| c.definition.name == "Serra Angel").unwrap().id;
    assert_eq!(pt(&g, copy), (4, 4));
    assert!(g.permanent_has_keyword(copy, &Keyword::Haste));
    assert!(g.permanent_has_keyword(copy, &Keyword::Flying));
}

#[test]
fn hashaton_pays_to_copy_a_discarded_creature() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::hashaton_scarabs_fist());
    g.add_card_to_hand(0, catalog::serra_angel());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let d = g.add_card_to_hand(0, catalog::mind_rot());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: d, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("discard own hand");
    drain_stack(&mut g);
    let copy = g.battlefield.iter().find(|c| c.definition.name == "Serra Angel" && c.is_token).map(|c| c.id);
    let copy = copy.expect("a token copy");
    assert_eq!(pt(&g, copy), (4, 4));
    assert!(g.battlefield_find(copy).unwrap().tapped);
}

#[test]
fn on_wings_of_gold_lifts_zombies_and_tokens() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::on_wings_of_gold());
    let z = g.add_card_to_battlefield(0, catalog::wayward_servant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, z), (3, 3));
    assert!(g.permanent_has_keyword(z, &Keyword::Flying));
    assert_eq!(pt(&g, bear), (2, 2));
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let n = g.add_card_to_battlefield(0, catalog::necrogenesis());
    activate(&mut g, n, 0, Some(Target::Permanent(dead))).expect("exile from the graveyard");
    assert_eq!(count_named(&g, 0, "Zombie"), 1, "a card left the graveyard");
}

#[test]
fn plague_belcher_shrinks_itself_and_drains_on_zombie_deaths() {
    let mut g = main_phase(3);
    let p = g.add_card_to_hand(0, catalog::plague_belcher());
    cast(&mut g, p, &[]);
    assert_eq!(pt(&g, p), (3, 2));
    let z = g.add_card_to_battlefield(0, catalog::wayward_servant());
    let lives = [g.players[1].life, g.players[2].life];
    let kill = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, kill, &[Target::Permanent(z)]);
    assert_eq!([g.players[1].life, g.players[2].life], [lives[0] - 1, lives[1] - 1]);
}

#[test]
fn priest_of_the_crossing_counts_the_turns_dead() {
    let mut g = main_phase(2);
    let priest = g.add_card_to_battlefield(0, catalog::priest_of_the_crossing());
    for _ in 0..2 {
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let kill = g.add_card_to_hand(0, catalog::murder());
        cast(&mut g, kill, &[Target::Permanent(bear)]);
    }
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(pt(&g, priest), (5, 5));
}

#[test]
fn prophet_of_the_scarab_draws_the_greater_zombie_count() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::wayward_servant());
    }
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::plains());
    }
    let p = g.add_card_to_hand(0, catalog::prophet_of_the_scarab());
    cast(&mut g, p, &[]);
    assert_eq!(g.players[0].hand.len(), 3, "three Zombie cards beat one Zombie");
}

/// CR 702.29 — a creature card in hand gains cycling {1}{U}.
#[test]
fn rhet_tomb_mystic_gives_hand_creatures_cycling() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::rhet_tomb_mystic());
    g.add_card_to_library(0, catalog::plains());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let land = g.add_card_to_hand(0, catalog::plains());
    flood(&mut g, 0);
    assert!(g.perform_action(GameAction::Cycle { card_id: land, x_value: None }).is_err(), "not a creature");
    g.perform_action(GameAction::Cycle { card_id: bear, x_value: None }).expect("cycle the bear");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
}

/// X is the number of opponents: two at a three-seat table.
#[test]
fn rot_hulk_returns_a_zombie_per_opponent() {
    let mut g = main_phase(3);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::wayward_servant());
    }
    let h = g.add_card_to_hand(0, catalog::rot_hulk());
    cast(&mut g, h, &[]);
    assert_eq!(count_named(&g, 0, "Wayward Servant"), 2);
}

#[test]
fn temmet_loots_on_attack_and_pumps_zombies_per_draw() {
    let mut g = main_phase(2);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::plains());
    }
    g.add_card_to_hand(0, catalog::swamp());
    let t = g.add_card_to_battlefield(0, catalog::temmet_naktamuns_will());
    declare(&mut g, t, 1).expect("attack");
    drain_stack(&mut g);
    assert_eq!(pt(&g, t), (5, 5), "one draw, one pump");
    assert_eq!(g.players[0].graveyard.len(), 1, "then a discard");
}

#[test]
fn wayward_servant_drains_when_a_zombie_arrives() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(0, catalog::wayward_servant());
    let lives = [g.players[0].life, g.players[1].life, g.players[2].life];
    let z = g.add_card_to_hand(0, catalog::binding_mummy());
    cast(&mut g, z, &[]);
    assert_eq!([g.players[0].life, g.players[1].life, g.players[2].life], [lives[0] + 1, lives[1] - 1, lives[2] - 1]);
}

/// Once each turn, an opponent's non-mana activation makes a Zombie.
#[test]
fn wizened_mentor_answers_an_opponents_activation_once_a_turn() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::wizened_mentor());
    g.active_player_idx = 1;
    let cat = g.add_card_to_battlefield(1, catalog::acorn_catapult());
    for _ in 0..2 {
        g.battlefield_find_mut(cat).unwrap().tapped = false;
        flood(&mut g, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::ActivateAbility {
            card_id: cat, ability_index: 0, target: Some(Target::Player(0)), additional_targets: vec![], x_value: None, mode: None,
        })
        .expect("activate");
        drain_stack(&mut g);
    }
    assert_eq!(count_named(&g, 0, "Zombie"), 1);
}
