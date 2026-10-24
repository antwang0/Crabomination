//! Commander: EDHREC ranks 1401-2000, single-card gaps (`decks::cmdr_rank2k_h`).

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

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast_action(id: CardId, target: Option<Target>) -> GameAction {
    GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None }
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(cast_action(id, target)).expect("cast");
    drain_stack(g);
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

/// Bard's Company recruits as it enters: a nonland discard makes a Human
/// Soldier, which its anthem pumps.
#[test]
fn bards_company_recruits_a_soldier() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let bard = g.add_card_to_hand(0, catalog::bards_company());
    cast(&mut g, bard, None);
    let soldier = g.battlefield.iter().find(|c| c.definition.name == "Human Soldier").expect("token").id;
    assert_eq!(g.computed_permanent(soldier).unwrap().power, 2);
    assert_eq!(g.players[0].graveyard.len(), 1, "the Bears were discarded");
}

/// Bard's Company has flash only while you control a Human.
#[test]
fn bards_company_flashes_beside_a_human() {
    for (human, ok) in [(true, true), (false, false)] {
        let mut g = pod(2);
        g.active_player_idx = 1;
        if human {
            ready(&mut g, 0, catalog::bards_company());
        }
        let bard = g.add_card_to_hand(0, catalog::bards_company());
        flood(&mut g, 0);
        g.priority.player_with_priority = 0;
        assert_eq!(g.perform_action(cast_action(bard, None)).is_ok(), ok);
    }
}

/// Blightbeetle stops +1/+1 counters on its opponents' creatures only.
#[test]
fn blightbeetle_blocks_enemy_counters() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::blightbeetle());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    for target in [mine, theirs] {
        let growth = g.add_card_to_hand(0, catalog::battlegrowth());
        cast(&mut g, growth, Some(Target::Permanent(target)));
    }
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(g.battlefield_find(theirs).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Courageous Resolve protects and draws; at 5 life it also locks your life
/// and your loss for the turn.
#[test]
fn courageous_resolve_fateful_hour() {
    for (life, locked) in [(5, true), (6, false)] {
        let mut g = pod(3);
        g.add_card_to_library(0, catalog::island());
        g.players[0].life = life;
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        let resolve = g.add_card_to_hand(0, catalog::courageous_resolve());
        cast(&mut g, resolve, Some(Target::Permanent(bear)));
        assert_eq!(g.players[0].hand.len(), 1, "drew a card");
        assert_eq!(g.players[0].cant_lose_this_turn, locked);
        assert_eq!(g.players[0].cant_lose_life_this_turn, locked);
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        flood(&mut g, 1);
        g.priority.player_with_priority = 1;
        assert!(g.perform_action(cast_action(bolt, Some(Target::Permanent(bear)))).is_err(), "protected from seat 2");
    }
}

/// Disruption Protocol taps an untapped artifact when it can, else costs {1}
/// more.
#[test]
fn disruption_protocol_taps_or_pays() {
    for artifact in [true, false] {
        let mut g = pod(2);
        let ring = artifact.then(|| ready(&mut g, 0, catalog::sol_ring()));
        let opt = g.add_card_to_hand(0, catalog::opt());
        flood(&mut g, 0);
        g.perform_action(cast_action(opt, None)).expect("Opt");
        let protocol = g.add_card_to_hand(0, catalog::disruption_protocol());
        g.players[0].mana_pool = Default::default();
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.priority.player_with_priority = 0;
        let res = g.perform_action(cast_action(protocol, Some(Target::Permanent(opt))));
        assert_eq!(res.is_ok(), artifact, "{{U}}{{U}} alone needs the artifact");
        if let Some(ring) = ring {
            assert!(g.battlefield_find(ring).unwrap().tapped);
            drain_stack(&mut g);
            assert!(g.players[0].graveyard.iter().any(|c| c.id == opt), "Opt was countered");
        }
    }
}

/// Extraction Specialist brings back a two-drop that can't attack or block.
#[test]
fn extraction_specialist_returns_a_pacified_two_drop() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spec = g.add_card_to_hand(0, catalog::extraction_specialist());
    cast(&mut g, spec, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).expect("returned");
    let kws = cp.keywords();
    assert!(kws.contains(&Keyword::CantAttack) && kws.contains(&Keyword::CantBlock));
    // CR 611.2b — "for as long as you control Extraction Specialist": it
    // ends when an opponent takes the Specialist, which stays on the field.
    let steal = crabomination::effect::Effect::GainControl {
        what: crabomination::effect::Selector::ExactObjects(vec![spec]),
        to: None,
        duration: crabomination::effect::Duration::Permanent,
    };
    g.resolve_effect(&steal, &EffectContext::for_spell(1, None, 0, 0)).unwrap();
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(spec).unwrap().controller, 1);
    let cp = g.computed_permanent(bear).unwrap();
    let kws = cp.keywords();
    assert!(!kws.contains(&Keyword::CantAttack) && !kws.contains(&Keyword::CantBlock));
}

/// Induced Amnesia swaps a hand for fresh cards, then gives it back when it
/// is destroyed.
#[test]
fn induced_amnesia_borrows_a_hand() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::island());
    }
    let a = g.add_card_to_hand(1, catalog::grizzly_bears());
    let b = g.add_card_to_hand(1, catalog::lightning_bolt());
    let amnesia = g.add_card_to_hand(0, catalog::induced_amnesia());
    cast(&mut g, amnesia, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 2);
    assert!(g.players[1].hand.iter().all(|c| c.id != a && c.id != b));
    let disenchant = g.add_card_to_hand(0, catalog::disenchant());
    cast(&mut g, disenchant, Some(Target::Permanent(amnesia)));
    assert_eq!(g.players[1].hand.len(), 4);
    assert!(g.players[1].hand.iter().any(|c| c.id == a));
}

/// Samut gives your instants split second: nobody can respond to them.
#[test]
fn samut_locks_the_stack() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::samut_tyrant_of_naktamun());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    g.perform_action(cast_action(bolt, Some(Target::Player(1)))).expect("bolt");
    let shock = g.add_card_to_hand(1, catalog::shock());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(cast_action(shock, Some(Target::Player(0)))).is_err());
}

/// Temporal Extortion is countered when an opponent pays half their life.
#[test]
fn temporal_extortion_can_be_bought_off() {
    for pays in [true, false] {
        let mut g = pod(2);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false), DecisionAnswer::Bool(pays)]));
        let life = g.players[1].life;
        let te = g.add_card_to_hand(0, catalog::temporal_extortion());
        cast(&mut g, te, None);
        assert_eq!(g.players[1].life, if pays { life - life / 2 } else { life });
        assert_eq!(g.players[0].extra_turns, if pays { 0 } else { 1 });
    }
}

/// Tyrant of Discord keeps taking random permanents until a land goes.
#[test]
fn tyrant_of_discord_eats_until_a_land() {
    let mut g = pod(2);
    let land = ready(&mut g, 1, catalog::island());
    for _ in 0..3 {
        ready(&mut g, 1, catalog::grizzly_bears());
    }
    let tyrant = g.add_card_to_hand(0, catalog::tyrant_of_discord());
    cast(&mut g, tyrant, Some(Target::Player(1)));
    assert!(g.battlefield_find(land).is_none(), "the land always goes eventually");
    assert!(g.battlefield_find(tyrant).is_some());
}

/// The Tyrant stops after the first land.
#[test]
fn tyrant_of_discord_stops_at_a_land() {
    let mut g = pod(2);
    for _ in 0..2 {
        ready(&mut g, 1, catalog::island());
    }
    let tyrant = g.add_card_to_hand(0, catalog::tyrant_of_discord());
    cast(&mut g, tyrant, Some(Target::Player(1)));
    assert_eq!(named(&g, "Island"), 1);
}
