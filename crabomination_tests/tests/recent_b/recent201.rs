//! Functionality tests for `catalog::sets::decks::recent201`.

use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Duskmourn's Domination steals a creature and shrinks + silences it.
#[test]
fn duskmourns_domination_steals_and_shrinks() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // A 4/4 flyer to steal; -3/-0 leaves a 1/4 you control, no flying.
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    let aura = g.add_card_to_hand(0, catalog::duskmourns_domination());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Duskmourn's Domination");
    drain_stack(&mut g);
    let cp = g.computed_permanent(victim).unwrap();
    assert_eq!(cp.controller, 0, "you control the enchanted creature");
    assert_eq!(cp.power, 1, "-3/-0 leaves 1 power");
    assert!(!cp.keywords.contains(&crabomination::card::Keyword::Flying), "lost its abilities");
}
