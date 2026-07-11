//! Functionality tests for `catalog::sets::decks::recent160` (Foundations).

use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Target, TurnStep};
use crate::game::*;
use crate::mana::Color;

fn fill_mana(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(8);
}

/// Erudite Wizard grows when you draw your second card in a turn.
#[test]
fn erudite_wizard_grows_on_second_draw() {
    let mut g = two_player_game();
    let wiz = g.add_card_to_battlefield(0, catalog::erudite_wizard());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let mut evs = vec![];
    g.draw_one(0, &mut evs);
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(wiz).unwrap().power, 3, "second draw → +1/+1");
}

/// Gorehorn Raider's Raid pings when you attacked this turn.
#[test]
fn gorehorn_raider_raid_pings() {
    let mut g = two_player_game();
    g.players[0].attacked_this_turn = true;
    let life = g.players[1].life;
    let raider = g.add_card_to_battlefield(0, catalog::gorehorn_raider());
    g.fire_self_etb_triggers(raider, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "Raid dealt 2");
}

/// Gutless Plunderer digs three on Raid.
#[test]
fn gutless_plunderer_raid_digs() {
    let mut g = two_player_game();
    g.players[0].attacked_this_turn = true;
    g.players[0].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let gy = g.players[0].graveyard.len();
    let plunderer = g.add_card_to_battlefield(0, catalog::gutless_plunderer());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![],
    }]));
    g.fire_self_etb_triggers(plunderer, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy + 2, "kept one, milled the other two");
}

/// Hinterland Sanctifier gains life when another creature you control enters.
#[test]
fn hinterland_sanctifier_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hinterland_sanctifier());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ally }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained 1 life");
}

/// Hungry Ghoul grows by sacrificing another creature.
#[test]
fn hungry_ghoul_sac_grows() {
    let mut g = two_player_game();
    let ghoul = g.add_card_to_battlefield(0, catalog::hungry_ghoul());
    g.clear_sickness(ghoul);
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    fill_mana(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ghoul, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Hungry Ghoul");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed the fodder");
    assert_eq!(g.computed_permanent(ghoul).unwrap().power, 3, "grew +1/+1");
}

/// Icewind Elemental loots on entry.
#[test]
fn icewind_elemental_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![forest])]));
    let hand = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::icewind_elemental());
    drain_stack(&mut g);
    // Drew one and discarded one → net hand unchanged, graveyard grew.
    assert_eq!(g.players[0].hand.len(), hand, "drew one, discarded one");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == forest), "discarded the forest");
}

/// Infestation Sage leaves a flying Insect when it dies.
#[test]
fn infestation_sage_dies_to_insect() {
    let mut g = two_player_game();
    let sage = g.add_card_to_battlefield(0, catalog::infestation_sage());
    let mut evs = g.remove_to_graveyard_with_triggers(sage);
    evs.push(GameEvent::CreatureDied { card_id: sage });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Insect" && c.controller == 0), "made an Insect");
}

/// Prideful Parent brings a Cat friend.
#[test]
fn prideful_parent_makes_a_cat() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::prideful_parent());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Cat" && c.controller == 0), "made a Cat");
}

/// Firespitter Whelp pings each opponent on a noncreature cast.
#[test]
fn firespitter_whelp_pings_on_noncreature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::firespitter_whelp());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::divination());
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Divination");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "noncreature cast pinged the opponent");
}

/// Guarded Heir brings two Knights.
#[test]
fn guarded_heir_makes_two_knights() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::guarded_heir());
    drain_stack(&mut g);
    let knights = g.battlefield.iter().filter(|c| c.definition.name == "Knight" && c.controller == 0).count();
    assert_eq!(knights, 2, "made two Knights");
}
