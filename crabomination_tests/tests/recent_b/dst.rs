//! Darksteel gap batch (`decks::recent310`).

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// The Horn/Feather cycle watches every player's casts of its colour.
#[test]
fn color_watch_artifacts_gain_life_on_a_matching_cast() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::angels_feather());
    g.add_card_to_battlefield(0, catalog::demons_horn());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast a red spell");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "neither watches red");
    let raise = g.add_card_to_hand(0, catalog::raise_dead());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: raise, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast a black spell");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21, "Demon's Horn only");
}

/// Darksteel Forge blankets your artifacts in indestructible.
#[test]
fn darksteel_forge_makes_your_artifacts_indestructible() {
    let mut g = main_phase();
    let plain = g.add_card_to_battlefield(0, catalog::coretapper());
    assert!(!g
        .computed_permanent(plain)
        .unwrap()
        .keywords
        .contains(&Keyword::Indestructible));
    g.add_card_to_battlefield(0, catalog::darksteel_forge());
    assert!(g
        .computed_permanent(plain)
        .unwrap()
        .keywords
        .contains(&Keyword::Indestructible));
}

/// Darksteel Brute animates into a 2/2 Beast that keeps its artifact type.
#[test]
fn darksteel_brute_animates_into_a_beast() {
    let mut g = main_phase();
    let brute = g.add_card_to_battlefield(0, catalog::darksteel_brute());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: brute, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(brute).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.card_types.contains(&CardType::Creature));
    assert!(cp.card_types.contains(&CardType::Artifact));
}

/// Arcane Spyglass banks a charge per draw, then cashes three in for another.
#[test]
fn arcane_spyglass_charges_then_cashes_in() {
    let mut g = main_phase();
    let glass = g.add_card_to_battlefield(0, catalog::arcane_spyglass());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
        g.battlefield_find_mut(glass).unwrap().tapped = false;
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: glass, ability_index: 0, target: None, additional_targets: vec![],
            x_value: None,
        })
        .expect("sac a land to draw");
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(glass).unwrap().counter_count(CounterType::Charge), 3);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: glass, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("remove three charges");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.battlefield_find(glass).unwrap().counter_count(CounterType::Charge), 0);
}

/// Coretapper's sacrifice charges an artifact twice.
#[test]
fn coretapper_sacrifices_for_two_charges() {
    let mut g = main_phase();
    let myr = g.add_card_to_battlefield(0, catalog::coretapper());
    let target = g.add_card_to_battlefield(0, catalog::arcane_spyglass());
    g.perform_action(GameAction::ActivateAbility {
        card_id: myr, ability_index: 1, target: Some(Target::Permanent(target)),
        additional_targets: vec![], x_value: None,
    })
    .expect("sacrifice for two");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::Charge), 2);
    assert!(g.battlefield_find(myr).is_none());
}

/// Drill-Skimmer only has shroud while another artifact creature is around.
#[test]
fn drill_skimmer_gains_shroud_with_a_friend() {
    let mut g = main_phase();
    let skimmer = g.add_card_to_battlefield(0, catalog::drill_skimmer());
    assert!(!g.computed_permanent(skimmer).unwrap().keywords.contains(&Keyword::Shroud));
    g.add_card_to_battlefield(0, catalog::coretapper());
    assert!(g.computed_permanent(skimmer).unwrap().keywords.contains(&Keyword::Shroud));
}

/// Dross Golem's affinity for Swamps discounts it.
#[test]
fn dross_golem_costs_less_per_swamp() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    let golem = g.add_card_to_hand(0, catalog::dross_golem());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: golem, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("{5} minus three Swamps");
    drain_stack(&mut g);
    assert!(g.battlefield_find(golem).is_some());
}

/// Auriok Glaivemaster only grows while it's carrying something.
#[test]
fn auriok_glaivemaster_grows_when_equipped() {
    let mut g = main_phase();
    let kor = g.add_card_to_battlefield(0, catalog::auriok_glaivemaster());
    assert_eq!(g.computed_permanent(kor).map(|c| (c.power, c.toughness)), Some((1, 1)));
    let sword = g.add_card_to_battlefield(0, catalog::short_bow());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(kor);
    let cp = g.computed_permanent(kor).unwrap();
    // 1/1 base + Short Bow's +1/+1 + the Glaivemaster's own equipped bonus.
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
}

/// Chittering Rats tucks a card off the opponent's hand.
#[test]
fn chittering_rats_tucks_a_card() {
    let mut g = main_phase();
    let theirs = g.add_card_to_hand(1, catalog::grizzly_bears());
    let rats = g.add_card_to_battlefield(0, catalog::chittering_rats());
    g.fire_self_etb_triggers(rats, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0);
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(theirs));
}

/// Burden of Greed scales with the target's tapped artifacts.
#[test]
fn burden_of_greed_counts_tapped_artifacts() {
    let mut g = main_phase();
    for _ in 0..3 {
        let a = g.add_card_to_battlefield(1, catalog::coretapper());
        g.battlefield_find_mut(a).unwrap().tapped = true;
    }
    g.add_card_to_battlefield(1, catalog::coretapper()); // untapped, doesn't count
    let burden = g.add_card_to_hand(0, catalog::burden_of_greed());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: burden, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// Crazed Goblin has to swing.
#[test]
fn crazed_goblin_must_attack() {
    let g = main_phase();
    assert!(catalog::crazed_goblin().keywords.contains(&Keyword::MustAttack));
    let _ = g;
}

/// Carry Away steals the Equipment it enchants and knocks it loose.
#[test]
fn carry_away_steals_the_equipment() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(1, catalog::short_bow());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(bear);
    let aura = g.add_card_to_hand(0, catalog::carry_away());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(sword)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast the Aura");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sword).unwrap().controller, 0, "you control it now");
}
