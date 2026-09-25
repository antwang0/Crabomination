//! Commander: the Limit Break precon (FIC, Cloud, Ex-SOLDIER,
//! `decks::cmdr_cloud_fic`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn act(g: &mut GameState, a: GameAction) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(a).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target: targets.first().cloned(),
            additional_targets: targets.iter().skip(1).cloned().collect(),
            mode: None,
            x_value: None,
        },
    )
}

fn fire(g: &mut GameState, step: TurnStep) {
    g.step = step;
    g.fire_step_triggers(step);
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

/// Avalanche of Sector 7's power is the artifacts its opponents control.
#[test]
fn avalanche_counts_opposing_artifacts() {
    let mut g = main_phase(2);
    let av = g.add_card_to_battlefield(0, catalog::avalanche_of_sector_7());
    g.add_card_to_battlefield(1, catalog::sol_ring());
    g.add_card_to_battlefield(1, catalog::mind_stone());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    assert_eq!(g.computed_permanent(av).unwrap().power, 2);
}

/// Bugenhagen draws at upkeep only with a 7-power creature out.
#[test]
fn bugenhagen_draws_with_a_big_creature() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::bugenhagen_wise_elder());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    fire(&mut g, TurnStep::Upkeep);
    assert_eq!(g.players[0].hand.len(), hand, "no 7-power creature");
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    g.battlefield_find_mut(wurm).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    fire(&mut g, TurnStep::Upkeep);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Cid flies on your turn only, and Equipment costs {1} less.
#[test]
fn cid_jumps_and_discounts_equipment() {
    let mut g = main_phase(2);
    let cid = g.add_card_to_battlefield(0, catalog::cid_freeflier_pilot());
    assert!(g.computed_permanent(cid).unwrap().keywords().contains(&Keyword::Flying));
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(cid).unwrap().keywords().contains(&Keyword::Flying));
    g.active_player_idx = 0;
    let heirloom = g.add_card_to_hand(0, catalog::heros_heirloom());
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, heirloom, &[]).expect("{2} less 1");
    assert!(g.battlefield_find(heirloom).is_some());
}

/// Hero's Heirloom: +2/+1 always; trample and haste only on a legend.
#[test]
fn heros_heirloom_favours_legends() {
    let mut g = main_phase(2);
    let heirloom = g.add_card_to_battlefield(0, catalog::heros_heirloom());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(heirloom).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3));
    assert!(!cp.keywords().contains(&Keyword::Trample));
    let cid = g.add_card_to_battlefield(0, catalog::cid_freeflier_pilot());
    g.battlefield_find_mut(heirloom).unwrap().attached_to = Some(cid);
    let cp = g.computed_permanent(cid).unwrap();
    assert!(cp.keywords().contains(&Keyword::Trample) && cp.keywords().contains(&Keyword::Haste));
}

/// Wrecking Ball Arm makes its creature a 7/7; equipping a legend costs {3}.
#[test]
fn wrecking_ball_arm_sets_seven_seven() {
    let mut g = main_phase(2);
    let arm = g.add_card_to_battlefield(0, catalog::wrecking_ball_arm());
    let cid = g.add_card_to_battlefield(0, catalog::cid_freeflier_pilot());
    g.players[0].mana_pool.add_colorless(3);
    act(&mut g, GameAction::Equip { equipment: arm, target: cid }).expect("equip legendary {3}");
    let cp = g.computed_permanent(cid).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7));
}

/// Elena returns a non-Assassin historic card and grows on historic spells.
#[test]
fn elena_returns_a_historic_card() {
    let mut g = main_phase(2);
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let elena = g.add_card_to_hand(0, catalog::elena_turk_recruit());
    flood(&mut g, 0);
    cast(&mut g, elena, &[Target::Permanent(ring)]).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == ring));
    cast(&mut g, ring, &[]).expect("a historic spell");
    assert_eq!(g.battlefield_find(elena).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Heidegger makes a Soldier per opponent with more creatures than you.
#[test]
fn heidegger_counts_bigger_boards() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(0, catalog::heidegger_shinra_executive());
    for _ in 0..2 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    g.add_card_to_battlefield(2, catalog::grizzly_bears());
    fire(&mut g, TurnStep::End);
    assert_eq!(named(&g, 0, "Soldier"), 1, "only seat 1 has more than Heidegger's one");
}

/// Lifestream's Blessing draws X; foretold it also gains 2X life.
#[test]
fn lifestreams_blessing_foretold_gains_life() {
    for foretold in [false, true] {
        let mut g = main_phase(2);
        for _ in 0..8 {
            g.add_card_to_library(0, catalog::forest());
        }
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::lifestreams_blessing());
        flood(&mut g, 0);
        let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
        if foretold {
            g.perform_action(GameAction::Foretell { card_id: spell }).expect("foretell");
            g.foretold_this_turn.clear();
            act(
                &mut g,
                GameAction::CastForetold { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None },
            )
            .expect("cast foretold");
        } else {
            cast(&mut g, spell, &[]).expect("cast");
        }
        assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "X = 2");
        assert_eq!(g.players[0].life, life + if foretold { 4 } else { 0 });
    }
}

/// Ultimate Magic: Meteor, foretold: 7 to each creature and each opponent
/// loses an artifact or land.
#[test]
fn ultimate_magic_meteor_foretold() {
    let mut g = main_phase(3);
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let land = g.add_card_to_battlefield(2, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::ultimate_magic_meteor());
    flood(&mut g, 0);
    g.perform_action(GameAction::Foretell { card_id: spell }).expect("foretell");
    g.foretold_this_turn.clear();
    act(&mut g, GameAction::CastForetold { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast foretold");
    assert!(g.battlefield_find(wurm).is_none());
    assert!(g.battlefield_find(ring).is_none() && g.battlefield_find(land).is_none());
}

/// Sephiroth returns from the graveyard by sacrificing a modified creature.
#[test]
fn sephiroth_returns_for_a_modified_creature() {
    let mut g = main_phase(2);
    let seph = g.add_card_to_graveyard(0, catalog::sephiroth_fallen_hero());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    flood(&mut g, 0);
    act(
        &mut g,
        GameAction::ActivateAbility {
            card_id: seph,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        },
    )
    .expect("the Reunion");
    assert!(g.battlefield_find(bear).is_none(), "sacrificed");
    assert!(g.battlefield_find(seph).is_some_and(|c| c.tapped));
}

/// Summoning Materia lets you cast a creature from the top of your library
/// while it is attached.
#[test]
fn summoning_materia_casts_from_the_top() {
    let mut g = main_phase(2);
    let materia = g.add_card_to_battlefield(0, catalog::summoning_materia());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    assert!(cast(&mut g, top, &[]).is_err(), "not while unattached");
    g.battlefield_find_mut(materia).unwrap().attached_to = Some(bear);
    cast(&mut g, top, &[]).expect("from the top");
    assert!(g.battlefield_find(top).is_some());
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4);
}

/// Yuffie steals a noncreature artifact.
#[test]
fn yuffie_steals_an_artifact() {
    let mut g = main_phase(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let yuffie = g.add_card_to_hand(0, catalog::yuffie_materia_hunter());
    flood(&mut g, 0);
    cast(&mut g, yuffie, &[Target::Permanent(ring)]).expect("cast");
    assert_eq!(g.battlefield_find(ring).unwrap().controller, 0);
}
