//! Commander: the 2026-10-13 rescan's one-card gaps (`decks::cmdr_edhrec22`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
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

fn loyalty(g: &mut GameState, id: CardId, index: usize) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

/// Black Vise hits the chosen opponent for hand size minus four on their
/// upkeep, and for nothing (not a negative amount) on a short hand.
#[test]
fn black_vise_punishes_a_full_hand() {
    for (hand, life) in [(7, 17), (2, 20)] {
        let mut g = pod(2);
        let vise = g.add_card_to_hand(0, catalog::black_vise());
        cast(&mut g, 0, vise, &[]).expect("cast");
        for _ in 0..hand {
            g.add_card_to_hand(1, catalog::island());
        }
        g.active_player_idx = 1;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life, "{hand} cards");
    }
}

/// Mercy Killing: the creature's controller sacrifices it and gets one Elf
/// Warrior per point of its power.
#[test]
fn mercy_killing_pays_the_controller() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let id = g.add_card_to_hand(0, catalog::mercy_killing());
    cast(&mut g, 0, id, &[Target::Permanent(giant)]).expect("cast");
    assert!(g.battlefield_find(giant).is_none());
    let elves: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Elf Warrior").collect();
    assert_eq!(elves.len(), 3);
    assert!(elves.iter().all(|c| c.controller == 1));
}

/// Delay (CR 702.62a): the countered spell goes to exile with three time
/// counters and gains suspend.
#[test]
fn delay_suspends_the_countered_spell() {
    let mut g = pod(2);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt cast");
    let delay = g.add_card_to_hand(0, catalog::delay());
    cast(&mut g, 0, delay, &[Target::Permanent(bolt)]).expect("delay");
    let card = g.exile.iter().find(|c| c.id == bolt).expect("exiled");
    assert!(card.is_suspended());
    assert_eq!(card.counter_count(CounterType::Time), 3);
}

/// Basri's Lieutenant makes a Knight when a creature of yours with a +1/+1
/// counter dies, and not for one without.
#[test]
fn basris_lieutenant_knights_for_countered_deaths() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let plain = ready(&mut g, 0, catalog::grizzly_bears());
    let lt = g.add_card_to_hand(0, catalog::basris_lieutenant());
    cast(&mut g, 0, lt, &[Target::Permanent(bear)]).expect("cast");
    for id in [plain, bear] {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        cast(&mut g, 1, bolt, &[Target::Permanent(id)]).expect("bolt");
        g.check_state_based_actions();
        drain_stack(&mut g);
    }
    assert_eq!(named(&g, "Knight"), 1);
}

/// Loran returns an artifact card when it or another legendary creature
/// enters.
#[test]
fn loran_regrows_artifacts_on_legends() {
    let mut g = pod(2);
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let loran = g.add_card_to_hand(0, catalog::loran_disciple_of_history());
    cast(&mut g, 0, loran, &[]).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == ring));
}

/// Tainted Indulgence discards only without five mana values in the
/// graveyard.
#[test]
fn tainted_indulgence_discard_gate() {
    for (filled, hand) in [(false, 1), (true, 2)] {
        let mut g = pod(2);
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        if filled {
            for def in [catalog::island, catalog::lightning_bolt, catalog::shock, catalog::grizzly_bears, catalog::hill_giant] {
                g.add_card_to_graveyard(0, def());
            }
            g.add_card_to_graveyard(0, catalog::craw_wurm());
        }
        let id = g.add_card_to_hand(0, catalog::tainted_indulgence());
        cast(&mut g, 0, id, &[]).expect("cast");
        assert_eq!(g.players[0].hand.len(), hand, "filled graveyard: {filled}");
    }
}

/// Tenacious Dead pays {1}{B} to come back tapped.
#[test]
fn tenacious_dead_returns_tapped() {
    let mut g = pod(2);
    let dead = ready(&mut g, 0, catalog::tenacious_dead());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, &[Target::Permanent(dead)]).expect("bolt");
    g.check_state_based_actions();
    drain_stack(&mut g);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Tenacious Dead").expect("returned");
    assert!(back.tapped);
}

/// The Misty Mountains Cold trades four Treasures' worth of chapters for a
/// 6/6 Dragon, sacrificing itself.
#[test]
fn misty_mountains_makes_a_dragon_at_four_treasures() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
    }
    let saga = g.add_card_to_hand(0, catalog::the_misty_mountains_cold());
    cast(&mut g, 0, saga, &[]).expect("cast");
    assert_eq!(named(&g, "Dragon"), 1);
    assert!(g.battlefield_find(saga).is_none(), "sacrificed");
}

/// Champion of the Path beholds an Elemental and burns each opponent for
/// another Elemental's power as it enters.
#[test]
fn champion_of_the_path_beholds_and_burns() {
    let mut g = pod(3);
    g.add_card_to_hand(0, catalog::champion_of_the_path());
    let champ = g.players[0].hand.last().unwrap().id;
    g.add_card_to_hand(0, catalog::champion_of_the_path());
    cast(&mut g, 0, champ, &[]).expect("behold the other Champion");
    assert_eq!(g.exile.len(), 1, "the beheld Elemental");
    let other = g.add_card_to_hand(0, catalog::champion_of_the_path());
    g.add_card_to_hand(0, catalog::champion_of_the_path());
    cast(&mut g, 0, other, &[]).expect("second");
    assert_eq!((g.players[1].life, g.players[2].life), (13, 13), "7 to each opponent");
}

/// Jace, Reality Sculptor: +1 empowers a Jace token by Islands; the 0 does
/// nothing short of 25 loyalty among your Jaces.
#[test]
fn jace_reality_sculptor_empowers_and_gates() {
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 0, catalog::island());
    }
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let jace = ready(&mut g, 0, catalog::jace_reality_sculptor());
    loyalty(&mut g, jace, 0).expect("+1");
    let token = g.battlefield.iter().find(|c| c.is_token && c.definition.is_planeswalker()).expect("Jace token");
    assert_eq!(token.counter_count(CounterType::Loyalty), 3);
    let mut g2 = pod(2);
    for _ in 0..5 {
        g2.add_card_to_library(1, catalog::island());
    }
    let jace = ready(&mut g2, 0, catalog::jace_reality_sculptor());
    loyalty(&mut g2, jace, 2).expect("0");
    assert_eq!(g2.players[1].library.len(), 5, "under 25 loyalty");
}

/// Zuko gains first strike from a noncreature spell.
#[test]
fn zuko_first_strike_on_noncreature_spells() {
    let mut g = pod(2);
    let zuko = ready(&mut g, 0, catalog::zuko_seeking_honor());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, &[Target::Player(1)]).expect("bolt");
    assert!(g.computed_permanent(zuko).unwrap().keywords().contains(&Keyword::FirstStrike));
}

/// Garruk's Horde casts a creature off the top of the library.
#[test]
fn garruks_horde_casts_from_the_top() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::garruks_horde());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    cast(&mut g, 0, bear, &[]).expect("cast from the top");
    assert!(g.battlefield_find(bear).is_some());
}

/// Feywild Caretaker takes the initiative and makes a Faerie Dragon at your
/// end step while you hold it.
#[test]
fn feywild_caretaker_dragons_with_the_initiative() {
    let mut g = pod(2);
    let id = g.add_card_to_hand(0, catalog::feywild_caretaker());
    cast(&mut g, 0, id, &[]).expect("cast");
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(named(&g, "Faerie Dragon"), 1);
}

/// Bug fix (seat 1067's pod, seed 22104): a "an opponent gains control"
/// donation named `EachOpponent` in a singular slot and panicked at 3+ seats.
/// Discerning Financier now hands its Treasure to one opponent.
#[test]
fn discerning_financier_donates_to_one_opponent_in_a_pod() {
    let mut g = pod(3);
    let fin = ready(&mut g, 0, catalog::discerning_financier());
    let treasure = g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
    g.add_card_to_library(0, catalog::island());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fin,
        ability_index: 0,
        target: Some(Target::Permanent(treasure)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("donate");
    drain_stack(&mut g);
    let to = g.battlefield.iter().find(|c| c.id == treasure).expect("still there").controller;
    assert!(to == 1 || to == 2);
    assert_eq!(g.players[0].hand.len(), 1);
}
