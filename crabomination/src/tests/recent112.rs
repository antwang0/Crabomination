//! Functionality tests for `catalog::sets::decks::recent112` — {Q} untap
//! costs, Phyrexian Unlife, and spellslinger bodies.

use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::mana::Color;

/// CR 107.17 — {Q}: the source must be tapped; paying untaps it.
#[test]
fn cr_107_17_pili_pala_untap_cost() {
    let mut g = two_player_game();
    let pala = g.add_card_to_battlefield(0, catalog::pili_pala());
    g.clear_sickness(pala);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    // Untapped: {Q} is unpayable.
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: pala, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    });
    assert!(err.is_err(), "{{Q}} needs a tapped source");
    g.battlefield_find_mut(pala).unwrap().tapped = true;
    g.perform_action(GameAction::ActivateAbility {
        card_id: pala, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("pay {2}, {Q}");
    assert!(!g.battlefield_find(pala).unwrap().tapped, "untapped as the cost");
    assert_eq!(g.players[0].mana_pool.total(), 1, "one mana of any color added");
}

/// CR 602.5h — a summoning-sick creature can't pay {Q} either.
#[test]
fn cr_602_5h_untap_cost_summoning_sick() {
    let mut g = two_player_game();
    let pala = g.add_card_to_battlefield(0, catalog::pili_pala());
    g.battlefield_find_mut(pala).unwrap().tapped = true;
    g.battlefield_find_mut(pala).unwrap().summoning_sick = true;
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: pala, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    });
    assert!(matches!(err, Err(GameError::SummoningSickness(_))));
}

/// Phyrexian Unlife — no loss at ≤ 0 life; damage lands as poison instead.
#[test]
fn phyrexian_unlife_zero_life_mode() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::phyrexian_unlife());
    g.players[0].life = -3;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    assert!(!g.players[0].eliminated, "no loss from life with Unlife");
    // Damage at ≤ 0 life becomes poison, not further life loss.
    let ctx = crate::game::effects::EffectContext::for_spell(1, Some(Target::Player(0)), 0, 0);
    let events = g
        .resolve_effect(
            &crate::effect::Effect::DealDamage {
                to: crate::effect::Selector::Target(0),
                amount: crate::effect::Value::Const(4),
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    assert_eq!(g.players[0].life, -3, "life untouched");
    assert_eq!(g.players[0].poison_counters, 4, "damage became poison");
    // Ten poison still loses through Unlife.
    g.players[0].poison_counters = 10;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    assert!(g.players[0].eliminated, "poison loss is not gated");
}

/// Salvage Titan's alternative cost eats three artifacts.
#[test]
fn salvage_titan_alt_cost() {
    let mut g = two_player_game();
    let a1 = g.add_card_to_battlefield(0, catalog::chrome_mox());
    let a2 = g.add_card_to_battlefield(0, catalog::shriekhorn());
    let a3 = g.add_card_to_battlefield(0, catalog::darksteel_garrison());
    let titan = g.add_card_to_hand(0, catalog::salvage_titan());
    g.step = crate::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: titan, target: None, additional_targets: vec![], mode: None,
        x_value: None, pitch_card: None,
    })
    .expect("free via three sacrifices");
    drain_stack(&mut g);
    assert!(g.battlefield_find(titan).is_some());
    assert!(
        g.battlefield_find(a1).is_none()
            && g.battlefield_find(a2).is_none()
            && g.battlefield_find(a3).is_none(),
        "three artifacts sacrificed"
    );
}

/// Salvage Titan returns from the graveyard by exiling three artifacts.
#[test]
fn salvage_titan_graveyard_return() {
    let mut g = two_player_game();
    let titan = g.add_card_to_graveyard(0, catalog::salvage_titan());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::chrome_mox());
    }
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: titan, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("return from graveyard");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == titan), "Titan to hand");
    assert_eq!(
        g.exile.iter().filter(|c| c.definition.name == "Chrome Mox").count(),
        3,
        "three artifacts exiled as the cost"
    );
}

/// Qasali Ambusher is free (with flash) only under attack with Forest+Plains.
#[test]
fn qasali_ambusher_free_when_ambushing() {
    let mut g = two_player_game();
    let cat = g.add_card_to_hand(0, catalog::qasali_ambusher());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::savannah()); // Forest Plains dual
    g.step = crate::game::TurnStep::DeclareBlockers;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    // No attacker yet → the alternative cast is rejected.
    let err = g.perform_action(GameAction::CastSpellAlternative {
        card_id: cat, target: None, additional_targets: vec![], mode: None,
        x_value: None, pitch_card: None,
    });
    assert!(err.is_err(), "no attacker → no free cast");
    let raider = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking.push(crate::game::types::Attack {
        attacker: raider,
        target: crate::game::types::AttackTarget::Player(0),
    });
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: cat, target: None, additional_targets: vec![], mode: None,
        x_value: None, pitch_card: None,
    })
    .expect("ambush: free flash cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cat).is_some());
}

/// Boros Reckoner bounces damage dealt to it at any target.
#[test]
fn boros_reckoner_bounces_damage() {
    let mut g = two_player_game();
    let reckoner = g.add_card_to_battlefield(0, catalog::boros_reckoner());
    let life1 = g.players[1].life;
    let ctx = crate::game::effects::EffectContext::for_spell(1, Some(Target::Permanent(reckoner)), 0, 0);
    let events = g
        .resolve_effect(
            &crate::effect::Effect::DealDamage {
                to: crate::effect::Selector::Target(0),
                amount: crate::effect::Value::Const(3),
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 3, "3 damage bounced at the opponent");
}

/// Blistercoil Weird grows and untaps on an instant cast.
#[test]
fn blistercoil_weird_untaps_on_cast() {
    let mut g = two_player_game();
    let weird = g.add_card_to_battlefield(0, catalog::blistercoil_weird());
    g.battlefield_find_mut(weird).unwrap().tapped = true;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(weird).unwrap();
    assert!(!c.tapped, "untapped by the trigger");
    let cp = g.computed_permanent(weird).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "+1/+1 EOT");
}

/// Sage of the Falls offers a loot when a non-Human creature enters.
#[test]
fn sage_of_the_falls_loots() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let sage = g.add_card_to_battlefield(0, catalog::sage_of_the_falls());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: sage }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "drew one, discarded one");
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Elusive Spellfist pumps and slips through on a noncreature cast.
#[test]
fn elusive_spellfist_unblockable_on_cast() {
    let mut g = two_player_game();
    let monk = g.add_card_to_battlefield(0, catalog::elusive_spellfist());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(monk).unwrap();
    assert_eq!(cp.power, 2, "+1/+0");
    assert!(cp.keywords.contains(&crate::card::Keyword::Unblockable));
}
