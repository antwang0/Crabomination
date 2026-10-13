//! Commander: The Fourteenth Doctor's last gaps (`decks::cmdr_edhrec51`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
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

fn cast(g: &mut GameState, id: CardId) -> Result<(), String> {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// The Fourteenth Doctor bins the Doctors among the top fourteen as it's cast,
/// then enters as a copy of one with haste.
#[test]
fn the_fourteenth_doctor_regenerates_from_the_library() {
    let mut g = pod(2);
    g.players[0].library.clear();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let fifteenth = g.add_card_to_library(0, catalog::the_fifteenth_doctor());
    let fourteenth = g.add_card_to_hand(0, catalog::the_fourteenth_doctor());
    cast(&mut g, fourteenth).expect("cast");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fifteenth), "the Doctor was binned");
    let cp = g.computed_permanent(fourteenth).expect("on the battlefield");
    assert_eq!(cp.def.name, "The Fifteenth Doctor");
    assert!(cp.keywords().contains(&Keyword::Haste));
}

/// Rose Noble draws off a Doctor spell; the Fifteenth Doctor's mill finds a
/// two-mana artifact.
#[test]
fn rose_noble_and_the_fifteenth_doctor() {
    let mut g = pod(2);
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::jhoiras_timebug());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.add_card_to_battlefield(0, catalog::rose_noble());
    g.players[0].hand.clear();
    let doc = g.add_card_to_hand(0, catalog::the_fifteenth_doctor());
    cast(&mut g, doc).expect("doctor");
    let names: Vec<_> = g.players[0].hand.iter().map(|c| c.definition.name).collect();
    assert!(names.contains(&"Jhoira's Timebug"), "the milled artifact: {names:?}");
    assert_eq!(names.len(), 2, "Rose Noble's draw and the Timebug");
}
