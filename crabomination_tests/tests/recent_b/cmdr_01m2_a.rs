//! Commander: the 2026-10-24 (`01M2Ffkh`) Isu the Abominable closers
//! (`decks::cmdr_01m2_a`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, GameError, TurnStep};
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

fn cast(g: &mut GameState, id: CardId) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

/// Isu plays a snow land and casts a snow spell off the top of the library
/// (never a non-snow one), and a snow arrival buys a +1/+1 counter for the
/// chosen colour.
#[test]
fn isu_plays_snow_off_the_top_and_grows() {
    let mut g = pod(2);
    let isu = ready(&mut g, 0, catalog::isu_the_abominable());
    let land = g.add_card_to_library(0, catalog::snow_covered_forest());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0), DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::PlayLand(land)).expect("a snow land off the top");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_some());
    assert_eq!(g.battlefield_find(isu).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "paid {{G}}");

    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    assert!(cast(&mut g, bears).is_err(), "not a snow spell");
    g.players[0].library.retain(|c| c.id != bears);
    let yeti = g.add_card_to_library(0, catalog::frostpeak_yeti());
    g.players[0].mana_pool.add(Color::Blue, 4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(2), DecisionAnswer::Bool(false)]));
    cast(&mut g, yeti).expect("a snow spell off the top");
    assert!(g.battlefield_find(yeti).is_some());
    assert_eq!(g.battlefield_find(isu).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "declined");
}

/// Diamond Faerie's {1}{S} pumps snow creatures only, paid from snow lands.
#[test]
fn diamond_faerie_pumps_only_snow_creatures() {
    let mut g = pod(2);
    let faerie = ready(&mut g, 0, catalog::diamond_faerie());
    let yeti = ready(&mut g, 0, catalog::frostpeak_yeti());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..2 {
        ready(&mut g, 0, catalog::snow_covered_island());
    }
    activate(&mut g, faerie).expect("{1}{S} from two snow lands");
    assert_eq!(pt(&g, faerie), (4, 4));
    assert_eq!(pt(&g, yeti), (4, 4));
    assert_eq!(pt(&g, bears), (2, 2));
}

/// Snow Devil: flying always; first strike only while blocking with a snow
/// land under its controller.
#[test]
fn snow_devil_first_strike_needs_a_block_and_a_snow_land() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let devil = g.add_card_to_battlefield(0, catalog::snow_devil());
    g.battlefield_find_mut(devil).unwrap().attached_to = Some(bear);
    let has = |g: &GameState, k: Keyword| g.computed_permanent(bear).unwrap().keywords().contains(&k);
    assert!(has(&g, Keyword::Flying));
    let attacker = ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::snow_covered_island());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(!has(&g, Keyword::FirstStrike), "not blocking yet");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, attacker)])).expect("block");
    drain_stack(&mut g);
    assert!(has(&g, Keyword::FirstStrike));
}

/// Glacial Plating: +3/+3 for each age counter on it.
#[test]
fn glacial_plating_scales_with_age_counters() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let plating = g.add_card_to_battlefield(0, catalog::glacial_plating());
    g.battlefield_find_mut(plating).unwrap().attached_to = Some(bear);
    assert_eq!(pt(&g, bear), (2, 2));
    g.battlefield_find_mut(plating).unwrap().add_counters(CounterType::Age, 2);
    assert_eq!(pt(&g, bear), (8, 8));
}

/// CR 615 — Cover of Winter prevents X of each attacking creature's combat
/// damage, X its age counters.
#[test]
fn cover_of_winter_shaves_creature_combat_damage() {
    let mut g = pod(2);
    let cover = g.add_card_to_battlefield(0, catalog::cover_of_winter());
    g.battlefield_find_mut(cover).unwrap().add_counters(CounterType::Age, 1);
    let attacker = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    let life = g.players[0].life;
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    assert_eq!(g.players[0].life, life - 1, "2 power, 1 prevented");
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(6);
}

/// Bladeback Sliver: hellbent — only an empty hand gives Slivers the ping.
#[test]
fn bladeback_sliver_pings_only_when_hellbent() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    let sliver = ready(&mut g, 0, catalog::bladeback_sliver());
    g.add_card_to_hand(0, catalog::island());
    let ping = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: sliver,
            ability_index: 0,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(ping(&mut g).is_err(), "a card in hand");
    g.players[0].hand.clear();
    let life = g.players[1].life;
    ping(&mut g).expect("hellbent");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
}

/// CR 506.2 — Hellkite Whelp's attack ping hits a creature of the player it
/// attacks, not another opponent's (three seats).
#[test]
fn hellkite_whelp_pings_the_defending_players_creature() {
    let mut g = pod(3);
    let whelp = ready(&mut g, 0, catalog::hellkite_whelp());
    let theirs = ready(&mut g, 2, catalog::llanowar_elves());
    let other = ready(&mut g, 1, catalog::llanowar_elves());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: whelp, target: AttackTarget::Player(2) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "the defending player's 1/1");
    assert!(g.battlefield_find(other).is_some());
}

/// Prophetic Ravings gives its host haste and a {T}, discard: draw loot.
#[test]
fn prophetic_ravings_grants_haste_and_a_loot() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ravings = g.add_card_to_battlefield(0, catalog::prophetic_ravings());
    g.battlefield_find_mut(ravings).unwrap().attached_to = Some(bear);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Haste));
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, bear).expect("summoning sick, but hasty");
    assert_eq!(g.players[0].hand.len(), 1, "discarded one, drew one");
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Volatile Claws: +2/+0 and every creature type until end of turn.
#[test]
fn volatile_claws_pumps_and_grants_every_type() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let claws = g.add_card_to_hand(0, catalog::volatile_claws());
    flood(&mut g, 0);
    cast(&mut g, claws).expect("cast");
    assert_eq!(pt(&g, bear), (4, 2));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Changeling));
}

/// Vandalize — "choose one or both": an artifact and a land in one cast.
#[test]
fn vandalize_destroys_an_artifact_and_a_land() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let land = g.add_card_to_battlefield(1, catalog::island());
    let v = g.add_card_to_hand(0, catalog::vandalize());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: v,
        spree_modes: vec![0, 1],
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![Target::Permanent(land)],
        x_value: None,
    })
    .expect("both modes");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_none() && g.battlefield_find(land).is_none());
}
