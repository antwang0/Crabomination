//! Functionality tests for `catalog::sets::decks::recent93` (blue Wizard payoffs).

use crate::catalog;
use crate::card::CardType;
use crate::game::two_player_game;
use crate::game::types::Target;
use crate::game::*;
use crate::mana::Color;

#[test]
fn galecaster_bounces_by_tapping_a_wizard() {
    let mut g = two_player_game();
    let colossus = g.add_card_to_battlefield(0, catalog::galecaster_colossus());
    // A second Wizard to tap for the cost.
    g.add_card_to_battlefield(0, catalog::gadwick_the_wizened());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for c in g.battlefield.iter_mut() { c.summoning_sick = false; }
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: colossus, ability_index: 0, target: Some(Target::Permanent(foe)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("tap a Wizard to bounce");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "the enemy creature was bounced");
    assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "returned to its owner's hand");
}

#[test]
fn gadwick_draws_x_and_taps_on_blue_cast() {
    let mut g = two_player_game();
    // Cast Gadwick with X = 2.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let gadwick = g.add_card_to_hand(0, catalog::gadwick_the_wizened());
    for _ in 0..3 { g.players[0].mana_pool.add(Color::Blue, 1); }
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: gadwick, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast Gadwick for X=2");
    drain_stack(&mut g);
    // Gadwick left hand (−1) and drew 2 (+2) → net +1.
    assert_eq!(g.players[0].hand.len(), before + 1, "drew X = 2 (net +1 after leaving hand)");
    // Cast a blue spell → tap an opponent's untapped permanent.
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::brainstorm());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a blue spell");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "the blue cast tapped the opponent's creature");
}

#[test]
fn sphinx_of_lost_truths_discards_when_not_kicked() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let sphinx = g.add_card_to_battlefield(0, catalog::sphinx_of_lost_truths());
    let before = g.players[0].hand.len();
    g.fire_self_etb_triggers(sphinx, 0);
    drain_stack(&mut g);
    // Drew 3, discarded 3 (not kicked) → net 0.
    assert_eq!(g.players[0].hand.len(), before, "unkicked: draw 3 then discard 3");
    assert!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Island").count() >= 3,
        "three cards discarded");
}

#[test]
fn rielle_grows_with_instants_in_graveyard() {
    let mut g = two_player_game();
    let rielle = g.add_card_to_battlefield(0, catalog::rielle_the_everwise());
    assert_eq!(g.computed_permanent(rielle).unwrap().power, 0, "0 power with an empty graveyard");
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::brainstorm());
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // a creature — not counted
    let cp = g.computed_permanent(rielle).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3), "+1/+0 per instant/sorcery in graveyard");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.card_types.contains(&CardType::Creature)));
}
