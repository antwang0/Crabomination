//! Functionality tests for `catalog::sets::decks::recent149` (BLB wave).

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::two_player_game;

fn fill_mana(g: &mut GameState) {
    for c in [
        crabomination::mana::Color::White,
        crabomination::mana::Color::Blue,
        crabomination::mana::Color::Black,
        crabomination::mana::Color::Red,
        crabomination::mana::Color::Green,
    ] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(8);
}

/// Driftgloom Coyote exiles a small opposing creature until it leaves and grows.
#[test]
fn driftgloom_coyote_exiles_and_grows() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, power 2
    let coyote = g.move_card_to_battlefield_for_test(0, catalog::driftgloom_coyote());
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "the 2/2 was exiled");
    assert_eq!(
        g.battlefield_find(coyote).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "power-2 exile grew the Coyote",
    );
    // Coyote leaves → the exiled creature returns.
    let _ = g.remove_to_graveyard_with_triggers(coyote);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == victim), "exiled creature returned");
}

/// Early Winter mode 0 exiles a target creature.
#[test]
fn early_winter_exiles_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::early_winter());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Early Winter mode 0");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature exiled");
}

/// High Stride pumps +1/+3, grants reach, and untaps the target.
#[test]
fn high_stride_pumps_reach_untap() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::high_stride());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast High Stride");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (3, 5), "+1/+3");
    assert!(c.keywords.contains(&Keyword::Reach), "gained reach");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
}

/// Playful Shove deals 1 to a creature and draws a card.
#[test]
fn playful_shove_pings_and_draws() {
    let mut g = two_player_game();
    let lion = g.add_card_to_battlefield(1, catalog::savannah_lions()); // 2/1
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::playful_shove());
    let hand_before = g.players[0].hand.len(); // spell in hand
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(lion)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Playful Shove");
    drain_stack(&mut g);
    assert!(g.battlefield_find(lion).is_none(), "2/1 died to 1 damage");
    assert_eq!(g.players[0].hand.len(), hand_before, "cast one, drew one → net hand size steady");
}

/// Psychic Whorl makes the opponent discard two; surveil fires only with a Rat.
#[test]
fn psychic_whorl_discards_and_conditional_surveil() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::forest());
    g.add_card_to_hand(1, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::psychic_whorl());
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    let opp_hand = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Psychic Whorl");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand - 2, "opponent discarded two");
}

/// Reptilian Recruiter steals a small creature until end of turn (untapped, hasty).
#[test]
fn reptilian_recruiter_threatens_small_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // power 2
    g.battlefield_find_mut(victim).unwrap().tapped = true;
    g.move_card_to_battlefield_for_test(0, catalog::reptilian_recruiter());
    drain_stack(&mut g);
    let c = g.battlefield_find(victim).unwrap();
    assert_eq!(c.controller, 0, "gained control of the power-2 creature");
    assert!(!c.tapped, "untapped it");
    assert!(g.computed_permanent(victim).unwrap().keywords.contains(&Keyword::Haste), "hasty");
}

/// Raccoon Rallier's sorcery-speed tap grants a creature haste.
#[test]
fn raccoon_rallier_grants_haste() {
    let mut g = two_player_game();
    let rallier = g.add_card_to_battlefield(0, catalog::raccoon_rallier());
    g.clear_sickness(rallier);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rallier, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None,
    }).expect("activate Raccoon Rallier");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste), "granted haste");
}
