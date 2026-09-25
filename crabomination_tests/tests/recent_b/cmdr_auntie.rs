//! Commander: the Blight Curse precon (ECC, Auntie Ool, `decks::cmdr_auntie`),
//! and its primitives: blight as an optional additional cost (CR 701.68,
//! 601.2b), a clash that repeats while won (CR 701.30), a life / toughness
//! exchange with a target player (CR 119.7), removing counters for cards, and
//! "a counter was put on a creature this turn".

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
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

fn cast_with(
    g: &mut GameState,
    seat: usize,
    id: CardId,
    target: Option<Target>,
    extra: Vec<Target>,
    x: Option<u32>,
) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: extra, mode: None, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    cast_with(g, seat, id, target, vec![], None)
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

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn minus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::MinusOneMinusOne))
}

fn put_minus(g: &mut GameState, id: CardId, n: i32) -> Vec<GameEvent> {
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g
        .resolve_effect(
            &Effect::AddCounter {
                what: Selector::ExactObjects(vec![id]),
                kind: CounterType::MinusOneMinusOne,
                amount: crabomination::card::Value::Const(n),
            },
            &ctx,
        )
        .expect("counters");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
    evs
}

fn kill(g: &mut GameState, id: CardId) {
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![id]) }, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
    g.step = TurnStep::PreCombatMain;
}

fn on_top(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.next_id();
    g.players[seat].add_to_library_top(id, def);
    id
}

/// CR 701.68 / 601.2b — Burning Curiosity's optional blight 1: paid, a
/// creature you control takes a -1/-1 counter and three cards are exiled;
/// unpaid, two.
#[test]
fn cr_701_68_burning_curiosity_optional_blight() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    flood(&mut g, 0);
    let bc = g.add_card_to_hand(0, catalog::burning_curiosity());
    let exiled = g.exile.len();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: bc,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("blighted");
    drain_stack(&mut g);
    assert_eq!(minus(&g, giant), 1, "blight 1 paid on the Giant");
    assert_eq!(g.exile.len(), exiled + 3);
    let bc2 = g.add_card_to_hand(0, catalog::burning_curiosity());
    cast(&mut g, 0, bc2, None).expect("unblighted");
    assert_eq!(g.exile.len(), exiled + 5);
    assert_eq!(minus(&g, giant), 1);
}

/// CR 701.68b — no creature, no blight: the kicked cast is refused.
#[test]
fn cr_701_68b_blight_cost_needs_a_creature() {
    let mut g = pod(2);
    flood(&mut g, 0);
    let bc = g.add_card_to_hand(0, catalog::burning_curiosity());
    g.priority.player_with_priority = 0;
    let r = g.perform_action(GameAction::CastSpellKicked {
        card_id: bc,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(r.is_err());
}

/// CR 701.30 — Hoarder's Greed repeats while its clashes are won.
#[test]
fn cr_701_30_hoarders_greed_repeats_while_winning() {
    let mut g = pod(2);
    // Seat 0, from the top: two draws, a Hill Giant that wins the clash, two
    // more draws, then a Sol Ring that ties the opponent's.
    g.players[0].library.clear();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::sol_ring());
    }
    on_top(&mut g, 0, catalog::island());
    on_top(&mut g, 0, catalog::hill_giant());
    on_top(&mut g, 0, catalog::island());
    on_top(&mut g, 0, catalog::island());
    flood(&mut g, 0);
    let hg = g.add_card_to_hand(0, catalog::hoarders_greed());
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, hg, None).expect("Hoarder's Greed");
    assert_eq!(g.players[0].life, 16, "two rounds of 2 life");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 4, "two rounds of two cards");
}

/// CR 119.7 — Tree of Perdition swaps an opponent's life with its toughness.
#[test]
fn cr_119_7_tree_of_perdition_exchanges() {
    let mut g = pod(2);
    let tree = g.add_card_to_battlefield(0, catalog::tree_of_perdition());
    g.clear_sickness(tree);
    activate(&mut g, 0, tree, 0, Some(Target::Player(1))).expect("exchange");
    assert_eq!(g.players[1].life, 13);
    assert_eq!(g.computed_permanent(tree).unwrap().toughness, 20);
}

/// Eventide's Shadow — the headless pick strips the opponent's +1/+1 counters
/// and your -1/-1 ones: three removed, three drawn, three life lost.
#[test]
fn eventides_shadow_draws_per_counter() {
    let mut g = pod(2);
    let theirs = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.battlefield_find_mut(theirs).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let mine = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.battlefield_find_mut(mine).unwrap().add_counters(CounterType::MinusOneMinusOne, 1);
    flood(&mut g, 0);
    let es = g.add_card_to_hand(0, catalog::eventides_shadow());
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, es, None).expect("Eventide's Shadow");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3);
    assert_eq!(g.players[0].life, 17);
    assert_eq!(g.battlefield_find(theirs).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(minus(&g, mine), 0);
}

/// Lasting Tarfire — each end step, 2 to each opponent, but only on a turn a
/// counter went on a creature.
#[test]
fn lasting_tarfire_needs_a_counter_this_turn() {
    let mut g = pod(3);
    g.turn_number = 5;
    g.add_card_to_battlefield(0, catalog::lasting_tarfire());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    step(&mut g, TurnStep::End);
    assert_eq!((g.players[1].life, g.players[2].life), (20, 20), "no counter yet");
    put_minus(&mut g, bear, 1);
    step(&mut g, TurnStep::End);
    assert_eq!((g.players[1].life, g.players[2].life), (18, 18));
    g.turn_number = 6;
    step(&mut g, TurnStep::End);
    assert_eq!(g.players[1].life, 18, "a new turn");
}

/// Blowfly Infestation — a creature dying with a -1/-1 counter passes one on.
#[test]
fn blowfly_infestation_passes_the_counter() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::blowfly_infestation());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    put_minus(&mut g, a, 1);
    kill(&mut g, a);
    assert_eq!(minus(&g, b), 1);
}

/// Kulrath Knight — an opponent's creature with a counter can't attack or
/// block; one without can.
#[test]
fn kulrath_knight_locks_countered_creatures() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::kulrath_knight());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    put_minus(&mut g, a, 1);
    let kws = |id| g.computed_permanent(id).unwrap().keywords().to_vec();
    assert!(kws(a).contains(&Keyword::CantAttack) && kws(a).contains(&Keyword::CantBlock));
    assert!(!kws(b).contains(&Keyword::CantAttack));
}

/// The Reaper — an opponent's creature dying with a -1/-1 counter comes over,
/// once each turn.
#[test]
fn reaper_steals_once_each_turn() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::the_reaper_king_no_more());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    put_minus(&mut g, a, 1);
    put_minus(&mut g, b, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    kill(&mut g, a);
    assert_eq!(g.battlefield_find(a).map(|c| c.controller), Some(0), "stolen");
    kill(&mut g, b);
    assert!(g.battlefield_find(b).is_none(), "only once each turn");
}

/// Oft-Nabbed Goat — an opponent's {1} draws, steals it and adds a counter;
/// dying with counters, its owner draws that many and the others lose that
/// much.
#[test]
fn oft_nabbed_goat_changes_hands() {
    let mut g = pod(3);
    let goat = g.add_card_to_battlefield(0, catalog::oft_nabbed_goat());
    flood(&mut g, 1);
    g.active_player_idx = 1; // only as a sorcery: on seat 1's turn
    let hand = g.players[1].hand.len();
    activate(&mut g, 1, goat, 0, None).expect("an opponent activates");
    assert_eq!(g.battlefield_find(goat).unwrap().controller, 1);
    assert_eq!(minus(&g, goat), 1);
    assert_eq!(g.players[1].hand.len(), hand + 1);
    assert!(activate(&mut g, 1, goat, 0, None).is_err(), "only opponents of its controller");
    let owner_hand = g.players[0].hand.len();
    kill(&mut g, goat);
    assert_eq!(g.players[0].hand.len(), owner_hand + 1, "the owner draws");
    assert_eq!((g.players[1].life, g.players[2].life), (19, 19));
    assert_eq!(g.players[0].life, 20);
}

/// Ferrafor — a Saproling per counter among the target player's creatures.
#[test]
fn ferrafor_counts_counters() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.battlefield_find_mut(a).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(b).unwrap().add_counters(CounterType::MinusOneMinusOne, 1);
    flood(&mut g, 0);
    let f = g.add_card_to_hand(0, catalog::ferrafor_young_yew());
    cast(&mut g, 0, f, None).expect("Ferrafor");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Saproling").len(), 3);
}

/// Wickersmith's Tools — a charge counter per -1/-1 placement; sacrificed, X
/// tapped Scarecrows.
#[test]
fn wickersmiths_tools_makes_scarecrows() {
    let mut g = pod(2);
    let tools = g.add_card_to_battlefield(0, catalog::wickersmiths_tools());
    let a = g.add_card_to_battlefield(1, catalog::hill_giant());
    put_minus(&mut g, a, 2);
    put_minus(&mut g, a, 1);
    assert_eq!(g.battlefield_find(tools).unwrap().counter_count(CounterType::Charge), 2);
    flood(&mut g, 0);
    activate(&mut g, 0, tools, 1, None).expect("sacrifice");
    let crows = named(&g, 0, "Scarecrow");
    assert_eq!(crows.len(), 2);
    assert!(crows.iter().all(|&c| g.battlefield_find(c).unwrap().tapped));
}

/// Incremental Blight — 1, 2 and 3 counters on three different creatures.
#[test]
fn incremental_blight_three_targets() {
    let mut g = pod(2);
    let ids: Vec<CardId> = (0..3).map(|_| g.add_card_to_battlefield(1, catalog::craw_wurm())).collect();
    flood(&mut g, 0);
    let ib = g.add_card_to_hand(0, catalog::incremental_blight());
    cast_with(
        &mut g,
        0,
        ib,
        Some(Target::Permanent(ids[0])),
        vec![Target::Permanent(ids[1]), Target::Permanent(ids[2])],
        None,
    )
    .expect("Incremental Blight");
    assert_eq!((minus(&g, ids[0]), minus(&g, ids[1]), minus(&g, ids[2])), (1, 2, 3));
}

/// Dusk Urchins — dies and draws a card per -1/-1 counter on it.
#[test]
fn dusk_urchins_draws_per_counter() {
    let mut g = pod(2);
    let u = g.add_card_to_battlefield(0, catalog::dusk_urchins());
    put_minus(&mut g, u, 2);
    let hand = g.players[0].hand.len();
    kill(&mut g, u);
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// The Scorpion God — draws when a countered creature dies, and comes back to
/// hand at the next end step.
#[test]
fn scorpion_god_draws_and_returns() {
    let mut g = pod(2);
    let god = g.add_card_to_battlefield(0, catalog::the_scorpion_god());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    put_minus(&mut g, a, 1);
    let hand = g.players[0].hand.len();
    kill(&mut g, a);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    kill(&mut g, god);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == god));
    step(&mut g, TurnStep::End);
    assert!(g.players[0].hand.iter().any(|c| c.id == god), "back to hand");
}

/// Hapatra — a Snake whenever -1/-1 counters are put on a creature.
#[test]
fn hapatra_makes_snakes() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::hapatra_vizier_of_poisons());
    let a = g.add_card_to_battlefield(1, catalog::hill_giant());
    put_minus(&mut g, a, 2);
    let snakes = named(&g, 0, "Snake");
    assert_eq!(snakes.len(), 1, "one or more counters: one Snake");
    assert!(g.computed_permanent(snakes[0]).unwrap().keywords().contains(&Keyword::Deathtouch));
}

/// Flourishing Defenses — an Elf Warrior per -1/-1 counter.
#[test]
fn flourishing_defenses_token_per_counter() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::flourishing_defenses());
    let a = g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    put_minus(&mut g, a, 3);
    assert_eq!(named(&g, 0, "Elf Warrior").len(), 3);
}

/// Channeler Initiate — its mana ability spends its own -1/-1 counters.
#[test]
fn channeler_initiate_taps_counters_for_mana() {
    let mut g = pod(2);
    let ci = g.add_card_to_battlefield(0, catalog::channeler_initiate());
    g.clear_sickness(ci);
    g.battlefield_find_mut(ci).unwrap().add_counters(CounterType::MinusOneMinusOne, 3);
    activate(&mut g, 0, ci, 0, None).expect("mana");
    assert_eq!(minus(&g, ci), 2);
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Dread Tiller — a countered creature dying puts a land from your graveyard
/// onto the battlefield tapped.
#[test]
fn dread_tiller_lands_a_land() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::dread_tiller());
    let land = g.add_card_to_graveyard(0, catalog::swamp());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    put_minus(&mut g, a, 1);
    kill(&mut g, a);
    let c = g.battlefield_find(land).expect("the Swamp entered");
    assert!(c.tapped);
}

/// Puca's Covenant — a creature you control dying with two counters returns a
/// permanent card with mana value 2 or less.
#[test]
fn pucas_covenant_returns_by_counter_count() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::pucas_covenant());
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let a = g.add_card_to_battlefield(0, catalog::craw_wurm());
    g.battlefield_find_mut(a).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    kill(&mut g, a);
    assert!(g.players[0].hand.iter().any(|c| c.id == ring), "Sol Ring (1) back");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == giant), "Hill Giant (4) stays");
}

/// Fire Covenant — pay X life, X damage divided among creatures (the
/// headless split is even).
#[test]
fn fire_covenant_pays_life_for_damage() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    flood(&mut g, 0);
    let fc = g.add_card_to_hand(0, catalog::fire_covenant());
    cast_with(&mut g, 0, fc, Some(Target::Permanent(a)), vec![Target::Permanent(b)], Some(6)).expect("X = 6");
    assert_eq!(g.players[0].life, 14);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "an even 3 + 3 split kills both");
}
