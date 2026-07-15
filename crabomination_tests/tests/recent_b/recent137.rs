//! Functionality tests for `catalog::sets::decks::recent137` (WOE wave 10).

use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn cast_adventure(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastAdventure {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast adventure");
    drain_stack(g);
}

/// Storyteller Pixie draws when you cast an Adventure spell.
#[test]
fn storyteller_pixie_draws_on_adventure() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::storyteller_pixie());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    // Ride the Rails (Minecart Daredevil's adventure): +2/+1.
    let daredevil = g.add_card_to_hand(0, catalog::minecart_daredevil());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_adventure(&mut g, daredevil, Some(Target::Permanent(bear)));
    // Cast consumed the adventurer (−1) but the Pixie drew (+1) → net even.
    assert_eq!(g.players[0].hand.len(), hand_before, "Pixie drew for the Adventure cast");
}

/// Desperate Parry weakens a blocker with -4/-0.
#[test]
fn desperate_parry_shrinks_power() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let card = g.add_card_to_hand(0, catalog::obyras_attendants());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_adventure(&mut g, card, Some(Target::Permanent(angel)));
    assert_eq!(g.computed_permanent(angel).unwrap().power, 0, "-4/-0 zeroed the angel's power");
}

/// High Fae Negotiator's bargained ETB drains for 3.
#[test]
fn high_fae_negotiator_bargain_drain() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // A Treasure token to sacrifice for Bargain.
    let treasure = g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
    let card = g.add_card_to_hand(0, catalog::high_fae_negotiator());
    g.players[0].life = 20;
    g.players[1].life = 20;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpellBargain {
        card_id: card,
        sacrifice: Some(treasure),
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bargained");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "opponent lost 3 to the bargained ETB");
    assert_eq!(g.players[0].life, 23, "you gained 3");
}

/// Fell Horseman's Deathly Ride returns a creature card from the graveyard.
#[test]
fn deathly_ride_returns_from_graveyard() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let card = g.add_card_to_hand(0, catalog::fell_horseman());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_adventure(&mut g, card, Some(Target::Permanent(dead)));
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "Grizzly Bears returned to hand",
    );
}

/// Shrouded Shepherd's ETB pumps a creature you control.
#[test]
fn shrouded_shepherd_etb_pump() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let card = g.add_card_to_hand(0, catalog::shrouded_shepherd());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, card, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 4, "+2/+2 on the bear");
}

/// Intrepid Trufflesnout makes a Food only when it attacks alone.
#[test]
fn trufflesnout_food_on_solo_attack() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let boar = g.add_card_to_battlefield(0, catalog::intrepid_trufflesnout());
    g.step = TurnStep::PreCombatMain;
    g.clear_sickness(boar);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: boar,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack alone");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Food"),
        "attacking alone made a Food",
    );
}
