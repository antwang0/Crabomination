//! The one-primitive backlog batch (`decks::recent329`).

use crabomination::card::CardType;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood_mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 12);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// Resolve an enchantment ETB, which is what Eerie watches.
fn cast_enchantment(g: &mut GameState) {
    let e = g.add_card_to_hand(0, catalog::goblin_bombardment());
    g.perform_action(GameAction::CastSpell {
        card_id: e,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("enchantment");
    drain_stack(g);
}

/// Victor's Eerie escalates across the turn: surveil, then a discard, then a
/// reanimation.
#[test]
fn victor_eerie_escalates_through_three_branches() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::victor_valgavoths_seneschal());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());

    flood_mana(&mut g, 0);

    // 1st resolution: surveil 2 only.
    cast_enchantment(&mut g);
    assert_eq!(g.players[1].hand.len(), 1, "no discard yet");

    // 2nd: the opponent discards.
    cast_enchantment(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "second resolution discards");

    // 3rd: a creature card comes back under Victor's controller.
    cast_enchantment(&mut g);
    assert!(
        g.battlefield
            .iter()
            .any(|c| c.definition.name == "Grizzly Bears" && c.controller == 0),
        "third resolution reanimates"
    );
}

/// Two escalating sources keep independent tallies (CR 603 — "this ability").
#[test]
fn nth_resolution_tally_is_per_source() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::victor_valgavoths_seneschal());
    let second = g.add_card_to_battlefield(0, catalog::victor_valgavoths_seneschal());
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(1, catalog::island());
    flood_mana(&mut g, 0);
    // One enchantment ETB fires both Victors' first branch (surveil), so
    // neither reaches the discard branch.
    cast_enchantment(&mut g);
    assert_eq!(g.players[1].hand.len(), 1, "both sources are still on branch 1");
    assert!(g.battlefield_find(second).is_some());
}

/// Alania copies the turn's first sorcery and only that one.
#[test]
fn alania_copies_only_the_first_sorcery() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::alania_divergent_storm());
    let a = g.add_card_to_hand(0, catalog::divination());
    let b = g.add_card_to_hand(0, catalog::divination());
    for _ in 0..12 {
        g.add_card_to_library(0, catalog::island());
    }
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::island());
    }
    flood_mana(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let opp_before = g.players[1].hand.len();
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: a,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("first sorcery");
    drain_stack(&mut g);
    // Divination draws 2; the copy draws 2 more, and the opponent drew one.
    assert_eq!(g.players[1].hand.len(), opp_before + 1, "the opponent was gifted a card");
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 4, "the sorcery was copied");

    let opp_after = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: b,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("second sorcery");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_after, "the second sorcery doesn't trigger");
}

/// Heirloom Epic's {4} is payable entirely by tapping creatures (CR 702.51 on
/// an activated ability).
#[test]
fn heirloom_epic_convokes_its_activation() {
    let mut g = main_phase();
    let epic = g.add_card_to_battlefield(0, catalog::heirloom_epic());
    let helpers: Vec<_> = (0..4)
        .map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears()))
        .collect();
    for c in &helpers {
        g.battlefield_find_mut(*c).unwrap().summoning_sick = false;
    }
    g.add_card_to_library(0, catalog::island());
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbilityWaterbend {
        card_id: epic,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        helpers: helpers.clone(),
    })
    .expect("convoked activation");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "the draw resolved with no mana");
    assert!(helpers.iter().all(|c| g.battlefield_find(*c).unwrap().tapped), "helpers tapped");
}

/// Eriette steals the enchanted permanent, and gives it back when the Aura
/// leaves.
#[test]
fn eriette_steals_while_the_aura_stays_attached() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::eriette_the_beguiler());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::pacifism());
    flood_mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("aura");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 0, "stolen");

    let mut evs = Vec::new();
    g.destroy_permanent(aura, false, &mut evs);
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 1, "returned when the Aura left");
}

/// A pricier host is out of Eriette's reach.
#[test]
fn eriette_ignores_a_host_above_the_auras_mana_value() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::eriette_the_beguiler());
    let victim = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    assert!(
        catalog::colossal_dreadmaw().card_types.contains(&CardType::Creature),
        "the host is a nonland permanent"
    );
    let aura = g.add_card_to_hand(0, catalog::pacifism());
    flood_mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("aura");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 1, "too expensive to steal");
}

