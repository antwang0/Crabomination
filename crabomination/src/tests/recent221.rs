//! Functionality tests for `catalog::sets::decks::recent221`.

use crate::card::{CardType, Supertype};
use crate::catalog;
use crate::game::types::TurnStep;
use crate::game::{drain_stack, two_player_game, GameAction};
use crate::mana::Color;

/// Diamond Mare gains life only when you cast a spell of its chosen color.
#[test]
fn diamond_mare_gains_on_chosen_color() {
    let mut g = two_player_game();
    let mare = g.add_card_to_battlefield(0, catalog::diamond_mare());
    g.battlefield_find_mut(mare).unwrap().chosen_color = Some(Color::Red);
    // A red spell to cast.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast red spell");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained 1 life for the red spell");
}

/// The Aetherdrift Tyrant legends have their printed stats and supertype.
#[test]
fn tyrant_legends_have_printed_stats() {
    type Make = fn() -> crate::card::CardDefinition;
    for (make, pt) in [
        (catalog::kalakscion_hunger_tyrant as Make, (7, 2)),
        (catalog::tyrox_saurid_tyrant as Make, (4, 1)),
        (catalog::terrian_world_tyrant as Make, (9, 7)),
        (catalog::sundial_dawn_tyrant as Make, (3, 3)),
    ] {
        let def = make();
        assert!(def.supertypes.contains(&Supertype::Legendary), "{} is legendary", def.name);
        assert!(def.card_types.contains(&CardType::Creature), "{} is a creature", def.name);
        assert_eq!((def.power, def.toughness), pt, "{} stats", def.name);
    }
    assert!(catalog::sundial_dawn_tyrant().card_types.contains(&CardType::Artifact), "Sundial is an artifact");
}
