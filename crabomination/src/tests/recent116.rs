//! Functionality tests for `catalog::sets::decks::recent116`.

use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// Untamed Kavu enters with three +1/+1 counters when kicked (5/5).
#[test]
fn untamed_kavu_kicked_is_five_five() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let spell = g.add_card_to_hand(0, catalog::untamed_kavu());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4); // {1}{G} + kicker {3}
    g.perform_action(GameAction::CastSpellKicked {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Untamed Kavu kicked");
    drain_stack(&mut g);
    let kavu = g.battlefield.iter().find(|c| c.definition.name == "Untamed Kavu").expect("resolved");
    assert_eq!(g.computed_permanent(kavu.id).unwrap().power, 5, "2/2 + three +1/+1 counters");
}

/// Unkicked, Untamed Kavu is a plain 2/2.
#[test]
fn untamed_kavu_unkicked_is_two_two() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let spell = g.add_card_to_hand(0, catalog::untamed_kavu());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Untamed Kavu");
    drain_stack(&mut g);
    let kavu = g.battlefield.iter().find(|c| c.definition.name == "Untamed Kavu").expect("resolved");
    assert_eq!(g.computed_permanent(kavu.id).unwrap().power, 2, "no counters");
}

/// Manaform Hellkite mints an X/X Dragon token equal to the mana spent on a
/// noncreature spell (Divination = 3 → a 3/3).
#[test]
fn manaform_hellkite_mints_mana_spent_token() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::manaform_hellkite());
    for _ in 0..4 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let spell = g.add_card_to_hand(0, catalog::divination()); // {2}{U}
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Divination");
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.definition.name == "Dragon Illusion").expect("token minted");
    assert_eq!(g.computed_permanent(token.id).unwrap().power, 3, "X = 3 mana spent");
    assert!(token.definition.keywords.contains(&crate::card::Keyword::Haste), "has haste");
}

/// The Dragon Illusion token is exiled at the next end step.
#[test]
fn manaform_token_exiles_at_end_step() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::manaform_hellkite());
    for _ in 0..4 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let spell = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Divination");
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.definition.name == "Dragon Illusion").unwrap().id;
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    // The token leaves the battlefield at end step; as a token it then ceases
    // to exist (CR 111.7), so it lingers in no zone.
    assert!(g.battlefield.iter().all(|c| c.id != token), "token exiled at end step");
}
