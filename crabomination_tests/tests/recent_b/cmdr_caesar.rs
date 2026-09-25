//! Commander: the Hail, Caesar precon (PIP, Caesar, Legion's Emperor,
//! `decks::cmdr_caesar`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..seats {
        for _ in 0..8 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    act(g, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    act(g, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode: None,
    })
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    for a in attackers {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|a| Attack { attacker: *a, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

/// CR 601.2c — V.A.T.S. destroys any number of target creatures of equal
/// toughness; a pair of different toughness is an illegal set.
#[test]
fn cr_601_2c_vats_needs_equal_toughness() {
    let mut g = main_phase(2);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let vats = g.add_card_to_hand(0, catalog::v_a_t_s());
    assert!(
        cast(&mut g, vats, &[Target::Permanent(a), Target::Permanent(giant)]).is_err(),
        "a 2/2 and a 3/3 don't share a toughness",
    );
    cast(&mut g, vats, &[Target::Permanent(a), Target::Permanent(b)]).expect("two 2/2s");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert!(g.battlefield_find(giant).is_some());
}

/// CR 702.157 — squad's non-mana cost: Ruthless Radrat exiles four
/// graveyard cards per copy, and can't squad past what the graveyard holds.
#[test]
fn cr_702_157_ruthless_radrat_squads_with_its_graveyard() {
    let mut g = main_phase(2);
    for _ in 0..5 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    let rat = g.add_card_to_hand(0, catalog::ruthless_radrat());
    flood(&mut g, 0);
    let squad = |times| GameAction::CastSpellSquad {
        card_id: rat,
        times,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    };
    assert!(act(&mut g, squad(2)).is_err(), "eight cards needed, five in the graveyard");
    act(&mut g, squad(1)).expect("squad once");
    assert_eq!(named(&g, 0, "Ruthless Radrat").len(), 2);
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// CR 702.138 — Desdemona grants escape until end of turn to a cheap
/// creature card in your graveyard: its mana cost plus two other cards.
#[test]
fn cr_702_138_desdemona_grants_escape() {
    let mut g = main_phase(2);
    let desi = g.add_card_to_battlefield(0, catalog::desdemona_freedoms_edge());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let fodder: Vec<CardId> = (0..2).map(|_| g.add_card_to_graveyard(0, catalog::forest())).collect();
    attack(&mut g, &[desi]);
    g.step = TurnStep::PostCombatMain;
    flood(&mut g, 0);
    act(&mut g, GameAction::CastEscape {
        card_id: bears,
        exile_cards: fodder,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("escape");
    assert!(g.battlefield_find(bears).is_some());
}

/// The Nipton Lottery leaves exactly one creature, yours until end of turn.
#[test]
fn nipton_lottery_leaves_one_creature() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::hill_giant());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let lottery = g.add_card_to_hand(0, catalog::the_nipton_lottery());
    cast(&mut g, lottery, &[]).expect("cast");
    let left: Vec<_> = g.battlefield.iter().filter(|c| c.definition.is_creature()).collect();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].controller, 0);
}

/// CR 701.38 — Vault 11's chapter II vote: the most-voted creature is
/// destroyed.
#[test]
fn cr_701_38_vault_11_destroys_the_most_voted() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let saga = g.add_card_to_battlefield(0, catalog::vault_11_voters_dilemma());
    g.battlefield.iter_mut().filter(|c| c.id == saga).for_each(|c| c.add_counters(CounterType::Lore, 1));
    g.step = TurnStep::Draw;
    to_main(&mut g);
    assert!(g.battlefield_find(giant).is_none(), "both headless votes went to the biggest");
    assert!(g.battlefield_find(bears).is_some());
}

/// Advance from the draw step into the precombat main phase, firing the
/// saga's lore counter.
fn to_main(g: &mut GameState) {
    for _ in 0..4 {
        if g.step == TurnStep::PreCombatMain {
            break;
        }
        if let Ok(events) = g.advance_step(Vec::new()) {
            g.dispatch_triggers_for_events(&events);
        }
        drain_stack(g);
    }
}

/// Legate Lanius decimates: an opponent with three creatures sacrifices
/// one (a tenth, rounded up), and that sacrifice grows the Legate.
#[test]
fn legate_lanius_decimates() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    let legate = g.add_card_to_hand(0, catalog::legate_lanius_caesars_ace());
    cast(&mut g, legate, &[]).expect("cast");
    assert_eq!(named(&g, 1, "Grizzly Bears").len(), 2);
    assert_eq!(g.battlefield_find(legate).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Luck Bobblehead: an even roll makes a tapped Treasure.
#[test]
fn luck_bobblehead_rolls_for_treasure() {
    let mut g = main_phase(2);
    let luck = g.add_card_to_battlefield(0, catalog::luck_bobblehead());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(4)]));
    activate(&mut g, luck, 1, &[]).expect("roll");
    let t = named(&g, 0, "Treasure");
    assert_eq!(t.len(), 1);
    assert!(g.battlefield_find(t[0]).unwrap().tapped);
}

/// Overseer of Vault 76 banks quest counters, then spends three at combat
/// to grow the team.
#[test]
fn overseer_of_vault_76_spends_quest_counters() {
    let mut g = main_phase(2);
    let overseer = g.add_card_to_hand(0, catalog::overseer_of_vault_76());
    cast(&mut g, overseer, &[]).expect("cast");
    for _ in 0..2 {
        let b = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast(&mut g, b, &[]).expect("cast");
    }
    assert_eq!(g.battlefield_find(overseer).unwrap().counter_count(CounterType::Quest), 3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(overseer).unwrap().counter_count(CounterType::Quest), 0);
    assert_eq!(g.battlefield_find(overseer).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Yes Man goes to an opponent, draws you two and leaves with a quest
/// counter.
#[test]
fn yes_man_changes_hands() {
    let mut g = main_phase(2);
    let yes = g.add_card_to_battlefield(0, catalog::yes_man_personal_securitron());
    g.clear_sickness(yes);
    let hand = g.players[0].hand.len();
    activate(&mut g, yes, 0, &[Target::Player(1)]).expect("give it away");
    let c = g.battlefield_find(yes).unwrap();
    assert_eq!(c.controller, 1);
    assert_eq!(c.counter_count(CounterType::Quest), 1);
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// Caesar: attacking, it sacrifices another creature and picks two modes.
#[test]
fn caesar_sacrifices_for_two_modes() {
    let mut g = main_phase(2);
    let caesar = g.add_card_to_battlefield(0, catalog::caesar_legions_emperor());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    attack(&mut g, &[caesar]);
    assert!(named(&g, 0, "Grizzly Bears").is_empty(), "sacrificed");
    assert_eq!(named(&g, 0, "Soldier").len(), 2, "two hasty Soldiers");
    assert_eq!(g.players[0].hand.len(), hand + 1, "and a card");
}

/// Caesar on a prompting seat: the sacrifice and the two modes are asked
/// and answered through the pending-decision channel.
#[test]
fn caesar_prompting_seat_answers_cleanly() {
    use crabomination::decision::Decision;
    let mut g = main_phase(2);
    g.players[0].wants_ui = true;
    let caesar = g.add_card_to_battlefield(0, catalog::caesar_legions_emperor());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack(&mut g, &[caesar]);
    for _ in 0..10 {
        let Some(pending) = g.pending_decision.as_ref() else { break };
        let answer = match &pending.decision {
            Decision::ChooseModes { .. } => DecisionAnswer::Modes(vec![0, 2]),
            _ => DecisionAnswer::Bool(true),
        };
        g.submit_decision(answer).expect("answer");
        drain_stack(&mut g);
    }
    assert!(g.pending_decision.is_none());
    assert_eq!(named(&g, 0, "Soldier").len(), 2);
    assert_eq!(g.players[1].life, 18, "two creature tokens' worth of damage");
}

/// Craig Boone: attacking with two, it shoots a creature whose prompting
/// controller may take the damage instead.
#[test]
fn craig_boone_prompting_opponent_answers_cleanly() {
    let mut g = main_phase(2);
    g.players[1].wants_ui = true;
    let boone = g.add_card_to_battlefield(0, catalog::craig_boone_novac_guard());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let life = g.players[1].life;
    attack(&mut g, &[boone, bear]);
    for _ in 0..10 {
        if g.pending_decision.is_none() {
            break;
        }
        g.submit_decision(DecisionAnswer::Bool(true)).expect("answer");
        drain_stack(&mut g);
    }
    assert!(g.pending_decision.is_none());
    assert_eq!(g.battlefield_find(boone).unwrap().counter_count(CounterType::Quest), 2);
    assert!(g.battlefield_find(giant).is_some());
    assert_eq!(g.players[1].life, life - 2, "they took it");
}
