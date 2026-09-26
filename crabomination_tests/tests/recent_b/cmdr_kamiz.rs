//! Commander: the Obscura Operation precon (NCC, Kamiz,
//! `decks::cmdr_kamiz`).

use crabomination::card::{CardId, CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
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

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    act(g, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, id, targets, None)
}

fn cast_mode(g: &mut GameState, id: CardId, targets: &[Target], mode: usize) -> Result<(), String> {
    act(g, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: Some(mode),
        x_value: None,
    })
}

fn stock(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::grizzly_bears());
    }
}

fn attack(g: &mut GameState, attackers: &[CardId], defender: usize) {
    for a in attackers {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn connect(g: &mut GameState, attackers: &[CardId], defender: usize) {
    attack(g, attackers, defender);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = defender;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(g);
    for _ in 0..6 {
        if g.step == TurnStep::PostCombatMain {
            break;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn has(g: &GameState, id: CardId, k: &Keyword) -> bool {
    g.computed_permanent(id).expect("on the battlefield").keywords().contains(k)
}

/// Archon of Coronation: the monarch takes damage but loses no life
/// (CR 120.3a).
#[test]
fn archon_of_coronation_shields_the_monarch() {
    let mut g = main_phase(2);
    let a = g.add_card_to_hand(0, catalog::archon_of_coronation());
    cast_at(&mut g, a, &[]).expect("cast");
    assert_eq!(g.monarch, Some(0));
    let life = g.players[0].life;
    let b = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, b, &[Target::Player(0)]).expect("bolt yourself");
    assert_eq!(g.players[0].life, life);
}

/// Cephalid Facetaker copies another creature as a 1/4 unblockable.
#[test]
fn cephalid_facetaker_takes_a_face() {
    let mut g = main_phase(2);
    let c = g.add_card_to_battlefield(0, catalog::cephalid_facetaker());
    g.add_card_to_battlefield(1, catalog::serra_angel());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    for _ in 0..6 {
        if g.step == TurnStep::BeginCombat {
            break;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    let cp = g.computed_permanent(c).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 4));
    assert!(has(&g, c, &Keyword::Flying) && has(&g, c, &Keyword::Unblockable));
}

/// Change of Plans: two creatures each connive on their own discard.
#[test]
fn change_of_plans_connives_each() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.add_card_to_hand(0, catalog::change_of_plans());
    cast_x(&mut g, c, &[Target::Permanent(a), Target::Permanent(b)], Some(2)).expect("cast");
    let counters = |id| g.find_card_anywhere(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne));
    assert_eq!(counters(a) + counters(b), 1, "one nonland discard between them, not two");
}

/// Commit tucks a permanent second from the top; Memory resets both hands.
#[test]
fn commit_memory_tucks_and_resets() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    stock(&mut g, 1, 9);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let c = g.add_card_to_hand(0, catalog::commit_memory());
    act(&mut g, GameAction::CastSpell {
        card_id: c, target: Some(Target::Permanent(ring)), additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("Commit, the permanent mode");
    assert_eq!(g.players[1].library.get(1).map(|c| c.id), Some(ring));
    stock(&mut g, 0, 8);
    act(&mut g, GameAction::CastAftermath { card_id: c, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Memory");
    assert_eq!(g.players[1].hand.len(), 7);
}

/// Daring Saboteur loots on connecting.
#[test]
fn daring_saboteur_loots() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    let d = g.add_card_to_battlefield(0, catalog::daring_saboteur());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    connect(&mut g, &[d], 1);
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Dragonlord Ojutai has hexproof only while untapped.
#[test]
fn dragonlord_ojutai_hides_while_untapped() {
    let mut g = main_phase(2);
    let o = g.add_card_to_battlefield(0, catalog::dragonlord_ojutai());
    assert!(has(&g, o, &Keyword::Hexproof));
    g.battlefield_find_mut(o).unwrap().tapped = true;
    assert!(!has(&g, o, &Keyword::Hexproof));
}

/// Graveblade Marauder drains by your graveyard's creature cards.
#[test]
fn graveblade_marauder_drains_by_graveyard() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let m = g.add_card_to_battlefield(0, catalog::graveblade_marauder());
    let life = g.players[1].life;
    connect(&mut g, &[m], 1);
    assert_eq!(g.players[1].life, life - 1 - 3);
}

/// Identity Thief exiles another creature and becomes a copy of it.
#[test]
fn identity_thief_steals_an_identity() {
    let mut g = main_phase(2);
    let t = g.add_card_to_battlefield(0, catalog::identity_thief());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, &[t], 1);
    assert!(g.exile.iter().any(|c| c.id == angel));
    assert_eq!(g.computed_permanent(t).unwrap().power, 4);
}

/// In Too Deep turns a creature into a colorless Clue.
#[test]
fn in_too_deep_makes_a_clue() {
    let mut g = main_phase(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let i = g.add_card_to_hand(0, catalog::in_too_deep());
    cast_at(&mut g, i, &[Target::Permanent(angel)]).expect("cast");
    let cp = g.computed_permanent(angel).unwrap();
    assert!(!cp.card_types().contains(&CardType::Creature));
    assert!(cp.card_types().contains(&CardType::Artifact));
    assert!(!cp.keywords().contains(&Keyword::Flying));
}

/// Jailbreak frees an opponent's permanent and one of yours no bigger.
#[test]
fn jailbreak_frees_both_sides() {
    let mut g = main_phase(2);
    let theirs = g.add_card_to_graveyard(1, catalog::serra_angel());
    let mine = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let j = g.add_card_to_hand(0, catalog::jailbreak());
    cast_at(&mut g, j, &[Target::Permanent(theirs)]).expect("cast");
    assert!(g.battlefield_find(theirs).is_some_and(|c| c.controller == 1));
    assert!(g.battlefield_find(mine).is_some_and(|c| c.controller == 0));
}

/// Kamiz: an attacker goes unblockable and connives; a smaller one gets
/// double strike.
#[test]
fn kamiz_runs_the_operation() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    let k = g.add_card_to_battlefield(0, catalog::kamiz_obscura_oculus());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let lib = g.players[0].library.len();
    attack(&mut g, &[k, bear], 1);
    assert_eq!(g.players[0].library.len(), lib - 1, "the target connived");
    let unblockable: Vec<CardId> = [k, bear].into_iter().filter(|&id| has(&g, id, &Keyword::Unblockable)).collect();
    assert_eq!(unblockable.len(), 1);
    let other = if unblockable[0] == k { bear } else { k };
    let pw = |id| g.computed_permanent(id).unwrap().power;
    assert_eq!(has(&g, other, &Keyword::DoubleStrike), pw(other) < pw(unblockable[0]));
}

/// Mask of Riddles gives fear; Mask of the Schemer connives by the damage.
#[test]
fn masks_equip_their_riddles() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 5);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let home = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let riddles = g.add_card_to_battlefield(0, catalog::mask_of_riddles());
    act(&mut g, GameAction::Equip { equipment: riddles, target: home }).expect("equip");
    assert!(has(&g, home, &Keyword::Fear));
    let schemer = g.add_card_to_battlefield(0, catalog::mask_of_the_schemer());
    act(&mut g, GameAction::Equip { equipment: schemer, target: bear }).expect("equip");
    let lib = g.players[0].library.len();
    connect(&mut g, &[bear], 1);
    assert_eq!(g.players[0].library.len(), lib - 2, "connives 2 for the 2 damage");
}

/// Obscura Charm's removal mode kills a small creature.
#[test]
fn obscura_charm_kills_small() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let c = g.add_card_to_hand(0, catalog::obscura_charm());
    cast_mode(&mut g, c, &[Target::Permanent(bear)], 2).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
}

/// Oskar is cheaper per distinct mana value in your graveyard, and a
/// discarded nonland card may be cast from there.
#[test]
fn oskar_reclaims_the_rubbish() {
    let mut g = main_phase(2);
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let o = g.add_card_to_hand(0, catalog::oskar_rubbish_reclaimer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: o, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{3}{U}{B} less 2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(o).is_some());
}

/// Tivit's council's dilemma pays out a Clue or a Treasure per vote.
#[test]
fn tivit_sells_secrets() {
    let mut g = main_phase(2);
    let t = g.add_card_to_hand(0, catalog::tivit_seller_of_secrets());
    cast_at(&mut g, t, &[]).expect("cast");
    let payouts = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.is_token && (c.definition.name == "Clue" || c.definition.name == "Treasure"))
        .count();
    assert_eq!(payouts, 3, "your two votes and the opponent's one");
}

/// Writ of Return reanimates tapped, then ciphers.
#[test]
fn writ_of_return_reanimates() {
    let mut g = main_phase(2);
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    let w = g.add_card_to_hand(0, catalog::writ_of_return());
    cast_at(&mut g, w, &[Target::Permanent(angel)]).expect("cast");
    assert!(g.battlefield_find(angel).is_some_and(|c| c.tapped));
}

/// Obscura Confluence: a connive and a 1/1 on the default picks.
#[test]
fn obscura_confluence_resolves_its_picks() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let c = g.add_card_to_hand(0, catalog::obscura_confluence());
    cast_at(&mut g, c, &[Target::Permanent(angel), Target::Permanent(bear)]).expect("default picks");
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(!cp.keywords().contains(&Keyword::Flying));
    assert_eq!(g.players[0].hand.len(), 1, "the creature card came back");
}

/// Skyway Robber escapes, exiling five other cards with it.
#[test]
fn skyway_robber_escapes() {
    let mut g = main_phase(2);
    let fodder: Vec<CardId> = (0..5).map(|_| g.add_card_to_graveyard(0, catalog::lightning_bolt())).collect();
    let s = g.add_card_to_graveyard(0, catalog::skyway_robber());
    act(&mut g, GameAction::CastEscape {
        card_id: s,
        exile_cards: fodder,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("escape");
    assert!(g.battlefield_find(s).is_some());
    assert_eq!(g.exile.iter().filter(|c| c.exiled_with == Some(s)).count(), 5);
}
