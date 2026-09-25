//! Commander: the Doom Prevails precon (MSC, Doctor Doom, King of Latveria,
//! `decks::cmdr_doom`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector, Value};
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn cast_as(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
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

fn plus_ones(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

fn discard(g: &mut GameState, seat: usize, id: CardId) {
    let mut evs = Vec::new();
    assert!(g.discard_card(seat, id, &mut evs));
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn begin_combat(g: &mut GameState) {
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(g);
}

/// CR 701.50a / 701.50c — Doctor Doom's beginning-of-combat trigger makes a
/// Villain of yours connive; Iron Monger ("whenever a creature you control
/// connives") puts a +1/+1 counter on each Villain you control, and Glorious
/// Purpose puts one on the conniver and a plan counter on itself.
#[test]
fn cr_701_50_doom_connives_and_the_connive_payoffs_fire() {
    let mut g = main_phase(4);
    let doom = g.add_card_to_battlefield(0, catalog::doctor_doom_king_of_latveria());
    let monger = g.add_card_to_battlefield(0, catalog::iron_monger_sadistic_tycoon());
    let plan = g.add_card_to_battlefield(0, catalog::glorious_purpose());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    begin_combat(&mut g);
    let doom_counters = plus_ones(&g, doom);
    let monger_counters = plus_ones(&g, monger);
    // Iron Monger: one on each Villain. Glorious Purpose: one more on the
    // conniver. Connive itself may add one more for a nonland discard.
    assert!(doom_counters >= 1 && monger_counters >= 1, "each Villain got Iron Monger's counter");
    assert!(doom_counters + monger_counters >= 3, "the conniver also got Glorious Purpose's counter");
    assert_eq!(plus_ones(&g, bear), 0, "a non-Villain gets no counter");
    assert_eq!(g.battlefield_find(plan).unwrap().counter_count(CounterType::Plan), 1);
    let conniver = if doom_counters > monger_counters { doom } else { monger };
    let c = g.computed_permanent(conniver).unwrap();
    assert!(c.keywords().contains(&Keyword::Menace), "the target gains menace");
}

/// CR 701.9a — Doctor Doom: discarding a land card drains each opponent 2;
/// a nonland discard does nothing.
#[test]
fn cr_701_9_doom_punishes_land_discards() {
    let mut g = main_phase(4);
    g.add_card_to_battlefield(0, catalog::doctor_doom_king_of_latveria());
    let start = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    discard(&mut g, 0, bolt);
    assert!((1..4).all(|s| g.players[s].life == start));
    let land = g.add_card_to_hand(0, catalog::island());
    discard(&mut g, 0, land);
    assert!((1..4).all(|s| g.players[s].life == start - 2), "each opponent loses 2");
    assert_eq!(g.players[0].life, start);
}

/// CR 601.2 — Madame Hydra: casting a Villain spell makes a 2/1 menace
/// Villain; a non-Villain spell doesn't.
#[test]
fn cr_601_2_madame_hydra_rewards_villain_spells() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::madame_hydra());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_as(&mut g, 0, bears, None).expect("bears");
    assert!(named(&g, 0, "Villain").is_empty());
    let monger = g.add_card_to_hand(0, catalog::iron_monger_sadistic_tycoon());
    cast_as(&mut g, 0, monger, None).expect("Iron Monger");
    let tokens = named(&g, 0, "Villain");
    assert_eq!(tokens.len(), 1);
    assert_eq!(pt(&g, tokens[0]), (2, 1));
    assert!(g.computed_permanent(tokens[0]).unwrap().keywords().contains(&Keyword::Menace));
}

/// CR 601.2 — The Frightful Four: an opponent's first noncreature spell
/// each turn costs them its mana value in life; their second doesn't.
#[test]
fn cr_601_2_frightful_four_taxes_the_first_noncreature_spell() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::the_frightful_four());
    let first = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, first, Some(Target::Player(0))).expect("bolt");
    assert_eq!(g.players[1].life, 19, "the first costs its mana value");
    let second = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, second, Some(Target::Player(0))).expect("bolt");
    assert_eq!(g.players[1].life, 19, "the second is free");
}

/// CR 613.4c — The Squadron Sinister: other Villains you control get +2/+2
/// with flying and haste; it doesn't pump itself or a non-Villain.
#[test]
fn cr_613_4c_squadron_sinister_anthems_villains() {
    let mut g = main_phase(2);
    let squad = g.add_card_to_battlefield(0, catalog::the_squadron_sinister());
    let monger = g.add_card_to_battlefield(0, catalog::iron_monger_sadistic_tycoon());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, squad), (5, 5));
    assert_eq!(pt(&g, monger), (4, 4));
    assert_eq!(pt(&g, bear), (2, 2));
    let m = g.computed_permanent(monger).unwrap();
    assert!(m.keywords().contains(&Keyword::Haste) && m.keywords().contains(&Keyword::Flying));
}

/// CR 121.2 — Kang, Temporal Tyrant: the second card you draw each turn
/// makes each opponent lose 1 and you gain 1 (once, not per opponent).
#[test]
fn cr_121_2_kang_drains_on_the_second_draw() {
    let mut g = main_phase(4);
    let kang = g.add_card_to_battlefield(0, catalog::kang_temporal_tyrant());
    let start = g.players[0].life;
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(kang);
    let ev = g
        .resolve_effect(
&Effect::Draw { who: Selector::You, amount: Value::Const(2) }, &ctx)
        .expect("draw");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!((1..4).all(|s| g.players[s].life == start - 1), "each opponent loses 1");
    assert_eq!(g.players[0].life, start + 1, "you gain 1 once");
}

/// CR 601.2 — Endless Ranks of HYDRA makes a 2/1 menace Villain per
/// opponent.
#[test]
fn cr_601_2_endless_ranks_makes_one_villain_per_opponent() {
    let mut g = main_phase(4);
    let ranks = g.add_card_to_hand(0, catalog::endless_ranks_of_hydra());
    cast_as(&mut g, 0, ranks, None).expect("Endless Ranks");
    assert_eq!(named(&g, 0, "Villain").len(), 3);
}

/// CR 702.62a — Kang Prime entering exiles from the top until a nonland
/// card and gives it two time counters and suspend.
#[test]
fn cr_702_62a_kang_prime_suspends_the_first_nonland_card() {
    let mut g = main_phase(2);
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let kang = g.add_card_to_hand(0, catalog::kang_prime());
    cast_as(&mut g, 0, kang, None).expect("Kang Prime");
    let exiled = g.exile.iter().find(|c| c.id == bolt).expect("the nonland card is exiled");
    assert_eq!(exiled.counter_count(CounterType::Time), 2);
    assert!(g.players[0].library.is_empty(), "the lands above it were exiled too");
}

/// CR 714.2 — Age of Ultron's chapter II makes a 2/2 Robot Villain per
/// opponent.
#[test]
fn cr_714_2_age_of_ultron_builds_robots_per_opponent() {
    let mut g = main_phase(4);
    let id = g.add_card_to_battlefield(0, catalog::age_of_ultron());
    g.battlefield.iter_mut().filter(|c| c.id == id).for_each(|c| c.add_counters(CounterType::Lore, 1));
    g.step = TurnStep::Draw;
    for _ in 0..4 {
        if g.step == TurnStep::PreCombatMain {
            break;
        }
        if let Ok(events) = g.advance_step(Vec::new()) {
            g.dispatch_triggers_for_events(&events);
        }
        drain_stack(&mut g);
    }
    let robots = named(&g, 0, "Robot Villain");
    assert_eq!(robots.len(), 3);
    assert_eq!(pt(&g, robots[0]), (2, 2));
}
