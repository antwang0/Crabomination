//! Functionality tests for the `catalog::sets::decks::recent4` batch.

use crate::catalog;
use crate::game::actions::extra_cost_for_spell;
use crate::game::types::TurnStep;
use crate::game::*;
use crate::mana::Color;

/// Ritual of Soot destroys creatures with mana value 3 or less, sparing bigger.
#[test]
fn ritual_of_soot_kills_small_creatures_only() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let big = g.add_card_to_battlefield(1, catalog::grave_titan()); // MV 6
    let id = g.add_card_to_hand(0, catalog::ritual_of_soot());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ritual of Soot");
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_none(), "MV2 creature destroyed");
    assert!(g.battlefield_find(big).is_some(), "MV6 creature survives");
}

/// Recurring Nightmare: sac a creature + bounce itself to reanimate a graveyard
/// creature.
#[test]
fn recurring_nightmare_reanimates_sacrificing_a_creature() {
    let mut g = two_player_game();
    let nightmare = g.add_card_to_battlefield(0, catalog::recurring_nightmare());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dead = g.add_card_to_graveyard(0, catalog::grave_titan());
    g.perform_action(GameAction::ActivateAbility {
        card_id: nightmare, ability_index: 0,
        target: Some(Target::Permanent(dead)), additional_targets: vec![], x_value: None,
    }).expect("activate Recurring Nightmare");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "graveyard creature reanimated");
    assert!(g.battlefield_find(fodder).is_none(), "fodder creature sacrificed");
    assert!(g.players[0].hand.iter().any(|c| c.id == nightmare),
        "Recurring Nightmare returned to its owner's hand");
}

/// Survival of the Fittest: pay {G} + discard a creature → tutor a creature to
/// hand.
#[test]
fn survival_of_the_fittest_tutors_a_creature() {
    let mut g = two_player_game();
    let survival = g.add_card_to_battlefield(0, catalog::survival_of_the_fittest());
    let discard = g.add_card_to_hand(0, catalog::grizzly_bears());
    let fetch = g.add_card_to_library(0, catalog::grave_titan());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(fetch)),
    ]));
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: survival, ability_index: 0,
        target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Survival of the Fittest");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == fetch),
        "tutored creature is in hand");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == discard),
        "discarded creature is in the graveyard");
}

/// Footsteps of the Goryo reanimates a graveyard creature, then sacrifices it
/// at the next end step.
#[test]
fn footsteps_of_the_goryo_reanimates_then_sacrifices() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grave_titan());
    let id = g.add_card_to_hand(0, catalog::footsteps_of_the_goryo());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(dead)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Footsteps of the Goryo");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "creature reanimated");
    // Walk to the end step — the delayed trigger sacrifices it.
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_none(), "reanimated creature sacrificed at end step");
}

/// Apprentice Necromancer: {B}, {T}, sac itself → reanimate a graveyard creature
/// with haste; sacrifice it at the next end step.
#[test]
fn apprentice_necromancer_reanimates_with_haste() {
    let mut g = two_player_game();
    let appr = g.add_card_to_battlefield(0, catalog::apprentice_necromancer());
    g.clear_sickness(appr);
    let dead = g.add_card_to_graveyard(0, catalog::grave_titan());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: appr, ability_index: 0,
        target: Some(Target::Permanent(dead)), additional_targets: vec![], x_value: None,
    }).expect("activate Apprentice Necromancer");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "creature reanimated");
    assert!(g.computed_permanent(dead).unwrap().keywords.contains(&crate::card::Keyword::Haste),
        "reanimated creature has haste");
    assert!(g.battlefield_find(appr).is_none(), "Apprentice Necromancer sacrificed itself");
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_none(), "reanimated creature sacrificed at end step");
}

/// Deafening Silence: a second noncreature spell can't be cast; creature spells
/// are unaffected.
#[test]
fn deafening_silence_limits_noncreature_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::deafening_silence());
    let bolt1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("first noncreature spell ok");
    drain_stack(&mut g);
    let second = g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(matches!(second, Err(GameError::SpellLimitReached)),
        "second noncreature spell blocked, got {second:?}");
    // A creature spell is still castable.
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("creature spell still castable");
}

/// Ethersworn Canonist: a second nonartifact spell can't be cast; artifact
/// spells are unaffected.
#[test]
fn ethersworn_canonist_limits_nonartifact_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ethersworn_canonist());
    let bolt1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let sol = g.add_card_to_hand(0, catalog::sol_ring());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("first nonartifact spell ok");
    drain_stack(&mut g);
    let second = g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(matches!(second, Err(GameError::SpellLimitReached)),
        "second nonartifact spell blocked, got {second:?}");
    g.perform_action(GameAction::CastSpell {
        card_id: sol, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("artifact spell still castable");
}

/// Defense Grid taxes spells {3} unless cast during the caster's own turn.
#[test]
fn defense_grid_taxes_off_turn_spells() {
    let mut g = two_player_game(); // player 0 active
    g.add_card_to_battlefield(0, catalog::defense_grid());
    let id = g.add_card_to_hand(1, catalog::lightning_bolt());
    let bolt = g.players[1].hand.iter().find(|c| c.id == id).unwrap().clone();
    assert_eq!(extra_cost_for_spell(&g, 1, &bolt, None), 3,
        "opponent's spell taxed {{3}} on the active player's turn");
    assert_eq!(extra_cost_for_spell(&g, 0, &bolt, None), 0,
        "active player's own spell untaxed");
}

/// Bontu's Last Reckoning destroys all creatures and keeps the caster's lands
/// from untapping next untap step.
#[test]
fn bontus_last_reckoning_wipes_board_and_locks_lands() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::bontus_last_reckoning());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bontu's Last Reckoning");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "all creatures destroyed");
    // Tap the land and run the untap step — it must stay tapped (charge spent).
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(land).unwrap().tapped, "land stays tapped after Bontu's");
    // The lock is one-shot: the following untap step untaps normally.
    g.do_untap();
    assert!(!g.battlefield_find(land).unwrap().tapped, "land untaps the next step");
}
