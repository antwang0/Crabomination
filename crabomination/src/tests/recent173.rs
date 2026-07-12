//! Functionality tests for `catalog::sets::decks::recent173`.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::types::Target;
use crate::game::*;

/// Roadside Assistance attaches, mints a boosted Pilot, and grants +1/+1 +
/// lifelink.
#[test]
fn roadside_assistance_aura_and_pilot() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::roadside_assistance());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.cast_spell(aura, Some(Target::Permanent(bear)), vec![], None, None).expect("cast the Aura");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
    assert!(cp.keywords.contains(&Keyword::Lifelink));
    let pilot = g.battlefield.iter().find(|c| c.definition.name == "Pilot").expect("Pilot minted");
    assert_eq!(g.crew_saddle_power_bonus(pilot.id), 2, "boosted Pilot");
}

/// Trade the Helm swaps control of two permanents.
#[test]
fn trade_the_helm_swaps_control() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::trade_the_helm());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.cast_spell(spell, Some(Target::Permanent(mine)), vec![Target::Permanent(theirs)], None, None)
        .expect("cast Trade the Helm");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1, "my bear went to the opponent");
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0, "their angel came to me");
}

/// Voyage Home draws three, gains 3, and gets Affinity for artifacts.
#[test]
fn voyage_home_affinity_draw_and_life() {
    let mut g = two_player_game();
    // Two artifacts → {2} off {5}{W}{U}.
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
    let spell = g.add_card_to_hand(0, catalog::voyage_home());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3); // 5 total = {5}{W}{U} - {2} affinity
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    let life = g.players[0].life;
    g.cast_spell(spell, None, vec![], None, None).expect("affinity pays the reduced cost");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3, "drew three (spell left hand)");
    assert_eq!(g.players[0].life, life + 3, "gained 3");
}

/// Aggressive Negotiations exiles a nonland from the opponent's hand and puts a
/// counter on your creature.
#[test]
fn aggressive_negotiations_exiles_and_counters() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::lightning_bolt()); // nonland
    g.add_card_to_hand(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::aggressive_negotiations());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.cast_spell(spell, Some(Target::Permanent(mine)), vec![], None, None)
        .expect("cast Aggressive Negotiations");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.definition.name == "Lightning Bolt"), "nonland exiled");
    assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Forest"), "land kept");
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}
