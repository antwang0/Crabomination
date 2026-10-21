//! Commander: the 2026-10-21 (`01KabMxS`) Attraction-rider closers
//! (`decks::cmdr_01ka_a`) and the Attraction events they read.

use crabomination::card::{CardId, CardInstance, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn stock_attraction_deck(g: &mut GameState, p: usize, defs: Vec<crabomination::card::CardDefinition>) {
    for def in defs {
        let id = g.next_id();
        g.players[p].attraction_deck.push(CardInstance::new(id, def, p));
    }
}

/// Seat 0 rolls to visit its Attractions off Command Performance (its "roll to
/// visit" and ticket modes), the d6 scripted to `roll`, then `rest`.
fn roll_to_visit(g: &mut GameState, roll: u8, rest: Vec<DecisionAnswer>) {
    let cp = g.add_card_to_hand(0, catalog::command_performance());
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new(std::iter::once(DecisionAnswer::DieRoll(roll)).chain(rest)));
    g.perform_action(GameAction::CastSpellSpree {
        card_id: cp,
        spree_modes: vec![1, 2],
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

/// CR 701.52a / 706.2 — rolling to visit is a die roll, so Dee Kay's "whenever
/// you roll a 2" drains off it; CR 717.5 — the visit counts toward "you've
/// visited an Attraction this turn" (Soul Swindler's indestructible), and a
/// miss visits nothing.
#[test]
fn cr_701_52a_a_visit_roll_is_a_die_roll_and_counts_as_a_visit() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::information_booth()); // lights 2, 6
    ready(&mut g, 0, catalog::dee_kay_finder_of_the_lost());
    let swindler = ready(&mut g, 0, catalog::soul_swindler());
    let indestructible = |g: &GameState| g.computed_permanent(swindler).unwrap().keywords().contains(&Keyword::Indestructible);
    assert!(!indestructible(&g));
    roll_to_visit(&mut g, 4, vec![]);
    assert!(!indestructible(&g), "a 4 lights nothing on the Booth");
    let (me, opps) = (g.players[0].life, [g.players[1].life, g.players[2].life]);
    roll_to_visit(&mut g, 2, vec![]);
    assert_eq!([g.players[1].life, g.players[2].life], [opps[0] - 1, opps[1] - 1], "each opponent loses 1");
    assert_eq!(g.players[0].life, me + 1, "you gain 1 once");
    assert!(indestructible(&g), "visited an Attraction this turn");
    g.turn_number += 1;
    assert!(!indestructible(&g), "the tally is this turn's only");
}

/// Squirrel Squatters makes one tapped-and-attacking Squirrel per Attraction
/// visited this turn (CR 508.3a) — two lit Attractions on one roll are two.
#[test]
fn squirrel_squatters_mints_a_squirrel_per_attraction_visited() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::information_booth()); // lights 2, 6
    g.add_card_to_battlefield(0, catalog::roller_coaster()); // lights 2, 6
    let sq = ready(&mut g, 0, catalog::squirrel_squatters());
    roll_to_visit(&mut g, 6, vec![]);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: sq, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let squirrels: Vec<_> =
        g.battlefield.iter().filter(|c| c.definition.name == "Squirrel" && c.controller == 0).collect();
    assert_eq!(squirrels.len(), 2);
    assert!(squirrels.iter().all(|c| c.tapped));
}

/// Motion Sickness: "whenever you visit an Attraction" fires off the Aura (not
/// the Attraction, CR 717.5's visit ability) and moves it onto a tapped
/// creature (CR 701.3).
#[test]
fn motion_sickness_moves_to_a_tapped_creature_on_a_visit() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::information_booth()); // lights 2, 6
    let first = ready(&mut g, 1, catalog::grizzly_bears());
    let tapped = ready(&mut g, 1, catalog::hill_giant());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    let aura = g.add_card_to_battlefield(0, catalog::motion_sickness());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(first);
    roll_to_visit(&mut g, 6, vec![DecisionAnswer::Bool(true)]);
    assert_eq!(g.battlefield_find(aura).unwrap().attached_to, Some(tapped));
}

/// CR 701.51c / 702.159b — The Most Dangerous Gamer grows once per Attraction
/// actually opened (an empty Attraction deck opens nothing), and claiming
/// Pick-a-Beeble's prize (six luck counters) destroys a target permanent.
#[test]
fn cr_702_159b_the_most_dangerous_gamer_reads_opens_and_prizes() {
    let mut g = pod(2);
    let gamer = g.add_card_to_hand(0, catalog::the_most_dangerous_gamer());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: gamer, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let p1p1 = |g: &GameState| g.battlefield_find(gamer).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied();
    assert_eq!(p1p1(&g), None, "no Attraction deck — nothing opened");
    stock_attraction_deck(&mut g, 0, vec![catalog::pick_a_beeble(), catalog::information_booth()]);
    let soul = g.add_card_to_hand(0, catalog::soul_swindler());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: soul, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(p1p1(&g), Some(1), "Soul Swindler opened Pick-a-Beeble");
    let beeble = g.battlefield.iter().find(|c| c.definition.name == "Pick-a-Beeble").unwrap().id;
    g.battlefield_find_mut(beeble).unwrap().counters.insert(CounterType::Luck, 5);
    let victim = ready(&mut g, 1, catalog::serra_angel());
    // Visit on a 2, Beeble's own roll a 1: six luck counters, the prize.
    roll_to_visit(&mut g, 2, vec![DecisionAnswer::DieRoll(1)]);
    assert!(g.battlefield_find(beeble).is_none(), "sacrificed for the prize");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Information Booth"), "and opened the next");
    assert_eq!(p1p1(&g), Some(2), "the prize's open grows it too");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 3);
    assert!(g.battlefield_find(victim).is_none(), "the claimed prize destroyed a target permanent");
}

/// Done for the Day — at your end step, an Employee makes it pay a ticket or a
/// Treasure (an intervening-if, CR 603.4); with no Employee, Performer or Robot
/// it does nothing.
#[test]
fn cr_603_4_done_for_the_day_pays_only_with_an_employee() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::done_for_the_day());
    let end_step = |g: &mut GameState| {
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        drain_stack(g);
    };
    end_step(&mut g);
    let paid = |g: &GameState| g.players[0].tickets + g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count() as u32;
    assert_eq!(paid(&g), 0);
    ready(&mut g, 0, catalog::soul_swindler());
    end_step(&mut g);
    assert_eq!(paid(&g), 1);
}
