//! Commander: the 2026-10-25 (`01M2Ffkh`) Living Lightning, Charged Up
//! closers (`decks::cmdr_01m2_b`).

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

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast_at(g: &mut GameState, id: CardId, target: Target) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(target),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate_at(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// CR 506.2 / 603.2c — Contested Game Ball: an opponent's attack that
/// connects hands it (untapped) to the attacking player, once however many
/// attackers connect.
#[test]
fn contested_game_ball_goes_to_the_attacking_player() {
    let mut g = pod(3);
    let ball = g.add_card_to_battlefield(0, catalog::contested_game_ball());
    g.battlefield_find_mut(ball).unwrap().tapped = true;
    let a = ready(&mut g, 2, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    g.active_player_idx = 2;
    g.priority.player_with_priority = 2;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(0) },
        Attack { attacker: b, target: AttackTarget::Player(0) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    let card = g.battlefield_find(ball).unwrap();
    assert_eq!(card.controller, 2, "the attacking player");
    assert!(!card.tapped, "and untapped");
}

/// Contested Game Ball's {2}, {T}: a card and a point counter; the fifth
/// counter trades it for a Treasure.
#[test]
fn contested_game_ball_cashes_in_at_five_points() {
    let mut g = pod(2);
    let ball = g.add_card_to_battlefield(0, catalog::contested_game_ball());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    g.battlefield_find_mut(ball).unwrap().add_counters(CounterType::Point, 3);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    activate_at(&mut g, ball, None);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.battlefield_find(ball).unwrap().counter_count(CounterType::Point), 4, "four: kept");
    g.battlefield_find_mut(ball).unwrap().tapped = false;
    g.players[0].mana_pool.add_colorless(2);
    activate_at(&mut g, ball, None);
    assert!(g.battlefield_find(ball).is_none(), "fifth counter: sacrificed");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure" && c.controller == 0));
}

/// CR 701.23a — Jester's Cap: the activator picks three cards from the
/// target player's library and exiles them.
#[test]
fn jesters_cap_exiles_three_of_the_target_players_cards() {
    let mut g = pod(2);
    let cap = ready(&mut g, 0, catalog::jesters_cap());
    let picks: Vec<CardId> = (0..3).map(|_| g.add_card_to_library(1, catalog::lightning_bolt())).collect();
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::island());
    }
    g.decider = Box::new(ScriptedDecider::new(picks.iter().map(|&c| DecisionAnswer::Search(Some(c)))));
    g.players[0].mana_pool.add_colorless(2);
    activate_at(&mut g, cap, Some(Target::Player(1)));
    assert!(g.battlefield_find(cap).is_none(), "sacrificed as a cost");
    for c in picks {
        assert!(g.exile.iter().any(|e| e.id == c), "exiled");
    }
    assert_eq!(g.players[1].library.len(), 3);
}

/// Lightning Visionary has prowess.
#[test]
fn lightning_visionary_has_prowess() {
    let mut g = pod(2);
    let v = ready(&mut g, 0, catalog::lightning_visionary());
    assert!(g.computed_permanent(v).unwrap().keywords().contains(&Keyword::Prowess));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Player(1));
    let c = g.computed_permanent(v).unwrap();
    assert_eq!((c.power, c.toughness), (3, 2));
}

/// Lightning Prowess: the host gains haste and a 1-damage tap ability.
#[test]
fn lightning_prowess_grants_haste_and_a_ping() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::lightning_prowess());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Haste));
    let life = g.players[1].life;
    activate_at(&mut g, bear, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 1, "summoning-sick, but hasty");
}

/// Lightning Storm deals 3 (no charge counters).
#[test]
fn lightning_storm_deals_three() {
    let mut g = pod(2);
    let storm = g.add_card_to_hand(0, catalog::lightning_storm());
    g.players[0].mana_pool.add(Color::Red, 3);
    let life = g.players[1].life;
    cast_at(&mut g, storm, Target::Player(1));
    assert_eq!(g.players[1].life, life - 3);
}

/// Radiating Lightning: 3 to the player, 1 to each of that player's
/// creatures and none of anyone else's.
#[test]
fn radiating_lightning_hits_the_player_and_their_creatures() {
    let mut g = pod(3);
    let theirs = ready(&mut g, 1, catalog::llanowar_elves());
    let other = ready(&mut g, 2, catalog::llanowar_elves());
    let mine = ready(&mut g, 0, catalog::llanowar_elves());
    let spell = g.add_card_to_hand(0, catalog::radiating_lightning());
    g.players[0].mana_pool.add(Color::Red, 4);
    let life = g.players[1].life;
    cast_at(&mut g, spell, Target::Player(1));
    assert_eq!(g.players[1].life, life - 3);
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(other).is_some());
    assert!(g.battlefield_find(mine).is_some());
}

/// Riddle of Lightning: scry 3, then the revealed top card's mana value is
/// the damage.
#[test]
fn riddle_of_lightning_deals_the_top_cards_mana_value() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::serra_angel());
    g.add_card_to_library(0, catalog::island());
    let riddle = g.add_card_to_hand(0, catalog::riddle_of_lightning());
    g.players[0].mana_pool.add(Color::Red, 5);
    g.decider = Box::new(ScriptedDecider::new([]));
    let life = g.players[1].life;
    cast_at(&mut g, riddle, Target::Player(1));
    assert_eq!(g.players[1].life, life - 5, "Serra Angel, mana value 5");
}

/// Lightning Javelin: 3 damage, then scry 1.
#[test]
fn lightning_javelin_deals_three() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::lightning_javelin());
    g.players[0].mana_pool.add(Color::Red, 4);
    let life = g.players[1].life;
    cast_at(&mut g, spell, Target::Player(1));
    assert_eq!(g.players[1].life, life - 3);
}
