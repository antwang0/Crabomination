//! Commander: the Chaos Incarnate deck (SCD, Kardur, `decks::cmdr_kardur`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
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

fn cast_by(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Step from `from` into `to` on `active`'s turn, resolving what triggers.
fn step_into(g: &mut GameState, active: usize, from: TurnStep, to: TurnStep) {
    g.active_player_idx = active;
    g.step = from;
    g.priority.player_with_priority = active;
    while g.step != to {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

fn attack(g: &mut GameState, attacker: CardId, defender: usize) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(defender) }]))
        .expect("attack");
    drain_stack(g);
}

fn creatures_of(g: &GameState, seat: usize) -> Vec<&'static str> {
    let mut v: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_creature())
        .map(|c| c.definition.name)
        .collect();
    v.sort();
    v
}

/// Archfiend of Depravity: at an opponent's end step they keep two creatures
/// (a bot keeps its best) and sacrifice the rest; your own end step is spared.
#[test]
fn archfiend_of_depravity_trims_each_opponent_to_two() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::archfiend_of_depravity());
    for def in [catalog::grizzly_bears(), catalog::craw_wurm(), catalog::hill_giant(), catalog::grizzly_bears()] {
        g.add_card_to_battlefield(1, def);
    }
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    step_into(&mut g, 1, TurnStep::PostCombatMain, TurnStep::End);
    assert_eq!(creatures_of(&g, 1), vec!["Craw Wurm", "Hill Giant"]);
    step_into(&mut g, 0, TurnStep::PostCombatMain, TurnStep::End);
    assert_eq!(creatures_of(&g, 0).len(), 4, "your own end step doesn't trigger it");
}

/// Dredge the Mire: each opponent gives up one creature card of their own
/// graveyard (a bot, its least valuable); you get them all.
#[test]
fn dredge_the_mire_takes_one_from_each_opponent() {
    let mut g = pod(3);
    let bears = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::craw_wurm());
    let giant = g.add_card_to_graveyard(2, catalog::hill_giant());
    let dm = g.add_card_to_hand(0, catalog::dredge_the_mire());
    cast_by(&mut g, 0, dm, &[]);
    for id in [bears, giant] {
        assert_eq!(g.battlefield_find(id).map(|c| c.controller), Some(0));
    }
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Craw Wurm"));
}

/// Explosion of Riches: you draw; another player accepts and draws, the
/// third declines. Two draws, two 5-damage hits among your opponents.
#[test]
fn explosion_of_riches_burns_per_card_drawn() {
    let mut g = pod(3);
    for s in 0..3 {
        for _ in 0..3 {
            g.add_card_to_library(s, catalog::island());
        }
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(false)]));
    let er = g.add_card_to_hand(0, catalog::explosion_of_riches());
    let before: i32 = g.players[1].life + g.players[2].life;
    cast_by(&mut g, 0, er, &[]);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[1].hand.len() + g.players[2].hand.len(), 1);
    assert_eq!(before - g.players[1].life - g.players[2].life, 10);
    assert_eq!(g.players[0].life, g.players[0].starting_life, "never you");
}

/// CR 701.38 — Kardur goads every creature your opponents control, including
/// one that enters later (its ruling), until your next turn; an attacking
/// creature dying drains each opponent 1 for 1.
#[test]
fn kardur_goads_the_table_until_your_next_turn() {
    let mut g = pod(3);
    let early = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let kardur = g.add_card_to_hand(0, catalog::kardur_doomscourge());
    cast_by(&mut g, 0, kardur, &[]);
    assert!(g.battlefield_find(early).unwrap().goaded_by.contains(&0));
    let late = g.add_card_to_hand(2, catalog::hill_giant());
    g.active_player_idx = 2;
    cast_by(&mut g, 2, late, &[]);
    assert!(g.battlefield_find(late).unwrap().goaded_by.contains(&0), "entered later, goaded too");
    let mine = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    cast_by(&mut g, 0, mine, &[]);
    assert!(g.battlefield_find(mine).unwrap().goaded_by.is_empty());

    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(early).unwrap().goaded_by.is_empty(), "lapsed at your next turn");
    let after = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    cast_by(&mut g, 1, after, &[]);
    assert!(g.battlefield_find(after).unwrap().goaded_by.is_empty(), "the watch lapsed too");

    // An attacking creature dies: each opponent loses 1, you gain 1.
    g.active_player_idx = 1;
    attack(&mut g, early, 2);
    let (l0, l1, l2) = (g.players[0].life, g.players[1].life, g.players[2].life);
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast_by(&mut g, 0, murder, &[Target::Permanent(early)]);
    assert_eq!((g.players[0].life, g.players[1].life, g.players[2].life), (l0 + 1, l1 - 1, l2 - 1));
}

/// Molten Slagheap banks storage counters and cashes X of them for X mana.
#[test]
fn molten_slagheap_stores_and_cashes_in() {
    let mut g = pod(2);
    let heap = g.add_card_to_battlefield(0, catalog::molten_slagheap());
    for _ in 0..2 {
        g.players[0].mana_pool.add_colorless(1);
        activate(&mut g, heap, 1, None, None);
        g.battlefield_find_mut(heap).unwrap().tapped = false;
    }
    assert_eq!(g.battlefield_find(heap).unwrap().counter_count(CounterType::Storage), 2);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, heap, 2, None, Some(2));
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.amount(Color::Black) + pool.amount(Color::Red), 2);
    assert_eq!(g.battlefield_find(heap).unwrap().counter_count(CounterType::Storage), 0);
}

/// Rakshasa Debaser attacking takes a creature card from the defending
/// player's graveyard.
#[test]
fn rakshasa_debaser_raids_the_defenders_graveyard() {
    let mut g = pod(3);
    let deb = g.add_card_to_battlefield(0, catalog::rakshasa_debaser());
    let wurm = g.add_card_to_graveyard(2, catalog::craw_wurm());
    g.add_card_to_graveyard(1, catalog::hill_giant());
    attack(&mut g, deb, 2);
    assert_eq!(g.battlefield_find(wurm).map(|c| c.controller), Some(0));
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Hill Giant"), "not the other opponent's");
}

/// Scythe Specter: each opponent discards; the greatest mana value discarded
/// costs its discarder that much life.
#[test]
fn scythe_specter_punishes_the_biggest_discard() {
    let mut g = pod(3);
    g.add_card_to_hand(1, catalog::craw_wurm());
    g.add_card_to_hand(2, catalog::grizzly_bears());
    let spec = g.add_card_to_battlefield(0, catalog::scythe_specter());
    attack(&mut g, spec, 1);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.players[1].hand.is_empty() && g.players[2].hand.is_empty(), "each opponent discarded");
    assert_eq!(g.players[1].life, g.players[1].starting_life - 4 - 6, "four combat, six for the Wurm");
    assert_eq!(g.players[2].life, g.players[2].starting_life, "the Bears weren't the greatest");
}

/// Theater of Horrors: the exiled card is only playable on your turn once an
/// opponent has lost life this turn.
#[test]
fn theater_of_horrors_opens_after_an_opponent_bleeds() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::theater_of_horrors());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    step_into(&mut g, 0, TurnStep::Untap, TurnStep::Draw);
    assert!(g.exile.iter().any(|c| c.id == bear), "exiled at upkeep");
    g.step = TurnStep::PreCombatMain;
    let cast = |g: &mut GameState| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastFromZoneWithoutPaying {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    flood(&mut g, 0);
    assert!(cast(&mut g).is_err(), "no opponent has lost life yet");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_by(&mut g, 0, bolt, &[Target::Player(1)]);
    let mana = g.players[0].mana_pool.total();
    cast(&mut g).expect("now it's open");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(g.players[0].mana_pool.total(), mana - 2, "paid its own cost");
}

/// Titan Hunter: at each player's end step with no creature death this turn,
/// that player takes 4.
#[test]
fn titan_hunter_punishes_peaceful_turns() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::titan_hunter());
    step_into(&mut g, 1, TurnStep::PostCombatMain, TurnStep::End);
    assert_eq!(g.players[1].life, g.players[1].starting_life - 4);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.active_player_idx = 0;
    g.step = TurnStep::PostCombatMain;
    cast_by(&mut g, 0, murder, &[Target::Permanent(bear)]);
    step_into(&mut g, 0, TurnStep::PostCombatMain, TurnStep::End);
    assert_eq!(g.players[0].life, g.players[0].starting_life, "a creature died this turn");
}

/// Unlicensed Disintegration: with an artifact, 3 to the dead creature's
/// controller; without, just the kill.
#[test]
fn unlicensed_disintegration_burns_with_an_artifact() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ud = g.add_card_to_hand(0, catalog::unlicensed_disintegration());
    cast_by(&mut g, 0, ud, &[Target::Permanent(bear)]);
    assert_eq!(g.players[1].life, g.players[1].starting_life);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    let ud = g.add_card_to_hand(0, catalog::unlicensed_disintegration());
    cast_by(&mut g, 0, ud, &[Target::Permanent(bear)]);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[1].life, g.players[1].starting_life - 3);
}

/// Wildfire Devils: a random player's instant or sorcery is exiled and you
/// cast a copy — with every graveyard holding a Divination, you draw two.
#[test]
fn wildfire_devils_recasts_a_random_graveyard_spell() {
    let mut g = pod(3);
    for s in 0..3 {
        g.add_card_to_graveyard(s, catalog::divination());
        g.add_card_to_library(0, catalog::island());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let wd = g.add_card_to_hand(0, catalog::wildfire_devils());
    cast_by(&mut g, 0, wd, &[]);
    assert_eq!(g.exile.iter().filter(|c| c.definition.name == "Divination").count(), 1);
    assert_eq!(g.players[0].hand.len(), 2, "the copy drew two");
}
