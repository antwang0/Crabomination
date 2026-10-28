//! Commander: EDHREC ranks 2001-2500, two-card gaps (`decks::cmdr_edhrec61`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
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

fn fill_pool(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
    g.priority.player_with_priority = seat;
}

fn cast_at(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    fill_pool(g, 0);
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

/// Cleon's heroic exiles the top card to play; Emerge Unscathed targeting it
/// triggers it (CR 702.141 heroic wording) and rebounds (CR 702.88).
#[test]
fn cleon_heroic_and_emerge_unscathed() {
    let mut g = pod(2);
    g.players[0].library.clear();
    let top = g.add_card_to_library(0, catalog::island());
    let cleon = ready(&mut g, 0, catalog::cleon_merry_champion());
    let spell = g.add_card_to_hand(0, catalog::emerge_unscathed());
    cast_at(&mut g, spell, Some(Target::Permanent(cleon))).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == top), "heroic exiled the top card");
    assert!(g.exile.iter().any(|c| c.id == spell), "rebound exiled the spell");
    assert!(g.computed_permanent(cleon).unwrap().keywords().iter().any(|k| matches!(k, Keyword::Protection(_))));
}

/// Bloodmark Mentor gives red creatures first strike; Slumbering Dragon can't
/// attack until it has five +1/+1 counters, and grows when you're attacked.
#[test]
fn bloodmark_mentor_and_slumbering_dragon() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::bloodmark_mentor());
    let dragon = ready(&mut g, 0, catalog::slumbering_dragon());
    assert!(g.computed_permanent(dragon).unwrap().keywords().contains(&Keyword::FirstStrike));
    g.step = TurnStep::DeclareAttackers;
    let atk = vec![Attack { attacker: dragon, target: AttackTarget::Player(1) }];
    assert!(g.perform_action(GameAction::DeclareAttackers(atk)).is_err(), "asleep");

    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bears, target: AttackTarget::Player(0) }]))
        .expect("their attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(dragon).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Faith's Reward returns what died this turn, not older graveyard cards.
#[test]
fn faiths_reward_returns_this_turns_dead() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let old = g.add_card_to_graveyard(0, catalog::hill_giant());
    let mut evs = Vec::new();
    g.destroy_permanent(bears, false, &mut evs);
    drain_stack(&mut g);
    let spell = g.add_card_to_hand(0, catalog::faiths_reward());
    cast_at(&mut g, spell, None).expect("cast");
    assert!(g.battlefield_find(bears).is_some());
    assert!(g.battlefield_find(old).is_none());
}

/// Treasure Keeper's death reveals (not exiles, unlike cascade) down to the
/// first nonland card with mana value 3 OR LESS, casts it free, and bottoms
/// the passed-over cards.
#[test]
fn treasure_keeper_casts_on_death() {
    let mut g = pod(2);
    g.players[0].library.clear();
    let forest = g.add_card_to_library(0, catalog::forest());
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    let courser = g.add_card_to_library(0, catalog::centaur_courser());
    let under = g.add_card_to_library(0, catalog::grizzly_bears());
    let keeper = ready(&mut g, 0, catalog::treasure_keeper());
    let mut evs = Vec::new();
    g.destroy_permanent(keeper, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(courser).is_some(), "the three-drop was cast free");
    assert!(g.exile.is_empty(), "the walk reveals; nothing is exiled");
    let lib: Vec<CardId> = g.players[0].library.iter().map(|c| c.id).collect();
    assert_eq!(lib[0], under, "the unrevealed card stays on top");
    assert!(lib[1..].contains(&forest) && lib[1..].contains(&angel), "the misses go to the bottom");
}

/// Katilda counts Spirits and enchantments; disturbed, her Aura pumps the
/// enchanted creature by the same count (itself an enchantment).
#[test]
fn katilda_counts_spirits_and_enchantments() {
    let mut g = pod(2);
    let katilda = ready(&mut g, 0, catalog::katilda_dawnhart_martyr());
    ready(&mut g, 0, catalog::rage_reflection());
    assert_eq!(g.computed_permanent(katilda).unwrap().power, 2, "herself and the enchantment");

    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let card = g.add_card_to_graveyard(0, catalog::katilda_dawnhart_martyr());
    fill_pool(&mut g, 0);
    g.perform_action(GameAction::CastDisturb { card_id: card, target: Some(Target::Permanent(bears)), additional_targets: vec![] })
        .expect("disturb");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!(cp.power, 3, "+1 for the Aura itself");
    assert!(cp.keywords().contains(&Keyword::Lifelink));
}
