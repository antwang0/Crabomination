//! Functionality tests for `catalog::sets::decks::recent136` (WOE wave 9).

use crate::card::Keyword;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

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

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}


/// Royal Treatment grants hexproof and mints a Royal Role.
#[test]
fn royal_treatment_hexproof_and_role() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::royal_treatment());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Hexproof), "gained hexproof");
    assert_eq!(cp.power, 3, "Royal Role gives +1/+1");
    assert!(
        g.battlefield.iter().any(|c| c.attached_to == Some(bear) && c.definition.name == "Royal"),
        "Royal Role attached",
    );
}

/// Merfolk Coralsmith's {1} ability shifts +1/-1 until end of turn.
#[test]
fn merfolk_coralsmith_self_pump() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let fish = g.add_card_to_battlefield(0, catalog::merfolk_coralsmith());
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, fish, 0, None);
    let cp = g.computed_permanent(fish).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2), "+1/-1 until end of turn");
}

/// Living Lectern sacrifices to draw and mint a Sorcerer Role on another creature.
#[test]
fn living_lectern_draws_and_roles() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let lectern = g.add_card_to_battlefield(0, catalog::living_lectern());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(1);
    g.clear_sickness(lectern);
    activate(&mut g, lectern, 0, Some(Target::Permanent(ally)));
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    assert!(g.battlefield_find(lectern).is_none(), "Lectern sacrificed");
    assert!(
        g.battlefield.iter().any(|c| c.attached_to == Some(ally) && c.definition.name == "Sorcerer"),
        "Sorcerer Role on the ally",
    );
}

/// Stingblade Assassin destroys a creature that was dealt damage this turn.
#[test]
fn stingblade_kills_damaged_creature() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    // Mark the angel as damaged this turn.
    let ctx = crate::game::effects::EffectContext::for_ability(
        victim, 0, Some(Target::Permanent(victim)),
    );
    g.resolve_effect(
        &crate::effect::Effect::DealDamage {
            to: crate::effect::Selector::Target(0),
            amount: crate::effect::Value::Const(1),
        },
        &ctx,
    )
    .unwrap();
    let assassin = g.add_card_to_hand(0, catalog::stingblade_assassin());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, assassin, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none(), "damaged angel destroyed");
}

/// Lord Skitter's Butcher mode 0 makes a Rat token.
#[test]
fn lord_skitters_butcher_makes_rat() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Force mode 0 (create a Rat).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    let butcher = g.add_card_to_hand(0, catalog::lord_skitters_butcher());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, butcher, None);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Rat"),
        "mode 0 created a Rat token",
    );
}

/// Provisions Merchant enters with a Food token.
#[test]
fn provisions_merchant_makes_food() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let merch = g.add_card_to_hand(0, catalog::provisions_merchant());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, merch, None);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Food"),
        "ETB created a Food token",
    );
}

/// Scarecrow Guide's once-per-turn mana ability can't be activated twice.
#[test]
fn scarecrow_guide_once_per_turn_mana() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let guide = g.add_card_to_battlefield(0, catalog::scarecrow_guide());
    g.clear_sickness(guide);
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, guide, 0, None);
    // Second activation this turn must be rejected (once-per-turn gate).
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: guide,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    });
    assert!(err.is_err(), "second activation blocked by once-per-turn");
}
