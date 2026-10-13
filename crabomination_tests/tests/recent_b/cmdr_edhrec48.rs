//! Commander: Kaima's and Verrak's last gaps (`decks::cmdr_edhrec48`).

use crabomination::card::{CardId, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Draconic Destiny makes a flying, hasty Dragon and comes back to hand when
/// it dies; Undying Rage returns from the graveyard on its own.
#[test]
fn kaimas_auras_come_back() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let destiny = g.add_card_to_hand(0, catalog::draconic_destiny());
    cast(&mut g, destiny, Some(Target::Permanent(bear))).expect("destiny");
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords().contains(&Keyword::Flying) && cp.keywords().contains(&Keyword::Haste));
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Dragon));
    assert_eq!(cp.power, 3);
    let rage = g.add_card_to_hand(0, catalog::undying_rage());
    cast(&mut g, rage, Some(Target::Permanent(bear))).expect("rage");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.battlefield_find_mut(bear).unwrap().damage = 3;
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("bolt");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.id == destiny), "Destiny back to hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == rage), "Rage back to hand");
}

/// Fruit of the First Tree pays off the enchanted creature's toughness.
#[test]
fn fruit_of_the_first_tree_pays_toughness() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let fruit = g.add_card_to_hand(0, catalog::fruit_of_the_first_tree());
    cast(&mut g, fruit, Some(Target::Permanent(giant))).expect("fruit");
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(giant))).expect("bolt");
    assert_eq!(g.players[0].life, life + 3);
    assert_eq!(g.players[0].hand.len(), hand + 3);
}

/// Arguel's Blood Fast draws for life and turns into Temple of Aclazotz at
/// five life or less.
#[test]
fn arguels_blood_fast_transforms_when_low() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let fast = g.add_card_to_battlefield(0, catalog::arguels_blood_fast());
    g.players[0].life = 5;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(fast).unwrap().definition.name, "Temple of Aclazotz");
}

/// Axis of Mortality swaps two players' life totals.
#[test]
fn axis_of_mortality_swaps_life() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::axis_of_mortality());
    g.players[0].life = 3;
    g.players[2].life = 30;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let lives: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    assert!(lives.contains(&30) && g.players[0].life != 3, "the low seat traded up: {lives:?}");
}
