//! Functionality tests for the `catalog::sets::decks::recent11` batch.

use crate::card::CounterType;
use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::mana::Color;

/// Earthbending Lesson earthbends 4 from a sorcery.
#[test]
fn earthbending_lesson_earthbends_four() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::earthbending_lesson());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast_at(&mut g, id, Target::Permanent(land));
    assert_eq!(
        g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
        4
    );
}

/// Dai Li Indoctrination's earthbend mode (mode 1) earthbends 2.
#[test]
fn dai_li_indoctrination_earthbend_mode() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::dai_li_indoctrination());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("cast earthbend mode");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2
    );
}

/// Dai Li Indoctrination's discard mode (mode 0) makes an opponent discard.
#[test]
fn dai_li_indoctrination_discard_mode() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let before = g.players[1].hand.len();
    let id = g.add_card_to_hand(0, catalog::dai_li_indoctrination());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("cast discard mode");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), before - 1, "opponent discarded a nonland");
}
