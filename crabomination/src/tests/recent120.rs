//! Functionality tests for `catalog::sets::decks::recent120`.

use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// Fire a `LifeGained` event so lifegain-matters triggers dispatch.
fn gain_life(g: &mut GameState, seat: usize, amount: i32) {
    let before = g.players[seat].life;
    g.adjust_life(seat, amount);
    let delta = g.players[seat].life - before;
    if delta > 0 {
        g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: seat, amount: delta as u32 }]);
        drain_stack(g);
    }
}

fn token_count(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

/// Crypt Feaster gets +2/+0 on attack only under threshold.
#[test]
fn crypt_feaster_threshold_attack_pump() {
    let mut g = two_player_game();
    let feaster = g.add_card_to_battlefield(0, catalog::crypt_feaster());
    g.clear_sickness(feaster);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: feaster, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(feaster).unwrap().power, 3, "no threshold → no pump");
}

/// With seven cards in the graveyard Crypt Feaster's attack pumps it to 5/4.
#[test]
fn crypt_feaster_pumps_with_seven_in_graveyard() {
    let mut g = two_player_game();
    let feaster = g.add_card_to_battlefield(0, catalog::crypt_feaster());
    g.clear_sickness(feaster);
    for _ in 0..7 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: feaster, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(feaster).unwrap().power, 5, "threshold → +2/+0");
}

/// Elfsworn Giant makes a 1/1 Elf Warrior whenever a land you control enters.
#[test]
fn elfsworn_giant_landfall_token() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::elfsworn_giant());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play a land");
    drain_stack(&mut g);
    assert_eq!(token_count(&g, 0, "Elf Warrior"), 1, "landfall made an Elf Warrior");
}

/// Elvish Regrower returns a permanent card from the graveyard on ETB.
#[test]
fn elvish_regrower_returns_permanent_card() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let reg = g.add_card_to_battlefield(0, catalog::elvish_regrower());
    g.fire_self_etb_triggers(reg, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "the bear returned to hand");
}

/// Courageous Goblin pumps + gains menace only while you control a 4-power creature.
#[test]
fn courageous_goblin_conditional_attack() {
    let mut g = two_player_game();
    let goblin = g.add_card_to_battlefield(0, catalog::courageous_goblin());
    g.clear_sickness(goblin);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: goblin, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(goblin).unwrap().power, 2, "no 4-power ally → no bonus");

    // Reset and add a big creature so the intervening-if is satisfied.
    let mut g = two_player_game();
    let goblin = g.add_card_to_battlefield(0, catalog::courageous_goblin());
    g.clear_sickness(goblin);
    let big = g.add_card_to_battlefield(0, catalog::elfsworn_giant()); // 5/3
    g.clear_sickness(big);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: goblin, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(goblin).unwrap();
    assert_eq!(cp.power, 3, "4-power ally → +1/+0");
    assert!(cp.keywords.contains(&crate::card::Keyword::Menace), "and gains menace");
}

/// Eager Trufflesnout makes a Food when it connects with a player.
#[test]
fn eager_trufflesnout_food_on_combat_damage() {
    let mut g = two_player_game();
    let boar = g.add_card_to_battlefield(0, catalog::eager_trufflesnout());
    g.clear_sickness(boar);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: boar, target: AttackTarget::Player(1),
    }])).expect("attack");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(token_count(&g, 0, "Food"), 1, "combat damage made a Food");
}

/// Cat Collector makes a Food on ETB and a Cat on the first lifegain each turn.
#[test]
fn cat_collector_food_etb_and_first_lifegain_cat() {
    let mut g = two_player_game();
    let cc = g.add_card_to_battlefield(0, catalog::cat_collector());
    g.fire_self_etb_triggers(cc, 0);
    drain_stack(&mut g);
    assert_eq!(token_count(&g, 0, "Food"), 1, "ETB Food");

    // First lifegain on your turn → a Cat; the second does nothing.
    gain_life(&mut g, 0, 2);
    assert_eq!(token_count(&g, 0, "Cat"), 1, "first lifegain → Cat");
    gain_life(&mut g, 0, 2);
    assert_eq!(token_count(&g, 0, "Cat"), 1, "second lifegain same turn → no Cat");
}

/// Dawnwing Marshal's activated ability pumps the team +1/+1.
#[test]
fn dawnwing_marshal_team_pump() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let marshal = g.add_card_to_battlefield(0, catalog::dawnwing_marshal());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: marshal, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "bear 2/2 → 3/3");
    assert_eq!(g.computed_permanent(marshal).unwrap().toughness, 3, "marshal 2/2 → 3/3");
}

/// Clinquant Skymage grows with a +1/+1 counter each time you draw.
#[test]
fn clinquant_skymage_grows_on_draw() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::clinquant_skymage());
    let drawn = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::CardDrawn { player: 0, card_id: drawn }]);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mage).unwrap().power, 2, "1/1 + a draw → 2/2");
}

/// Elementalist Adept has flash and prowess.
#[test]
fn elementalist_adept_flash_prowess() {
    let mut g = two_player_game();
    let adept = g.add_card_to_battlefield(0, catalog::elementalist_adept());
    let cp = g.computed_permanent(adept).unwrap();
    assert!(cp.keywords.contains(&crate::card::Keyword::Flash), "has flash");
    assert!(cp.keywords.contains(&crate::card::Keyword::Prowess), "has prowess");
}

/// Divine Resilience unkicked protects one creature; kicked it protects the team.
#[test]
fn divine_resilience_kicked_protects_team() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::divine_resilience());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: spell, target: Some(Target::Permanent(a)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Divine Resilience kicked");
    drain_stack(&mut g);
    assert!(g.computed_permanent(a).unwrap().keywords.contains(&crate::card::Keyword::Indestructible));
    assert!(g.computed_permanent(b).unwrap().keywords.contains(&crate::card::Keyword::Indestructible),
        "kicked → both creatures protected");
}
