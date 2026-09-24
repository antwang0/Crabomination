//! Commander: the Symbiotic Swarm precon (C20, Kathril, `decks::cmdr_kathril`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn cast_by(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    cast_by(g, 0, id, targets).expect("cast");
}

fn has(g: &GameState, id: CardId, kw: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.keywords().contains(&kw))
}

/// Kathril: a keyword counter (CR 122.1b) per listed keyword in your
/// graveyard's creature cards, and a +1/+1 counter per counter placed.
#[test]
fn kathril_harvests_graveyard_keywords() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::serra_angel()); // flying, vigilance
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let k = g.add_card_to_hand(0, catalog::kathril_aspect_warper());
    cast(&mut g, k, &[]);
    assert_eq!(g.battlefield_find(k).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    let flying = [bear, k].into_iter().filter(|&id| has(&g, id, Keyword::Flying)).count();
    let vigilance = [bear, k].into_iter().filter(|&id| has(&g, id, Keyword::Vigilance)).count();
    assert_eq!((flying, vigilance), (1, 1));
}

/// Cairn Wanderer has a keyword while a creature card in any graveyard does.
#[test]
fn cairn_wanderer_borrows_from_graveyards() {
    let mut g = pod(2);
    let cw = g.add_card_to_battlefield(0, catalog::cairn_wanderer());
    assert!(!has(&g, cw, Keyword::Flying));
    g.add_card_to_graveyard(1, catalog::serra_angel());
    assert!(has(&g, cw, Keyword::Flying));
    assert!(has(&g, cw, Keyword::Vigilance));
}

/// Soulflayer: a delved creature card's keywords (CR 702.66).
#[test]
fn soulflayer_keeps_what_it_delved() {
    let mut g = pod(2);
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    let s = g.add_card_to_hand(0, catalog::soulflayer());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellDelve {
        card_id: s,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        delve_cards: vec![angel],
    })
    .expect("delve");
    drain_stack(&mut g);
    assert!(has(&g, s, Keyword::Flying));
    assert!(!has(&g, s, Keyword::Trample));
}

/// Archon of Valor's Reach: nobody casts the chosen type (CR 601.2 — the
/// cast is illegal).
#[test]
fn archon_bars_the_chosen_type() {
    let mut g = pod(2);
    let a = g.add_card_to_hand(0, catalog::archon_of_valors_reach());
    cast(&mut g, a, &[]);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    assert!(cast_by(&mut g, 1, bolt, &[Target::Player(0)]).is_err(), "instant is the default pick");
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    assert!(cast_by(&mut g, 1, bear, &[]).is_ok());
}

/// Nikara draws when another creature of yours leaves with counters.
#[test]
fn nikara_draws_on_counters_leaving() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::nikara_lair_scavenger());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(plain)]);
    assert_eq!(g.players[0].hand.len(), 0);
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(bear)]);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Selective Adaptation: one keyworded card per keyword; one to the
/// battlefield, the rest chosen to hand, everything else to the graveyard.
#[test]
fn selective_adaptation_sorts_by_keyword() {
    let mut g = pod(2);
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::selective_adaptation());
    cast(&mut g, s, &[]);
    assert!(g.battlefield_find(angel).is_some());
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
}

/// Majestic Myriarch counts double and borrows keywords at each combat.
#[test]
fn majestic_myriarch_grows_and_borrows() {
    let mut g = pod(2);
    let mm = g.add_card_to_battlefield(0, catalog::majestic_myriarch());
    g.add_card_to_battlefield(0, catalog::serra_angel());
    assert_eq!(g.computed_permanent(mm).map(|c| (c.power, c.toughness)), Some((4, 4)));
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(has(&g, mm, Keyword::Flying));
    assert!(has(&g, mm, Keyword::Vigilance));
}

/// Slippery Bogbonder gathers your creatures' counters onto its target.
#[test]
fn slippery_bogbonder_gathers_counters() {
    let mut g = pod(2);
    let donor = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(donor).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let host = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let sb = g.add_card_to_hand(0, catalog::slippery_bogbonder());
    cast(&mut g, sb, &[Target::Permanent(host)]);
    // The ETB picks its own target; every +1/+1 counter ends up on it.
    let t = [donor, host, sb]
        .into_iter()
        .find(|&id| g.battlefield_find(id).is_some_and(|c| !c.keyword_counters.is_empty()))
        .expect("a hexproof counter landed");
    assert!(has(&g, t, Keyword::Hexproof));
    assert_eq!(g.battlefield_find(t).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Netherborn Altar returns your commander to hand for 3 life a soul counter
/// (CR 903.8 — a card in hand is cast normally, tax included).
#[test]
fn netherborn_altar_fetches_the_commander() {
    let mut g = pod(2);
    g.seat_commanders(0, vec![catalog::kathril_aspect_warper()]);
    let na = g.add_card_to_battlefield(0, catalog::netherborn_altar());
    g.perform_action(GameAction::ActivateAbility {
        card_id: na,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("altar");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Kathril, Aspect Warper"));
    assert_eq!(g.players[0].starting_life - g.players[0].life, 3);
}

/// Abzan Ascendancy counters the team and leaves Spirits; Ever After brings
/// one back and goes to the bottom of the library.
#[test]
fn abzan_ascendancy_and_ever_after() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aa = g.add_card_to_hand(0, catalog::abzan_ascendancy());
    cast(&mut g, aa, &[]);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(bear)]);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Spirit"));
    let ea = g.add_card_to_hand(0, catalog::ever_after());
    cast(&mut g, ea, &[Target::Permanent(bear)]);
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(ea), "bottom of its owner's library");
}
