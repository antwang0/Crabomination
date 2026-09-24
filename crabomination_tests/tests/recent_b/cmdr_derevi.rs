//! Commander: the Evasive Maneuvers precon (C13, Derevi, `decks::cmdr_derevi`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = if n == 2 { two_player_game() } else { multi_player_game(n) };
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

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    cast_x(g, id, targets, None);
}

fn activate(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn attack(g: &mut GameState, attacker: CardId, defender: usize) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(defender) }]))
        .expect("attack");
    drain_stack(g);
}

/// Curse of Predation and Curse of the Forsaken: a creature attacking the
/// cursed player grows, and its controller gains 1.
#[test]
fn curses_reward_attacking_the_cursed_player() {
    let mut g = pod(3);
    let p = g.add_card_to_hand(0, catalog::curse_of_predation());
    cast(&mut g, p, &[Target::Player(1)]);
    let f = g.add_card_to_hand(0, catalog::curse_of_the_forsaken());
    cast(&mut g, f, &[Target::Player(1)]);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    attack(&mut g, bear, 1);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].life, life + 1);
    // Attacking someone else triggers neither.
    let mut g = pod(3);
    let p = g.add_card_to_hand(0, catalog::curse_of_predation());
    cast(&mut g, p, &[Target::Player(1)]);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack(&mut g, bear, 2);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Control Magic takes the creature.
#[test]
fn control_magic_steals() {
    let mut g = pod(2);
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let cm = g.add_card_to_hand(0, catalog::control_magic());
    cast(&mut g, cm, &[Target::Permanent(wurm)]);
    assert_eq!(g.battlefield_find(wurm).unwrap().controller, 0);
}

/// Unexpectedly Absent with X = 2 tucks the permanent third from the top.
#[test]
fn unexpectedly_absent_tucks_beneath_x() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::island());
    }
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let ua = g.add_card_to_hand(0, catalog::unexpectedly_absent());
    cast_x(&mut g, ua, &[Target::Permanent(wurm)], Some(2));
    assert_eq!(g.players[1].library.get(2).map(|c| c.id), Some(wurm));
}

/// Surveyor's Scope finds a basic per player at least two lands ahead.
#[test]
fn surveyors_scope_counts_players_ahead() {
    let mut g = pod(3);
    for _ in 0..2 {
        g.add_card_to_battlefield(1, catalog::plains());
    }
    g.add_card_to_battlefield(2, catalog::plains());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let scope = g.add_card_to_battlefield(0, catalog::surveyors_scope());
    activate(&mut g, scope, &[]).expect("scope");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Forest").count(), 1);
}

/// Hada Spy Patrol: level 1 makes it a 2/2 that can't be blocked; level 3 a
/// 3/3 with shroud.
#[test]
fn hada_spy_patrol_levels() {
    let mut g = pod(2);
    let h = g.add_card_to_battlefield(0, catalog::hada_spy_patrol());
    activate(&mut g, h, &[]).expect("level 1");
    let cp = g.computed_permanent(h).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.keywords().contains(&Keyword::Unblockable));
    activate(&mut g, h, &[]).expect("level 2");
    activate(&mut g, h, &[]).expect("level 3");
    let cp = g.computed_permanent(h).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords().contains(&Keyword::Shroud));
}

/// Diviner Spirit: you and the damaged player each draw that many.
#[test]
fn diviner_spirit_shares_the_draw() {
    let mut g = pod(2);
    for s in 0..2 {
        for _ in 0..3 {
            g.add_card_to_library(s, catalog::island());
        }
    }
    let ds = g.add_card_to_battlefield(0, catalog::diviner_spirit());
    attack(&mut g, ds, 1);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!(g.players[1].hand.len(), 2);
}

/// Djinn of Infinite Deceits swaps two creatures, but not during combat.
#[test]
fn djinn_swaps_outside_combat_only() {
    let mut g = pod(2);
    let djinn = g.add_card_to_battlefield(0, catalog::djinn_of_infinite_deceits());
    g.clear_sickness(djinn);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.step = TurnStep::BeginCombat;
    assert!(activate(&mut g, djinn, &[Target::Permanent(mine), Target::Permanent(theirs)]).is_err());
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, djinn, &[Target::Permanent(mine), Target::Permanent(theirs)]).expect("swap");
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1);
}

/// Borrowing 100,000 Arrows draws per tapped creature the opponent has.
#[test]
fn borrowing_arrows_counts_tapped_creatures() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    for tapped in [true, true, false] {
        let c = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(c).unwrap().tapped = tapped;
    }
    let b = g.add_card_to_hand(0, catalog::borrowing_100_000_arrows());
    cast(&mut g, b, &[Target::Player(1)]);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Skyward Eye Prophets: a land on top goes onto the battlefield, a spell to
/// hand.
#[test]
fn skyward_eye_prophets_sorts_the_top() {
    let mut g = pod(2);
    let p = g.add_card_to_battlefield(0, catalog::skyward_eye_prophets());
    g.clear_sickness(p);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let forest = g.add_card_to_library(0, catalog::forest());
    // `add_card_to_library` appends; the Forest is not on top yet.
    let top = g.players[0].library.first().map(|c| c.id);
    activate(&mut g, p, &[]).expect("tap");
    let moved = top.unwrap();
    if moved == forest {
        assert!(g.battlefield_find(forest).is_some());
    } else {
        assert!(g.players[0].hand.iter().any(|c| c.id == moved));
    }
}

/// Derevi's command-zone ability puts it onto the battlefield.
#[test]
fn derevi_enters_from_the_command_zone() {
    let mut g = pod(2);
    g.seat_commanders(0, vec![catalog::derevi_empyrial_tactician()]);
    let d = g.players[0].command.first().map(|c| c.id).expect("seated");
    activate(&mut g, d, &[]).expect("from the command zone");
    assert!(g.battlefield_find(d).is_some());
}

/// Restore takes a land from any graveyard.
#[test]
fn restore_reclaims_a_land() {
    let mut g = pod(2);
    let land = g.add_card_to_graveyard(1, catalog::forest());
    let r = g.add_card_to_hand(0, catalog::restore());
    cast(&mut g, r, &[Target::Permanent(land)]);
    assert_eq!(g.battlefield_find(land).map(|c| c.controller), Some(0));
}
