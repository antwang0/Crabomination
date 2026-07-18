//! Functionality tests for `catalog::sets::decks::blb` (Bloomburrow gaps).

use crabomination::card::{ArtifactSubtype, CardType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Sugar Coat turns the enchanted creature into a colorless, abilityless Food
/// artifact carrying only the sacrifice-for-3-life ability.
#[test]
fn sugar_coat_makes_a_food() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // flyer/vigilance
    let aura = g.add_card_to_hand(0, catalog::sugar_coat());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(angel)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Sugar Coat");
    drain_stack(&mut g);
    let cp = g.computed_permanent(angel).unwrap();
    assert!(cp.card_types.contains(&CardType::Artifact), "now an artifact");
    assert!(!cp.card_types.contains(&CardType::Creature), "no longer a creature");
    assert!(cp.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Food), "is a Food");
    assert!(cp.colors.is_empty(), "colorless");
    assert!(cp.keywords.is_empty(), "lost flying and vigilance");
    // Only the granted sac ability remains.
    let abilities = g.granted_abilities_for(angel);
    assert_eq!(abilities.len(), 1, "exactly the sac-for-life ability");
    assert!(abilities[0].sac_cost, "sacrifice cost");
}
