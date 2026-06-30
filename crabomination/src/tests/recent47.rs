//! Functionality tests for `catalog::sets::decks::recent47`.

use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::two_player_game;
use crate::game::*;
use crate::mana::Color;

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: idx, target, additional_targets: Vec::new(), x_value: None,
    }).expect("ability activates");
    drain_stack(g);
}

#[test]
fn multani_grows_with_lands_on_board_and_in_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_graveyard(0, catalog::forest());
    let m = g.add_card_to_battlefield(0, catalog::multani_yavimayas_avatar());
    let cp = g.computed_permanent(m).unwrap();
    assert_eq!(cp.power, 3, "2 lands on board + 1 in graveyard");
    assert_eq!(cp.toughness, 3);
}

#[test]
fn nullmage_shepherd_taps_four_to_destroy() {
    let mut g = two_player_game();
    let shep = g.add_card_to_battlefield(0, catalog::nullmage_shepherd());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    let ench = g.add_card_to_battlefield(1, catalog::mark_of_asylum());
    activate(&mut g, shep, 0, Some(Target::Permanent(ench)));
    assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
}

#[test]
fn magus_of_the_wheel_refills_both_hands() {
    let mut g = two_player_game();
    let magus = g.add_card_to_battlefield(0, catalog::magus_of_the_wheel());
    for p in 0..2 {
        for _ in 0..3 { g.add_card_to_hand(p, catalog::forest()); }
        for _ in 0..10 { g.add_card_to_library(p, catalog::forest()); }
    }
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, magus, 0, None);
    assert_eq!(g.players[0].hand.len(), 7, "P0 drew a fresh seven");
    assert_eq!(g.players[1].hand.len(), 7, "P1 drew a fresh seven");
}

#[test]
fn bankrupt_in_blood_sacs_two_for_three_cards() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let bib = g.add_card_to_hand(0, catalog::bankrupt_in_blood());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bib, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bankrupt in Blood");
    drain_stack(&mut g);
    // -Bankrupt (cast) +3 drawn.
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 3, "drew three");
    let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 0, "both creatures sacrificed as the additional cost");
}

#[test]
fn sidisi_exploit_tutors() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wanted = g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),            // yes, exploit the fodder
        DecisionAnswer::Search(Some(wanted)),  // tutor the bolt
    ]));
    let etb = catalog::sidisi_undead_vizier().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(crate::card::CardId(99), 0, None, 0);
    g.resolve_effect(&etb, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder exploited");
    assert!(g.players[0].hand.iter().any(|c| c.id == wanted), "tutored card in hand");
}

#[test]
fn nighthawk_scavenger_scales_off_opponent_graveyard_types() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::lightning_bolt()); // instant
    g.add_card_to_graveyard(1, catalog::grizzly_bears()); // creature
    let nh = g.add_card_to_battlefield(0, catalog::nighthawk_scavenger());
    let cp = g.computed_permanent(nh).unwrap();
    assert_eq!(cp.power, 3, "1 + two card types (instant, creature)");
    assert_eq!(cp.toughness, 3);
}

#[test]
fn speaker_of_the_heavens_makes_angel_only_when_high_on_life() {
    let mut g = two_player_game();
    let speaker = g.add_card_to_battlefield(0, catalog::speaker_of_the_heavens());
    // At 20 life the ability is illegal — no Angel.
    g.players[0].life = 20;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: speaker, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).is_err(), "blocked below the +7 life threshold");
    // Untap and climb to 27 — now it fires.
    g.battlefield_find_mut(speaker).unwrap().tapped = false;
    g.players[0].life = 27;
    activate(&mut g, speaker, 0, None);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Angel"), "Angel token created");
}
