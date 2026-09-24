//! Commander: the Divine Convocation precon (MOC, Kasla, `decks::cmdr_kasla`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = if n == 2 { two_player_game() } else { multi_player_game(n) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

/// Convoke with exactly `helpers` paying part of the cost (the rest floods).
fn convoke(g: &mut GameState, id: CardId, targets: &[Target], helpers: &[CardId]) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellConvoke {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
        convoke_creatures: helpers.to_vec(),
    })
    .expect("convoke cast");
    drain_stack(g);
}

fn library(g: &mut GameState, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(0, catalog::island());
    }
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == name).count()
}

/// CR 702.51 — tapping a creature to convoke taps it, so its "becomes tapped"
/// trigger fires (Fallowsage draws); Kasla sees a spell that has convoke.
#[test]
fn convoking_taps_and_kasla_draws() {
    let mut g = pod(2);
    library(&mut g, 6);
    let fs = g.add_card_to_battlefield(0, catalog::fallowsage());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mm = g.add_card_to_hand(0, catalog::meeting_of_minds());
    convoke(&mut g, mm, &[], &[fs]);
    assert!(g.battlefield_find(fs).unwrap().tapped);
    assert_eq!(g.players[0].hand.len(), 3, "Meeting of Minds 2 + Fallowsage 1");
    let mut g = pod(2);
    library(&mut g, 6);
    g.add_card_to_battlefield(0, catalog::kasla_the_broken_halo());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mm = g.add_card_to_hand(0, catalog::meeting_of_minds());
    convoke(&mut g, mm, &[], &[bear]);
    assert_eq!(g.players[0].hand.len(), 3, "Meeting of Minds 2 + Kasla 1");
}

/// Wand of the Worldsoul: the next spell has convoke (CR 702.51), and it
/// counts as a spell that has convoke for Joyful Stormsculptor.
#[test]
fn wand_grants_the_next_spell_convoke() {
    let mut g = pod(3);
    let wand = g.add_card_to_battlefield(0, catalog::wand_of_the_worldsoul());
    g.battlefield_find_mut(wand).unwrap().tapped = false;
    g.add_card_to_battlefield(0, catalog::joyful_stormsculptor());
    let helper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: wand,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("wand");
    drain_stack(&mut g);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    convoke(&mut g, bear, &[], &[helper]);
    assert_eq!(g.players[1].starting_life - g.players[1].life, 1);
    assert_eq!(g.players[2].starting_life - g.players[2].life, 1);
    // Spent: the next spell has no convoke.
    let bear2 = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    let helper2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(
        g.perform_action(GameAction::CastSpellConvoke {
            card_id: bear2,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
            convoke_creatures: vec![helper2],
        })
        .is_err()
    );
}

/// Venerated Loxodon counters each creature that convoked it.
#[test]
fn venerated_loxodon_rewards_its_convokers() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let vl = g.add_card_to_hand(0, catalog::venerated_loxodon());
    convoke(&mut g, vl, &[], &[a, b]);
    for (id, n) in [(a, 1), (b, 1), (c, 0)] {
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), n);
    }
}

/// Saint Traft and Rem Karolus: tapped (as a convoker) it makes a Human; the
/// convoke spell untaps it; tapped again, a Spirit.
#[test]
fn saint_traft_escalates_and_untaps() {
    let mut g = pod(2);
    let st = g.add_card_to_battlefield(0, catalog::saint_traft_and_rem_karolus());
    let mm = g.add_card_to_hand(0, catalog::meeting_of_minds());
    library(&mut g, 4);
    convoke(&mut g, mm, &[], &[st]);
    assert!(!g.battlefield_find(st).unwrap().tapped, "untapped by the convoke cast");
    assert_eq!(named(&g, "Human"), 1);
    let mm = g.add_card_to_hand(0, catalog::meeting_of_minds());
    convoke(&mut g, mm, &[], &[st]);
    assert_eq!(named(&g, "Spirit"), 1);
}

/// Cut Short destroys a tapped creature, not an untapped one.
#[test]
fn cut_short_needs_a_tapped_creature() {
    let mut g = pod(2);
    let tapped = g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    let untapped = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let cs = g.add_card_to_hand(0, catalog::cut_short());
    flood(&mut g, 0);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: cs,
            target: Some(Target::Permanent(untapped)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err()
    );
    cast(&mut g, cs, &[Target::Permanent(tapped)], None);
    assert!(g.battlefield_find(tapped).is_none());
}

/// Nesting Dovehawk populates at the beginning of your combat (CR 701.32)
/// and grows as the copy enters.
#[test]
fn nesting_dovehawk_populates() {
    let mut g = pod(2);
    let nd = g.add_card_to_battlefield(0, catalog::nesting_dovehawk());
    let p = g.add_card_to_hand(0, catalog::path_of_the_ghosthunter());
    cast(&mut g, p, &[], Some(1));
    assert_eq!(named(&g, "Spirit"), 1);
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(named(&g, "Spirit"), 2);
    assert_eq!(g.battlefield_find(nd).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Wildfire Awakener makes X Elementals.
#[test]
fn wildfire_awakener_makes_x_elementals() {
    let mut g = pod(2);
    let wa = g.add_card_to_hand(0, catalog::wildfire_awakener());
    cast(&mut g, wa, &[], Some(3));
    assert_eq!(named(&g, "Elemental"), 3);
}

/// Mistmeadow Vanisher, tapped, exiles a permanent until the next end step.
#[test]
fn mistmeadow_vanisher_flickers_on_tap() {
    let mut g = pod(2);
    let mv = g.add_card_to_battlefield(0, catalog::mistmeadow_vanisher());
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let mm = g.add_card_to_hand(0, catalog::meeting_of_minds());
    library(&mut g, 2);
    convoke(&mut g, mm, &[], &[mv]);
    assert!(g.battlefield_find(wurm).is_none());
    assert!(g.exile.iter().any(|c| c.id == wurm));
}
