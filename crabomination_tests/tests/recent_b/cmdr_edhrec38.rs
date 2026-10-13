//! Commander: Jared Carthalion's and Cleopatra's last gaps
//! (`decks::cmdr_edhrec38`).

use crabomination::card::{CardId, CounterType};
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn p1p1(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

/// CR 725 — Jared hands an opponent the monarchy and you can't take it back
/// this turn, even by a Court entering; while you're the monarch, damage to
/// Jared becomes +1/+1 counters.
#[test]
fn cr_725_jared_gives_the_crown_away_and_grows_while_wearing_it() {
    let mut g = pod(3);
    let jared = g.add_card_to_hand(0, catalog::jared_carthalion_true_heir());
    cast(&mut g, jared, None).expect("jared");
    let monarch = g.monarch.expect("someone is the monarch");
    assert_ne!(monarch, 0);
    let court = g.add_card_to_hand(0, catalog::court_of_bounty());
    cast(&mut g, court, None).expect("court");
    assert_eq!(g.monarch, Some(monarch), "you can't become the monarch this turn");
    // Next turn, wearing the crown: shocks become counters.
    g.monarch = Some(0);
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, Some(Target::Permanent(jared))).expect("shock");
    assert!(g.battlefield_find(jared).is_some());
    assert_eq!(p1p1(&g, jared), 2, "two damage became two counters");
}

/// Court of Garenbrig distributes two counters and, as the monarch, doubles
/// them.
#[test]
fn court_of_garenbrig_doubles_for_the_monarch() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let court = g.add_card_to_hand(0, catalog::court_of_garenbrig());
    cast(&mut g, court, None).expect("court");
    assert_eq!(g.monarch, Some(0));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(p1p1(&g, bear), 4, "two counters, doubled");
}

/// CR 603.10 — Cleopatra's Betrayal draws per counter on a dying legendary
/// creature, and Grakmaw's own death trigger still sees its three counters:
/// the first of the two to resolve used to tear the shared snapshot down.
#[test]
fn cleopatra_draws_off_grakmaws_counters() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    g.add_card_to_battlefield(0, catalog::cleopatra_exiled_pharaoh());
    let grakmaw = g.add_card_to_hand(0, catalog::grakmaw_skyclave_ravager());
    cast(&mut g, grakmaw, None).expect("grakmaw");
    assert_eq!(p1p1(&g, grakmaw), 3);
    let hand = g.players[0].hand.len();
    let life = g.players[0].life;
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(grakmaw))).expect("murder");
    assert_eq!(g.players[0].hand.len(), hand + 3, "three counters, three cards");
    assert_eq!(g.players[0].life, life - 2);
    let hydra = g.battlefield.iter().find(|c| c.definition.name == "Hydra").map(|c| c.id).expect("hydra");
    let cp = g.computed_permanent(hydra).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}
