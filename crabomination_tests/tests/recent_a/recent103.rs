//! CR 702.141 Encore — Commander Legends batch (`decks::recent103`).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::TurnStep;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};

/// Encore: exiling Impulsive Pilferer from the graveyard for {3}{R} mints a
/// hasty token copy per opponent that must attack; it's sacrificed at the
/// next end step.
#[test]
fn cr_702_141_encore_mints_attacking_copies() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::impulsive_pilferer());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dead, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("encore from the graveyard");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == dead), "source exiled as the cost");
    let copy = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Impulsive Pilferer")
        .expect("one token copy (one opponent)");
    assert!(g.computed_permanent(copy.id).unwrap().keywords.contains(&Keyword::Haste));
    assert_eq!(copy.goaded_by, vec![0], "attacks-if-able requirement");
    let copy_id = copy.id;
    // The copy is sacrificed at the beginning of the next end step; its
    // dies-trigger still mints the Treasure.
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(copy_id).is_none(), "token sacrificed at end step");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
        "dies trigger fired off the sacrifice");
}

/// Encore respects sorcery timing: not activatable during an opponent's turn
/// priority window off-main.
#[test]
fn encore_is_sorcery_speed() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::trove_tracker());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(5);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: dead, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).is_err(), "sorcery-only activation rejected mid-combat");
}

/// Kilnmouth Dragon's amplify counters power its {T} ping (Amplify 3 with one
/// Dragon in hand → 3 counters → 3 damage).
#[test]
fn kilnmouth_dragon_pings_for_its_counters() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::kilnmouth_dragon()); // a Dragon to reveal
    let kiln = g.move_card_to_battlefield_for_test(0, catalog::kilnmouth_dragon());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(kiln).unwrap()
        .counter_count(crabomination::card::CounterType::PlusOnePlusOne), 3, "amplify 3 × 1 Dragon");
    g.clear_sickness(kiln);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: kiln, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("tap to ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "3 damage");
}
