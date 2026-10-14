//! Commander: three-card gaps of EDHREC ranks 4501-5500, second batch (`decks::cmdr_rank5k_d`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x });
}

fn keywords(g: &GameState, id: CardId) -> Vec<Keyword> {
    g.computed_permanent(id).unwrap().keywords().to_vec()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

/// Triumphant Reckoning returns every artifact, enchantment and planeswalker
/// card from the graveyard, and leaves creatures there.
#[test]
fn triumphant_reckoning_returns_noncreature_permanents() {
    let mut g = pod(2);
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let verse = g.add_card_to_graveyard(0, catalog::spider_verse());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::triumphant_reckoning());
    cast(&mut g, id, None, None);
    assert!(g.battlefield_find(ring).is_some());
    assert!(g.battlefield_find(verse).is_some());
    assert!(g.battlefield_find(bear).is_none());
}

/// Blue Marvel grows on the second card drawn in a turn.
#[test]
fn blue_marvel_grows_on_the_second_draw() {
    let mut g = pod(2);
    let bm = ready(&mut g, 0, catalog::blue_marvel_adam_brashear());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::cast_away_doubt());
    cast(&mut g, id, None, None);
    assert_eq!(g.battlefield_find(bm).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Leonardo grows whenever you gain life — here off his own lifelink.
#[test]
fn leonardo_grows_on_life_gain() {
    let mut g = pod(2);
    let leo = ready(&mut g, 0, catalog::leonardo_cutting_edge());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: leo,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(leo).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// The Wondrous Wasp: the tapped creature has no abilities only while the Wasp stays.
#[test]
fn the_wondrous_wasp_strips_while_it_remains() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let wasp = g.add_card_to_hand(0, catalog::the_wondrous_wasp());
    cast(&mut g, wasp, None, None);
    assert!(g.battlefield_find(angel).unwrap().tapped);
    assert!(!keywords(&g, angel).contains(&Keyword::Flying));
    let evs = g.remove_from_battlefield_to_hand(wasp);
    g.dispatch_triggers_for_events(&evs);
    assert!(keywords(&g, angel).contains(&Keyword::Flying), "back once the Wasp leaves");
}

/// Exhaustion: the opponent's creatures and lands stay tapped through their next untap.
#[test]
fn exhaustion_skips_the_opponents_untap() {
    let mut g = pod(2);
    let land = ready(&mut g, 1, catalog::forest());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    for id in [land, bear] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    let id = g.add_card_to_hand(0, catalog::exhaustion());
    cast(&mut g, id, Some(Target::Player(1)), None);
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(land).unwrap().tapped && g.battlefield_find(bear).unwrap().tapped);
}

/// Pang Tong's ability only works on its controller's turn before attacks.
#[test]
fn pang_tong_only_before_attackers_on_your_turn() {
    let mut g = pod(2);
    let pang = ready(&mut g, 0, catalog::pang_tong_young_phoenix());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::PostCombatMain;
    let a = GameAction::ActivateAbility {
        card_id: pang,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    };
    assert!(g.perform_action(a.clone()).is_err(), "after combat");
    g.step = TurnStep::PreCombatMain;
    act(&mut g, a);
    assert_eq!(pt(&g, bear), (2, 4));
}

/// The Princess Takes Flight: chapter III returns what chapter I exiled.
#[test]
fn the_princess_takes_flight_returns_its_exile() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let saga = g.add_card_to_hand(0, catalog::the_princess_takes_flight());
    cast(&mut g, saga, None, None);
    assert!(g.battlefield_find(bear).is_none(), "chapter I exiled it");
    // Two more precombat mains: chapters II and III.
    for _ in 0..2 {
        g.step = TurnStep::Draw;
        while g.step != TurnStep::PreCombatMain {
            let evs = g.advance_step(Vec::new()).unwrap();
            g.dispatch_triggers_for_events(&evs);
            drain_stack(&mut g);
        }
    }
    assert!(g.battlefield_find(bear).is_some_and(|c| c.controller == 1), "chapter III returned it");
}
