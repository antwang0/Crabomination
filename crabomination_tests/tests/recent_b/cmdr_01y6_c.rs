//! Commander: the 2026-10-25 (`01Y6ZRFx`) Stockman, Mad Fly-entist closers
//! (`decks::cmdr_01y6_c`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn pass_to(g: &mut GameState, step: TurnStep) {
    for _ in 0..200 {
        if g.step == step {
            return;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    panic!("never reached {step:?}");
}

/// The skulkins grant their keyword to a creature of their colour only.
#[test]
fn the_skulkins_grant_to_their_color_only() {
    for (skulkin, kw, color_fits) in [
        (catalog::fang_skulkin(), Keyword::Wither, catalog::sengir_vampire()),
        (catalog::antler_skulkin(), Keyword::Persist, catalog::savannah_lions()),
    ] {
        let mut g = pod(2);
        let s = ready(&mut g, 0, skulkin);
        let fits = g.add_card_to_battlefield(0, color_fits);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add_colorless(2);
        assert!(activate(&mut g, s, Some(Target::Permanent(bear)), None).is_err(), "green is no target");
        activate(&mut g, s, Some(Target::Permanent(fits)), None).expect("grant");
        assert!(g.permanent_has_keyword(fits, &kw));
    }
}

/// Clockwork Beast winds back to seven counters at most.
#[test]
fn clockwork_beast_winds_to_seven() {
    let mut g = pod(2);
    let b = ready(&mut g, 0, catalog::clockwork_beast());
    g.battlefield_find_mut(b).unwrap().add_counters(CounterType::PlusOnePlusZero, 4);
    g.step = TurnStep::Upkeep;
    g.players[0].mana_pool.add_colorless(5);
    activate(&mut g, b, None, Some(5)).expect("wind");
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusZero), 7);
    assert_eq!(g.computed_permanent(b).unwrap().power, 7);
}

/// CR 603.8 — Ronom Serpent attacks only a player with a snow land and is
/// sacrificed the moment its controller has none.
#[test]
fn ronom_serpent_needs_snow_on_both_sides() {
    let mut g = pod(2);
    let mine = g.add_card_to_battlefield(0, catalog::snow_covered_island());
    let s = ready(&mut g, 0, catalog::ronom_serpent());
    g.step = TurnStep::DeclareAttackers;
    let attack = vec![Attack { attacker: s, target: AttackTarget::Player(1) }];
    assert!(g.declare_attackers(attack.clone()).is_err(), "no snow land across");
    g.add_card_to_battlefield(1, catalog::snow_covered_island());
    assert!(g.declare_attackers(attack).is_ok());
    let _ = g.remove_to_graveyard_with_triggers(mine);
    g.check_state_based_actions();
    assert!(g.battlefield_find(s).is_none());
}

/// Bazaar of Wonders exiles the graveyards as it enters, then counters a
/// spell sharing a name with a graveyard card or a nontoken permanent.
#[test]
fn bazaar_of_wonders_counters_repeated_names() {
    let mut g = pod(2);
    let gy = g.add_card_to_graveyard(1, catalog::island());
    let baz = g.add_card_to_hand(0, catalog::bazaar_of_wonders());
    g.players[0].mana_pool.add(Color::Blue, 5);
    g.perform_action(GameAction::CastSpell { card_id: baz, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == gy), "graveyards exiled");
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    for (def, target, countered) in [
        (catalog::grizzly_bears(), None, true),
        (catalog::lightning_bolt(), Some(Target::Player(1)), true),
        (catalog::savannah_lions(), None, false),
    ] {
        let id = g.add_card_to_hand(0, def);
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add(Color::White, 1);
        g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
            .expect("cast");
        drain_stack(&mut g);
        let name = g.find_card_anywhere(id).map(|c| c.definition.name.to_string()).unwrap_or_default();
        if countered {
            assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "{name} countered");
        } else {
            assert!(g.battlefield_find(id).is_some(), "{name} resolved");
        }
    }
}

/// Balduvian Shaman rewrites a white enchantment you control and saddles it
/// with cumulative upkeep {1}; one that already has cumulative upkeep is no
/// target.
#[test]
fn balduvian_shaman_saddles_a_white_enchantment_with_upkeep() {
    let mut g = pod(2);
    let shaman = ready(&mut g, 0, catalog::balduvian_shaman());
    let decree = g.add_card_to_battlefield(0, catalog::royal_decree());
    let cop = g.add_card_to_battlefield(0, catalog::circle_of_protection_red());
    assert!(activate(&mut g, shaman, Some(Target::Permanent(decree)), None).is_err(), "already has cumulative upkeep");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red), DecisionAnswer::Color(Color::Black)]));
    activate(&mut g, shaman, Some(Target::Permanent(cop)), None).expect("rewrite");
    let c = g.computed_permanent(cop).unwrap();
    assert!(c.keywords().iter().any(|k| matches!(k, Keyword::CumulativeUpkeep(_))));
}

/// Arcum's Whistle: unpaid, the active player's creature must attack and is
/// destroyed at the end step when it didn't; paying its mana value spares it.
#[test]
fn arcums_whistle_forces_an_attack_or_a_payment() {
    for pays in [false, true] {
        let mut g = pod(2);
        for p in 0..2 {
            for _ in 0..3 {
                g.add_card_to_library(p, catalog::island());
            }
        }
        let whistle = g.add_card_to_battlefield(0, catalog::arcums_whistle());
        let giant = ready(&mut g, 1, catalog::hill_giant());
        g.active_player_idx = 1;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(3);
        if pays {
            g.players[1].mana_pool.add_colorless(4);
        }
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(pays)]));
        activate(&mut g, whistle, Some(Target::Permanent(giant)), None).expect("whistle");
        assert_eq!(g.permanent_has_keyword(giant, &Keyword::MustAttack), !pays);
        // It never attacks (no attack declared): the end step decides.
        g.priority.player_with_priority = 1;
        g.step = TurnStep::PostCombatMain;
        pass_to(&mut g, TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(giant).is_some(), pays);
    }
}

/// Acidic Dagger: its creature's combat damage destroys the non-Wall
/// creature it hits, whatever its toughness; a Wall survives.
#[test]
fn acidic_dagger_melts_what_its_creature_hits() {
    for (blocker, dies) in [(catalog::craw_wurm(), true), (catalog::wall_of_stone(), false)] {
        let mut g = pod(2);
        let dagger = g.add_card_to_battlefield(0, catalog::acidic_dagger());
        let lions = ready(&mut g, 0, catalog::savannah_lions());
        let b = ready(&mut g, 1, blocker);
        g.players[0].mana_pool.add_colorless(4);
        activate(&mut g, dagger, Some(Target::Permanent(lions)), None).expect("dagger");
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: lions, target: AttackTarget::Player(1) }]))
            .expect("attack");
        drain_stack(&mut g);
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::DeclareBlockers(vec![(b, lions)])).expect("block");
        drain_stack(&mut g);
        g.step = TurnStep::CombatDamage;
        let ev = g.resolve_combat().expect("damage");
        g.dispatch_triggers_for_events(&ev);
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(b).is_none(), dies);
        // The Wurm kills the Lions and the Dagger goes with them; the Wall
        // deals nothing back.
        assert_eq!(g.battlefield_find(lions).is_none(), dies);
        assert_eq!(g.battlefield_find(dagger).is_none(), dies);
    }
}
