//! Commander: the Tinker Time precon (MOC, Gimbal, Gremlin Prodigy,
//! `decks::cmdr_gimbal`). The primitives it needed are tested in
//! `core_rules/commander_cards.rs`.

use crabomination::card::{CardDefinition, CardId, CounterType, Keyword};
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

fn cast_as(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, mode: Option<usize>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    cast_as(g, 0, id, target, None)
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn ready(g: &mut GameState, seat: usize, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

/// Two differently named artifact tokens: a Treasure and a Clue.
fn treasure_and_clue(g: &mut GameState) {
    use crabomination::effect::shortcut::{investigate, mint_treasures};
    use crabomination::game::effects::EffectContext;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&mint_treasures(1), &ctx).expect("a Treasure");
    g.resolve_effect(&investigate(1), &ctx).expect("a Clue");
    assert_eq!(named(g, 0, "Treasure").len(), 1);
    assert_eq!(named(g, 0, "Clue").len(), 1);
}

/// CR 122.1 — Gimbal's Gremlin counts the differently named artifact tokens,
/// itself included.
#[test]
fn gimbal_makes_a_gremlin_per_distinct_artifact_token() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::gimbal_gremlin_prodigy());
    treasure_and_clue(&mut g);
    step(&mut g, TurnStep::End);
    let gremlin = named(&g, 0, "Gremlin")[0];
    assert_eq!(pt(&g, gremlin), (3, 3), "Treasure, Clue, Gremlin");
    assert!(g.permanent_has_keyword(gremlin, &Keyword::Trample));
}

/// CR 702.139a — Aid from the Cowl needs revolt; then a revealed permanent
/// card may go onto the battlefield.
#[test]
fn aid_from_the_cowl_deploys_under_revolt() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::aid_from_the_cowl());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    step(&mut g, TurnStep::End);
    assert!(g.battlefield_find(bear).is_none(), "no revolt yet");
    let seer = g.add_card_to_battlefield(0, catalog::viscera_seer());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    activate(&mut g, seer, 0, None).expect("sacrifice the Ornithopter");
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    step(&mut g, TurnStep::End);
    assert!(g.battlefield_find(bear).is_some());
}

/// CR 207.2c (parley) — Cutthroat Negotiator makes a tapped Treasure per
/// nonland card revealed, then each player draws.
#[test]
fn cutthroat_negotiator_parleys_for_treasure() {
    let mut g = pod(3);
    let orc = ready(&mut g, 0, catalog::cutthroat_negotiator());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::grizzly_bears());
    g.add_card_to_library(2, catalog::island());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: orc, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let treasures = named(&g, 0, "Treasure");
    assert_eq!(treasures.len(), 2);
    assert!(treasures.iter().all(|&t| g.battlefield_find(t).unwrap().tapped));
    assert!(g.players.iter().all(|p| p.hand.len() == 1), "everyone drew");
}

/// CR 601.2 — Dance with Calamity casts everything it exiled free when the
/// total is 13 or less.
#[test]
fn dance_with_calamity_casts_the_exiled_spells() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let dance = g.add_card_to_hand(0, catalog::dance_with_calamity());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 4]));
    cast(&mut g, dance, None).expect("cast");
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 4, "eight mana value, all cast");
}

/// CR 602.2 — Ghirapur Aether Grid taps two artifacts to ping.
#[test]
fn ghirapur_aether_grid_taps_artifacts_to_ping() {
    let mut g = pod(2);
    let grid = g.add_card_to_battlefield(0, catalog::ghirapur_aether_grid());
    let a = g.add_card_to_battlefield(0, catalog::ornithopter());
    let b = g.add_card_to_battlefield(0, catalog::ornithopter());
    let life = g.players[1].life;
    activate(&mut g, grid, 0, Some(Target::Player(1))).expect("tap both");
    assert_eq!(g.players[1].life, life - 1);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
    assert!(activate(&mut g, grid, 0, Some(Target::Player(1))).is_err(), "no untapped artifacts left");
}

/// CR 603.2 — Hedron Detonator pings an opponent per artifact; sacrificing
/// two exiles the top card to play this turn.
#[test]
fn hedron_detonator_pings_and_impulses() {
    let mut g = pod(2);
    let det = ready(&mut g, 0, catalog::hedron_detonator());
    let life = g.players[1].life;
    for _ in 0..2 {
        let t = g.add_card_to_hand(0, catalog::ornithopter());
        cast(&mut g, t, None).expect("an artifact");
    }
    assert_eq!(g.players[1].life, life - 2);
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    activate(&mut g, det, 0, None).expect("sacrifice both");
    let card = g.exile.iter().find(|c| c.id == top).expect("exiled");
    assert!(card.may_play_until.is_some());
}

/// CR 702.126a — Inspiring Statuary lets artifacts help pay for a
/// nonartifact spell.
#[test]
fn inspiring_statuary_grants_improvise() {
    let mut g = pod(2);
    let statuary = g.add_card_to_battlefield(0, catalog::inspiring_statuary());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    assert!(g.helper_tap_candidates(0, bolt).contains(&statuary));
}

/// CR 707.2 — Masterful Replication's second mode turns every other artifact
/// into a copy of the target until end of turn.
#[test]
fn masterful_replication_copies_an_artifact_onto_the_rest() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let thopter = g.add_card_to_battlefield(0, catalog::ornithopter());
    let mr = g.add_card_to_hand(0, catalog::masterful_replication());
    cast_as(&mut g, 0, mr, Some(Target::Permanent(ring)), Some(1)).expect("mode two");
    assert_eq!(g.computed_permanent(thopter).unwrap().def.name, "Sol Ring");
    let golems = g.add_card_to_hand(0, catalog::masterful_replication());
    cast_as(&mut g, 0, golems, None, Some(0)).expect("mode one");
    assert_eq!(named(&g, 0, "Golem").len(), 2);
}

/// CR 603.2 — Pain Distributor: a Treasure for each player's first spell of
/// the turn, a point of damage for each opponent's dying artifact.
#[test]
fn pain_distributor_pays_the_first_spell_and_punishes_artifacts() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::pain_distributor());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, Some(Target::Player(0)), None).expect("first");
    assert_eq!(named(&g, 1, "Treasure").len(), 1);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, Some(Target::Player(0)), None).expect("second");
    assert_eq!(named(&g, 1, "Treasure").len(), 1, "only the first");
    let thopter = g.add_card_to_battlefield(1, catalog::ornithopter());
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(thopter))).expect("kill it");
    assert_eq!(g.players[1].life, life - 1);
}

/// CR 701.23 — Path of the Animist fetches two basics tapped.
#[test]
fn path_of_the_animist_ramps_two() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    let path = g.add_card_to_hand(0, catalog::path_of_the_animist());
    cast(&mut g, path, None).expect("cast");
    let lands: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).collect();
    assert_eq!(lands.len(), 2);
    assert!(lands.iter().all(|c| c.tapped));
}

/// CR 601.2 — Rashmi and Ragavan steals a cheap spell off an opponent's
/// library on your first spell of the turn.
#[test]
fn rashmi_and_ragavan_casts_an_opponents_card() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::rashmi_and_ragavan());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::ornithopter());
    }
    let bear = g.add_card_to_library(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    cast(&mut g, bolt, Some(Target::Player(1))).expect("first spell");
    assert_eq!(named(&g, 0, "Treasure").len(), 1);
    let card = g.battlefield_find(bear).expect("cast free: 2 < 4 artifacts");
    assert_eq!(card.controller, 0);
}

/// CR 701.39a — Sandsteppe War Riders bolsters by the distinct artifact
/// tokens.
#[test]
fn sandsteppe_war_riders_bolsters() {
    let mut g = pod(2);
    let riders = g.add_card_to_battlefield(0, catalog::sandsteppe_war_riders());
    treasure_and_clue(&mut g);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(pt(&g, riders), (4, 4));
}

/// CR 707.2 — Schema Thief copies an artifact of the player it hit.
#[test]
fn schema_thief_copies_an_artifact() {
    let mut g = pod(2);
    let thief = ready(&mut g, 0, catalog::schema_thief());
    g.add_card_to_battlefield(1, catalog::sol_ring());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: thief, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(named(&g, 0, "Sol Ring").len(), 1);
}

/// CR 702.33 — kicked Skyclave Relic makes two tapped copies.
#[test]
fn skyclave_relic_kicked_makes_three() {
    let mut g = pod(2);
    let relic = g.add_card_to_hand(0, catalog::skyclave_relic());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: relic,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked");
    drain_stack(&mut g);
    let relics = named(&g, 0, "Skyclave Relic");
    assert_eq!(relics.len(), 3);
    assert_eq!(relics.iter().filter(|&&id| g.battlefield_find(id).unwrap().tapped).count(), 2);
}

/// CR 701.5 — Spell Swindle counters and pays out the spell's mana value.
#[test]
fn spell_swindle_counters_for_treasure() {
    let mut g = pod(2);
    let colossus = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: colossus,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("opponent casts");
    let swindle = g.add_card_to_hand(0, catalog::spell_swindle());
    cast(&mut g, swindle, Some(Target::Permanent(colossus))).expect("counter");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == colossus), "countered");
    assert_eq!(named(&g, 0, "Treasure").len(), 1, "mana value one");
}

/// CR 303.4 — Weirding Wood investigates and the land taps for two of one
/// color.
#[test]
fn weirding_wood_investigates_and_doubles_the_land() {
    let mut g = pod(2);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let wood = g.add_card_to_hand(0, catalog::weirding_wood());
    cast(&mut g, wood, Some(Target::Permanent(forest))).expect("cast");
    assert_eq!(named(&g, 0, "Clue").len(), 1);
    assert_eq!(g.battlefield_find(wood).unwrap().attached_to, Some(forest));
}
