//! Functionality tests for `catalog::sets::decks::recent251` (token hate +
//! Merfolk untapper).

use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game};

/// Kraul Whipcracker destroys an opponent's token on ETB (and can't hit a
/// nontoken creature).
#[test]
fn kraul_whipcracker_destroys_opponent_token() {
    use crabomination::card::{CardType, CreatureType, Subtypes, TokenDefinition};
    let mut g = two_player_game();
    // A real token for the opponent.
    let tok = TokenDefinition {
        name: "Bird".into(),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    };
    let token = g.add_token_to_battlefield(1, &tok);
    let whip = g.add_card_to_battlefield(0, catalog::kraul_whipcracker());
    g.fire_self_etb_triggers(whip, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(token).is_none(), "opponent's token destroyed");
}

/// Forensic Researcher untaps another target permanent you control.
#[test]
fn forensic_researcher_untaps_your_permanent() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let researcher = g.add_card_to_battlefield(0, catalog::forensic_researcher());
    g.clear_sickness(researcher);
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.perform_action(GameAction::ActivateAbility {
        card_id: researcher,
        ability_index: 0,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate the untap ability");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(land).unwrap().tapped, "the land was untapped");
}
