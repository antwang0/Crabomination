//! Commander: the Built From Scratch precon (C14, Daretti, `decks::cmdr_daretti`).

use crabomination::card::{CardId, CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn cast_with(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    cast_with(g, id, targets).expect("cast");
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
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

fn to_end_step(g: &mut GameState) {
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

/// CR 614.5 — damage between the two chosen players doubles both ways; a
/// third player is untouched.
#[test]
fn bitter_feud_doubles_damage_between_you_and_the_chosen_player() {
    let mut g = main_phase(3);
    let feud = g.add_card_to_hand(0, catalog::bitter_feud());
    cast(&mut g, feud, &[]);
    let rival = g.battlefield_find(feud).and_then(|c| c.chosen_player).expect("chose a player");
    let other = if rival == 1 { 2 } else { 1 };
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(rival)]);
    assert_eq!(g.players[rival].life, 20 - 6);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(other)]);
    assert_eq!(g.players[other].life, 20 - 3);
}

/// Cast from hand it is a 1/1; dying, it goes to exile with three time
/// counters and suspend. Put onto the battlefield otherwise, it is a 4/4.
#[test]
fn epochrasite_returns_bigger_through_suspend() {
    let mut g = main_phase(2);
    let e = g.add_card_to_hand(0, catalog::epochrasite());
    cast(&mut g, e, &[]);
    assert_eq!(pt(&g, e), (1, 1), "cast from hand: no counters");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(e)]);
    let exiled = g.exile.iter().find(|c| c.id == e).expect("exiled, not in the graveyard");
    assert_eq!(exiled.counter_count(CounterType::Time), 3);

    let mut g = main_phase(2);
    let e = g.add_card_to_graveyard(0, catalog::epochrasite());
    let reanimate = g.add_card_to_hand(0, catalog::reanimate());
    cast(&mut g, reanimate, &[Target::Permanent(e)]);
    assert_eq!(pt(&g, e), (4, 4), "not cast: three counters");
}

#[test]
fn feldon_borrows_a_hasty_artifact_copy_for_the_turn() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let feldon = g.add_card_to_battlefield(0, catalog::feldon_of_the_third_path());
    g.clear_sickness(feldon);
    activate(&mut g, feldon, 0, Some(Target::Permanent(giant))).expect("activate");
    let copy = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Hill Giant").map(|c| c.id);
    let copy = copy.expect("a token copy");
    let cp = g.computed_permanent(copy).expect("copy");
    assert!(cp.card_types().contains(&CardType::Artifact));
    assert!(cp.keywords().contains(&Keyword::Haste));
    to_end_step(&mut g);
    assert!(g.battlefield_find(copy).is_none(), "sacrificed at the end step");
}

#[test]
fn hoard_smelter_dragon_melts_an_artifact_for_its_mana_value() {
    let mut g = main_phase(2);
    let d = g.add_card_to_battlefield(0, catalog::hoard_smelter_dragon());
    let lotus = g.add_card_to_battlefield(1, catalog::gilded_lotus());
    activate(&mut g, d, 0, Some(Target::Permanent(lotus))).expect("activate");
    assert!(g.battlefield_find(lotus).is_none());
    assert_eq!(pt(&g, d), (10, 5));
}

/// X is the largest single-source hit this turn — five from Lava Axe.
#[test]
fn impact_resonance_echoes_the_turns_biggest_hit() {
    let mut g = main_phase(2);
    let axe = g.add_card_to_hand(0, catalog::lava_axe());
    cast(&mut g, axe, &[Target::Player(1)]);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ir = g.add_card_to_hand(0, catalog::impact_resonance());
    cast(&mut g, ir, &[Target::Permanent(giant), Target::Permanent(bear)]);
    assert!(g.battlefield_find(giant).is_none() && g.battlefield_find(bear).is_none(), "5 split 3/2 kills both");
}

#[test]
fn incite_rebellion_charges_each_player_for_their_own_army() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::hill_giant());
    g.add_card_to_battlefield(0, catalog::hill_giant());
    let lone = g.add_card_to_battlefield(1, catalog::hill_giant());
    let ir = g.add_card_to_hand(0, catalog::incite_rebellion());
    cast(&mut g, ir, &[]);
    assert_eq!(g.players[0].life, 18);
    assert_eq!(g.players[1].life, 19);
    assert!(g.battlefield.iter().filter(|c| c.controller == 0).all(|c| c.damage == 2));
    assert_eq!(g.battlefield_find(lone).map(|c| c.damage), Some(1));
}

#[test]
fn liquimetal_coating_makes_a_land_an_artifact() {
    let mut g = main_phase(2);
    let coat = g.add_card_to_battlefield(0, catalog::liquimetal_coating());
    let land = g.add_card_to_battlefield(1, catalog::island());
    activate(&mut g, coat, 0, Some(Target::Permanent(land))).expect("activate");
    let cp = g.computed_permanent(land).expect("land");
    assert!(cp.card_types().contains(&CardType::Artifact) && cp.card_types().contains(&CardType::Land));
}

#[test]
fn panic_spellbomb_stops_a_blocker_and_cycles() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::mountain());
    let bomb = g.add_card_to_battlefield(0, catalog::panic_spellbomb());
    let wall = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    activate(&mut g, bomb, 0, Some(Target::Permanent(wall))).expect("activate");
    assert!(g.permanent_has_keyword(wall, &Keyword::CantBlock));
    assert_eq!(g.players[0].hand.len(), 1, "paid {{R}} and drew");
}

/// Every player exiles their artifact cards, sacrifices their artifacts,
/// then gets the exiled ones back.
#[test]
fn scrap_mastery_swaps_each_players_artifacts() {
    let mut g = main_phase(2);
    let on_board = g.add_card_to_battlefield(1, catalog::ornithopter());
    let mine_gy = g.add_card_to_graveyard(0, catalog::gilded_lotus());
    let theirs_gy = g.add_card_to_graveyard(1, catalog::memnite());
    let sm = g.add_card_to_hand(0, catalog::scrap_mastery());
    cast(&mut g, sm, &[]);
    assert!(g.battlefield_find(on_board).is_none());
    assert!(g.players[1].graveyard.iter().any(|c| c.id == on_board), "sacrificed after the exile step");
    assert_eq!(g.battlefield_find(mine_gy).map(|c| c.controller), Some(0));
    assert_eq!(g.battlefield_find(theirs_gy).map(|c| c.controller), Some(1));
}

/// Evoked, it is sacrificed on entry and its leave trigger still fires.
#[test]
fn spitebellows_evoked_is_a_six_damage_removal_spell() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let sb = g.add_card_to_hand(0, catalog::spitebellows());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: sb,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("evoke");
    drain_stack(&mut g);
    assert!(g.battlefield_find(sb).is_none(), "evoked: sacrificed");
    assert!(g.battlefield_find(giant).is_none(), "six damage to the Giant");

    // Hard-cast and killed, the same trigger.
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let sb = g.add_card_to_hand(0, catalog::spitebellows());
    cast(&mut g, sb, &[]);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(sb)]);
    assert!(g.battlefield_find(giant).is_none());
}

#[test]
fn trading_post_trades() {
    let mut g = main_phase(2);
    let post = g.add_card_to_battlefield(0, catalog::trading_post());
    g.add_card_to_hand(0, catalog::island());
    let life = g.players[0].life;
    activate(&mut g, post, 0, None).expect("discard for life");
    assert_eq!(g.players[0].life, life + 4);
    assert!(g.players[0].hand.is_empty());
    g.battlefield_find_mut(post).expect("post").tapped = false;
    activate(&mut g, post, 1, None).expect("life for a Goat");
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Goat"));
    g.battlefield_find_mut(post).expect("post").tapped = false;
    g.add_card_to_library(0, catalog::island());
    activate(&mut g, post, 3, None).expect("the Post itself for a card");
    assert!(g.battlefield_find(post).is_none());
    assert_eq!(g.players[0].hand.len(), 1);
}

/// CR 207.2c lieutenant — the attack trigger and the +2/+2 need your
/// commander on the battlefield.
#[test]
fn tyrants_familiar_is_a_lieutenant() {
    let mut g = main_phase(2);
    let cmdr = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    let fam = g.add_card_to_battlefield(0, catalog::tyrants_familiar());
    assert_eq!(pt(&g, fam), (5, 5));
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmdr,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("cast commander");
    drain_stack(&mut g);
    assert_eq!(pt(&g, fam), (7, 7));
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.clear_sickness(fam);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: fam,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "seven to a creature the defender controls");
}

/// The opponent's-choice halves are made by an opponent, and never hit the
/// caster's own permanents ("you don't control" reads the caster).
#[test]
fn volcanic_offering_makes_an_opponent_pick_the_second_victims() {
    let mut g = main_phase(3);
    let my_land = g.add_card_to_battlefield(0, catalog::command_tower());
    let my_giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let lands: Vec<CardId> = (1..3).map(|s| g.add_card_to_battlefield(s, catalog::command_tower())).collect();
    let giants: Vec<CardId> = (1..3).map(|s| g.add_card_to_battlefield(s, catalog::hill_giant())).collect();
    let vo = g.add_card_to_hand(0, catalog::volcanic_offering());
    cast(&mut g, vo, &[Target::Permanent(lands[0]), Target::Permanent(giants[0])]);
    assert!(g.battlefield_find(my_land).is_some() && g.battlefield_find(my_giant).is_some());
    assert!(lands.iter().all(|&l| g.battlefield_find(l).is_none()), "the target and the opponent's pick");
    assert!(giants.iter().all(|&c| g.battlefield_find(c).is_none()));
}

#[test]
fn warmonger_hellkite_makes_every_creature_attack() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::warmonger_hellkite());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(g.permanent_has_keyword(theirs, &Keyword::MustAttack));
}

/// Split second: nobody can respond; the stolen permanent is untapped and
/// hasty for the turn.
#[test]
fn word_of_seizing_steals_any_permanent() {
    let mut g = main_phase(2);
    let lotus = g.add_card_to_battlefield(1, catalog::gilded_lotus());
    g.battlefield_find_mut(lotus).expect("lotus").tapped = true;
    let w = g.add_card_to_hand(0, catalog::word_of_seizing());
    cast(&mut g, w, &[Target::Permanent(lotus)]);
    let c = g.battlefield_find(lotus).expect("lotus");
    assert_eq!(c.controller, 0);
    assert!(!c.tapped);
}
