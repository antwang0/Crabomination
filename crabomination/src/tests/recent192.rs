//! Functionality tests for `catalog::sets::decks::recent192` (OTJ gaps).

use crate::catalog;
use crate::game::two_player_game;
use crate::game::*;
use crate::mana::Color;

/// Pillage the Bog digs twice your land count and takes a card.
#[test]
fn pillage_the_bog_digs_by_lands() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest()); // 3 lands → dig 6
    }
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::island());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::pillage_the_bog());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Pillage the Bog");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == top), "took the top card from the dig");
}

/// Hell to Pay burns a creature for X and makes Treasures from excess damage.
#[test]
fn hell_to_pay_excess_makes_treasures() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::hell_to_pay());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(5),
    })
    .expect("cast Hell to Pay for X=5");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "5 damage killed the 2/2");
    let treasures = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Treasure" && c.controller == 0)
        .count();
    assert_eq!(treasures, 3, "5 - 2 lethal = 3 excess → 3 Treasures");
    assert!(
        g.battlefield.iter().filter(|c| c.definition.name == "Treasure").all(|c| c.tapped),
        "Treasures enter tapped",
    );
}
