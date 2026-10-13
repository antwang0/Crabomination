//! Commander: Gale's, Jhoira's and Cecily's last gaps (`decks::cmdr_edhrec44`).

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

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Gale: an instant from hand lets you cast a sorcery from your graveyard,
/// which is exiled as it resolves.
#[test]
fn gale_casts_the_other_type_from_the_graveyard() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    g.add_card_to_battlefield(0, catalog::gale_waterdeep_prodigy());
    let div = g.add_card_to_graveyard(0, catalog::divination());
    let shock = g.add_card_to_hand(0, catalog::shock());
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, shock, Some(Target::Player(1))).expect("shock");
    assert!(g.exile.iter().any(|c| c.id == div), "the flashed-back Divination is exiled");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2);
}

/// CR 903 — Scion of Halaster: while you control your commander, the turn's
/// first draw looks at two, bins one, then draws.
#[test]
fn cr_903_scion_of_halaster_sculpts_the_first_draw() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let cmd = g.seat_commanders(0, vec![catalog::gale_waterdeep_prodigy()])[0];
    g.add_card_to_battlefield(0, catalog::scion_of_halaster());
    let gy = g.players[0].graveyard.len();
    let mut events = Vec::new();
    g.draw_one(0, &mut events);
    assert_eq!(g.players[0].graveyard.len(), gy, "no commander on the battlefield: a plain draw");
    let pos = g.players[0].command.iter().position(|c| c.id == cmd).unwrap();
    let gale = g.players[0].command.remove(pos);
    g.battlefield.push(gale);
    g.players[0].cards_drawn_this_turn = 0;
    let hand = g.players[0].hand.len();
    g.draw_one(0, &mut events);
    g.draw_one(0, &mut events);
    assert_eq!(g.players[0].graveyard.len(), gy + 1, "one of the top two binned, first draw only");
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// CR 903.8 — Font of Magic discounts instants and sorceries by the times a
/// commander was cast from the command zone.
#[test]
fn cr_903_8_font_of_magic_counts_commander_casts() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let cmd = g.seat_commanders(0, vec![catalog::gale_waterdeep_prodigy()])[0];
    g.add_card_to_battlefield(0, catalog::font_of_magic());
    g.commander_cast_count.insert(cmd, 2);
    let div = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: div, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{2}{U} less {2}");
}

/// Jhoira: each tap banks two ingenuity counters, then drops an artifact with
/// mana value up to the count.
#[test]
fn jhoira_cheats_artifacts_by_ingenuity() {
    let mut g = pod(2);
    let jhoira = g.add_card_to_battlefield(0, catalog::jhoira_ageless_innovator());
    g.clear_sickness(jhoira);
    let mox = g.add_card_to_hand(0, catalog::sewer_veillance_cam());
    activate(&mut g, jhoira, 0, None).expect("tap");
    assert_eq!(g.battlefield_find(jhoira).unwrap().counter_count(CounterType::Ingenuity), 2);
    assert!(g.battlefield_find(mox).is_some());
}

/// Sewer-veillance Cam taps a creature as it enters; Forging the Anchor keeps
/// every artifact among the top five.
#[test]
fn cam_taps_and_the_anchor_keeps_artifacts() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(1)]));
    let cam = g.add_card_to_hand(0, catalog::sewer_veillance_cam());
    cast(&mut g, cam, None).expect("cam");
    assert!(g.battlefield_find(bear).unwrap().tapped);
    for def in [catalog::sol_ring(), catalog::forest(), catalog::sol_ring(), catalog::forest(), catalog::forest()] {
        g.add_card_to_library(0, def);
    }
    let hand = g.players[0].hand.len();
    let anchor = g.add_card_to_hand(0, catalog::forging_the_anchor());
    cast(&mut g, anchor, None).expect("anchor");
    assert_eq!(g.players[0].hand.len(), hand + 2, "both Sol Rings");
}

/// Cecily attacking draws and drains her controller; with eleven cards in hand
/// she casts an instant or sorcery free.
#[test]
fn cecily_casts_free_at_eleven_cards() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let cecily = g.add_card_to_battlefield(0, catalog::cecily_haunted_mage());
    g.clear_sickness(cecily);
    g.players[0].hand.clear();
    g.add_card_to_hand(0, catalog::lightning_bolt());
    for _ in 0..9 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let life = (g.players[0].life, g.players[1].life);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: cecily, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life.0 - 1);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"), "cast free");
    assert_eq!(g.players[1].life, life.1 - 3);
}

/// Othelm returns a creature that died this turn, tapped.
#[test]
fn othelm_undoes_a_death() {
    let mut g = pod(2);
    let othelm = g.add_card_to_battlefield(0, catalog::othelm_sigardian_outcast());
    g.clear_sickness(othelm);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, Some(Target::Permanent(bear))).expect("shock");
    activate(&mut g, othelm, 0, Some(Target::Permanent(bear))).expect("return");
    let b = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("back");
    assert!(b.tapped);
}
