//! Commander: the Silverquill Statement precon (C21, Breena,
//! `decks::cmdr_breena`).

use crabomination::card::{CardId, CounterType};
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

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    cast_x(g, seat, id, target, None)
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: x,
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

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

/// Seat `seat` (made active) declares `attacks`.
fn declare(g: &mut GameState, seat: usize, attacks: Vec<(CardId, usize)>) -> Result<(), String> {
    g.active_player_idx = seat;
    for (a, _) in &attacks {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, d)| Attack { attacker, target: AttackTarget::Player(d) }).collect(),
    ))
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// CR 508.1 — Breena fires when any player attacks one of its controller's
/// opponents who has more life than another of them: the attacker draws, and
/// Breena's controller gets two +1/+1 counters.
#[test]
fn breena_rewards_attacks_on_the_healthier_opponent() {
    let mut g = pod(3);
    stock_libraries(&mut g, 5);
    let breena = g.add_card_to_battlefield(0, catalog::breena_the_demagogue());
    g.players[1].life = 30;
    g.players[2].life = 20;
    // Seat 1 attacks seat 2 — the poorer opponent: nothing.
    let h1 = g.players[1].hand.len();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    declare(&mut g, 1, vec![(a, 2)]).expect("attack");
    assert_eq!(g.players[1].hand.len(), h1);
    assert_eq!(g.battlefield_find(breena).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    // Seat 2 attacks seat 1 — the richer one: seat 2 draws, Breena grows.
    let mut g2 = pod(3);
    stock_libraries(&mut g2, 5);
    let breena = g2.add_card_to_battlefield(0, catalog::breena_the_demagogue());
    g2.players[1].life = 30;
    g2.players[2].life = 20;
    let h2 = g2.players[2].hand.len();
    let b = g2.add_card_to_battlefield(2, catalog::grizzly_bears());
    declare(&mut g2, 2, vec![(b, 1)]).expect("attack");
    assert_eq!(g2.players[2].hand.len(), h2 + 1, "the attacking player draws");
    assert_eq!(g2.battlefield_find(breena).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Combat Calligrapher gives any player attacking an opponent of yours a
/// tapped Inkling attacking that opponent — and Inklings can't attack you.
#[test]
fn combat_calligrapher_hands_out_attacking_inklings() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::combat_calligrapher());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    declare(&mut g, 1, vec![(a, 2)]).expect("attack");
    let ink = named(&g, 1, "Inkling");
    assert_eq!(ink.len(), 1);
    assert!(g.attacking.iter().any(|x| x.attacker == ink[0] && x.target == AttackTarget::Player(2)));
    assert!(g.battlefield_find(ink[0]).unwrap().tapped);
}

/// Nils taxes each creature with counters by its counter count to attack you.
#[test]
fn nils_taxes_countered_attackers() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::nils_discipline_enforcer());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.players[1].mana_pool.add_colorless(1);
    assert!(declare(&mut g, 1, vec![(bear, 0)]).is_err(), "two counters, one mana");
    g.players[1].mana_pool.add_colorless(1);
    declare(&mut g, 1, vec![(bear, 0)]).expect("pays two");
}

/// Nils's end step puts a +1/+1 counter on a creature of each player.
#[test]
fn nils_counters_a_creature_of_each_player() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::nils_discipline_enforcer());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    for id in [a, b] {
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }
}

/// CR 701.15 — Bloodthirsty Blade moves onto an opponent's creature and
/// goads it (by static, `decks::cmdr_nelly`).
#[test]
fn bloodthirsty_blade_goads_its_host() {
    let mut g = pod(3);
    let blade = g.add_card_to_battlefield(0, catalog::bloodthirsty_blade());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, blade, 0, Some(Target::Permanent(bear)), None).expect("attach");
    assert_eq!(g.battlefield_find(blade).unwrap().attached_to, Some(bear));
    assert_eq!(pt(&g, bear), (4, 2));
    g.active_player_idx = 1;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(g.goaded_by_player(g.battlefield_find(bear).unwrap(), 0));
}

/// Parasitic Impetus drains the host's controller when it attacks.
#[test]
fn parasitic_impetus_drains_on_attack() {
    let mut g = pod(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pi = g.add_card_to_hand(0, catalog::parasitic_impetus());
    cast(&mut g, 0, pi, Some(Target::Permanent(bear))).expect("cast");
    assert_eq!(pt(&g, bear), (4, 4));
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    declare(&mut g, 1, vec![(bear, 2)]).expect("attack");
    assert_eq!((g.players[0].life, g.players[1].life), (l0 + 2, l1 - 2));
}

/// Tempting Contract: each accepting opponent gets a Treasure and so do you.
#[test]
fn tempting_contract_pays_per_acceptor() {
    let mut g = pod(4);
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(true),
    ]));
    g.add_card_to_battlefield(0, catalog::tempting_contract());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let accepted = (1..4).filter(|&s| !named(&g, s, "Treasure").is_empty()).count();
    assert_eq!(named(&g, 0, "Treasure").len(), accepted);
}

/// Keen Duelist: each loses the other's top card's mana value and takes their
/// own top card.
#[test]
fn keen_duelist_trades_top_cards() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::keen_duelist());
    let mine = g.add_card_to_library(0, catalog::serra_angel());
    let theirs = g.add_card_to_library(1, catalog::grizzly_bears());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!((g.players[0].life, g.players[1].life), (l0 - 2, l1 - 5));
    assert!(g.players[0].hand.iter().any(|c| c.id == mine));
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs));
}

/// Deathbringer Regent wipes the board only with five other creatures.
#[test]
fn deathbringer_regent_needs_five_others() {
    let mut g = pod(2);
    let bears: Vec<CardId> = (0..5).map(|i| g.add_card_to_battlefield(i % 2, catalog::grizzly_bears())).collect();
    let dr = g.add_card_to_hand(0, catalog::deathbringer_regent());
    cast(&mut g, 0, dr, None).expect("cast");
    assert!(bears.iter().all(|&b| g.battlefield_find(b).is_none()));
    assert!(g.battlefield_find(dr).is_some());
}

/// Tragic Arrogance keeps one of each nonland permanent type per player.
#[test]
fn tragic_arrogance_keeps_one_of_each_type() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_battlefield(1, catalog::sol_ring());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let ta = g.add_card_to_hand(0, catalog::tragic_arrogance());
    cast(&mut g, 0, ta, None).expect("cast");
    let creatures = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_creature()).count();
    assert_eq!(creatures, 1);
    assert_eq!(named(&g, 1, "Sol Ring").len(), 1);
    assert!(g.battlefield_find(land).is_some());
}

/// Stinging Study draws and costs the commander's mana value.
#[test]
fn stinging_study_reads_your_commander() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    g.seat_commanders(0, vec![catalog::breena_the_demagogue()]);
    let (h, l) = (g.players[0].hand.len(), g.players[0].life);
    let ss = g.add_card_to_hand(0, catalog::stinging_study());
    cast(&mut g, 0, ss, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), h + 3);
    assert_eq!(g.players[0].life, l - 3);
}

/// CR 903.3 — X is a commander you own "on the battlefield or in the command
/// zone": one left in the graveyard (its owner declined the CR 903.9a move)
/// adds nothing.
#[test]
fn stinging_study_ignores_a_commander_in_the_graveyard() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let cmd = g.seat_commanders(0, vec![catalog::breena_the_demagogue()])[0];
    let pos = g.players[0].command.iter().position(|c| c.id == cmd).unwrap();
    let card = g.players[0].command.remove(pos);
    g.players[0].graveyard.push(card);
    g.commander_return_declined.push(cmd);
    let (h, l) = (g.players[0].hand.len(), g.players[0].life);
    let ss = g.add_card_to_hand(0, catalog::stinging_study());
    cast(&mut g, 0, ss, None).expect("cast");
    assert_eq!((g.players[0].hand.len(), g.players[0].life), (h, l));
}

/// Author of Shadows exiles the opponents' graveyards and lets you cast a
/// nonland card from among them.
#[test]
fn author_of_shadows_steals_a_spell() {
    let mut g = pod(3);
    let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.add_card_to_graveyard(2, catalog::forest());
    let au = g.add_card_to_hand(0, catalog::author_of_shadows());
    cast(&mut g, 0, au, None).expect("cast");
    assert!(g.players[1].graveyard.is_empty() && g.players[2].graveyard.is_empty());
    assert!(g.exile.iter().any(|c| c.id == bolt && c.may_play_until.is_some()));
}

/// Boreas Charger fetches the land gap to the landiest opponent in Plains.
#[test]
fn boreas_charger_closes_the_land_gap() {
    let mut g = pod(3);
    for _ in 0..4 {
        g.add_card_to_battlefield(2, catalog::forest());
    }
    g.add_card_to_battlefield(0, catalog::plains());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::plains());
    }
    let bc = g.add_card_to_battlefield(0, catalog::boreas_charger());
    let hand = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(bc))).expect("bolt");
    assert_eq!(named(&g, 0, "Plains").len(), 2, "one onto the battlefield");
    assert_eq!(g.players[0].hand.len(), hand + 2, "the other two to hand");
}

/// Pendant of Prosperity enters under an opponent; its activation feeds both
/// its controller and its owner.
#[test]
fn pendant_of_prosperity_feeds_both_players() {
    let mut g = pod(2);
    for s in 0..2 {
        g.add_card_to_library(s, catalog::grizzly_bears());
    }
    let pp = g.add_card_to_hand(0, catalog::pendant_of_prosperity());
    cast(&mut g, 0, pp, None).expect("cast");
    assert_eq!(g.battlefield_find(pp).unwrap().controller, 1);
    let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
    activate(&mut g, 1, pp, 0, None, None).expect("activate");
    assert_eq!((g.players[0].hand.len(), g.players[1].hand.len()), (h0 + 1, h1 + 1));
}

/// Inkshield makes an Inkling per point of unblocked power swinging at you and
/// prevents that damage.
#[test]
fn inkshield_turns_the_swing_into_inklings() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    declare(&mut g, 1, vec![(giant, 0)]).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    let ink = g.add_card_to_hand(0, catalog::inkshield());
    cast(&mut g, 0, ink, None).expect("cast");
    assert_eq!(named(&g, 0, "Inkling").len(), 3);
    let life = g.players[0].life;
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, life);
}

/// Bold Plagiarist copies the +1/+1 counters an opponent puts on their own
/// creature.
#[test]
fn bold_plagiarist_copies_counters() {
    let mut g = pod(2);
    let bp = g.add_card_to_battlefield(0, catalog::bold_plagiarist());
    let wd = g.add_card_to_battlefield(1, catalog::willowdusk_essence_seer());
    g.clear_sickness(wd);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[1].life_gained_this_turn = 2;
    g.active_player_idx = 1;
    activate(&mut g, 1, wd, 0, Some(Target::Permanent(bear)), None).expect("two counters");
    assert_eq!(g.battlefield_find(bp).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Deathbringer Liege taps with a white spell and kills the tapped with a
/// black one.
#[test]
fn deathbringer_liege_taps_then_kills() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::deathbringer_liege());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let warden = g.add_card_to_hand(0, catalog::soul_warden());
    cast(&mut g, 0, warden, None).expect("a white spell");
    assert!(g.battlefield_find(angel).unwrap().tapped, "tapped");
    let bp = g.add_card_to_hand(0, catalog::bold_plagiarist());
    cast(&mut g, 0, bp, None).expect("a black spell");
    assert!(g.battlefield_find(angel).is_none(), "destroyed while tapped");
}

/// Two Bold Plagiarists across the table don't feed each other: the counters a
/// Plagiarist is given are put on a creature the opponent doesn't control.
#[test]
fn bold_plagiarists_do_not_loop() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::bold_plagiarist());
    let b = g.add_card_to_battlefield(1, catalog::bold_plagiarist());
    let wd = g.add_card_to_battlefield(1, catalog::willowdusk_essence_seer());
    g.clear_sickness(wd);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[1].life_gained_this_turn = 1;
    g.active_player_idx = 1;
    activate(&mut g, 1, wd, 0, Some(Target::Permanent(bear)), None).expect("a counter");
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// CR 702.16b/e — Guardian Archon's reveal: you and the target permanent gain
/// protection from the chosen opponent until end of turn. That player can't
/// target you or it and their damage to you is prevented; another opponent
/// is unaffected, and the protection ends at cleanup.
#[test]
fn cr_702_16_guardian_archon_protects_you_and_a_permanent_from_one_player() {
    use crabomination::decision::Decider;
    struct Seat3;
    impl Decider for Seat3 {
        fn decide(&mut self, d: &crabomination::decision::Decision) -> DecisionAnswer {
            match d {
                crabomination::decision::Decision::ChooseOption { options, .. } => {
                    DecisionAnswer::Amount(options.iter().position(|o| o == "Player 3").unwrap() as u32)
                }
                other => crabomination::decision::AutoDecider.decide(other),
            }
        }
        fn kind(&self) -> crabomination::decision::DeciderKind {
            crabomination::decision::DeciderKind::Scripted { answers: vec![], asked: vec![] }
        }
    }
    let mut g = pod(3);
    g.decider = Box::new(Seat3);
    let archon = g.move_card_to_battlefield_for_test(0, catalog::guardian_archon());
    g.decider = Box::new(crabomination::decision::AutoDecider);
    assert_eq!(g.battlefield_find(archon).unwrap().chosen_player, Some(2));
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, archon, 0, Some(Target::Permanent(bears)), None).expect("reveal");
    for seat in [2, 1] {
        let bolt = g.add_card_to_hand(seat, catalog::lightning_bolt());
        let at_you = cast(&mut g, seat, bolt, Some(Target::Player(0)));
        let bolt = g.add_card_to_hand(seat, catalog::lightning_bolt());
        let at_bears = cast(&mut g, seat, bolt, Some(Target::Permanent(bears)));
        assert_eq!((at_you.is_err(), at_bears.is_err()), (seat == 2, seat == 2), "seat {seat}: {at_you:?} {at_bears:?}");
    }
    assert_eq!(g.players[0].life, 17, "only seat 1's bolt landed");
    assert!(g.battlefield_find(bears).is_none(), "seat 1's second bolt killed the Bears");
    g.step = TurnStep::End;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(2, catalog::lightning_bolt());
    cast(&mut g, 2, bolt, Some(Target::Player(0))).expect("the protection ended at cleanup");
}
