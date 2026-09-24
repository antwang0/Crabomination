//! Commander: the Vampiric Bloodline precon (VOC, Strefan, `decks::cmdr_strefan`).

use crabomination::card::{CardId, Keyword};
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

fn cast_with(g: &mut GameState, id: CardId, targets: &[Target], mode: Option<usize>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    cast_with(g, id, targets, None).expect("cast");
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

fn connect(g: &mut GameState, attackers: &[CardId], defender: usize) {
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
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = defender;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(g);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// A Blood token per opponent; each is an Equipment with equip {2} and +2/+0.
#[test]
fn arterial_alchemy_turns_blood_into_equipment() {
    let mut g = main_phase(3);
    let aa = g.add_card_to_hand(0, catalog::arterial_alchemy());
    cast(&mut g, aa, &[]);
    let bloods: Vec<CardId> =
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Blood").map(|c| c.id).collect();
    assert_eq!(bloods.len(), 2, "one per opponent");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: bloods[0], target: bear }).expect("equip {2}");
    assert_eq!(pt(&g, bear), (4, 2));
}

/// CR 702.35 — a Vampire card discarded with Falkenrath Gorger out is cast
/// for its madness cost (its own mana cost) when the pool can pay.
#[test]
fn falkenrath_gorger_gives_vampires_madness() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::falkenrath_gorger());
    let outlet = g.add_card_to_battlefield(0, catalog::stromkirk_condemned());
    let vamp = g.add_card_to_hand(0, catalog::stromkirk_condemned());
    activate(&mut g, outlet, 0, None).expect("discard the Vampire");
    assert!(g.battlefield_find(vamp).is_some(), "cast for madness from exile");

    // Without the Gorger the same discard is just a discard.
    let mut g = main_phase(2);
    let outlet = g.add_card_to_battlefield(0, catalog::stromkirk_condemned());
    let vamp = g.add_card_to_hand(0, catalog::stromkirk_condemned());
    activate(&mut g, outlet, 0, None).expect("discard");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == vamp));
}

#[test]
fn imposing_grandeur_refills_to_the_commanders_mana_value() {
    let mut g = main_phase(2);
    g.seat_commanders(0, vec![catalog::hill_giant()]);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::swamp());
        g.add_card_to_library(1, catalog::swamp());
    }
    g.add_card_to_hand(0, catalog::swamp());
    let theirs = g.add_card_to_hand(1, catalog::swamp());
    let ig = g.add_card_to_hand(0, catalog::imposing_grandeur());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(false)]));
    cast(&mut g, ig, &[]);
    assert_eq!(g.players[0].hand.len(), 4, "discarded one, drew Hill Giant's four");
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs), "the other player declined");
}

#[test]
fn kamber_profits_from_an_opponents_dead() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::kamber_the_plunderer());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(victim)]);
    assert_eq!(g.players[0].life, life + 1);
    assert_eq!(count_named(&g, 0, "Blood"), 1);
}

#[test]
fn laurine_goads_for_an_artifact() {
    let mut g = main_phase(2);
    let laurine = g.add_card_to_battlefield(0, catalog::laurine_the_diversion());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, laurine, 0, Some(Target::Permanent(theirs))).expect("goad");
    assert!(g.battlefield_find(theirs).is_some_and(|c| !c.goaded_by.is_empty()));
}

/// Fights on its own entry and each other Vampire's, and a creature it
/// damaged dying makes Blood.
#[test]
fn markov_enforcer_fights_as_vampires_arrive() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let me = g.add_card_to_hand(0, catalog::markov_enforcer());
    cast(&mut g, me, &[]);
    assert!(g.battlefield_find(bear).is_none(), "fought on entry");
    assert_eq!(count_named(&g, 0, "Blood"), 1, "the Bears it damaged died");
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let vamp = g.add_card_to_hand(0, catalog::stromkirk_condemned());
    cast(&mut g, vamp, &[]);
    assert!(g.battlefield_find(giant).is_none(), "fought again as another Vampire entered");
}

/// X = Vampires you control; artifacts with mana abilities can't be targeted.
#[test]
fn midnight_arsonist_burns_artifacts_without_mana_abilities() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::stromkirk_condemned());
    let thopter = g.add_card_to_battlefield(1, catalog::ornithopter());
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let ma = g.add_card_to_hand(0, catalog::midnight_arsonist());
    assert!(cast_with(&mut g, ma, &[Target::Permanent(ring)], None).is_err() || g.battlefield_find(ring).is_some());
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::stromkirk_condemned());
    let thopter2 = g.add_card_to_battlefield(1, catalog::ornithopter());
    let ma = g.add_card_to_hand(0, catalog::midnight_arsonist());
    cast(&mut g, ma, &[Target::Permanent(thopter2)]);
    assert!(g.battlefield_find(thopter2).is_none());
    let _ = thopter;
}

#[test]
fn mob_rule_steals_the_big_ones() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(1, catalog::serra_angel());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mr = g.add_card_to_hand(0, catalog::mob_rule());
    cast_with(&mut g, mr, &[], Some(0)).expect("mode 0");
    assert_eq!(g.battlefield_find(giant).map(|c| c.controller), Some(0));
    assert_eq!(g.battlefield_find(bear).map(|c| c.controller), Some(1));
}

#[test]
fn predators_hour_steals_the_top_card_on_a_hit() {
    let mut g = main_phase(2);
    let top = g.add_card_to_library(1, catalog::grizzly_bears());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ph = g.add_card_to_hand(0, catalog::predators_hour());
    cast(&mut g, ph, &[]);
    assert!(g.permanent_has_keyword(bear, &Keyword::Menace));
    connect(&mut g, &[bear], 1);
    assert!(g.exile.iter().any(|c| c.id == top), "exiled");
    g.step = TurnStep::PostCombatMain;
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: top,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the stolen card, any mana");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(top).map(|c| c.controller), Some(0));
}

#[test]
fn scion_of_opulence_mints_treasure_for_fallen_vampires() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::scion_of_opulence());
    let vamp = g.add_card_to_battlefield(0, catalog::stromkirk_condemned());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(vamp)]);
    assert_eq!(count_named(&g, 0, "Treasure"), 1);
}

/// Each opponent sacrifices their biggest; you gain the biggest power given up.
#[test]
fn shadowgrange_archfiend_takes_each_opponents_best() {
    let mut g = main_phase(3);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(2, catalog::hill_giant());
    let life = g.players[0].life;
    let sa = g.add_card_to_hand(0, catalog::shadowgrange_archfiend());
    cast(&mut g, sa, &[]);
    assert!(g.battlefield_find(angel).is_none() && g.battlefield_find(giant).is_none());
    assert!(g.battlefield_find(bear1).is_some());
    assert_eq!(g.players[0].life, life + 4, "the Angel's 4 is the greatest");
}

#[test]
fn sinister_waltz_returns_two_of_three_at_random() {
    let mut g = main_phase(2);
    let ids: Vec<CardId> = (0..3).map(|_| g.add_card_to_graveyard(0, catalog::grizzly_bears())).collect();
    let sw = g.add_card_to_hand(0, catalog::sinister_waltz());
    cast(&mut g, sw, &ids.iter().map(|&i| Target::Permanent(i)).collect::<Vec<_>>());
    let back = ids.iter().filter(|&&i| g.battlefield_find(i).is_some()).count();
    let bottom = ids.iter().filter(|&&i| g.players[0].library.iter().any(|c| c.id == i)).count();
    assert_eq!((back, bottom), (2, 1));
}

/// Blood per player who lost life; attacking, two Blood buy a Vampire from
/// hand into the attack, indestructible.
#[test]
fn strefan_brings_vampires_to_the_fight() {
    let mut g = main_phase(3);
    let strefan = g.add_card_to_battlefield(0, catalog::strefan_maurer_progenitor());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(1)]);
    g.players[0].life -= 1;
    g.players[0].lost_life_this_turn = true;
    for s in 0..3 {
        g.add_card_to_library(s, catalog::swamp());
        g.add_card_to_library(s, catalog::swamp());
    }
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Blood"), 2, "seat 1 and you lost life");
    let vamp = g.add_card_to_hand(0, catalog::stromkirk_condemned());
    g.step = TurnStep::PreCombatMain;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Cards(vec![vamp])]));
    g.clear_sickness(strefan);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: strefan, target: AttackTarget::Player(2) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Blood"), 0);
    assert!(g.attacking().iter().any(|a| a.attacker == vamp), "tapped and attacking");
    assert!(g.permanent_has_keyword(vamp, &Keyword::Indestructible));
}

#[test]
fn stromkirk_condemned_pumps_the_bloodline_once_a_turn() {
    let mut g = main_phase(2);
    let sc = g.add_card_to_battlefield(0, catalog::stromkirk_condemned());
    let other = g.add_card_to_battlefield(0, catalog::falkenrath_gorger());
    g.add_card_to_hand(0, catalog::swamp());
    g.add_card_to_hand(0, catalog::swamp());
    activate(&mut g, sc, 0, None).expect("discard");
    assert_eq!(pt(&g, sc), (3, 3));
    assert_eq!(pt(&g, other), (3, 2));
    assert!(activate(&mut g, sc, 0, None).is_err(), "once each turn");
}

#[test]
fn stromkirk_occultist_impulses_on_a_hit() {
    let mut g = main_phase(2);
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    let so = g.add_card_to_battlefield(0, catalog::stromkirk_occultist());
    connect(&mut g, &[so], 1);
    assert!(g.exile.iter().any(|c| c.id == top));
}

/// Bug fix (CR 601.2c): the bot aimed both halves of a divided-damage spell
/// at the same face — or the second at its own — so the cast was rejected
/// and Avacyn's Judgment / Forked Bolt went unplayed in 200 pods.
#[test]
fn the_bot_casts_a_divided_damage_spell_at_distinct_hostile_targets() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    for card in [catalog::avacyns_judgment(), catalog::forked_bolt()] {
        let mut g = main_phase(2);
        let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
        g.players[1].life = 2;
        g.players[0].hostile_player_targets = true;
        flood(&mut g, 0);
        let id = g.add_card_to_hand(0, card);
        let action = HeuristicBot::new().next_action(&g, 0);
        assert!(
            matches!(&action, Some(GameAction::CastSpell { card_id, target: Some(Target::Player(1)), additional_targets, .. })
                if *card_id == id && additional_targets == &vec![Target::Permanent(elf)]),
            "{action:?}"
        );
    }
}
