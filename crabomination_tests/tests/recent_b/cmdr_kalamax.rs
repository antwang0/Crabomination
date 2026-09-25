//! Commander: the Arcane Maelstrom precon (C20, Kalamax, `decks::cmdr_kalamax`).

use crabomination::card::{CardId, CounterType};
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

fn cast_by(g: &mut GameState, seat: usize, id: CardId, targets: &[Target], x: Option<u32>) {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    cast_by(g, 0, id, targets, None);
}

fn lost(g: &GameState, s: usize) -> i32 {
    g.players[s].starting_life - g.players[s].life
}

/// Kalamax, tapped, copies the first instant each turn (CR 707.10), and the
/// copy grows it; Twinning Staff adds one more copy.
#[test]
fn kalamax_copies_and_grows() {
    let mut g = pod(2);
    let k = g.add_card_to_battlefield(0, catalog::kalamax_the_stormsire());
    g.battlefield_find_mut(k).unwrap().tapped = true;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(1)]);
    assert_eq!(lost(&g, 1), 6, "the bolt and its copy");
    assert_eq!(g.battlefield_find(k).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let mut g = pod(2);
    let k = g.add_card_to_battlefield(0, catalog::kalamax_the_stormsire());
    g.battlefield_find_mut(k).unwrap().tapped = true;
    g.add_card_to_battlefield(0, catalog::twinning_staff());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(1)]);
    assert_eq!(lost(&g, 1), 9, "the bolt and two copies");
}

/// Whiplash Trap's {U} alternative cost needs an opponent's two creature
/// entries this turn.
#[test]
fn whiplash_trap_alt_cost_condition() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wt = g.add_card_to_hand(0, catalog::whiplash_trap());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let alt = |g: &mut GameState| {
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: wt,
            pitch_card: None,
            target: Some(Target::Permanent(a)),
            additional_targets: vec![Target::Permanent(b)],
            mode: None,
            x_value: None,
        })
    };
    assert!(alt(&mut g).is_err(), "nothing entered this turn");
    g.players[1].creatures_entered_this_turn.push(a);
    g.players[1].creatures_entered_this_turn.push(b);
    alt(&mut g).expect("alt cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Rashmi: the first spell each turn reveals the top; a cheaper spell is cast
/// free (CR 601.2), a land goes to hand.
#[test]
fn rashmi_casts_the_cheaper_top() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::rashmi_eternities_crafter());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    cast(&mut g, wurm, &[]);
    assert!(g.battlefield_find(bear).is_some() || g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Nascent Metamorph attacking becomes a copy of the opponent's first
/// creature card from the top.
#[test]
fn nascent_metamorph_shifts() {
    let mut g = pod(2);
    let nm = g.add_card_to_battlefield(0, catalog::nascent_metamorph());
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::craw_wurm());
    g.clear_sickness(nm);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: nm,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(nm).map(|c| (c.power, c.toughness)), Some((6, 4)));
}

/// Glademuse draws a caster a card off their own turn.
#[test]
fn glademuse_rewards_off_turn_spells() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::glademuse());
    g.add_card_to_library(1, catalog::island());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_by(&mut g, 1, bolt, &[Target::Player(0)], None);
    assert_eq!(g.players[1].hand.len(), 1);
}

/// Curious Herd counts the target opponent's artifacts.
#[test]
fn curious_herd_counts_artifacts() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_battlefield(1, catalog::ornithopter());
    }
    let ch = g.add_card_to_hand(0, catalog::curious_herd());
    cast(&mut g, ch, &[Target::Player(1)]);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Beast").count(), 2);
}

/// Wort grants conspire (CR 702.78): a red instant copies off two tapped
/// red creatures.
#[test]
fn wort_grants_conspire() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::wort_the_raidmother());
    let goblins: Vec<CardId> = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Goblin Warrior")
        .map(|c| c.id)
        .collect();
    let (a, b) = if goblins.len() >= 2 {
        (goblins[0], goblins[1])
    } else {
        (
            g.add_card_to_battlefield(0, catalog::goblin_guide()),
            g.add_card_to_battlefield(0, catalog::goblin_guide()),
        )
    };
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellConspire {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
        conspire_creatures: [a, b],
    })
    .expect("conspire");
    drain_stack(&mut g);
    assert_eq!(lost(&g, 1), 6);
}
