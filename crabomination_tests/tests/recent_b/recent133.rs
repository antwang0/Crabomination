//! Functionality tests for `catalog::sets::decks::recent133` (WOE wave 6).

use crabomination::card::{CardType, Keyword};
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn role_on(g: &GameState, host: CardId, name: &str) -> bool {
    g.battlefield.iter().any(|c| c.attached_to == Some(host) && c.definition.name == name)
}

/// Rat Out shrinks a creature and leaves a Rat.
#[test]
fn rat_out_shrink_and_rat() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::rat_out());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(enemy)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Rat Out");
    drain_stack(&mut g);
    let cp = g.computed_permanent(enemy).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "-1/-1");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Rat"));
}

/// Eriette's Whisper discards two and hangs a Wicked Role that drains on death.
#[test]
fn eriettes_whisper_discard_and_wicked_role() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[1].hand.clear();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::eriettes_whisper());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Eriette's Whisper");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "opponent discarded both cards");
    assert!(role_on(&g, bear, "Wicked"), "Wicked Role attached");
    // The Role draining on death.
    g.players[1].life = 20;
    let role = g.battlefield.iter().find(|c| c.definition.name == "Wicked").unwrap().id;
    let ctx = crabomination::game::effects::EffectContext::for_ability(role, 0, Some(Target::Permanent(role)));
    g.resolve_effect(
        &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
        &ctx,
    )
    .unwrap();
    g.dispatch_triggers_for_events(&[]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "opponent loses 1 when the Wicked Role dies");
}

/// Edgewall Pack makes a Rat on entry.
#[test]
fn edgewall_pack_rat_on_etb() {
    let mut g = two_player_game();
    let pack = g.add_card_to_battlefield(0, catalog::edgewall_pack());
    g.fire_self_etb_triggers(pack, 0);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Rat"));
}

/// Spider Food destroys a flyer and makes a Food.
#[test]
fn spider_food_destroy_and_food() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // flying
    let spell = g.add_card_to_hand(0, catalog::spider_food());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(flyer)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Spider Food");
    drain_stack(&mut g);
    assert!(g.battlefield_find(flyer).is_none(), "flyer destroyed");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"));
}

/// Cursed Courtier shrinks itself to 1/1 with a Cursed Role.
#[test]
fn cursed_courtier_self_cursed_role() {
    let mut g = two_player_game();
    let courtier = g.add_card_to_battlefield(0, catalog::cursed_courtier());
    g.fire_self_etb_triggers(courtier, 0);
    drain_stack(&mut g);
    let cp = g.computed_permanent(courtier).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "Cursed Role makes it 1/1");
    assert!(cp.keywords.contains(&Keyword::Lifelink));
}

/// Dutiful Griffin returns itself from the graveyard for two enchantments.
#[test]
fn dutiful_griffin_graveyard_recursion() {
    let mut g = two_player_game();
    // Griffin in graveyard, two enchantments to sacrifice.
    let griffin = g.add_card_to_hand(0, catalog::dutiful_griffin());
    let i = g.players[0].hand.iter().position(|c| c.id == griffin).unwrap();
    let c = g.players[0].hand.remove(i);
    g.players[0].graveyard.push(c);
    g.add_card_to_battlefield(0, dummy_enchantment());
    g.add_card_to_battlefield(0, dummy_enchantment());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: griffin,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate graveyard return");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == griffin), "Griffin back in hand");
}

/// Tuinvale Guide grows and gains lifelink under Celebration.
#[test]
fn tuinvale_guide_celebration() {
    let mut g = two_player_game();
    let guide = g.add_card_to_battlefield(0, catalog::tuinvale_guide());
    // No celebration yet → bare 2/3, no lifelink.
    let cp = g.computed_permanent(guide).unwrap();
    assert!(!cp.keywords.contains(&Keyword::Lifelink));
    // Two nonland permanents entered this turn → celebration active.
    g.players[0].nonland_permanents_entered_this_turn = 2;
    let cp = g.computed_permanent(guide).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+0");
    assert!(cp.keywords.contains(&Keyword::Lifelink));
}

/// Candy Trail scrys on entry and cashes in for life + a card.
#[test]
fn candy_trail_scry_and_sac() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let trail = g.add_card_to_battlefield(0, catalog::candy_trail());
    g.fire_self_etb_triggers(trail, 0); // scry 2, no assert on order
    drain_stack(&mut g);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: trail,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("cash in Candy Trail");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "gained 3 life");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert!(g.battlefield_find(trail).is_none(), "artifact sacrificed");
}

fn dummy_enchantment() -> crabomination::card::CardDefinition {
    use crabomination::mana::{cost, generic};
    crabomination::card::CardDefinition {
        name: "Test Glyph",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    }
}
