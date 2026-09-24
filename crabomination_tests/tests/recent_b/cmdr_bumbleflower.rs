//! Commander: the Peace Offering precon (BLC, Ms. Bumbleflower,
//! `decks::cmdr_bumbleflower`).

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

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    act(g, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    act(g, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode: None,
    })
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0)
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn attack_with(g: &mut GameState, attackers: &[CardId], defender: usize) {
    for a in attackers {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::plains());
    }
}

/// Ms. Bumbleflower: each spell feeds an opponent a card and a creature a
/// counter and flying; the second trigger this turn draws you two.
#[test]
fn ms_bumbleflower_shares_then_draws_on_the_second_spell() {
    let mut g = main_phase(2);
    library(&mut g, 0, 3);
    library(&mut g, 1, 3);
    let mb = g.add_card_to_battlefield(0, catalog::ms_bumbleflower());
    g.add_card_to_battlefield(0, catalog::serra_angel());
    for i in 0..2 {
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Target(Target::Player(1)),
            DecisionAnswer::Target(Target::Permanent(mb)),
        ]));
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        flood(&mut g, 0);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None }).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), i + 1, "the opponent draws each time");
    }
    assert_eq!(g.players[0].hand.len(), 2, "the second resolution drew two");
    let counters: u32 = g.battlefield.iter().filter(|c| c.controller == 0).map(|c| plus(&g, c.id)).sum();
    assert_eq!(counters, 2, "one +1/+1 counter per trigger");
    let _ = mb;
}

/// Bloodroot Apothecary: a Treasure each, and an opponent sacrificing a
/// noncreature token takes two poison counters.
#[test]
fn bloodroot_apothecary_poisons_treasure_sacrifices() {
    let mut g = main_phase(2);
    let ba = g.add_card_to_hand(0, catalog::bloodroot_apothecary());
    cast_at(&mut g, ba, &[]).expect("cast");
    assert_eq!(named(&g, 0, "Treasure").len(), 1);
    let theirs = named(&g, 1, "Treasure");
    assert_eq!(theirs.len(), 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: theirs[0],
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("crack the Treasure");
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 2);
}

/// CR 604.3 — Body of Knowledge is as big as your hand and draws off damage.
#[test]
fn cr_604_3_body_of_knowledge_grows_with_the_hand() {
    let mut g = main_phase(2);
    library(&mut g, 0, 5);
    let bk = g.add_card_to_battlefield(0, catalog::body_of_knowledge());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::plains());
    }
    assert_eq!(pt(&g, bk), (3, 3));
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast_at(&mut g, shock, &[Target::Permanent(bk)]).expect("shock");
    assert_eq!(g.players[0].hand.len(), 5, "three, plus two drawn off the Shock");
}

/// Communal Brewing: opponents draw, ingredients pile up, and each creature
/// spell enters with that many extra counters.
#[test]
fn communal_brewing_seasons_creature_spells() {
    let mut g = main_phase(3);
    library(&mut g, 1, 2);
    library(&mut g, 2, 2);
    let cb = g.add_card_to_hand(0, catalog::communal_brewing());
    cast_at(&mut g, cb, &[Target::Player(1), Target::Player(2)]).expect("cast");
    let n = g.battlefield_find(cb).unwrap().counter_count(CounterType::Ingredient);
    let drew = g.players[1].hand.len() + g.players[2].hand.len();
    assert_eq!(n as usize, 1 + drew);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, bear, &[]).expect("cast");
    assert_eq!(plus(&g, bear), n);
}

/// Fisher's Talent: a land on top makes a Fish, which becomes a Shark at
/// level 2 and an Octopus at level 3.
#[test]
fn fishers_talent_levels_the_catch() {
    let mut g = main_phase(2);
    let ft = g.add_card_to_hand(0, catalog::fishers_talent());
    cast_at(&mut g, ft, &[]).expect("cast");
    let upkeep = |g: &mut GameState| {
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::forest());
        }
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(g);
    };
    upkeep(&mut g);
    assert_eq!(named(&g, 0, "Fish").len(), 1);
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, ft, 0, &[]).expect("level 2");
    upkeep(&mut g);
    assert_eq!(named(&g, 0, "Shark").len(), 1);
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, ft, 1, &[]).expect("level 3");
    upkeep(&mut g);
    let oct = named(&g, 0, "Octopus");
    assert_eq!(oct.len(), 1);
    assert_eq!(pt(&g, oct[0]), (8, 8));
}

/// Ghirapur Orrery refills an empty hand at its owner's upkeep.
#[test]
fn ghirapur_orrery_refills_empty_hands() {
    let mut g = main_phase(2);
    library(&mut g, 1, 4);
    g.add_card_to_battlefield(0, catalog::ghirapur_orrery());
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 3);
}

/// Kwain's offer: the takers draw and gain 1 life.
#[test]
fn kwain_offers_everyone_a_card() {
    let mut g = main_phase(2);
    library(&mut g, 0, 2);
    library(&mut g, 1, 2);
    let k = g.add_card_to_battlefield(0, catalog::kwain_itinerant_meddler());
    g.clear_sickness(k);
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false), DecisionAnswer::Bool(true)]));
    activate(&mut g, k, 0, &[]).expect("activate");
    assert_eq!(g.players[0].hand.len(), 0);
    assert_eq!((g.players[1].hand.len(), g.players[1].life), (1, life + 1));
}

/// CR 701.15 — Martial Impetus goads its host and pumps the attack.
#[test]
fn cr_701_15_martial_impetus_goads_and_pumps() {
    let mut g = main_phase(3);
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mi = g.add_card_to_hand(0, catalog::martial_impetus());
    cast_at(&mut g, mi, &[Target::Permanent(theirs)]).expect("cast");
    assert_eq!(pt(&g, theirs), (3, 3));
    assert!(g.battlefield_find(theirs).unwrap().goaded_by.contains(&0));
}

/// Mr. Foxglove draws up to the defender's hand size.
#[test]
fn mr_foxglove_draws_up_to_the_defenders_hand() {
    let mut g = main_phase(2);
    library(&mut g, 0, 5);
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::plains());
    }
    let mf = g.add_card_to_battlefield(0, catalog::mr_foxglove());
    attack_with(&mut g, &[mf], 1);
    assert_eq!(g.players[0].hand.len(), 3);
}

/// Octomancer copies a creature token that entered this turn at the end
/// step.
#[test]
fn octomancer_copies_a_fresh_token() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::octomancer());
    let kitt = g.add_card_to_hand(0, catalog::kitt_kanto_mayhem_diva());
    cast_at(&mut g, kitt, &[]).expect("a Citizen token");
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Citizen").len(), 2);
}

/// Perch Protection makes four fliers and exiles itself.
#[test]
fn perch_protection_makes_birds() {
    let mut g = main_phase(2);
    let pp = g.add_card_to_hand(0, catalog::perch_protection());
    cast_at(&mut g, pp, &[]).expect("cast");
    let birds = named(&g, 0, "Bird");
    assert_eq!(birds.len(), 4);
    assert!(g.computed_permanent(birds[0]).unwrap().keywords().contains(&Keyword::Flying));
    assert!(g.exile.iter().any(|c| c.id == pp));
}

/// Promise of Loyalty leaves each player one vowed creature that can't
/// attack its caster.
#[test]
fn promise_of_loyalty_keeps_one_vowed_creature_each() {
    let mut g = main_phase(3);
    for p in 1..3 {
        g.add_card_to_battlefield(p, catalog::grizzly_bears());
        g.add_card_to_battlefield(p, catalog::serra_angel());
    }
    let pl = g.add_card_to_hand(0, catalog::promise_of_loyalty());
    cast_at(&mut g, pl, &[]).expect("cast");
    for p in 1..3 {
        let left: Vec<CardId> = g.battlefield.iter().filter(|c| c.controller == p && c.definition.is_creature()).map(|c| c.id).collect();
        assert_eq!(left.len(), 1);
        assert_eq!(g.battlefield_find(left[0]).unwrap().counter_count(CounterType::Vow), 1);
        assert!(g.computed_permanent(left[0]).unwrap().keywords().contains(&Keyword::CantAttackPlayer(0)));
    }
}

/// CR 702.175 — Steelburr Champion's offspring makes a 1/1 copy; an
/// opponent's noncreature spell grows it.
#[test]
fn cr_702_175_steelburr_champion_offspring_and_growth() {
    let mut g = main_phase(2);
    let sc = g.add_card_to_hand(0, catalog::steelburr_champion());
    act(&mut g, GameAction::CastSpellKicked { card_id: sc, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("offspring");
    assert_eq!(named(&g, 0, "Steelburr Champion").len(), 2);
    let shock = g.add_card_to_hand(1, catalog::shock());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: shock, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("shock");
    drain_stack(&mut g);
    assert_eq!(plus(&g, sc), 1);
}

/// Tamiyo's −2 taps two permanents that skip their next untap; −7 leaves a
/// free-casting emblem.
#[test]
fn tamiyo_locks_down_and_ults() {
    let mut g = main_phase(2);
    library(&mut g, 0, 4);
    let t = g.add_card_to_battlefield(0, catalog::tamiyo_field_researcher());
    let a = g.add_card_to_battlefield(1, catalog::serra_angel());
    act(&mut g, GameAction::ActivateLoyaltyAbility { card_id: t, ability_index: 1, target: Some(Target::Permanent(a)), x_value: None })
        .expect("-2");
    assert!(g.battlefield_find(a).unwrap().tapped);

    let mut g = main_phase(2);
    library(&mut g, 0, 4);
    let t = g.add_card_to_battlefield(0, catalog::tamiyo_field_researcher());
    g.battlefield_find_mut(t).unwrap().add_counters(CounterType::Loyalty, 10);
    act(&mut g, GameAction::ActivateLoyaltyAbility { card_id: t, ability_index: 2, target: None, x_value: None })
        .expect("-7");
    assert_eq!(g.players[0].hand.len(), 3);
    assert_eq!(g.players[0].emblems.len(), 1);
}

/// Tenuous Truce: both draw at the enchanted opponent's end step; attacking
/// them breaks it.
#[test]
fn tenuous_truce_draws_until_broken() {
    let mut g = main_phase(3);
    library(&mut g, 0, 3);
    library(&mut g, 1, 3);
    let tt = g.add_card_to_hand(0, catalog::tenuous_truce());
    cast_at(&mut g, tt, &[Target::Player(1)]).expect("cast");
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!((g.players[0].hand.len(), g.players[1].hand.len()), (1, 1));
    g.active_player_idx = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack_with(&mut g, &[bear], 1);
    assert!(g.battlefield_find(tt).is_none(), "attacking them breaks the truce");
}

/// The Thriving lands tap for their color or a chosen other one.
#[test]
fn thriving_lands_choose_another_color() {
    for (def, color) in [
        (catalog::thriving_grove as fn() -> _, Color::Green),
        (catalog::thriving_heath, Color::White),
        (catalog::thriving_isle, Color::Blue),
    ] {
        let mut g = main_phase(2);
        let land = g.add_card_to_hand(0, def());
        act(&mut g, GameAction::PlayLand(land)).expect("play");
        let c = g.battlefield_find(land).unwrap();
        assert!(c.tapped);
        assert!(c.chosen_color.is_some_and(|k| k != color));
    }
}

/// Twenty-Toed Toad grows on a two-creature attack and wins at twenty.
#[test]
fn twenty_toed_toad_grows_and_wins() {
    let mut g = main_phase(2);
    library(&mut g, 0, 3);
    let toad = g.add_card_to_battlefield(0, catalog::twenty_toed_toad());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack_with(&mut g, &[toad, bear], 1);
    assert_eq!(plus(&g, toad), 1);
    assert_eq!(g.players[0].hand.len(), 1);

    let mut g = main_phase(2);
    let toad = g.add_card_to_battlefield(0, catalog::twenty_toed_toad());
    g.battlefield_find_mut(toad).unwrap().add_counters(CounterType::PlusOnePlusOne, 20);
    attack_with(&mut g, &[toad], 1);
    assert!(g.players[1].eliminated || g.is_game_over(), "twenty counters win");
}
