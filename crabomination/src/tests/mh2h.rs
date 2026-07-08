//! Functionality tests for `catalog::sets::decks::mh2h` — MH2 sweep batch 9.

use crate::card::{CardType, CounterType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::Target;
use crate::game::*;
use crate::mana::Color;

/// Arcbound Javelineer removes X counters to deal X to an attacker.
#[test]
fn arcbound_javelineer_remove_x() {
    let mut g = two_player_game();
    let jav = g.add_card_to_battlefield(0, catalog::arcbound_javelineer());
    g.battlefield_find_mut(jav).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    g.clear_sickness(jav);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    g.step = crate::game::TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![crate::game::types::Attack {
        attacker: bear,
        target: crate::game::types::AttackTarget::Player(0),
    }])).expect("attack");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: jav, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: Some(2),
    }).expect("remove 2 counters");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2 damage kills the attacking 2/2");
    assert_eq!(g.battlefield_find(jav).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Arcus Acolyte grants outlast to counterless creatures only.
#[test]
fn arcus_acolyte_grants_outlast() {
    let mut g = two_player_game();
    let acolyte = g.add_card_to_battlefield(0, catalog::arcus_acolyte());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(acolyte);
    g.clear_sickness(bear);
    assert_eq!(g.granted_abilities_for(bear).len(), 1, "counterless bear has outlast");
    // Outlast it once: tap + {G/W}.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let printed = catalog::grizzly_bears().activated_abilities.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear, ability_index: printed, target: None,
        additional_targets: vec![], x_value: None,
    }).expect("outlast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.battlefield_find(bear).unwrap().tapped);
    assert!(g.granted_abilities_for(bear).is_empty(), "countered bear loses the grant");
}

/// CR 702.16j — Serra's Emissary protects you and your team from the type.
#[test]
fn cr_702_16j_serras_emissary_protection() {
    let mut g = two_player_game();
    let angel = g.add_card_to_hand(0, catalog::serras_emissary());
    for _ in 0..4 {
        g.players[0].mana_pool.add(Color::White, 1);
    }
    g.players[0].mana_pool.add_colorless(3);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Choose "Creature" (mode 0).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    cast(&mut g, angel);
    assert_eq!(
        g.battlefield_find(angel).unwrap().chosen_card_type,
        Some(CardType::Creature)
    );
    // The team has protection from creatures → an attacking bear deals no
    // combat damage to the player.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    g.step = crate::game::TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![crate::game::types::Attack {
        attacker: bear,
        target: crate::game::types::AttackTarget::Player(0),
    }])).expect("attack");
    let life = g.players[0].life;
    g.step = crate::game::TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "no combat damage from a creature source");
}

/// Shattered Ego shrinks the host and buries it third from the top.
#[test]
fn shattered_ego_bury() {
    let mut g = two_player_game();
    let djinn = g.add_card_to_battlefield(1, catalog::mahamoti_djinn()); // 5/6
    let ego = g.add_card_to_hand(0, catalog::shattered_ego());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: ego, target: Some(Target::Permanent(djinn)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("aura");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(djinn).unwrap().power, 2, "-3/-0");
    // Give the owner some library to sink into.
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::island());
    }
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ego, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None,
    }).expect("bury");
    drain_stack(&mut g);
    assert!(g.battlefield_find(djinn).is_none());
    assert_eq!(g.players[1].library[2].definition.name, "Mahamoti Djinn", "third from top");
}

/// Verdant Command picks two modes (tokens + life here).
#[test]
fn verdant_command_two_modes() {
    let mut g = two_player_game();
    let cmd = catalog::verdant_command();
    let life = g.players[0].life;
    let mut ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Player(0)];
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![0, 3])]));
    let events = g.resolve_effect(&cmd.effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    let squirrels: Vec<_> =
        g.battlefield.iter().filter(|c| c.definition.name == "Squirrel").collect();
    assert_eq!(squirrels.len(), 2);
    assert!(squirrels.iter().all(|c| c.tapped), "tokens enter tapped");
    assert_eq!(g.players[0].life, life + 3);
}

/// Zabaz boosts modular counter moves and animates.
#[test]
fn zabaz_modular_bonus() {
    let mut g = two_player_game();
    let zabaz = g.add_card_to_battlefield(0, catalog::zabaz_the_glimmerwasp());
    g.battlefield_find_mut(zabaz).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let mouser = g.add_card_to_battlefield(0, catalog::arcbound_mouser());
    g.battlefield_find_mut(mouser).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    // Kill the mouser; its modular trigger moves 2+1 counters onto Zabaz.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(zabaz)),
    ]));
    let ctx = crate::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(mouser)), 0, 0);
    let events = g
        .resolve_effect(
            &crate::effect::Effect::Destroy { what: crate::effect::Selector::Target(0) },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(zabaz).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1 + 3,
        "modular 2 + Zabaz bonus 1"
    );
}

/// Lonis investigates on nontoken creatures and steals with sacrificed Clues.
#[test]
fn lonis_investigate_and_steal() {
    let mut g = two_player_game();
    let lonis = g.add_card_to_battlefield(0, catalog::lonis_cryptozoologist());
    g.clear_sickness(lonis);
    // A nontoken creature enters → investigate.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast(&mut g, bear);
    drain_stack(&mut g);
    let clues = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Clue")
        .count();
    assert_eq!(clues, 1, "investigated once");
    // Add a second clue, then sac both to steal a ≤2 MV permanent.
    g.add_token_to_battlefield(0, &crabomination_base::tokens::clue_token());
    g.add_card_to_library(1, catalog::parcel_myr()); // MV 2 — stealable
    g.add_card_to_library(1, catalog::island());
    g.perform_action(GameAction::ActivateAbility {
        card_id: lonis, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: Some(2),
    }).expect("steal");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Parcel Myr" && c.controller == 0),
        "stole the Myr"
    );
    // Both cost Clues were paid; the stolen Myr's own ETB re-investigated.
    let clues_after = g.battlefield.iter().filter(|c| c.definition.name == "Clue").count();
    assert_eq!(clues_after, 1, "cost clues paid; new investigate clue minted");
}

/// Carth digs seven for a planeswalker and taxes loyalty costs upward.
#[test]
fn carth_digs_and_taxes_loyalty() {
    let mut g = two_player_game();
    // ETB dig: a planeswalker in the top seven goes to hand.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::vivien_reid());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let carth = g.add_card_to_hand(0, catalog::carth_the_lion());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(None)]));
    cast(&mut g, carth);
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.is_planeswalker()),
        "dig found the planeswalker"
    );
    // Loyalty tax: Vivien's +1 now adds 2 loyalty.
    let viv_id = g.players[0].hand.iter().find(|c| c.definition.is_planeswalker()).unwrap().id;
    let viv = g.players[0].remove_from_hand(viv_id).unwrap();
    g.battlefield.push(viv);
    let base = g.battlefield_find(viv_id).map(|c| c.definition.base_loyalty).unwrap();
    g.battlefield_find_mut(viv_id).unwrap().counters.insert(CounterType::Loyalty, base);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: viv_id, ability_index: 0, target: None, x_value: None,
    }).expect("loyalty");
    let loyalty = g.battlefield_find(viv_id).unwrap().counter_count(CounterType::Loyalty);
    assert_eq!(loyalty, base + 2, "+1 ability pays as +2 under Carth");
}

/// Bloodbraid Marauder cascades only with delirium.
#[test]
fn bloodbraid_marauder_delirium_cascade() {
    let mut g = two_player_game();
    // No delirium: no cascade.
    g.add_card_to_library(0, catalog::sacred_cat()); // MV 1 — cascade hit
    let m1 = g.add_card_to_hand(0, catalog::bloodbraid_marauder());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast(&mut g, m1);
    drain_stack(&mut g);
    assert!(
        !g.battlefield.iter().any(|c| c.definition.name == "Sacred Cat"),
        "no cascade without delirium"
    );
    // Fill the graveyard with four card types → cascade fires.
    g.add_card_to_graveyard(0, catalog::island());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::parcel_myr());
    g.add_card_to_graveyard(0, catalog::shattered_ego());
    let m2 = g.add_card_to_hand(0, catalog::bloodbraid_marauder());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, m2);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Sacred Cat"),
        "delirium cascade free-cast the cat"
    );
}

/// Rise and Shine animates an artifact with four counters; overload hits all.
#[test]
fn rise_and_shine_animates() {
    let mut g = two_player_game();
    let myr = g.add_card_to_battlefield(0, catalog::parcel_myr());
    let mox = g.add_card_to_battlefield(0, catalog::chrome_mox());
    let over = catalog::rise_and_shine()
        .alternative_cost
        .as_ref()
        .unwrap()
        .effect_override
        .clone()
        .unwrap();
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(&over, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    // Parcel Myr is already a creature — untouched; the Mox animates 0/0+4.
    let mox_cp = g.computed_permanent(mox).unwrap();
    assert!(mox_cp.card_types.contains(&CardType::Creature));
    assert_eq!((mox_cp.power, mox_cp.toughness), (4, 4), "0/0 with four counters");
    assert_eq!(g.battlefield_find(mox).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
    let myr_cp = g.computed_permanent(myr).unwrap();
    assert_eq!(myr_cp.power, catalog::parcel_myr().power, "creature untouched");
}

/// The Zabaz flying pump works.
#[test]
fn zabaz_flying_pump() {
    let mut g = two_player_game();
    let zabaz = g.add_card_to_battlefield(0, catalog::zabaz_the_glimmerwasp());
    g.battlefield_find_mut(zabaz).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.clear_sickness(zabaz);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: zabaz, ability_index: 1, target: None,
        additional_targets: vec![], x_value: None,
    }).expect("flying");
    drain_stack(&mut g);
    assert!(g.computed_permanent(zabaz).unwrap().keywords.contains(&Keyword::Flying));
}
