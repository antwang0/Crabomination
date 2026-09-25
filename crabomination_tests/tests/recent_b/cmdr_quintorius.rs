//! Commander: the Lorehold Spirit precon (SOC, Quintorius,
//! `decks::cmdr_quintorius`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
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

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
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

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
}

/// Pull a card out of seat 0's graveyard with Regrowth (a "leaves your
/// graveyard" event).
fn regrow(g: &mut GameState, card: CardId) {
    let r = g.add_card_to_hand(0, catalog::regrowth());
    cast(g, 0, r, Some(Target::Permanent(card))).expect("regrowth");
}

/// Quintorius, History Chaser — cards leaving your graveyard make a 3/2
/// Spirit; −4 gives Spirits double strike.
#[test]
fn quintorius_makes_spirits_from_graveyard_departures() {
    let mut g = pod(2);
    let q = g.add_card_to_battlefield(0, catalog::quintorius_history_chaser());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    regrow(&mut g, bear);
    let sp = named(&g, 0, "Spirit");
    assert_eq!(sp.len(), 1);
    assert_eq!(pt(&g, sp[0]), (3, 2));
    g.battlefield_find_mut(q).unwrap().add_counters(CounterType::Loyalty, 5);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: q, ability_index: 1, target: None, x_value: None })
        .expect("−4");
    drain_stack(&mut g);
    assert!(g.computed_permanent(sp[0]).unwrap().keywords().contains(&Keyword::DoubleStrike));
}

/// Advanced Reconstruction level 2 deals 2 to each opponent when cards leave
/// your graveyard.
#[test]
fn advanced_reconstruction_level_two_burns() {
    let mut g = pod(3);
    let ar = g.add_card_to_hand(0, catalog::advanced_reconstruction());
    cast(&mut g, 0, ar, None).expect("cast");
    let ar = named(&g, 0, "Advanced Reconstruction")[0];
    activate(&mut g, 0, ar, 0, None).expect("level 2");
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    regrow(&mut g, bear);
    assert_eq!((g.players[1].life, g.players[2].life), (l1 - 2, l2 - 2));
}

/// Balefire Liege — +1/+1 per color to your other creatures; red spells burn
/// for 3, white spells gain 3.
#[test]
fn balefire_liege_pumps_and_triggers() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::balefire_liege());
    let spirit = g.add_card_to_battlefield(0, catalog::relic_retriever());
    assert_eq!(pt(&g, spirit), (3, 2));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let l = g.players[1].life;
    cast(&mut g, 0, bolt, Some(Target::Player(1))).expect("bolt");
    assert_eq!(g.players[1].life, l - 3 - 3, "Bolt plus the Liege's 3");
}

/// Ceaseless Conflict destroys everything and makes a Spirit per nontoken
/// creature of yours it destroyed.
#[test]
fn ceaseless_conflict_refunds_spirits() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::hill_giant());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cc = g.add_card_to_hand(0, catalog::ceaseless_conflict());
    cast(&mut g, 0, cc, None).expect("cast");
    assert!(named(&g, 1, "Grizzly Bears").is_empty());
    assert_eq!(named(&g, 0, "Spirit").len(), 2);
}

/// Augusta — each player exiles a card from their graveyard; an attacker gets
/// a counter per nonland card exiled.
#[test]
fn augusta_grows_an_attacker() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::augusta_order_returned());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.clear_sickness(a);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: a, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(g.players.iter().all(|p| p.graveyard.is_empty()));
}

/// Excava returns a small card from your graveyard as a 1/1 flying Spirit
/// with a finality counter.
#[test]
fn excava_returns_a_finality_spirit() {
    let mut g = pod(2);
    let ex = g.add_card_to_battlefield(0, catalog::excava_the_risen_past());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: ex, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let giant = named(&g, 0, "Grizzly Bears");
    assert_eq!(giant.len(), 1);
    let c = g.battlefield_find(giant[0]).unwrap();
    assert_eq!(c.counter_count(CounterType::Finality), 1);
    assert_eq!(pt(&g, giant[0]), (1, 1));
    assert!(g.computed_permanent(giant[0]).unwrap().keywords().contains(&Keyword::Flying));
}

/// Drumbellower untaps your creatures in another player's untap step.
#[test]
fn drumbellower_untaps_on_other_turns() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::drumbellower());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().tapped = true;
    g.active_player_idx = 1;
    g.do_untap();
    assert!(!g.battlefield_find(bear).unwrap().tapped);
}

/// Guardian of Faith phases out your other creatures (CR 702.26).
#[test]
fn guardian_of_faith_phases_out_the_team() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gf = g.add_card_to_hand(0, catalog::guardian_of_faith());
    cast(&mut g, 0, gf, None).expect("cast");
    assert!(g.battlefield_find(bear).is_none() && g.phased_out.iter().any(|c| c.id == bear));
}

/// Relic Retriever makes a Treasure at an end step after a card left your
/// graveyard.
#[test]
fn relic_retriever_treasures_on_departures() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::relic_retriever());
    step(&mut g, TurnStep::End);
    assert!(named(&g, 0, "Treasure").is_empty());
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    regrow(&mut g, bear);
    step(&mut g, TurnStep::End);
    assert_eq!(named(&g, 0, "Treasure").len(), 1);
}

/// Vanguard of the Restless: Spirits get +1/+1 per commander cast (CR 903.8's
/// count), and a Spirit entering may pay {2}{W} to bring it back.
#[test]
fn vanguard_of_the_restless_scales_and_returns() {
    let mut g = pod(2);
    let v = g.add_card_to_battlefield(0, catalog::vanguard_of_the_restless());
    assert_eq!(pt(&g, v), (2, 2));
    let mut g = pod(2);
    let v = g.add_card_to_graveyard(0, catalog::vanguard_of_the_restless());
    let rr = g.add_card_to_hand(0, catalog::relic_retriever());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    cast(&mut g, 0, rr, None).expect("a Spirit enters");
    assert!(g.battlefield_find(v).is_some() || !named(&g, 0, "Vanguard of the Restless").is_empty(), "paid {{2}}{{W}}");
}

/// Mistveil Plains puts a graveyard card on the bottom with two white
/// permanents.
#[test]
fn mistveil_plains_tucks_a_card() {
    let mut g = pod(2);
    let mp = g.add_card_to_battlefield(0, catalog::mistveil_plains());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    assert!(activate(&mut g, 0, mp, 1, Some(Target::Permanent(bolt))).is_err(), "no white permanents");
    g.add_card_to_battlefield(0, catalog::drumbellower());
    g.add_card_to_battlefield(0, catalog::guardian_of_faith());
    let mp2 = g.add_card_to_battlefield(0, catalog::mistveil_plains());
    activate(&mut g, 0, mp2, 1, Some(Target::Permanent(bolt))).expect("tuck");
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(bolt));
}

/// Currency Converter: {2}, {T}: loot; a discarded card is exiled with it,
/// and {T} turns a nonland one into a 2/2 Rogue.
#[test]
fn currency_converter_loots_and_converts() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let cc = g.add_card_to_battlefield(0, catalog::currency_converter());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    activate(&mut g, 0, cc, 0, None).expect("loot");
    assert_eq!(g.exile.len(), 1, "the discarded card is exiled with it");
    g.battlefield.iter_mut().find(|c| c.id == cc).unwrap().tapped = false;
    activate(&mut g, 0, cc, 1, None).expect("convert");
    assert!(g.exile.is_empty());
    assert_eq!(named(&g, 0, "Rogue").len() + named(&g, 0, "Treasure").len(), 1);
}

/// Naktamun Lorespinner prepares when a player is down to one card; Wheel of
/// Fortune refills everyone.
#[test]
fn naktamun_lorespinner_prepares_wheel() {
    let mut g = pod(2);
    stock_libraries(&mut g, 20);
    let nl = g.add_card_to_battlefield(0, catalog::naktamun_lorespinner());
    step(&mut g, TurnStep::Upkeep);
    assert_eq!(g.battlefield_find(nl).unwrap().counter_count(CounterType::Prepared), 1, "empty hands");
    g.step = TurnStep::PreCombatMain;
    flood(&mut g, 0);
    g.perform_action(GameAction::CastPrepareSpell { creature_id: nl, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Wheel of Fortune");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 7);
    assert_eq!(g.players[1].hand.len(), 7);
}

/// Spirit of Resilience grows (and may become a copy) when a card leaves your
/// graveyard.
#[test]
fn spirit_of_resilience_grows() {
    let mut g = pod(2);
    let s = g.add_card_to_battlefield(0, catalog::spirit_of_resilience());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    regrow(&mut g, bolt);
    assert_eq!(g.battlefield_find(s).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Fateful Tempest's council's dilemma resolves for every voter (CR 701.38).
#[test]
fn fateful_tempest_resolves_the_votes() {
    let mut g = pod(3);
    stock_libraries(&mut g, 10);
    let ft = g.add_card_to_hand(0, catalog::fateful_tempest());
    let gy = g.players[0].graveyard.len();
    let ex = g.exile.len();
    cast(&mut g, 0, ft, None).expect("cast");
    let milled = g.players[0].graveyard.len() - gy - 1;
    let exiled = g.exile.len() - ex;
    assert_eq!(milled + exiled, 3, "one card per vote");
}

/// The Mountain Plains duals enter tapped.
#[test]
fn lorehold_duals_enter_tapped() {
    for f in [catalog::glittering_massif, catalog::sacred_peaks, catalog::turbulent_steppe] {
        let mut g = pod(2);
        let l = g.add_card_to_hand(0, f());
        g.perform_action(GameAction::PlayLand(l)).expect("land");
        assert!(g.battlefield_find(l).unwrap().tapped);
    }
}
