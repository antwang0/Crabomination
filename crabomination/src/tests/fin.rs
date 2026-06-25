//! Functionality tests for the `catalog::sets::fin` (Final Fantasy) batch.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::types::TurnStep;
use crate::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Iron Giant ships as a 6/6 with vigilance, reach, and trample.
#[test]
fn iron_giant_keywords() {
    let g = catalog::iron_giant();
    assert_eq!((g.power, g.toughness), (6, 6));
    for kw in [Keyword::Vigilance, Keyword::Reach, Keyword::Trample] {
        assert!(g.keywords.contains(&kw), "Iron Giant has {kw:?}");
    }
}

/// Sazh's Chocobo grows with a +1/+1 counter on landfall.
#[test]
fn sazhs_chocobo_grows_on_landfall() {
    let mut g = two_player_game();
    let bird = g.add_card_to_battlefield(0, catalog::sazhs_chocobo());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bird).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "landfall added a +1/+1 counter"
    );
}

/// Sephiroth's Intervention destroys a creature and gains 2 life.
#[test]
fn sephiroths_intervention_kills_and_gains() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::sephiroths_intervention());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sephiroth's Intervention");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature destroyed");
    assert_eq!(g.players[0].life, 22, "gained 2 life");
}

/// Cactuar bounces itself at end step, but not the turn it enters.
#[test]
fn cactuar_bounces_at_end_step_unless_fresh() {
    let mut g = two_player_game();
    // Freshly entered this turn → stays.
    let fresh = g.add_card_to_battlefield(0, catalog::cactuar());
    let t = g.turn_number;
    g.battlefield_find_mut(fresh).unwrap().entered_turn = Some(t);
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(fresh).is_some(), "fresh Cactuar stays");

    // A Cactuar that didn't enter this turn returns to hand.
    let mut g2 = two_player_game();
    let old = g2.add_card_to_battlefield(0, catalog::cactuar());
    g2.battlefield_find_mut(old).unwrap().entered_turn = None;
    advance_to(&mut g2, TurnStep::End);
    drain_stack(&mut g2);
    assert!(g2.battlefield_find(old).is_none(), "old Cactuar bounced");
    assert!(g2.players[0].hand.iter().any(|c| c.id == old), "returned to hand");
}

/// Magitek Armor enters as a Crew-1 Vehicle and mints a Hero token.
#[test]
fn magitek_armor_makes_a_hero() {
    let armor = catalog::magitek_armor();
    assert!(armor.keywords.contains(&Keyword::Crew(1)));
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::magitek_armor());
    drain_stack(&mut g);
    let heroes = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Hero").count();
    assert_eq!(heroes, 1, "one Hero token");
}

/// Chocobo Racetrack makes a Bird token on landfall.
#[test]
fn chocobo_racetrack_makes_bird_on_landfall() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::chocobo_racetrack());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    let birds = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Racetrack Bird").count();
    assert_eq!(birds, 1, "one Bird token from landfall");
}
