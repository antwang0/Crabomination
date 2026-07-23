//! CR conformance for rules exercised by this run's DIS/RTR gap wave:
//! CR 701.12b (fight snapshots power — near Guild Feud's dueling deploy),
//! CR 106.6 (a hybrid mana symbol makes one mana of a color — near Elemental
//! Resonance's cost-reading ramp), and CR 205.1/700.2 (a chosen card type
//! governs a resolution partition — near Vigean Intuition, an in-progress
//! "choose a card type" area).

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// CR 701.12b — in a fight each creature's damage equals its power measured
/// simultaneously, so an uneven pairing kills only the smaller creature. Guild
/// Feud deploys the top creature for each player, then fights the two.
#[test]
fn cr_701_12b_fight_snapshots_power() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::guild_feud());
    // You deploy a 3/2 Drake; the opponent a 1/1 Merfolk.
    g.add_card_to_library(0, catalog::snapping_drake());
    g.add_card_to_library(1, catalog::merfolk_of_the_pearl_trident());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    // The 1/1 dies; the 3/2 survives having taken exactly 1.
    let drake = g.battlefield.iter().find(|c| c.definition.name == "Snapping Drake").expect("drake survived");
    assert_eq!(drake.damage, 1, "the drake took the merfolk's 1 power");
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Merfolk of the Pearl Trident"),
        "the 1/1 was dealt lethal and died");
}

/// CR 106.6 — a hybrid mana symbol `{U/B}` is one mana of a color. Elemental
/// Resonance reading a `{U/B}{U/B}{U/B}` cost adds three colored mana.
#[test]
fn cr_106_6_hybrid_symbol_makes_colored_mana() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::nightveil_specter()); // {U/B}{U/B}{U/B}
    let aura = g.add_card_to_battlefield(0, catalog::elemental_resonance());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(host);
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    // Three hybrid pips → three mana, each a single color (no colorless).
    assert_eq!(g.players[0].mana_pool.total(), 3, "three hybrid pips → three mana");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 0, "hybrid pips are colored, not generic");
}

/// CR 205.1 / 700.2 — "choose a card type" fixes a single type that then drives
/// the effect. With Land chosen, Vigean Intuition routes lands (not creatures)
/// to hand.
#[test]
fn cr_205_1_chosen_type_drives_partition() {
    let mut g = two_player_game();
    let land = g.add_card_to_library(0, catalog::island());
    let creature = g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::vigean_intuition());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    // Options are [Creature, Instant, Sorcery, Artifact, Enchantment, PW, Land];
    // index 6 == Land.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(6)]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Vigean Intuition");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == land), "the chosen type (Land) went to hand");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == creature), "the creature was buried");
}
