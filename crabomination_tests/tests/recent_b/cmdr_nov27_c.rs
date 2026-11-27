//! Star Trek closers of the 2026-11-27 rescan: "face a dilemma" (CR 700.2),
//! Pick the Brain, Zero, Branching Timeline, the Klingons of Kor's deck.

use crabomination::card::{CardId, Keyword, MayPlayDuration};
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, mode: Option<usize>) {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn to_end_step(g: &mut GameState) {
    g.step = TurnStep::PostCombatMain;
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
    }
    drain_stack(g);
}

fn lives(g: &GameState) -> Vec<i32> {
    g.players.iter().map(|p| p.life).collect()
}

/// CR 700.2a — casting a modal spell faces a dilemma: Elim Garak drains each
/// opponent for 2 as Impossible Choice is cast; a non-modal spell doesn't.
#[test]
fn cr_700_2a_casting_a_modal_spell_faces_a_dilemma() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::elim_garak_simple_tailor());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Player(1)), None);
    assert_eq!(lives(&g), vec![20, 17, 20], "a bolt is not a dilemma");
    let choice = g.add_card_to_hand(0, catalog::impossible_choice());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.priority.player_with_priority = 0;
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    cast(&mut g, choice, None, Some(1));
    // Elim: +2 / -2 each opponent; Remain Fused: each player loses 2.
    assert_eq!(lives(&g), vec![20, 13, 16]);
    assert_eq!(g.players[0].hand.len(), 2, "Remain Fused drew two");
}

/// CR 700.2b — a modal triggered ability chooses its mode as it goes on the
/// stack: The Unicomplex entering untapped faces a dilemma (Obsidian Order
/// Operative drains 1).
#[test]
fn cr_700_2b_a_modal_trigger_faces_a_dilemma() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::obsidian_order_operative());
    g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_battlefield(0, catalog::swamp());
    let uni = g.add_card_to_hand(0, catalog::the_unicomplex());
    g.perform_action(GameAction::PlayLand(uni)).expect("land");
    drain_stack(&mut g);
    assert_eq!(lives(&g), vec![21, 19]);
}

/// Impossible Choice's Surgical Separation: the destroyed creature's
/// controller gets the two Officers (CR 608.2h — read off its LKI).
#[test]
fn impossible_choice_gives_the_victims_controller_two_officers() {
    let mut g = pod(3);
    let bear = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let choice = g.add_card_to_hand(0, catalog::impossible_choice());
    g.players[0].mana_pool.add(Color::Black, 3);
    cast(&mut g, choice, Some(Target::Permanent(bear)), Some(0));
    assert!(g.battlefield_find(bear).is_none());
    let officers = |s: usize| g.battlefield.iter().filter(|c| c.controller == s && c.definition.name == "Officer").count();
    assert_eq!((officers(0), officers(2)), (0, 2));
}

/// Pick the Brain: the chosen card alone; with delirium (four card types in
/// your graveyard) every card of that name in the opponent's graveyard, hand
/// and library goes too.
#[test]
fn pick_the_brain_takes_every_copy_under_delirium() {
    let run = |delirium: bool| {
        let mut g = pod(2);
        for _ in 0..2 {
            g.add_card_to_hand(1, catalog::grizzly_bears());
        }
        g.add_card_to_hand(1, catalog::island());
        g.add_card_to_library(1, catalog::grizzly_bears());
        g.add_card_to_library(1, catalog::island());
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
        if delirium {
            for def in [catalog::island(), catalog::hill_giant(), catalog::lightning_bolt(), catalog::sol_ring()] {
                g.add_card_to_graveyard(0, def);
            }
        }
        let pick = g.add_card_to_hand(0, catalog::pick_the_brain());
        g.players[0].mana_pool.add(Color::Black, 3);
        cast(&mut g, pick, Some(Target::Player(1)), None);
        let bears = |g: &GameState| g.exile.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
        (bears(&g), g.players[1].hand.len(), g.players[1].library.len(), g.players[1].graveyard.len())
    };
    assert_eq!(run(false), (1, 2, 2, 1));
    assert_eq!(run(true), (4, 1, 1, 0));
}

/// Zero (and Elite Spellbinder's shared effect): the exiled card stays
/// playable by its owner for as long as it remains exiled, for {2} more —
/// not only through their next turn (bug fix: the window used to lapse).
#[test]
fn zero_exiles_a_card_its_owner_may_play_while_it_stays_exiled() {
    let mut g = pod(2);
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    let zero = g.add_card_to_hand(0, catalog::zero_incorporeal_sensate());
    g.players[0].mana_pool.add(Color::White, 3);
    cast(&mut g, zero, None, None);
    let card = g.exile.iter().find(|c| c.id == bear).expect("exiled");
    let grant = card.may_play_until.expect("may play");
    assert_eq!((grant.player, grant.duration), (1, MayPlayDuration::WhileExiled));
    assert_eq!(card.granted_alt_cast_cost_eot.as_ref().map(|c| c.cmc()), Some(4), "{{1}}{{G}} + {{2}}");
    assert!(g.battlefield_find(zero).unwrap().definition.keywords.contains(&Keyword::Lifelink));
}

/// Branching Timeline: reveal X = differently named lands; every permanent
/// card enters tapped, the rest go to the bottom.
#[test]
fn branching_timeline_puts_every_revealed_permanent_in_tapped() {
    let mut g = pod(2);
    for land in [catalog::forest(), catalog::island(), catalog::forest()] {
        g.add_card_to_battlefield(0, land);
    }
    g.players[0].library.clear();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let deep = g.add_card_to_library(0, catalog::hill_giant());
    let tl = g.add_card_to_hand(0, catalog::branching_timeline());
    g.players[0].mana_pool.add(Color::Green, 8);
    cast(&mut g, tl, None, None);
    assert!(g.battlefield_find(bear).is_some_and(|c| c.tapped), "two names: the top two revealed");
    assert!(g.battlefield_find(deep).is_none());
    assert_eq!(g.players[0].library.iter().map(|c| c.id).collect::<Vec<_>>(), vec![deep, bolt]);
}

/// Gowron: sacrificing a Klingon draws and costs 1 life; a non-Klingon only
/// grants trample.
#[test]
fn gowron_draws_only_off_a_sacrificed_klingon() {
    let mut g = pod(2);
    let gowron = g.add_card_to_battlefield(0, catalog::gowron_eager_for_battle());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let sac = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: gowron,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("activate");
        drain_stack(g);
    };
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    sac(&mut g);
    assert_eq!((g.players[0].hand.len(), g.players[0].life), (0, 20));
    assert!(g.computed_permanent(gowron).unwrap().keywords().contains(&Keyword::Trample));
    g.add_card_to_battlefield(0, catalog::the_duras_sisters());
    sac(&mut g);
    // The Sisters die: their Klingon Warrior token, Gowron's counters (x2).
    assert_eq!((g.players[0].hand.len(), g.players[0].life), (1, 19));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Klingon Warrior"));
    assert_eq!(g.computed_permanent(gowron).unwrap().power, 5);
}

/// CR 700.2 — a dilemma is counted per player per turn (The Caretaker's "if
/// you faced a dilemma this turn"): Jett Reno grows on each one, and the
/// Caretaker's end step cascades into the opponent's library only after one.
#[test]
fn the_caretaker_cascades_only_after_a_dilemma() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::the_caretaker());
    g.players[1].library.clear();
    g.add_card_to_library(1, catalog::island());
    let bear = g.add_card_to_library(1, catalog::grizzly_bears());
    to_end_step(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "no dilemma, no cascade");
    // A modal spell next turn: the dilemma, then the end step's free cast.
    let mut g = pod(2);
    let jett = g.add_card_to_battlefield(0, catalog::jett_reno_problem_solver());
    g.add_card_to_battlefield(0, catalog::the_caretaker());
    g.players[1].library.clear();
    g.add_card_to_library(1, catalog::island());
    let bear = g.add_card_to_library(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let so = g.add_card_to_hand(0, catalog::make_it_so());
    g.players[0].mana_pool.add(Color::Blue, 4);
    cast(&mut g, so, None, Some(1));
    assert_eq!(g.players[0].dilemmas_faced_this_turn, 1);
    assert_eq!(g.computed_permanent(jett).unwrap().power, 2, "a +1/+1 counter per dilemma");
    to_end_step(&mut g);
    assert!(g.battlefield_find(bear).is_some_and(|c| c.controller == 0), "cast free off their library");
}

/// The Traveler: power counts differently named lands, toughness stays 3.
#[test]
fn the_traveler_counts_land_names_for_power_only() {
    let mut g = pod(2);
    let t = g.add_card_to_battlefield(0, catalog::the_traveler());
    for land in [catalog::forest(), catalog::forest(), catalog::island(), catalog::earth_sector_001()] {
        g.add_card_to_battlefield(0, land);
    }
    let cp = g.computed_permanent(t).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}
