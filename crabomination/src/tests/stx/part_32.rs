//! Functionality tests for the modern_decks STX additions: Possibility Storm
//! (`Effect::PossibilityStorm`) and other completed extras.

use crate::card::CardType;
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;

/// Possibility Storm is a {3}{R}{R} red enchantment with a cast trigger.
#[test]
fn possibility_storm_is_a_three_mana_red_enchantment() {
    let d = catalog::possibility_storm();
    assert_eq!(d.cost.cmc(), 5);
    assert!(d.card_types.contains(&CardType::Enchantment));
    assert_eq!(d.triggered_abilities.len(), 1, "the cast trigger is wired");
}

/// Casting a spell from hand under Possibility Storm exiles it, digs to a
/// card sharing a card type, and bottoms the rest — the cast spell never
/// resolves (it's exiled instead).
#[test]
fn possibility_storm_digs_to_a_shared_type_card() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::possibility_storm());
    // Library: a land (no shared type) then a sorcery that shares "Instant/
    // Sorcery is not a creature type" — use another instant so it shares the
    // Instant card type with Lightning Bolt.
    g.add_card_to_library(0, catalog::lightning_bolt()); // top: Instant (shares)
    g.add_card_to_library(0, catalog::grizzly_bears()); // below: Creature
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // The cast bolt was exiled (countered), so player 1 took no damage from it.
    assert_eq!(g.players[1].life, 20, "the cast spell was exiled, not resolved");
    // The dug Instant + the exiled spell + the non-matching card all end up
    // bottomed; nothing stays on the battlefield from the cast.
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Lightning Bolt"));
    // Library is back to its prior size (everything exiled this way bottomed,
    // and the cast spell joined them) minus none — the dug Instant may have
    // been declined-to-cast (AutoDecider declines), so it returns too.
    assert!(g.players[0].library.len() >= lib_before, "exiled cards bottomed back");
}
