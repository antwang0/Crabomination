//! Commander: EDHREC average-deck gaps, sixth batch (`decks::cmdr_edhrec6`):
//! Hylda of the Icy Crown's tappers, and the skip-next-untap rider they share.

use crabomination::card::{CardId, CreatureType};
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], mode: Option<usize>, x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode,
        x_value: x,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, id, targets, None, None)
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn tapped(g: &GameState, id: CardId) -> bool {
    g.battlefield_find(id).expect("on the battlefield").tapped
}

fn controller(g: &GameState, id: CardId) -> usize {
    g.battlefield_find(id).expect("on the battlefield").controller
}

/// Run `seat`'s untap step.
fn untap_step(g: &mut GameState, seat: usize) {
    g.active_player_idx = seat;
    g.do_untap();
}

/// `seat` casts a Grizzly Bears in its own main phase.
fn cast_on_own_turn(g: &mut GameState, seat: usize) -> CardId {
    let id = g.add_card_to_hand(seat, catalog::grizzly_bears());
    flood(g, seat);
    g.active_player_idx = seat;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
    g.active_player_idx = 0;
    id
}

fn perm(id: CardId) -> Target {
    Target::Permanent(id)
}

/// Hylda's Crown of Winter: the tap costs {1} less on your turn ({0}), full
/// price on another's; the sacrifice draws per tapped opposing creature.
#[test]
fn hyldas_crown_is_free_on_your_turn_and_counts_tapped_creatures() {
    let mut g = pod(3);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let crown = ready(&mut g, 0, catalog::hyldas_crown_of_winter());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    ready(&mut g, 2, catalog::grizzly_bears());
    activate(&mut g, crown, 0, Some(perm(a))).expect("free on your turn");
    assert!(tapped(&g, a));
    g.battlefield_find_mut(crown).unwrap().tapped = false;
    g.active_player_idx = 1;
    assert!(activate(&mut g, crown, 0, Some(perm(b))).is_err(), "{{1}} on another's turn, and no mana");
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, crown, 0, Some(perm(b))).expect("one paid");
    flood(&mut g, 0);
    activate(&mut g, crown, 1, None).expect("sacrifice");
    assert_eq!(g.players[0].hand.len(), 2, "two tapped creatures across two opponents");
}

/// Willbreaker (CR 611.2c): targeting an opponent's creature with your own
/// ability steals it; your own creature or an opponent's targeting doesn't;
/// losing Willbreaker hands the creature back.
#[test]
fn willbreaker_steals_what_you_target_while_you_control_it() {
    let mut g = pod(3);
    let wb = ready(&mut g, 0, catalog::willbreaker());
    let crown = ready(&mut g, 0, catalog::hyldas_crown_of_winter());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 2, catalog::hill_giant());
    activate(&mut g, crown, 0, Some(perm(mine))).expect("tap mine");
    assert_eq!(controller(&g, mine), 0);
    g.battlefield_find_mut(crown).unwrap().tapped = false;
    activate(&mut g, crown, 0, Some(perm(theirs))).expect("tap theirs");
    assert_eq!(controller(&g, theirs), 0, "stolen as it became the target");
    g.battlefield_find_mut(wb).unwrap().controller = 1;
    g.check_state_based_actions();
    assert_eq!(controller(&g, theirs), 2, "back once you don't control Willbreaker");
}

/// Illithid Harvester (CR 708.2): tapped nontoken creatures turn face down as
/// 2/2 Horrors; a double-faced one can't be turned face down (CR 712.16).
#[test]
fn cr_712_16_illithid_harvester_skips_a_double_faced_creature() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let delver = ready(&mut g, 1, catalog::delver_of_secrets());
    for id in [giant, delver] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    let h = g.add_card_to_hand(0, catalog::illithid_harvester());
    cast(&mut g, h, &[perm(giant), perm(delver)]).expect("cast");
    let c = g.computed_permanent(giant).unwrap();
    assert!(g.battlefield_find(giant).unwrap().face_down);
    assert_eq!((c.power, c.toughness), (2, 2));
    assert!(c.subtypes().creature_types.contains(&CreatureType::Horror));
    assert!(!g.battlefield_find(delver).unwrap().face_down, "CR 712.16");
}

/// Plant Tadpoles: exactly X targets, tapped and held through their next untap.
#[test]
fn plant_tadpoles_taps_x_creatures_and_holds_them() {
    let mut g = pod(2);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::hill_giant());
    let h = g.add_card_to_hand(0, catalog::illithid_harvester());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastAdventure {
        card_id: h,
        target: Some(perm(a)),
        additional_targets: vec![perm(b)],
        x_value: Some(2),
        mode: None,
    })
    .expect("cast Plant Tadpoles");
    drain_stack(&mut g);
    assert!(tapped(&g, a) && tapped(&g, b));
    untap_step(&mut g, 1);
    assert!(tapped(&g, a) && tapped(&g, b));
}

/// Cone of Cold (CR 706): a 20 also makes opposing creatures enter tapped
/// until your next turn (CR 614.1c); a 5 only taps.
#[test]
fn cone_of_cold_twenty_holds_the_door_until_your_next_turn() {
    let mut g = pod(3);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let cone = g.add_card_to_hand(0, catalog::cone_of_cold());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(20)]));
    cast(&mut g, cone, &[]).expect("cast");
    assert!(tapped(&g, a));
    let mine = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, mine, &[]).expect("cast mine");
    assert!(!tapped(&g, mine), "yours doesn't enter tapped");
    let late = cast_on_own_turn(&mut g, 2);
    assert!(tapped(&g, late), "an opponent's creature does");
    untap_step(&mut g, 1);
    assert!(tapped(&g, a), "and the 20 holds the tapped ones too");
    untap_step(&mut g, 0);
    let after = cast_on_own_turn(&mut g, 2);
    assert!(!tapped(&g, after), "over once your turn begins");

    let mut g = pod(2);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let cone = g.add_card_to_hand(0, catalog::cone_of_cold());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(5)]));
    cast(&mut g, cone, &[]).expect("cast");
    assert!(tapped(&g, a));
    untap_step(&mut g, 1);
    assert!(!tapped(&g, a), "a 5 doesn't hold");
}

/// Topplegeist: delirium taps a creature of the opponent whose upkeep it is.
#[test]
fn topplegeist_taps_on_each_opponents_upkeep_with_delirium() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::topplegeist());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(!tapped(&g, a), "no delirium yet");
    for def in [catalog::island(), catalog::grizzly_bears(), catalog::shock(), catalog::opt()] {
        g.add_card_to_graveyard(0, def);
    }
    g.add_card_to_graveyard(0, catalog::hyldas_crown_of_winter());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(tapped(&g, a));
}

/// Hammers of Moradin (CR 601.2c): attacking at three seats it taps one
/// creature of each opponent, and myriad copies it at the other one.
#[test]
fn hammers_of_moradin_taps_one_per_opponent() {
    let mut g = pod(3);
    let h = ready(&mut g, 0, catalog::hammers_of_moradin());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: h, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(tapped(&g, a) && tapped(&g, b));
    let copies = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Hammers of Moradin").count();
    assert_eq!(copies, 1, "myriad: a copy attacking seat 2");
}

/// Icy Blast: ferocious holds the tapped creatures; without it they untap.
#[test]
fn icy_blast_holds_only_with_ferocious() {
    for (ferocious, held) in [(false, false), (true, true)] {
        let mut g = pod(2);
        if ferocious {
            ready(&mut g, 0, catalog::illithid_harvester());
        }
        let a = ready(&mut g, 1, catalog::grizzly_bears());
        let blast = g.add_card_to_hand(0, catalog::icy_blast());
        cast_x(&mut g, blast, &[perm(a)], None, Some(1)).expect("cast");
        untap_step(&mut g, 1);
        assert_eq!(tapped(&g, a), held, "ferocious {ferocious}");
    }
}

/// Time of Ice: I locks a creature while you control the Saga; III bounces
/// every tapped creature.
#[test]
fn time_of_ice_locks_then_bounces_tapped_creatures() {
    let mut g = pod(2);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let saga = g.add_card_to_hand(0, catalog::time_of_ice());
    cast(&mut g, saga, &[perm(a)]).expect("cast");
    assert!(tapped(&g, a));
    untap_step(&mut g, 1);
    assert!(tapped(&g, a), "locked while you control the Saga");
    g.battlefield_find_mut(mine).unwrap().tapped = true;
    for _ in 0..2 {
        g.saga_advance(saga);
        drain_stack(&mut g);
        g.check_state_based_actions();
    }
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(mine).is_none());
    assert_eq!(g.players[1].hand.len(), 1);
    assert_eq!(g.players[0].hand.len(), 1);
}
