//! Functionality tests for `catalog::sets::decks::recent132` (WOE wave 5).

use crabomination::card::{
    CardDefinition, CardType, EnchantmentSubtype, Keyword, Subtypes,
};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::{cost, generic, Color};

/// A vanilla enchantment fixture for the enchantment-death triggers.
fn dummy_enchantment() -> CardDefinition {
    CardDefinition {
        name: "Test Glyph",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        ..Default::default()
    }
}

fn kill(g: &mut GameState, id: CardId) {
    let ctl = g.battlefield_find(id).unwrap().controller;
    let ctx = crabomination::game::effects::EffectContext::for_ability(id, ctl, Some(Target::Permanent(id)));
    g.resolve_effect(
        &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
        &ctx,
    )
    .unwrap();
    g.dispatch_triggers_for_events(&[]);
    drain_stack(g);
}

fn has_role_on(g: &GameState, host: CardId) -> bool {
    g.battlefield.iter().any(|c| {
        c.attached_to == Some(host)
            && c.definition.subtypes.enchantment_subtypes.contains(&EnchantmentSubtype::Role)
    })
}

/// Squeak By pumps and grants power-3+ evasion; the new keyword blocks a big
/// blocker but not a small one.
#[test]
fn squeak_by_pump_and_evasion() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let weak = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let card = g.add_card_to_hand(0, catalog::cheeky_house_mouse());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastAdventure {
        card_id: card,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Squeak By");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
    assert!(cp.keywords.contains(&Keyword::CantBeBlockedByPowerAtLeast(3)));
    assert!(!g.blocker_can_block_attacker(big, bear), "power-4 can't block");
    assert!(g.blocker_can_block_attacker(weak, bear), "power-2 can block");
}

/// Betroth the Beast attaches a Royal Role (+1/+1, ward).
#[test]
fn betroth_the_beast_royal_role() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let card = g.add_card_to_hand(0, catalog::besotted_knight());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastAdventure {
        card_id: card,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Betroth the Beast");
    drain_stack(&mut g);
    assert!(has_role_on(&g, bear), "Royal Role attached");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "Royal Role gives +1/+1");
    assert!(cp.keywords.iter().any(|k| matches!(k, Keyword::Ward(_))), "has ward");
}

/// Charmed Clothier hangs a Royal Role on another creature.
#[test]
fn charmed_clothier_royal_role() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let clothier = g.add_card_to_battlefield(0, catalog::charmed_clothier());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    g.fire_self_etb_triggers(clothier, 0);
    drain_stack(&mut g);
    assert!(has_role_on(&g, bear), "Royal Role on the bear");
}

/// Ashiok's Reaper draws when your enchantment dies.
#[test]
fn ashioks_reaper_draw_on_enchantment_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ashioks_reaper());
    let glyph = g.add_card_to_battlefield(0, dummy_enchantment());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    kill(&mut g, glyph);
    assert_eq!(g.players[0].hand.len(), before + 1, "drew a card");
}

/// Twice the Rage grants double strike.
#[test]
fn twice_the_rage_double_strike() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let card = g.add_card_to_hand(0, catalog::two_headed_hunter());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastAdventure {
        card_id: card,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Twice the Rage");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

/// That's Mine makes a Treasure.
#[test]
fn thats_mine_treasure() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let card = g.add_card_to_hand(0, catalog::grabby_giant());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastAdventure {
        card_id: card,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast That's Mine");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"));
}

/// Hollow Scavenger eats a Food for +2/+2.
#[test]
fn hollow_scavenger_food_pump() {
    let mut g = two_player_game();
    let scav = g.add_card_to_battlefield(0, catalog::hollow_scavenger());
    g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: scav,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("sac a Food");
    drain_stack(&mut g);
    let cp = g.computed_permanent(scav).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 4), "+2/+2");
    assert!(
        !g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"),
        "Food sacrificed",
    );
}

/// Skybeast Tracker makes a Food when you cast a 5-drop.
#[test]
fn skybeast_tracker_food_on_big_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::skybeast_tracker());
    let big = g.add_card_to_hand(0, CardDefinition {
        name: "Test Bomb",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Sorcery],
        ..Default::default()
    });
    g.players[0].mana_pool.add_colorless(5);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: big,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast 5-drop");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"));
}

/// Verdant Outrider grants itself power-2-or-less evasion.
#[test]
fn verdant_outrider_evasion() {
    let mut g = two_player_game();
    let rider = g.add_card_to_battlefield(0, catalog::verdant_outrider());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rider,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate Verdant Outrider");
    drain_stack(&mut g);
    assert!(g
        .computed_permanent(rider)
        .unwrap()
        .keywords
        .contains(&Keyword::CantBeBlockedByPowerAtMost(2)));
}
