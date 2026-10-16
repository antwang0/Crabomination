//! Commander: the 2026-10-16 third-leg rescan's gaps (`decks::cmdr_oct16_e`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn has(g: &GameState, id: CardId, k: &Keyword) -> bool {
    g.computed_permanent(id).expect("on the battlefield").keywords().contains(k)
}

/// The Howling Abomination: a spell targeting it pumps it and burns every
/// opponent (an ability targeting it does not); trample needs three spells.
#[test]
fn howling_abomination_punishes_every_opponent_when_targeted_by_a_spell() {
    let mut g = pod(4);
    let abom = ready(&mut g, 0, catalog::the_howling_abomination());
    assert!(!has(&g, abom, &Keyword::Trample));
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    let growth = g.add_card_to_hand(0, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, growth, Some(Target::Permanent(abom))).expect("Giant Growth");
    assert_eq!(g.computed_permanent(abom).unwrap().power, 5 + 3 + 2);
    assert_eq!(g.players[0].life, life[0]);
    for seat in 1..4 {
        assert_eq!(g.players[seat].life, life[seat] - 2, "seat {seat}");
    }
    for _ in 0..2 {
        assert!(!has(&g, abom, &Keyword::Trample));
        let growth = g.add_card_to_hand(0, catalog::giant_growth());
        g.players[0].mana_pool.add(Color::Green, 1);
        cast(&mut g, growth, Some(Target::Permanent(abom))).expect("Giant Growth");
    }
    assert!(has(&g, abom, &Keyword::Trample), "three spells this turn");
    assert_eq!(g.players[1].life, life[1] - 6);
}

/// Storm's ETB takes an artifact, an enchantment or a flier — never a
/// ground creature — and may take nothing.
#[test]
fn storm_destroys_a_flier_but_not_a_ground_creature() {
    let mut g = pod(3);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let bears = ready(&mut g, 2, catalog::grizzly_bears());
    let storm = g.add_card_to_hand(0, catalog::storm_shaker_of_skies());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, storm, None).expect("cast Storm");
    assert!(g.battlefield_find(angel).is_none(), "the flier goes");
    assert!(g.battlefield_find(bears).is_some() && g.battlefield_find(storm).is_some());
}

/// Wolfsbane's pump is once each turn.
#[test]
fn wolfsbane_pumps_once_each_turn() {
    let mut g = pod(2);
    let wolf = ready(&mut g, 0, catalog::wolfsbane_highland_hero());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    let act = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility { card_id: wolf, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
    };
    act(&mut g).expect("first pump");
    drain_stack(&mut g);
    assert!(act(&mut g).is_err(), "second pump this turn");
    assert_eq!(g.computed_permanent(wolf).unwrap().power, 4);
}

/// Extended Absence exiles a planeswalker and drains every opponent by one.
#[test]
fn extended_absence_exiles_and_pings_each_opponent() {
    let mut g = pod(3);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    let ea = g.add_card_to_hand(0, catalog::extended_absence());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, ea, Some(Target::Permanent(bears))).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == bears));
    assert_eq!(g.players.iter().map(|p| p.life).collect::<Vec<_>>(), vec![life[0] + 1, life[1] - 1, life[2] - 1]);
}

/// Hexhaven Dueling Arena's {2} ability wants a creature that attacked this
/// turn, and only a creature with a prepare spell can become prepared.
#[test]
fn hexhaven_dueling_arena_prepares_only_preparation_creatures() {
    let mut g = pod(2);
    let arena = ready(&mut g, 0, catalog::hexhaven_dueling_arena());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(4);
    let act = |g: &mut GameState, idx, t| {
        g.perform_action(GameAction::ActivateAbility { card_id: arena, ability_index: idx, target: Some(Target::Permanent(t)), additional_targets: vec![], x_value: None, mode: None })
    };
    assert!(act(&mut g, 2, bears).is_err(), "no prepare spell");
    assert!(act(&mut g, 1, bears).is_err(), "did not attack, and no prepare spell");
}
