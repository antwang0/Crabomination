//! Commander: the Revival Trance precon (FIC, Terra, Herald of Hope,
//! `decks::cmdr_terra`).

use crabomination::card::{CardId, CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    act(
        g,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target: targets.first().cloned(),
            additional_targets: targets.iter().skip(1).cloned().collect(),
            x_value: None,
            mode: None,
        },
    )
}

fn fire(g: &mut GameState, step: TurnStep) {
    g.step = step;
    g.fire_step_triggers(step);
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

/// Terra's Trance: mill two and fly for the turn.
#[test]
fn terra_trances_each_combat() {
    let mut g = main_phase(2);
    let terra = g.add_card_to_battlefield(0, catalog::terra_herald_of_hope());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    fire(&mut g, TurnStep::BeginCombat);
    assert_eq!(g.players[0].graveyard.len(), 2);
    assert!(g.computed_permanent(terra).unwrap().keywords().contains(&Keyword::Flying));
}

/// Rejoin the Fight: each of two opponents hands back one creature card, and
/// never the same one.
#[test]
fn rejoin_the_fight_takes_one_per_opponent() {
    let mut g = main_phase(3);
    let cards: Vec<CardId> = [catalog::grizzly_bears(), catalog::craw_wurm(), catalog::hill_giant()]
        .into_iter()
        .map(|c| g.add_card_to_graveyard(0, c))
        .collect();
    let spell = g.add_card_to_hand(0, catalog::rejoin_the_fight());
    flood(&mut g, 0);
    cast(&mut g, spell, &[]).expect("cast");
    let back = cards.iter().filter(|id| g.battlefield_find(**id).is_some_and(|c| c.controller == 0)).count();
    assert_eq!(back, 2, "one card per opponent");
    assert!(g.battlefield_find(cards[1]).is_none(), "opponents hand back the weakest first — not the Wurm");
}

/// Strago and Relm: the opponent digs to a creature, which is cast free with
/// haste and sacrificed at the end step.
#[test]
fn strago_and_relm_casts_a_hasty_borrowed_creature() {
    let mut g = main_phase(2);
    let sr = g.add_card_to_battlefield(0, catalog::strago_and_relm());
    g.clear_sickness(sr);
    let bear = g.add_card_to_library(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::forest());
    flood(&mut g, 0);
    activate(&mut g, sr, 0, &[Target::Player(1)]).expect("sketch");
    let c = g.battlefield_find(bear).expect("cast free");
    assert_eq!(c.controller, 0);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Haste));
    fire(&mut g, TurnStep::End);
    assert!(g.battlefield_find(bear).is_none(), "sacrificed at the end step");
}

/// Kefka casts a card exiled at random from each opponent's graveyard, and
/// its owner loses life equal to its mana value.
#[test]
fn kefka_casts_their_graveyard_and_bills_them() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::kefka_dancing_mad());
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    fire(&mut g, TurnStep::End);
    assert_eq!(g.battlefield_find(bear).map(|c| c.controller), Some(0));
    assert_eq!(g.players[1].life, life - 2);
}

/// Sabin blitzes from the graveyard (CR 702.152): haste, a discard as the
/// additional cost.
#[test]
fn sabin_blitzes_from_the_graveyard() {
    let mut g = main_phase(2);
    let sabin = g.add_card_to_graveyard(0, catalog::sabin_master_monk());
    g.add_card_to_hand(0, catalog::forest());
    flood(&mut g, 0);
    act(
        &mut g,
        GameAction::CastSpellAlternative {
            card_id: sabin,
            pitch_card: None,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        },
    )
    .expect("blitz from the graveyard");
    assert!(g.computed_permanent(sabin).unwrap().keywords().contains(&Keyword::Haste));
    assert!(g.players[0].hand.is_empty(), "discarded");
}

/// Cyan costs less per creature card in your graveyard and grows as they
/// leave it.
#[test]
fn cyan_grows_as_creatures_leave_the_graveyard() {
    let mut g = main_phase(2);
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let cyan = g.add_card_to_hand(0, catalog::cyan_vengeful_samurai());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, cyan, &[]).expect("{6}{W} less 4");
    let spell = g.add_card_to_hand(0, catalog::rejoin_the_fight());
    flood(&mut g, 0);
    cast(&mut g, spell, &[]).expect("cast");
    assert_eq!(g.battlefield_find(cyan).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "once per batch");
}

/// Coin of Fate: one exiled creature returns tapped, you become the monarch.
#[test]
fn coin_of_fate_returns_one_and_crowns_you() {
    let mut g = main_phase(2);
    let coin = g.add_card_to_battlefield(0, catalog::coin_of_fate());
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::craw_wurm());
    flood(&mut g, 0);
    activate(&mut g, coin, 0, &[]).expect("flip");
    let back: Vec<CardId> = [a, b].into_iter().filter(|id| g.battlefield_find(*id).is_some()).collect();
    assert_eq!(back.len(), 1);
    assert!(g.battlefield_find(back[0]).unwrap().tapped);
    assert_eq!(g.monarch, Some(0));
}

/// Siegfried: two counters per creature card in your graveyard after milling.
#[test]
fn siegfried_counts_creature_cards() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::craw_wurm());
    let s = g.add_card_to_hand(0, catalog::siegfried_famed_swordsman());
    flood(&mut g, 0);
    cast(&mut g, s, &[]).expect("cast");
    assert_eq!(g.battlefield_find(s).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

/// The Warring Triad is a creature only with eight cards in your graveyard.
#[test]
fn warring_triad_needs_eight_cards() {
    let mut g = main_phase(2);
    let triad = g.add_card_to_battlefield(0, catalog::the_warring_triad());
    let is_creature = |g: &GameState| g.computed_permanent(triad).unwrap().card_types().contains(&CardType::Creature);
    assert!(!is_creature(&g));
    for _ in 0..8 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    assert!(is_creature(&g));
}

/// Setzer brings The Blackjack, a legendary flying Vehicle with crew 2.
#[test]
fn setzer_brings_the_blackjack() {
    let mut g = main_phase(2);
    let s = g.add_card_to_hand(0, catalog::setzer_wandering_gambler());
    flood(&mut g, 0);
    cast(&mut g, s, &[]).expect("cast");
    let bj = named(&g, 0, "The Blackjack");
    assert_eq!(bj.len(), 1);
    let c = g.battlefield_find(bj[0]).unwrap();
    assert!(c.definition.keywords.contains(&Keyword::Crew(2)));
}

/// Legions to Ashes exiles the target and its same-named tokens.
#[test]
fn legions_to_ashes_takes_the_namesakes() {
    let mut g = main_phase(2);
    let s = g.add_card_to_battlefield(1, catalog::setzer_wandering_gambler());
    let spell = g.add_card_to_hand(0, catalog::legions_to_ashes());
    flood(&mut g, 0);
    cast(&mut g, spell, &[Target::Permanent(s)]).expect("cast");
    assert!(g.battlefield_find(s).is_none());
}

/// Espers to Magicite: an opponent's creature card comes back as a
/// noncreature artifact token copy.
#[test]
fn espers_to_magicite_crystallizes_a_creature() {
    let mut g = main_phase(2);
    g.add_card_to_graveyard(1, catalog::craw_wurm());
    let spell = g.add_card_to_hand(0, catalog::espers_to_magicite());
    flood(&mut g, 0);
    cast(&mut g, spell, &[]).expect("cast");
    let tok = g.battlefield.iter().find(|c| c.is_token && c.controller == 0).map(|c| c.id).expect("a token");
    let types = g.computed_permanent(tok).unwrap().card_types().to_vec();
    assert_eq!(types, vec![CardType::Artifact]);
}

/// Shadow may sacrifice a permanent on a hit: draw two, drain its mana value.
#[test]
fn shadow_throws_a_permanent() {
    let mut g = main_phase(2);
    let shadow = g.add_card_to_battlefield(0, catalog::shadow_mysterious_assassin());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::swamp());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let (hand, life) = (g.players[0].hand.len(), g.players[1].life);
    g.clear_sickness(shadow);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: shadow,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 2);
    assert_eq!(g.players[1].life, life - 3 - 1, "combat damage, then Sol Ring's mana value");
}

/// Umaro picks one of its three modes at random each combat.
#[test]
fn umaro_rolls_a_mode() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::umaro_raging_yeti());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::mountain());
    }
    let (life, hand) = (g.players[1].life, g.players[0].hand.len());
    fire(&mut g, TurnStep::BeginCombat);
    let pumped = g.computed_permanent(bear).unwrap().power == 5;
    let wheeled = g.players[0].hand.len() == hand + 4;
    let burned = g.players[1].life == life - 5 || g.battlefield.len() < 2;
    assert_eq!([pumped, wheeled, burned].iter().filter(|b| **b).count(), 1);
}
