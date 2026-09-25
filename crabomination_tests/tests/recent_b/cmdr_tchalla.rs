//! Commander: the Wakanda Forever precon (MSC, T'Challa, the Black Panther,
//! `decks::cmdr_tchalla`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
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

fn act(g: &mut GameState, seat: usize, a: GameAction) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(a).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    act(
        g,
        0,
        GameAction::CastSpell {
            card_id: id,
            target: targets.first().cloned(),
            additional_targets: targets.iter().skip(1).cloned().collect(),
            mode: None,
            x_value: x,
        },
    )
}

fn activate(g: &mut GameState, id: CardId, index: usize) -> Result<(), String> {
    act(
        g,
        0,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        },
    )
}

fn fire(g: &mut GameState, step: TurnStep) {
    g.step = step;
    g.fire_step_triggers(step);
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

/// T'Challa makes a tapped, indestructible Vibranium on entry and grows with
/// each artifact spell of mana value 4 or greater.
#[test]
fn tchalla_makes_vibranium_and_grows() {
    let mut g = main_phase(2);
    let t = g.add_card_to_hand(0, catalog::tchalla_the_black_panther());
    flood(&mut g, 0);
    cast(&mut g, t, &[], None).expect("cast");
    let v = named(&g, 0, "Vibranium");
    assert_eq!(v.len(), 1);
    let c = g.battlefield_find(v[0]).unwrap();
    assert!(c.tapped && c.definition.keywords.contains(&Keyword::Indestructible));
    let herb = g.add_card_to_hand(0, catalog::heart_shaped_herb());
    flood(&mut g, 0);
    cast(&mut g, herb, &[], None).expect("a four-mana artifact");
    assert_eq!(g.battlefield_find(t).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Bast can't attack or block with fewer than three creatures.
#[test]
fn bast_needs_company() {
    let mut g = main_phase(2);
    let bast = g.add_card_to_battlefield(0, catalog::bast_panther_goddess());
    assert!(g.computed_permanent(bast).unwrap().keywords().contains(&Keyword::CantAttack));
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bast).unwrap().keywords().contains(&Keyword::CantAttack));
}

/// Queen Mother Ramonda: while she keeps the crown, power-2 creatures can't
/// attack her controller (CR 508.1c through a gated static).
#[test]
fn ramonda_turns_away_small_attackers() {
    let mut g = main_phase(2);
    let r = g.add_card_to_hand(1, catalog::queen_mother_ramonda());
    flood(&mut g, 1);
    g.active_player_idx = 1;
    act(&mut g, 1, GameAction::CastSpell { card_id: r, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    assert_eq!(g.monarch, Some(1));
    g.active_player_idx = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    g.clear_sickness(bear);
    g.clear_sickness(wurm);
    g.step = TurnStep::DeclareAttackers;
    assert!(
        act(&mut g, 0, GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
            .is_err()
    );
    act(&mut g, 0, GameAction::DeclareAttackers(vec![Attack { attacker: wurm, target: AttackTarget::Player(1) }]))
        .expect("a 6-power attacker is fine");
}

/// Panther Habit turns damage to the equipped creature into +1/+1 counters.
#[test]
fn panther_habit_absorbs_damage() {
    let mut g = main_phase(2);
    let habit = g.add_card_to_battlefield(0, catalog::panther_habit());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(habit).unwrap().attached_to = Some(bear);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast(&mut g, bolt, &[Target::Permanent(bear)], None).expect("bolt");
    let c = g.battlefield_find(bear).expect("survives");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Kimoyo Beads: three end steps, three different beads.
#[test]
fn kimoyo_beads_uses_each_bead_once() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::kimoyo_beads());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
    }
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    for _ in 0..3 {
        fire(&mut g, TurnStep::End);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(named(&g, 0, "Soldier").len(), 2);
    assert_eq!(g.players[0].life, life + 3);
}

/// Wakanda Forever! puts the biggest permanent in with an indestructible
/// counter, the next into hand, and bins the rest.
#[test]
fn wakanda_forever_deploys_and_takes() {
    let mut g = main_phase(2);
    let wurm = g.add_card_to_library(0, catalog::craw_wurm());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
    }
    let spell = g.add_card_to_hand(0, catalog::wakanda_forever());
    flood(&mut g, 0);
    cast(&mut g, spell, &[], None).expect("cast");
    let c = g.battlefield_find(wurm).expect("deployed");
    assert_eq!(c.keyword_counters.get(&Keyword::Indestructible).copied(), Some(1));
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt));
}

/// Heart-Shaped Herb prevents 1 of each opposing source's damage to you.
#[test]
fn heart_shaped_herb_softens_burn() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::heart_shaped_herb());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    let life = g.players[0].life;
    act(&mut g, 1, GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    assert_eq!(g.players[0].life, life - 2);
}

/// Royal Talon Fighter Jet enters with X counters and makes that many
/// Soldiers.
#[test]
fn royal_talon_makes_soldiers_per_counter() {
    let mut g = main_phase(2);
    let jet = g.add_card_to_hand(0, catalog::royal_talon_fighter_jet());
    flood(&mut g, 0);
    cast(&mut g, jet, &[], Some(2)).expect("X = 2");
    assert_eq!(g.battlefield_find(jet).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(named(&g, 0, "Soldier").len(), 2);
}

/// Scourglass works only in your upkeep, and spares artifacts and lands.
#[test]
fn scourglass_is_an_upkeep_wrath() {
    let mut g = main_phase(2);
    let glass = g.add_card_to_battlefield(0, catalog::scourglass());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    assert!(activate(&mut g, glass, 0).is_err(), "not in the main phase");
    g.step = TurnStep::Upkeep;
    activate(&mut g, glass, 0).expect("upkeep");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(ring).is_some());
}

/// King Solomon's Frogs exiles an opponent's big permanent; they draw.
#[test]
fn king_solomons_frogs_trades_a_card_for_a_permanent() {
    let mut g = main_phase(2);
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.add_card_to_library(1, catalog::forest());
    let hand = g.players[1].hand.len();
    let frogs = g.add_card_to_hand(0, catalog::king_solomons_frogs());
    flood(&mut g, 0);
    cast(&mut g, frogs, &[Target::Permanent(wurm)], None).expect("cast");
    assert!(g.battlefield_find(wurm).is_none());
    assert_eq!(g.players[1].hand.len(), hand + 1);
}

/// M'Baku crowns an opponent at your end step when nobody is monarch.
#[test]
fn mbaku_crowns_an_opponent() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::mbaku_jabari_chieftain());
    fire(&mut g, TurnStep::End);
    assert_eq!(g.monarch, Some(1));
}

/// T'Chaka mills three and keeps an artifact or land from them.
#[test]
fn tchaka_keeps_an_artifact_or_land() {
    let mut g = main_phase(2);
    let ring = g.add_card_to_library(0, catalog::sol_ring());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::tchaka_venerable_king());
    flood(&mut g, 0);
    cast(&mut g, t, &[], None).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == ring));
    assert_eq!(g.players[0].graveyard.len(), 2);
}

/// Heart-Shaped Herb against an attacking creature: 2 damage becomes 1.
#[test]
fn heart_shaped_herb_softens_combat_damage() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::heart_shaped_herb());
    g.active_player_idx = 1;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    act(&mut g, 1, GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]))
        .expect("attack");
    let life = g.players[0].life;
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, life - 1);
}
