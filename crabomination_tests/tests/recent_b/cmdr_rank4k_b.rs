//! Commander: one-card gaps of EDHREC ranks 3501-4500 (`decks::cmdr_rank4k_b`).

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

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: x,
        },
    );
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(
        g,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        },
    );
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne)
}

fn has(g: &GameState, id: CardId, k: Keyword) -> bool {
    g.computed_permanent(id).unwrap().keywords().contains(&k)
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
    g.step = TurnStep::PreCombatMain;
}

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks
            .into_iter()
            .map(|(attacker, p)| Attack {
                attacker,
                target: AttackTarget::Player(p),
            })
            .collect(),
    ))
    .expect("attack");
    drain_stack(g);
}


fn kill(g: &mut GameState, id: CardId) {
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(g, murder, Some(Target::Permanent(id)), None);
}

fn loyalty(g: &mut GameState, id: CardId, index: usize) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: id,
        ability_index: index,
        target: None,
        x_value: None,
    })
    .expect("loyalty");
    drain_stack(g);
}

/// Ferocity of the Hunt returns the dead creature tapped under its owner's
/// control (CR 303.4).
#[test]
fn ferocity_of_the_hunt_brings_its_host_back_tapped() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::ferocity_of_the_hunt());
    cast(&mut g, aura, Some(Target::Permanent(bear)), None);
    assert!(has(&g, bear, Keyword::Deathtouch));
    kill(&mut g, bear);
    let back = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Grizzly Bears")
        .expect("returned");
    assert!(back.tapped);
}

/// CR 104.2b — Near-Death Experience wins at exactly 1 life.
#[test]
fn near_death_experience_wins_at_one_life() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::near_death_experience());
    g.players[0].life = 2;
    step(&mut g, TurnStep::Upkeep);
    assert!(!g.is_game_over());
    g.players[0].life = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    while !g.stack.is_empty() && !g.is_game_over() {
        let _ = g.perform_action(GameAction::PassPriority);
    }
    assert_eq!(g.game_over, Some(Some(0)));
}

/// CR 702.87 — Lighthouse Chronologist at level 7 takes an extra turn at an
/// opponent's end step, not its own.
#[test]
fn lighthouse_chronologist_steals_turns_at_level_seven() {
    let mut g = pod(2);
    let lh = ready(&mut g, 0, catalog::lighthouse_chronologist());
    for _ in 0..7 {
        activate(&mut g, lh, 0, None);
    }
    assert_eq!(
        (g.computed_permanent(lh).unwrap().power, g.computed_permanent(lh).unwrap().toughness),
        (3, 5)
    );
    step(&mut g, TurnStep::End);
    assert_eq!(g.players[0].extra_turns, 0, "not on its own turn");
    g.active_player_idx = 1;
    step(&mut g, TurnStep::End);
    assert_eq!(g.players[0].extra_turns, 1);
}

/// Fantastic Bounce costs {2} less aimed at a tapped creature.
#[test]
fn fantastic_bounce_is_cheaper_on_a_tapped_creature() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.add_card_to_library(0, catalog::island());
    let bounce = g.add_card_to_hand(0, catalog::fantastic_bounce());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bounce,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{1}{U} is enough");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Moment of Glory from the graveyard (flashback) counters every creature
/// you control; from hand, only the target.
#[test]
fn moment_of_glory_flashback_spreads_counters() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::moment_of_glory());
    cast(&mut g, m, Some(Target::Permanent(a)), None);
    assert_eq!((plus(&g, a), plus(&g, b)), (1, 0));
    act(
        &mut g,
        GameAction::CastFlashback {
            card_id: m,
            target: Some(Target::Permanent(a)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        },
    );
    assert_eq!((plus(&g, a), plus(&g, b)), (2, 1));
}

/// Repurposed Enforcer empowers Jace by your creature count.
#[test]
fn repurposed_enforcer_empowers_jace() {
    let mut g = pod(2);
    let cop = ready(&mut g, 0, catalog::repurposed_enforcer());
    ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, vec![(cop, 1)]);
    let jace = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.card_types.contains(&crabomination::card::CardType::Planeswalker))
        .expect("Jace token");
    assert_eq!(jace.counter_count(CounterType::Loyalty), 2);
}

/// Salvager of Ruin returns a permanent card that died this turn.
#[test]
fn salvager_of_ruin_rescues_the_dead() {
    let mut g = pod(2);
    let salvager = ready(&mut g, 0, catalog::salvager_of_ruin());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    kill(&mut g, bear);
    activate(&mut g, salvager, 0, Some(Target::Permanent(bear)));
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Joel draws and grows once a turn off creature tokens dying.
#[test]
fn joel_draws_once_a_turn_off_token_deaths() {
    let mut g = pod(2);
    let joel = ready(&mut g, 0, catalog::joel_resolute_survivor());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let kiora = g.add_card_to_hand(0, catalog::kiora_of_fire_and_ashes());
    cast(&mut g, kiora, None, None);
    let dragons: Vec<CardId> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Dragon")
        .map(|c| c.id)
        .collect();
    let hand = g.players[0].hand.len();
    kill(&mut g, dragons[0]);
    assert_eq!(plus(&g, joel), 1);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Greensleeves −3 puts the milled permanent cards into your hand, not the
/// instants.
#[test]
fn greensleeves_digs_permanents() {
    let mut g = pod(2);
    let gs = g.add_card_to_battlefield(0, catalog::greensleeves());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::grizzly_bears());
    loyalty(&mut g, gs, 1);
    let names: Vec<&str> = g.players[0].hand.iter().map(|c| c.definition.name).collect();
    assert_eq!(names.len(), 2, "{names:?}");
    assert!(names.contains(&"Island") && names.contains(&"Grizzly Bears"));
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Ashnod's attack sacrifice mints a tapped Powerstone.
#[test]
fn ashnod_trades_a_creature_for_a_powerstone() {
    let mut g = pod(2);
    let ashnod = ready(&mut g, 0, catalog::ashnod_flesh_mechanist());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    declare(&mut g, vec![(ashnod, 1)]);
    assert!(g.battlefield_find(bear).is_none());
    let stone = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Powerstone")
        .expect("Powerstone");
    assert!(stone.tapped);
}

/// CR 700.2 — with an empty hand Kirk's "choose one or more" never runs zero
/// modes: a named pick runs just those, an empty answer runs all three.
#[test]
fn captain_kirk_empty_hand_runs_one_or_more_modes() {
    let officers = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Officer").count();
    for (answer, tokens) in [(vec![1u8], 1), (vec![], 1)] {
        let mut g = pod(2);
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        g.players[0].hand.clear();
        g.add_card_to_library(0, catalog::forest());
        let kirk = g.add_card_to_hand(0, catalog::captain_james_t_kirk());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(answer.clone())]));
        cast(&mut g, kirk, None, None);
        assert_eq!(officers(&g), tokens, "{answer:?}");
        let pumped = g.computed_permanent(bear).unwrap().power == 3;
        assert_eq!(pumped, answer.is_empty(), "{answer:?}: the +1/+0 runs only with every mode");
    }
}
