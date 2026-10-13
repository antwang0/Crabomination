//! Commander: EDHREC ranks 2001-2500, one-card gaps, fourth batch
//! (`decks::cmdr_edhrec58`).

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

fn fill_pool(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
    g.priority.player_with_priority = 0;
}

fn cast_at(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    fill_pool(g);
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    fill_pool(g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn stock(g: &mut GameState, seats: usize) {
    for seat in 0..seats {
        for _ in 0..4 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
}

/// Jaya Ballard discards to shoot 3 at any target.
#[test]
fn jaya_ballard_shoots_for_a_card() {
    let mut g = pod(2);
    let jaya = ready(&mut g, 0, catalog::jaya_ballard_task_mage());
    g.add_card_to_hand(0, catalog::island());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    activate(&mut g, jaya, 1, Some(Target::Permanent(giant))).expect("3 damage");
    assert!(g.battlefield_find(giant).is_none());
    assert!(g.players[0].hand.is_empty(), "the discard was paid");
}

/// Teyo gives your creature hexproof and a +1/+1 counter.
#[test]
fn teyo_shields_and_grows() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let teyo = g.add_card_to_hand(0, catalog::teyo_lightshield_expert());
    cast_at(&mut g, teyo, Some(Target::Permanent(bears))).expect("cast");
    let cp = g.computed_permanent(bears).unwrap();
    assert!(cp.keywords().contains(&Keyword::Hexproof));
    assert_eq!(cp.power, 3);
}

/// Black Widow's combat damage draws two for both players.
#[test]
fn black_widow_shares_intel() {
    let mut g = pod(2);
    stock(&mut g, 2);
    let widow = ready(&mut g, 0, catalog::black_widow_intel_expert());
    let (mine, theirs) = (g.players[0].hand.len(), g.players[1].hand.len());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: widow, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::FirstStrikeDamage;
    let ev = g.resolve_first_strike_damage().expect("first-strike damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), mine + 2);
    assert_eq!(g.players[1].hand.len(), theirs + 2);
}

/// Danitha draws once a turn off an Equipment spell.
#[test]
fn danitha_draws_off_equipment() {
    let mut g = pod(2);
    stock(&mut g, 1);
    ready(&mut g, 0, catalog::danitha_sword_of_hope());
    let gear = g.add_card_to_hand(0, catalog::vulshok_morningstar());
    let gear2 = g.add_card_to_hand(0, catalog::vulshok_morningstar());
    let hand = g.players[0].hand.len();
    cast_at(&mut g, gear, None).expect("gear");
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    cast_at(&mut g, gear2, None).expect("gear 2");
    assert_eq!(g.players[0].hand.len(), hand - 1, "once each turn");
}

/// Kang's power-up grows him and takes an extra turn (CR 702.193).
#[test]
fn kang_powers_up_into_an_extra_turn() {
    let mut g = pod(2);
    let kang = ready(&mut g, 0, catalog::kang_the_conqueror());
    activate(&mut g, kang, 0, None).expect("power-up");
    assert_eq!(g.battlefield_find(kang).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(activate(&mut g, kang, 0, None).is_err(), "power-up only once");
    assert_eq!(g.players[0].extra_turns, 1);
}

/// Felidar Umbra gives lifelink and moves for {1}{W}.
#[test]
fn felidar_umbra_lifelinks_and_moves() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let other = ready(&mut g, 0, catalog::hill_giant());
    let umbra = g.add_card_to_hand(0, catalog::felidar_umbra());
    cast_at(&mut g, umbra, Some(Target::Permanent(bears))).expect("aura");
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::Lifelink));
    activate(&mut g, umbra, 0, Some(Target::Permanent(other))).expect("move");
    assert_eq!(g.battlefield_find(umbra).unwrap().attached_to, Some(other));
}

/// Dream Salvage draws as many as the target opponent discarded this turn.
#[test]
fn dream_salvage_counts_discards() {
    let mut g = pod(2);
    stock(&mut g, 1);
    g.players[1].cards_discarded_this_turn = 2;
    let spell = g.add_card_to_hand(0, catalog::dream_salvage());
    let hand = g.players[0].hand.len();
    cast_at(&mut g, spell, Some(Target::Player(1))).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2);
}

/// Halana: pay {2} as a creature enters to have it fight one-sidedly.
#[test]
fn halana_pays_to_bite() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::halana_kessig_ranger());
    let giant = ready(&mut g, 1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let maw = g.add_card_to_hand(0, catalog::colossal_dreadmaw());
    fill_pool(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: maw, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    g.players[0].mana_pool.add_colorless(2);
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none(), "the Dreadmaw bit the Bears");
}

/// Bower Passage keeps fliers from blocking your creatures.
#[test]
fn bower_passage_grounds_fliers() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::bower_passage());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let glider = ready(&mut g, 1, catalog::hookblade_veteran());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    // Hookblade Veteran flies only on its controller's turn; give it flying.
    g.battlefield_find_mut(glider).unwrap().definition_make_mut().keywords.push(Keyword::Flying);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(glider, bears)])).is_err());
}

/// Idyllic Grange enters untapped with three other Plains and grows a
/// creature; tapped otherwise.
#[test]
fn idyllic_grange_rewards_plains() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let first = g.add_card_to_hand(0, catalog::idyllic_grange());
    g.perform_action(GameAction::PlayLand(first)).expect("land");
    drain_stack(&mut g);
    assert!(g.battlefield_find(first).unwrap().tapped);
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::plains());
    }
    g.players[0].lands_played_this_turn = 0;
    let second = g.add_card_to_hand(0, catalog::idyllic_grange());
    g.perform_action(GameAction::PlayLand(second)).expect("land");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(second).unwrap().tapped);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Dragonlord Silumgar steals a creature while you control it.
#[test]
fn dragonlord_silumgar_steals() {
    let mut g = pod(2);
    let maw = ready(&mut g, 1, catalog::colossal_dreadmaw());
    let silumgar = g.add_card_to_hand(0, catalog::dragonlord_silumgar());
    cast_at(&mut g, silumgar, Some(Target::Permanent(maw))).expect("cast");
    assert_eq!(g.battlefield_find(maw).unwrap().controller, 0);
    let mut evs = Vec::new();
    g.destroy_permanent(silumgar, false, &mut evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(maw).unwrap().controller, 1, "back once Silumgar leaves");
}

/// Maximus tutors a two-drop Equipment and turns artifacts into energy.
#[test]
fn maximus_tutors_and_energizes() {
    let mut g = pod(2);
    g.players[0].library.clear();
    let gear = g.add_card_to_library(0, catalog::vulshok_morningstar());
    let max = g.add_card_to_hand(0, catalog::maximus_knight_apparent());
    cast_at(&mut g, max, None).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == gear));
    ready(&mut g, 0, catalog::sol_ring());
    activate(&mut g, max, 0, None).expect("energy");
    assert_eq!(g.players[0].energy, 2);
}

/// Zuko, Conflicted: each first main phase picks a fresh mode and costs 2 life.
#[test]
fn zuko_conflicted_picks_a_new_mode_each_turn() {
    let mut g = pod(2);
    stock(&mut g, 1);
    let zuko = ready(&mut g, 0, catalog::zuko_conflicted());
    let life = g.players[0].life;
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2);
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 4);
    assert_eq!(g.battlefield_find(zuko).map(|c| c.counter_count(CounterType::PlusOnePlusOne)), Some(1), "a second, different mode");
}

/// Jaws of Stone divides one damage per Mountain.
#[test]
fn jaws_of_stone_counts_mountains() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::jaws_of_stone());
    cast_at(&mut g, spell, Some(Target::Permanent(giant))).expect("cast");
    assert!(g.battlefield_find(giant).is_none(), "three to the 3/3");
}
