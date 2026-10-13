//! Commander: Mister Fantastic's last gaps (`decks::cmdr_edhrec52`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn power_up(g: &mut GameState, id: CardId) -> Result<(), String> {
    act(g, GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
}

fn p1p1(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

/// CR 702.193a/b — power-up costs {6}{R} less {2}{R} the turn Human Torch
/// entered ({4}), and activates only once.
#[test]
fn cr_702_193_power_up_is_cheaper_the_turn_it_enters_and_only_once() {
    let mut g = pod(2);
    let torch = g.add_card_to_hand(0, catalog::human_torch_johnny_storm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    act(&mut g, GameAction::CastSpell { card_id: torch, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("torch");
    g.players[0].mana_pool.add_colorless(4);
    power_up(&mut g, torch).expect("four generic is enough the turn it entered");
    assert_eq!(p1p1(&g, torch), 3);
    g.players[0].mana_pool.add(Color::Red, 7);
    assert!(power_up(&mut g, torch).is_err(), "only once");
}

/// CR 702.193a — a turn later the full {6}{R} is due.
#[test]
fn cr_702_193_power_up_costs_full_on_a_later_turn() {
    let mut g = pod(2);
    let torch = g.add_card_to_battlefield(0, catalog::human_torch_johnny_storm());
    g.battlefield_find_mut(torch).unwrap().entered_turn = Some(g.turn_number.saturating_sub(1));
    g.players[0].mana_pool.add_colorless(4);
    assert!(power_up(&mut g, torch).is_err(), "four generic isn't enough");
    g.players[0].mana_pool.add(Color::Red, 3);
    power_up(&mut g, torch).expect("the full cost");
    assert_eq!(p1p1(&g, torch), 3);
}

/// Human Torch pings an opponent when you draw beside another Hero — a Hero
/// dealing damage to a player grows The Thing; counters on another Hero make
/// Invisible Woman a Wall.
#[test]
fn the_fantastic_four_feed_each_other() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    g.add_card_to_battlefield(0, catalog::human_torch_johnny_storm());
    let thing = g.add_card_to_battlefield(0, catalog::the_thing_ben_grimm());
    g.add_card_to_battlefield(0, catalog::invisible_woman_sue_storm());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    let div = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    act(&mut g, GameAction::CastSpell { card_id: div, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("divination");
    assert_eq!(g.players[1].life, life - 2, "two draws, two pings");
    assert_eq!(p1p1(&g, thing), 4, "two Hero damage events");
    let walls = g.battlefield.iter().filter(|c| c.definition.name == "Wall").count();
    assert!(walls >= 1, "counters on The Thing made a Wall");
}
