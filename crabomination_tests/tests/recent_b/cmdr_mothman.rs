//! Commander: the Mutant Menace precon (PIP, The Wise Mothman,
//! `decks::cmdr_mothman`), and its primitives: radiation that heals (CR
//! 728), damage traded for rad counters (CR 615), a per-cast radiation mark,
//! an activation discount per rad counter, a batch's matching-card count, and
//! a card that returns from the graveyard as its Aura back face.

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, PlayerRef, Selector};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.turn_number = 3;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..12 {
            g.add_card_to_library(seat, catalog::sol_ring());
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

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
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

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
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

fn resolve(g: &mut GameState, seat: usize, eff: Effect) {
    let ctx = EffectContext::for_spell(seat, None, 0, 0);
    let evs = g.resolve_effect(&eff, &ctx).expect("resolve");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

fn on_top(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.next_id();
    g.players[seat].add_to_library_top(id, def);
    id
}

fn kill(g: &mut GameState, id: CardId) {
    resolve(g, 0, Effect::Destroy { what: Selector::ExactObjects(vec![id]) });
}

/// Pass from the draw step into the precombat main phase, where the rad
/// counters are spent (CR 728.2).
fn into_main(g: &mut GameState) {
    g.step = TurnStep::Draw;
    for _ in 0..20 {
        if g.step == TurnStep::PreCombatMain {
            break;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.step, TurnStep::PreCombatMain);
}

/// CR 728 — Strong turns radiation into life gain.
#[test]
fn cr_728_strong_heals_from_radiation() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::strong_the_brutish_thespian());
    for _ in 0..3 {
        on_top(&mut g, 0, catalog::grizzly_bears());
    }
    g.players[0].rad_counters = 3;
    into_main(&mut g);
    assert_eq!(g.players[0].life, 23, "three nonland mills heal");
    assert_eq!(g.players[0].rad_counters, 0);
}

/// CR 728 — without Strong, the same mills cost life.
#[test]
fn cr_728_radiation_costs_life() {
    let mut g = pod(2);
    for _ in 0..3 {
        on_top(&mut g, 0, catalog::grizzly_bears());
    }
    g.players[0].rad_counters = 3;
    into_main(&mut g);
    assert_eq!(g.players[0].life, 17);
}

/// CR 615 — Bloatfly Swarm trades damage for +1/+1 counters and gives each
/// player a rad counter per counter removed.
#[test]
fn cr_615_bloatfly_swarm_trades_damage_for_rad() {
    let mut g = pod(3);
    flood(&mut g, 0);
    let swarm = g.add_card_to_hand(0, catalog::bloatfly_swarm());
    cast(&mut g, 0, swarm, None).expect("Bloatfly Swarm");
    assert_eq!(plus(&g, swarm), 5);
    resolve(&mut g, 1, Effect::DealDamage { to: Selector::ExactObjects(vec![swarm]), amount: crabomination::card::Value::Const(3) });
    assert_eq!(plus(&g, swarm), 2, "three counters off");
    assert!(g.players.iter().all(|p| p.rad_counters == 3));
    assert_eq!(g.battlefield_find(swarm).unwrap().damage, 0, "the damage was prevented");
}

/// Nuka-Nuke Launcher's mark — the marked player gets two rad counters per
/// spell they cast.
#[test]
fn nuka_nuke_marks_spells_with_radiation() {
    let mut g = pod(2);
    resolve(&mut g, 0, Effect::RadOnCastUntilEndOfTheirNextTurn { who: PlayerRef::Seat(1), amount: 2 });
    flood(&mut g, 1);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0))).expect("Bolt");
    assert_eq!(g.players[1].rad_counters, 2);
    assert_eq!(g.players[0].rad_counters, 0);
}

/// Mariposa Military Base — its draw costs {1} less per rad counter.
#[test]
fn mariposa_discount_per_rad_counter() {
    let mut g = pod(2);
    let base = g.add_card_to_battlefield(0, catalog::mariposa_military_base());
    g.players[0].rad_counters = 3;
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    activate(&mut g, 0, base, 1, None).expect("{2} is enough with three rad counters");
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// The Wise Mothman — two nonland cards milled: two creatures get a counter.
#[test]
fn wise_mothman_counts_nonland_mills() {
    let mut g = pod(2);
    let moth = g.add_card_to_battlefield(0, catalog::the_wise_mothman());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    on_top(&mut g, 1, catalog::island());
    on_top(&mut g, 1, catalog::grizzly_bears());
    on_top(&mut g, 1, catalog::grizzly_bears());
    resolve(&mut g, 0, Effect::Mill { who: Selector::Player(PlayerRef::Seat(1)), amount: crabomination::card::Value::Const(3) });
    assert_eq!(plus(&g, wurm) + plus(&g, moth) + plus(&g, bear), 2, "X = 2 nonland cards");
    assert_eq!(plus(&g, wurm), 1, "the biggest creature first");
}

/// Screeching Scorchbeast — a Zombie Mutant per nonland card milled, once a
/// turn.
#[test]
fn scorchbeast_zombies_per_nonland_mill() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::screeching_scorchbeast());
    for _ in 0..3 {
        on_top(&mut g, 1, catalog::grizzly_bears());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    let mill3 = || Effect::Mill { who: Selector::Player(PlayerRef::Seat(1)), amount: crabomination::card::Value::Const(3) };
    resolve(&mut g, 0, mill3());
    assert_eq!(named(&g, 0, "Zombie Mutant").len(), 3);
    resolve(&mut g, 0, mill3());
    assert_eq!(named(&g, 0, "Zombie Mutant").len(), 3, "only once each turn");
}

/// Harold and Bob — dying, it returns as an Aura on a Forest, which gains
/// "{T}: Add three mana of any one color. You get two rad counters."
#[test]
fn harold_and_bob_returns_as_an_aura() {
    let mut g = pod(2);
    let hb = g.add_card_to_battlefield(0, catalog::harold_and_bob_first_numens());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    kill(&mut g, hb);
    let c = g.battlefield_find(hb).expect("back on the battlefield");
    assert_eq!(c.attached_to, Some(forest));
    assert!(!c.definition.is_creature(), "an Aura now");
    let printed = g.battlefield_find(forest).unwrap().definition.activated_abilities.len();
    activate(&mut g, 0, forest, printed, None).expect("the granted ability");
    assert_eq!(g.players[0].mana_pool.total(), 3);
    assert_eq!(g.players[0].rad_counters, 2);
}

/// Rampaging Yao Guai — X counters, and destroys artifacts / enchantments
/// totalling X.
#[test]
fn yao_guai_destroys_within_total_mana_value() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let shatter_target = g.add_card_to_battlefield(1, catalog::ornithopter());
    let big = g.add_card_to_battlefield(1, catalog::phyrexian_arena());
    flood(&mut g, 0);
    let yao = g.add_card_to_hand(0, catalog::rampaging_yao_guai());
    cast_x(&mut g, 0, yao, None, Some(2)).expect("X = 2");
    assert_eq!(plus(&g, yao), 2);
    assert!(g.battlefield_find(ring).is_none() && g.battlefield_find(shatter_target).is_none());
    assert!(g.battlefield_find(big).is_some(), "Phyrexian Arena (3) is over the cap");
}

/// Jason Bright — a Zombie or Mutant dying with its power changed draws.
#[test]
fn jason_bright_draws_on_changed_power() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::jason_bright_glowing_prophet());
    let a = g.add_card_to_battlefield(0, catalog::walking_corpse());
    let b = g.add_card_to_battlefield(0, catalog::walking_corpse());
    g.battlefield_find_mut(a).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let hand = g.players[0].hand.len();
    kill(&mut g, a);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    kill(&mut g, b);
    assert_eq!(g.players[0].hand.len(), hand + 1, "an unchanged Zombie draws nothing");
}

/// Lumbering Megasloth — {1} less per counter among players and permanents.
#[test]
fn megasloth_discount_per_counter() {
    let mut g = pod(2);
    let w = g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.battlefield_find_mut(w).unwrap().add_counters(CounterType::PlusOnePlusOne, 4);
    g.players[1].rad_counters = 3;
    g.players[0].poison_counters = 3;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(0);
    let sloth = g.add_card_to_hand(0, catalog::lumbering_megasloth());
    cast(&mut g, 0, sloth, None).expect("{10} off: {G}{G}");
    assert!(g.battlefield_find(sloth).unwrap().tapped, "enters tapped");
}

/// Nightkin Ambusher — unblockable against a player with a rad counter.
#[test]
fn nightkin_unblockable_against_radiation() {
    let mut g = pod(2);
    let nk = g.add_card_to_battlefield(0, catalog::nightkin_ambusher());
    g.clear_sickness(nk);
    let blocker = g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.players[1].rad_counters = 1;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: nk, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    let r = g.perform_action(GameAction::DeclareBlockers(vec![(blocker, nk)]));
    assert!(r.is_err(), "can't be blocked while they have a rad counter");
}

/// Lily Bowen — doubles its +1/+1 counters each upkeep up to power 16, then
/// resets to one and gains life for the rest.
#[test]
fn lily_bowen_doubles_then_resets() {
    let mut g = pod(2);
    let lily = g.add_card_to_battlefield(0, catalog::lily_bowen_raging_grandma());
    g.battlefield_find_mut(lily).unwrap().add_counters(CounterType::PlusOnePlusOne, 16);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(plus(&g, lily), 32, "power 16: doubled");
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(plus(&g, lily), 1);
    assert_eq!(g.players[0].life, 51, "31 removed, 31 life");
}

/// The Master — a creature milled this turn returns as a 3/3 green Mutant.
#[test]
fn the_master_reanimates_milled_creature() {
    let mut g = pod(2);
    let master = g.add_card_to_battlefield(0, catalog::the_master_transcendent());
    g.clear_sickness(master);
    let wurm = on_top(&mut g, 1, catalog::craw_wurm());
    let old = g.add_card_to_graveyard(1, catalog::craw_wurm());
    resolve(&mut g, 0, Effect::Mill { who: Selector::Player(PlayerRef::Seat(1)), amount: crabomination::card::Value::ONE });
    assert!(activate(&mut g, 0, master, 0, Some(Target::Permanent(old))).is_err(), "not milled this turn");
    activate(&mut g, 0, master, 0, Some(Target::Permanent(wurm))).expect("the milled Wurm");
    let c = g.computed_permanent(wurm).expect("under our control");
    assert_eq!((c.power, c.toughness), (3, 3));
    assert_eq!(g.battlefield_find(wurm).unwrap().controller, 0);
}

/// Struggle for Project Purity's Enclave — two rad counters per creature that
/// attacks you.
#[test]
fn struggle_enclave_irradiates_attackers() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    flood(&mut g, 0);
    let s = g.add_card_to_hand(0, catalog::struggle_for_project_purity());
    cast(&mut g, 0, s, None).expect("Struggle");
    g.active_player_idx = 1;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(b);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(0) },
        Attack { attacker: b, target: AttackTarget::Player(0) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].rad_counters, 4);
}

/// Infesting Radroach — its combat damage becomes rad counters.
#[test]
fn infesting_radroach_irradiates_on_hit() {
    let mut g = pod(2);
    let roach = g.add_card_to_battlefield(0, catalog::infesting_radroach());
    g.clear_sickness(roach);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: roach, target: AttackTarget::Player(1) }]))
        .expect("attack");
    for _ in 0..12 {
        if g.players[1].life < 20 {
            break;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[1].rad_counters, 2);
    assert!(g.battlefield_find(roach).unwrap().definition.keywords.contains(&Keyword::CantBlock));
}

/// CR 702.100b — Watchful Radstag copies itself when it *evolves*, i.e. after
/// its evolve ability puts the counter. A Radstag shrunk to 1/1 copied itself
/// before the counter landed, so every 2/2 copy re-triggered it: a six-seat
/// pod stacked 514 copy triggers. Now the copy follows the counter, the
/// Radstag is 2/2 when its copy enters, and the chain stops.
#[test]
fn cr_702_100b_watchful_radstag_copies_after_it_evolves() {
    let mut g = pod(3);
    let r = g.add_card_to_battlefield(0, catalog::watchful_radstag());
    g.battlefield_find_mut(r).unwrap().add_counters(CounterType::MinusOneMinusOne, 1);
    flood(&mut g, 0);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, 0, bear, None).expect("a 2/2 enters beside a 1/1 Radstag");
    let radstags = g.battlefield.iter().filter(|c| c.definition.name == "Watchful Radstag").count();
    assert_eq!(radstags, 2, "one evolve, one copy");
    assert!(g.stack.is_empty());
}
