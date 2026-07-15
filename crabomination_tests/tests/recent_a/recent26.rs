//! Functionality tests for `catalog::sets::decks::recent26` — Aetherdrift (DFT)
//! Mount/Saddle attack triggers + Exhaust activated abilities.

use crabomination::catalog;
use crabomination::card::{CounterType, Keyword, Supertype};
use crabomination::game::types::{Attack, AttackTarget};
use crabomination::game::*;
use crabomination::TurnStep;

fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == controller && c.definition.name == name)
        .count()
}

/// Saddle `id` and attack player 1 with it, resolving the triggers.
fn saddled_attack(g: &mut GameState, id: CardId) {
    g.battlefield_find_mut(id).unwrap().saddled = true;
    g.clear_sickness(id);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(g);
}

/// Jibbirik Omnivore is a vanilla 3/2.
#[test]
fn jibbirik_omnivore_stats() {
    let def = catalog::jibbirik_omnivore();
    assert_eq!((def.power, def.toughness), (3, 2));
    assert!(def.activated_abilities.is_empty() && def.triggered_abilities.is_empty());
}

/// Caelorna is a legendary 0/8 wall.
#[test]
fn caelorna_is_legendary_wall() {
    let def = catalog::caelorna_coral_tyrant();
    assert_eq!((def.power, def.toughness), (0, 8));
    assert!(def.supertypes.contains(&Supertype::Legendary));
}

/// Gilded Ghoda makes a Treasure when it attacks while saddled.
#[test]
fn gilded_ghoda_makes_treasure_when_saddled() {
    let mut g = two_player_game();
    let gg = g.add_card_to_battlefield(0, catalog::gilded_ghoda());
    saddled_attack(&mut g, gg);
    assert_eq!(count_named(&g, 0, "Treasure"), 1, "saddled attack made a Treasure");
}

/// Brightfield Mustang untaps and grows when it attacks while saddled.
#[test]
fn brightfield_mustang_untaps_and_grows() {
    let mut g = two_player_game();
    let bm = g.add_card_to_battlefield(0, catalog::brightfield_mustang());
    saddled_attack(&mut g, bm);
    let c = g.battlefield_find(bm).unwrap();
    assert!(!c.tapped, "untapped by its own trigger after attacking");
    assert_eq!(g.computed_permanent(bm).unwrap().power, 4, "+1/+1 counter → 4 power");
}

/// Draconautics Engineer's first exhaust grants team haste and grows itself.
#[test]
fn draconautics_engineer_exhaust_haste() {
    let mut g = two_player_game();
    let de = g.add_card_to_battlefield(0, catalog::draconautics_engineer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(de);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: de, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate exhaust");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste), "ally got haste");
    assert_eq!(g.computed_permanent(de).unwrap().power, 3, "self +1/+1 → 3 power");
    // Exhaust: can't activate the same ability twice.
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    let again = g.perform_action(GameAction::ActivateAbility {
        card_id: de, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    });
    assert!(again.is_err(), "exhaust ability is one-shot");
}

/// Afterburner Expert's exhaust puts two +1/+1 counters on it.
#[test]
fn afterburner_expert_exhaust_counters() {
    let mut g = two_player_game();
    let ae = g.add_card_to_battlefield(0, catalog::afterburner_expert());
    g.clear_sickness(ae);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ae, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate exhaust");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ae).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2);
}

/// Piranha Fly ships with flying and an enters-tapped static.
#[test]
fn piranha_fly_flies_enters_tapped() {
    let def = catalog::piranha_fly();
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.static_abilities.iter().any(|s| matches!(
        s.effect,
        crabomination::card::StaticEffect::EntersTapped { .. }
    )));
}

/// Ripchain Razorkin sacrifices a land to draw a card.
#[test]
fn ripchain_razorkin_sacs_land_to_draw() {
    let mut g = two_player_game();
    let rr = g.add_card_to_battlefield(0, catalog::ripchain_razorkin());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.clear_sickness(rr);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: rr, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Beastrider Vanguard digs three for a permanent card.
#[test]
fn beastrider_vanguard_digs_for_permanent() {
    let mut g = two_player_game();
    let bv = g.add_card_to_battlefield(0, catalog::beastrider_vanguard());
    g.add_card_to_library(0, catalog::grizzly_bears()); // a permanent on top
    g.clear_sickness(bv);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: bv, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "took a permanent into hand");
}

/// Fear of Exposure's additional cost taps two of your creatures/lands.
#[test]
fn fear_of_exposure_taps_two_to_cast() {
    let mut g = two_player_game();
    let fear = g.add_card_to_hand(0, catalog::fear_of_exposure());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: fear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Fear of Exposure");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fear).is_some(), "Fear of Exposure resolved onto the battlefield");
    assert_eq!(
        [a, b].iter().filter(|&&id| g.battlefield_find(id).unwrap().tapped).count(),
        2, "two creatures tapped for the additional cost",
    );
}

/// Vicious Clown pumps itself when a small creature you control enters.
#[test]
fn vicious_clown_pumps_on_small_creature_etb() {
    let mut g = two_player_game();
    let clown = g.add_card_to_battlefield(0, catalog::vicious_clown());
    // A 2/2 (power ≤ 2) entering pumps the Clown +2/+0.
    let small = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: small }]);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(clown).unwrap().power, 4, "Clown pumped to 4 power");
    // A big creature (power > 2) does not trigger the pump.
    let big = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: big }]);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(clown).unwrap().power, 4, "big creature did not pump further");
}
