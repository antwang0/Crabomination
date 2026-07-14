//! Functionality tests for `catalog::sets::decks::recent210`.

use crate::card::CounterType;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// A Guildgate enters tapped and taps for either of its two colors.
#[test]
fn guildgate_enters_tapped_dual() {
    let mut g = two_player_game();
    let gate = g.move_card_to_battlefield_for_test(0, catalog::izzet_guildgate());
    drain_stack(&mut g);
    assert!(g.battlefield_find(gate).unwrap().tapped, "Gate enters tapped");
    assert!(g.battlefield_find(gate).unwrap().definition.subtypes.land_types
        .contains(&crate::card::LandType::Gate));
    // Two mana abilities: {T}: Add {U} / {T}: Add {R}.
    assert_eq!(g.battlefield_find(gate).unwrap().definition.activated_abilities.len(), 2);
}

/// Heraldic Banner pumps creatures of the chosen color only.
#[test]
fn heraldic_banner_chosen_color_anthem() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Green)]));
    g.move_card_to_battlefield_for_test(0, catalog::heraldic_banner());
    drain_stack(&mut g);
    let elf = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green 2/2
    let red = g.add_card_to_battlefield(0, catalog::swab_goblin()); // red 2/2
    let ev = g.computed_permanent(elf).unwrap();
    assert_eq!((ev.power, ev.toughness), (3, 2), "green creature gets +1/+0");
    let rv = g.computed_permanent(red).unwrap();
    assert_eq!((rv.power, rv.toughness), (2, 2), "non-green creature unaffected");
}

/// Pirate's Cutlass auto-attaches to a Pirate on entry and buffs it.
#[test]
fn pirates_cutlass_attaches_to_pirate() {
    let mut g = two_player_game();
    let pirate = g.add_card_to_battlefield(0, catalog::swab_goblin()); // Goblin Pirate 2/2
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(pirate))]));
    g.move_card_to_battlefield_for_test(0, catalog::pirates_cutlass());
    drain_stack(&mut g);
    let pv = g.computed_permanent(pirate).unwrap();
    assert_eq!((pv.power, pv.toughness), (4, 3), "Pirate gets +2/+1 from the Cutlass");
}

/// Adventuring Gear pumps its wearer whenever a land enters.
#[test]
fn adventuring_gear_landfall_pump() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gear = g.add_card_to_battlefield(0, catalog::adventuring_gear());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: gear, target: bear }).expect("equip");
    drain_stack(&mut g);
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play a land");
    drain_stack(&mut g);
    let bv = g.computed_permanent(bear).unwrap();
    assert_eq!((bv.power, bv.toughness), (4, 4), "landfall gives +2/+2");
}

/// Gnarlback Rhino draws when you target it with your own spell.
#[test]
fn gnarlback_rhino_draws_on_self_target() {
    let mut g = two_player_game();
    let rhino = g.add_card_to_battlefield(0, catalog::gnarlback_rhino());
    let pump = g.add_card_to_hand(0, catalog::giant_growth());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: pump, target: Some(Target::Permanent(rhino)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("target the Rhino");
    drain_stack(&mut g);
    // -1 for casting Giant Growth, +1 from the draw trigger = net 0.
    assert_eq!(g.players[0].hand.len(), before);
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Giant Growth").count(), 1);
}

/// Mold Adder grows when an opponent casts a blue or black spell, but not a red one.
#[test]
fn mold_adder_grows_on_blue_or_black() {
    let mut g = two_player_game();
    let adder = g.add_card_to_battlefield(0, catalog::mold_adder());
    // A red spell does nothing.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("red spell");
    drain_stack(&mut g);
    assert_eq!(*g.battlefield_find(adder).unwrap().counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0), 0);
    // Player 0 puts a red bolt on the stack; the opponent counters it with a
    // blue spell — that blue cast grows Mold Adder.
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt on the stack");
    let blue = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: blue, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent counters with a blue spell");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    drain_stack(&mut g);
    assert_eq!(*g.battlefield_find(adder).unwrap().counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0), 1);
}
