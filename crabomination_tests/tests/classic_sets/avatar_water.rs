//! Functionality tests for `catalog::sets::decks::avatar_water` — Waterbend
//! (CR 701.67) as an additional cast cost and as an activated-ability cost.

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::mana::Color;
use crabomination::game::two_player_game;
use crabomination::game::*;

/// Tap a battlefield card's sickness so it can serve as a waterbend helper.
fn ready(g: &mut GameState, id: CardId) {
    g.clear_sickness(id);
}

#[test]
fn waterbend_helpers_pay_the_additional_cost() {
    // Benevolent River Spirit — {U}{U}, waterbend {5} (mandatory). With only UU
    // of real mana, tapping five helpers covers the whole waterbend sub-cost.
    let mut g = two_player_game();
    let mut helpers = Vec::new();
    for _ in 0..5 {
        let h = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        ready(&mut g, h);
        helpers.push(h);
    }
    let spirit = g.add_card_to_hand(0, catalog::benevolent_river_spirit());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpellWaterbend {
        card_id: spirit, target: None, additional_targets: vec![], mode: None, x_value: None,
        helpers: helpers.clone(),
    }).expect("waterbend helpers cover the {5}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == spirit), "Spirit resolved onto the battlefield");
    // All five helpers were tapped to pay.
    assert!(helpers.iter().all(|h| g.battlefield_find(*h).unwrap().tapped), "helpers tapped");
}

#[test]
fn mandatory_waterbend_paid_from_mana_on_plain_cast() {
    // The plain CastSpell path still pays a mandatory waterbend from real mana.
    let mut g = two_player_game();
    let spirit = g.add_card_to_hand(0, catalog::benevolent_river_spirit());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // UU only — short {5} for the waterbend, no helpers → rejected.
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: spirit, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "mandatory waterbend can't be skipped");
    // Add the {5} and the plain cast succeeds.
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: spirit, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("waterbend paid entirely from mana");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == spirit));
}

#[test]
fn too_many_helpers_rejected() {
    // Helpers are clamped to the waterbend amount; six helpers for waterbend {5}
    // is illegal (they'd otherwise discount the base cost too).
    let mut g = two_player_game();
    let mut helpers = Vec::new();
    for _ in 0..6 {
        let h = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        ready(&mut g, h);
        helpers.push(h);
    }
    let spirit = g.add_card_to_hand(0, catalog::benevolent_river_spirit());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    assert!(g.perform_action(GameAction::CastSpellWaterbend {
        card_id: spirit, target: None, additional_targets: vec![], mode: None, x_value: None,
        helpers,
    }).is_err(), "six helpers for waterbend {{5}} is rejected");
}

#[test]
fn optional_waterbend_branches_on_payment() {
    // Waterbending Lesson — draw 3, then discard unless you waterbend {2}.
    // Decline (plain cast): the discard fires.
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let lesson = g.add_card_to_hand(0, catalog::waterbending_lesson());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: lesson, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast without waterbend");
    drain_stack(&mut g);
    // Drew 3, discarded 1 → net +2 from the starting hand (minus the lesson itself).
    assert_eq!(g.players[0].hand.len(), before - 1 + 3 - 1, "declined → discard fires");
}

#[test]
fn optional_waterbend_paid_skips_the_downside() {
    // Pay the waterbend {2} (with two helpers) → no discard.
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let h1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let h2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    ready(&mut g, h1); ready(&mut g, h2);
    let lesson = g.add_card_to_hand(0, catalog::waterbending_lesson());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpellWaterbend {
        card_id: lesson, target: None, additional_targets: vec![], mode: None, x_value: None,
        helpers: vec![h1, h2],
    }).expect("cast paying waterbend");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before - 1 + 3, "paid → no discard");
}

#[test]
fn waterbend_x_reads_the_chosen_x() {
    // Waterbender's Restoration — waterbend {X}, exile X of your creatures and
    // return them at the next end step. X=2 paid by two helpers.
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    ready(&mut g, a); ready(&mut g, b);
    let spell = g.add_card_to_hand(0, catalog::waterbenders_restoration());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpellWaterbend {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
        helpers: vec![a, b],
    }).expect("waterbend {X=2} via two helpers");
    drain_stack(&mut g);
    // Both creatures were exiled (blinked).
    assert!(!g.battlefield.iter().any(|c| c.id == a || c.id == b), "X=2 exiled both");
}

#[test]
fn activated_waterbend_sets_base_pt() {
    // Flexible Waterbender — Waterbend {3}: base P/T becomes 5/2. Pay the {3}
    // with three helper taps.
    let mut g = two_player_game();
    let bender = g.add_card_to_battlefield(0, catalog::flexible_waterbender());
    ready(&mut g, bender);
    let mut helpers = Vec::new();
    for _ in 0..3 {
        let h = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        ready(&mut g, h);
        helpers.push(h);
    }
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbilityWaterbend {
        card_id: bender, ability_index: 0, target: None, additional_targets: vec![],
        x_value: None, helpers,
    }).expect("waterbend {3} ability");
    drain_stack(&mut g);
    let v = g.compute_battlefield();
    let b = v.iter().find(|c| c.id == bender).unwrap();
    assert_eq!((b.power, b.toughness), (5, 2), "base P/T set to 5/2");
}

#[test]
fn activated_waterbend_needs_full_amount() {
    // Two helpers for a Waterbend {3} ability, no mana → short {1}, rejected.
    let mut g = two_player_game();
    let bender = g.add_card_to_battlefield(0, catalog::flexible_waterbender());
    ready(&mut g, bender);
    let h1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let h2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    ready(&mut g, h1); ready(&mut g, h2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    assert!(g.perform_action(GameAction::ActivateAbilityWaterbend {
        card_id: bender, ability_index: 0, target: None, additional_targets: vec![],
        x_value: None, helpers: vec![h1, h2],
    }).is_err(), "two helpers short of {{3}} with no mana");
}

#[test]
fn giant_koi_becomes_unblockable() {
    let mut g = two_player_game();
    let koi = g.add_card_to_battlefield(0, catalog::giant_koi());
    ready(&mut g, koi);
    let mut helpers = Vec::new();
    for _ in 0..3 {
        let h = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        ready(&mut g, h);
        helpers.push(h);
    }
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbilityWaterbend {
        card_id: koi, ability_index: 0, target: None, additional_targets: vec![],
        x_value: None, helpers,
    }).expect("waterbend {3}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(koi).unwrap().has_keyword(&Keyword::Unblockable), "Koi unblockable");
}

#[test]
fn katara_water_tribes_hope_waterbend_x_team_pump() {
    // Waterbend {X}: creatures you control have base P/T X/X. X=3 via three
    // helpers; the Ally token and Katara both become 3/3 base.
    let mut g = two_player_game();
    let katara = g.add_card_to_battlefield(0, catalog::katara_water_tribes_hope());
    ready(&mut g, katara);
    let mut helpers = Vec::new();
    for _ in 0..3 {
        let h = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        ready(&mut g, h);
        helpers.push(h);
    }
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbilityWaterbend {
        card_id: katara, ability_index: 0, target: None, additional_targets: vec![],
        x_value: Some(3), helpers,
    }).expect("waterbend {X=3} ability");
    drain_stack(&mut g);
    let v = g.compute_battlefield();
    let k = v.iter().find(|c| c.id == katara).unwrap();
    assert_eq!((k.power, k.toughness), (3, 3), "Katara base 3/3 from waterbend X");
}

#[test]
fn waterbend_def_is_terse() {
    // The card carries the waterbend additional cost and the ability the flag.
    assert!(catalog::benevolent_river_spirit().waterbend.is_some());
    assert!(catalog::flexible_waterbender().activated_abilities[0].waterbend);
    // Mandatory vs optional flag is respected.
    assert!(!catalog::benevolent_river_spirit().waterbend.unwrap().optional);
    assert!(catalog::waterbending_lesson().waterbend.unwrap().optional);
    // Type lines.
    assert!(catalog::giant_koi().card_types.contains(&CardType::Creature));
    assert!(catalog::flexible_waterbender().keywords.contains(&Keyword::Vigilance));
    // A +1/+1 counter source for Katara end step.
    let _ = CounterType::PlusOnePlusOne;
}

#[test]
fn aangs_iceberg_exiles_then_sac_scrys() {
    // ETB exiles a nonland permanent; waterbend {3} sacrifice returns it.
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let iceberg = g.add_card_to_hand(0, catalog::aangs_iceberg());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: iceberg, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Aang's Iceberg");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "victim exiled by ETB");
    // Waterbend {3}: sacrifice the Iceberg → victim returns.
    let mut helpers = Vec::new();
    for _ in 0..3 { let h = g.add_card_to_battlefield(0, catalog::grizzly_bears()); g.clear_sickness(h); helpers.push(h); }
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbilityWaterbend {
        card_id: iceberg, ability_index: 0, target: None, additional_targets: vec![],
        x_value: None, helpers,
    }).expect("sac Iceberg via waterbend");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == victim), "victim returned on Iceberg leaving");
}

#[test]
fn waterbender_ascension_draws_at_four_quests() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let asc = g.add_card_to_battlefield(0, catalog::waterbender_ascension());
    // Seed three quest counters; the fourth (from real combat) triggers the draw.
    g.battlefield_find_mut(asc).unwrap().add_counters(CounterType::Quest, 3);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(
        g.battlefield_find(asc).unwrap().counters.get(&CounterType::Quest).copied().unwrap_or(0),
        4,
    );
    assert_eq!(g.players[0].hand.len(), before + 1, "reaching 4 quests draws a card");
}

#[test]
fn water_tribe_rallier_is_a_waterbend_ability() {
    let def = catalog::water_tribe_rallier();
    assert!(def.activated_abilities[0].waterbend);
}

#[test]
fn watery_grasp_and_unagi_are_defined() {
    assert!(catalog::watery_grasp().activated_abilities[0].waterbend);
    assert!(catalog::the_unagi_of_kyoshi_island().keywords.iter().any(|k| matches!(k, Keyword::Ward(_))));
}

/// Crashing Wave taps X target creatures (waterbend X), then distributes three
/// stun counters among tapped creatures an opponent controls.
#[test]
fn crashing_wave_taps_then_stuns() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::crashing_wave());
    g.players[0].mana_pool.add(Color::Blue, 4); // UU + waterbend {2}
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpellWaterbend {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
        helpers: vec![],
    }).expect("waterbend {X=2}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped, "target a tapped");
    assert!(g.battlefield_find(b).unwrap().tapped, "target b tapped");
    let stun_total = g.battlefield_find(a).unwrap().counter_count(CounterType::Stun)
        + g.battlefield_find(b).unwrap().counter_count(CounterType::Stun);
    assert_eq!(stun_total, 3, "three stun counters distributed among the tapped creatures");
}
