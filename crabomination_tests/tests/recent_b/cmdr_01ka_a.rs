//! Commander: the 2026-10-21 (`01KabMxS`) Attraction-rider closers
//! (`decks::cmdr_01ka_a`) and the Attraction events they read.

use crabomination::card::{CardId, CardInstance, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, GameError, TurnStep};
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

/// CR 717.2 / 717.2a — a pod seat whose deck opens Attractions brings an
/// Attraction deck of ten or more distinct names (in the command zone, not the
/// 100); a seat whose deck never opens one brings none.
#[test]
fn cr_717_2a_a_pod_seat_that_opens_attractions_brings_an_attraction_deck() {
    use crabomination::pod::{build_pod_template, target_decks};
    let field = target_decks();
    let gamer = *field.iter().find(|d| d.name.starts_with("The Most Dangerous Gamer")).expect("seat");
    let t = build_pod_template(&[gamer, field[0]]);
    let deck = &t.players[0].attraction_deck;
    assert!(deck.len() >= 10);
    let mut names: Vec<&str> = deck.iter().map(|c| c.definition.name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), deck.len(), "one of each name");
    assert_eq!(t.players[0].library.len(), 99, "outside the 100");
    assert!(t.players[1].attraction_deck.is_empty());
}

// ── Convoke / delve from the command zone and the graveyard; Hogaak ────────

fn commander_pod(def: crabomination::card::CardDefinition) -> (GameState, CardId) {
    let mut g = game_with_format(crabomination::format::Format::Commander, 3);
    let cmd = g.seat_commanders(0, vec![def])[0];
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    (g, cmd)
}

fn helper_cast(g: &mut GameState, card_id: CardId, convoke: &[CardId], delve: &[CardId]) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpellConvokeDelve {
        card_id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        convoke_creatures: convoke.to_vec(),
        delve_cards: delve.to_vec(),
    })
    .map(|_| ())
}

fn bodies(g: &mut GameState, n: usize, def: fn() -> crabomination::card::CardDefinition) -> Vec<CardId> {
    (0..n).map(|_| ready(g, 0, def())).collect()
}

fn graveyard(g: &mut GameState, n: usize) -> Vec<CardId> {
    (0..n).map(|_| g.add_card_to_graveyard(0, catalog::island())).collect()
}

/// CR 903.8 + 702.51a — the commander tax is an additional cost and convoke
/// applies after the total cost, so tapping creatures pays a convoke
/// commander's printed cost and its {2} tax from the command zone (Kasla).
#[test]
fn cr_903_8_convoke_pays_a_commanders_tax_from_the_command_zone() {
    let (mut g, kasla) = commander_pod(catalog::kasla_the_broken_halo());
    g.commander_cast_count.insert(kasla, 1);
    // {3}{U}{R}{W} + {2}: the {U}{R} from mana, the rest from six white creatures.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let helpers = bodies(&mut g, 6, catalog::savannah_lions);
    assert!(helper_cast(&mut g, kasla, &helpers[..5], &[]).is_err(), "one short of the generic, white pip and tax");
    helper_cast(&mut g, kasla, &helpers, &[]).expect("convoked from the command zone");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0, "the mana paid the blue and red pips");
    assert!(g.battlefield_find(kasla).is_some());
    assert!(helpers.iter().all(|h| g.battlefield_find(*h).unwrap().tapped));
    assert_eq!(g.commander_cast_count.get(&kasla), Some(&2), "a command-zone cast, counted");
}

/// Hogaak can't be cast with mana: a plain cast with a full pool fails, and
/// convoke + delve with the pool full leaves the pool untouched. Two black
/// creatures pay the {B/G}{B/G}; five graveyard cards the {5}.
#[test]
fn hogaak_is_paid_by_convoke_and_delve_never_by_mana() {
    let mut g = pod(3);
    let hogaak = g.add_card_to_hand(0, catalog::hogaak_arisen_necropolis());
    flood(&mut g, 0);
    let pool = g.players[0].mana_pool.clone();
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: hogaak, target: None, additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "you can't spend mana to cast it"
    );
    let knights = bodies(&mut g, 2, catalog::vampire_nighthawk);
    let gy = graveyard(&mut g, 5);
    assert!(helper_cast(&mut g, hogaak, &knights[..1], &gy).is_err(), "one short — mana may not cover it");
    assert!(!g.battlefield_find(knights[0]).unwrap().tapped, "a failed cast untaps its helper");
    helper_cast(&mut g, hogaak, &knights, &gy).expect("convoke + delve pay it all");
    drain_stack(&mut g);
    assert!(g.battlefield_find(hogaak).is_some());
    assert_eq!(g.players[0].mana_pool.total(), pool.total(), "no mana spent");
    assert!(gy.iter().all(|c| g.exile.iter().any(|e| e.id == *c)), "delved");
}

/// "You may cast this card from your graveyard" — Hogaak's own permission
/// (CR 601.2 from a zone other than the hand); and as a commander, the {2}
/// tax is paid by one more helper (CR 903.8).
#[test]
fn cr_903_8_hogaak_casts_from_the_graveyard_and_the_command_zone() {
    let mut g = pod(3);
    let hogaak = g.add_card_to_graveyard(0, catalog::hogaak_arisen_necropolis());
    let knights = bodies(&mut g, 2, catalog::vampire_nighthawk);
    let gy = graveyard(&mut g, 5);
    helper_cast(&mut g, hogaak, &knights, &gy).expect("cast from the graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(hogaak).is_some());

    let (mut g, cmd) = commander_pod(catalog::hogaak_arisen_necropolis());
    g.commander_cast_count.insert(cmd, 1);
    let knights = bodies(&mut g, 2, catalog::vampire_nighthawk);
    let gy = graveyard(&mut g, 5);
    assert!(helper_cast(&mut g, cmd, &knights, &gy).is_err(), "the tax is still owed");
    let more = graveyard(&mut g, 2);
    let gy: Vec<CardId> = gy.into_iter().chain(more).collect();
    helper_cast(&mut g, cmd, &knights, &gy).expect("seven delved pay {5} + {2} tax");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cmd).is_some());
}

/// CR 903.8 + 702.33 — a kicker commander is kicked from the command zone:
/// printed {2}{R}{R} + kicker {3} + {2} tax, the kicked ETB fires, and the cast
/// counts toward the next tax. A failed kicked cast leaves it in the command
/// zone.
#[test]
fn cr_903_8_a_commander_is_kicked_from_the_command_zone() {
    let (mut g, verix) = commander_pod(catalog::verix_bladewing());
    g.commander_cast_count.insert(verix, 1);
    let kick = |g: &mut GameState| {
        g.perform_action(GameAction::CastSpellKicked {
            card_id: verix,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    g.players[0].mana_pool.add(Color::Red, 8);
    assert!(kick(&mut g).is_err(), "eight mana is one short of nine");
    assert!(g.players[0].command.iter().any(|c| c.id == verix), "back in the command zone");
    assert_eq!(g.players[0].mana_pool.total(), 8, "and nothing spent");
    g.players[0].mana_pool.add(Color::Red, 1);
    kick(&mut g).expect("kicked from the command zone");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0);
    assert!(g.battlefield_find(verix).is_some());
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Karox Bladewing"), "kicked");
    assert_eq!(g.commander_cast_count.get(&verix), Some(&2));
}

/// CR 903.8 + 702.37 — a morph commander is cast face down from the command
/// zone for {3} plus its tax, as a 2/2 that is still the commander; the cast
/// counts toward the next tax.
#[test]
fn cr_903_8_a_morph_commander_is_cast_face_down_from_the_command_zone() {
    let (mut g, akroma) = commander_pod(catalog::akroma_angel_of_fury());
    g.commander_cast_count.insert(akroma, 1);
    g.players[0].mana_pool.add_colorless(4);
    let face_down = |g: &mut GameState| g.perform_action(GameAction::CastFaceDown { card_id: akroma });
    assert!(face_down(&mut g).is_err(), "three plus a two tax is five");
    assert!(g.players[0].command.iter().any(|c| c.id == akroma));
    g.players[0].mana_pool.add_colorless(1);
    face_down(&mut g).expect("face down from the command zone");
    drain_stack(&mut g);
    let p = g.battlefield_find(akroma).expect("on the battlefield");
    assert!(p.face_down);
    assert!(g.is_commander(akroma));
    assert_eq!(g.commander_cast_count.get(&akroma), Some(&2));
}
