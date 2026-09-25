//! Commander: the Blast from the Past precon (WHO, The Fourth Doctor + Sarah
//! Jane Smith, `decks::cmdr_fourth_doctor`), and its primitives: granted read
//! ahead (CR 714.3c), a non-legendary spell copy (CR 707.9b), a guess against
//! a live threshold with an else branch, one added counter of a chosen kind
//! per permanent, and "counter all other spells" without a draw.

use crabomination::card::{CardId, CounterType, Supertype, Value};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, PlayerRef, Selector};
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..12 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_hand(0, def);
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("castable");
    drain_stack(g);
    id
}

/// Resolve `source`'s `i`th triggered ability as if it fired.
fn fire(g: &mut GameState, source: CardId, i: usize) {
    let effect = g.battlefield_find(source).unwrap().definition.triggered_abilities[i].effect.clone();
    let ctx = EffectContext::for_ability(source, 0, None);
    let evs = g.resolve_effect(&effect, &ctx).expect("resolves");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

fn count(g: &GameState, id: CardId, kind: CounterType) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(kind))
}

/// CR 714.3c — Barbara Wright gives your Sagas read ahead: The Sea Devils
/// can start on chapter II, skipping chapter I's Salamander.
#[test]
fn barbara_wright_gives_your_sagas_read_ahead() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::barbara_wright());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Amount(2)]));
    let saga = cast(&mut g, catalog::the_sea_devils());
    assert_eq!(count(&g, saga, CounterType::Lore), 2);
    assert_eq!(named(&g, "Alien Salamander"), 1, "only chapter II fired");
}

/// Without Barbara the same Saga starts at chapter I.
#[test]
fn a_saga_without_read_ahead_starts_at_one() {
    let mut g = pod(2);
    let saga = cast(&mut g, catalog::the_sea_devils());
    assert_eq!(count(&g, saga, CounterType::Lore), 1);
    assert_eq!(named(&g, "Alien Salamander"), 1);
}

/// CR 707.9b — The Sixth Doctor copies a legendary spell, the copy not
/// legendary, so both survive the legend rule.
#[test]
fn the_sixth_doctor_copies_a_historic_spell_without_legendary() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::the_sixth_doctor());
    cast(&mut g, catalog::jo_grant());
    assert_eq!(named(&g, "Jo Grant"), 2);
    let copy = g.battlefield.iter().find(|c| c.definition.name == "Jo Grant" && c.is_token).expect("a token copy");
    assert!(!copy.definition.supertypes.contains(&Supertype::Legendary));
    cast(&mut g, catalog::barbara_wright());
    assert_eq!(named(&g, "Barbara Wright"), 1, "once each turn");
}

/// The Seventh Doctor: with no card in hand no spell is cast, so it
/// investigates.
#[test]
fn the_seventh_doctor_investigates_when_nothing_is_cast() {
    let mut g = pod(2);
    let doc = g.add_card_to_battlefield(0, catalog::the_seventh_doctor());
    g.players[0].hand.clear();
    fire(&mut g, doc, 0);
    assert_eq!(named(&g, "Clue"), 1);
}

/// Attacking, the defending player guesses: a right guess ("greater" for a
/// 5-drop over zero artifacts) casts nothing and investigates; a wrong one
/// casts the card free.
#[test]
fn the_seventh_doctor_casts_on_a_wrong_guess() {
    for (guess, clues, angels) in [(true, 1, 0), (false, 0, 1)] {
        let mut g = pod(2);
        let doc = g.add_card_to_battlefield(0, catalog::the_seventh_doctor());
        g.clear_sickness(doc);
        g.players[0].hand.clear();
        g.add_card_to_hand(0, catalog::serra_angel());
        g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(guess)]));
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: doc, target: AttackTarget::Player(1) }]))
            .expect("attack");
        drain_stack(&mut g);
        assert_eq!((named(&g, "Clue"), named(&g, "Serra Angel")), (clues, angels), "guess {guess}");
    }
}

/// The Caves of Androzani II: one more counter of a kind already there on
/// each non-Saga permanent — good ones on yours, bad ones on theirs, none on
/// a Saga.
#[test]
fn the_caves_of_androzani_adds_a_counter_to_each_permanent() {
    let mut g = pod(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let stunned = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let saga = g.add_card_to_battlefield(0, catalog::the_sea_devils());
    for (id, kind) in [
        (mine, CounterType::PlusOnePlusOne),
        (theirs, CounterType::PlusOnePlusOne),
        (stunned, CounterType::Stun),
        (saga, CounterType::Lore),
    ] {
        g.battlefield_find_mut(id).unwrap().add_counters(kind, 1);
    }
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let def = catalog::the_caves_of_androzani();
    let evs = g.resolve_effect(&def.saga_chapters[1].1, &ctx).expect("chapter II");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(count(&g, mine, CounterType::PlusOnePlusOne), 2);
    assert_eq!(count(&g, theirs, CounterType::PlusOnePlusOne), 1, "not an opponent's +1/+1");
    assert_eq!(count(&g, stunned, CounterType::Stun), 2);
    assert_eq!(count(&g, saga, CounterType::Lore), 1, "non-Saga permanents only");
}

/// Reverse the Polarity's first mode counters every other spell and draws
/// nothing (unlike Swift Silence).
#[test]
fn counter_all_other_spells_draws_nothing() {
    let mut g = pod(2);
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    flood(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    let hand = g.players[0].hand.len();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&Effect::CounterAllOtherSpells, &ctx).expect("counter");
    g.dispatch_triggers_for_events(&evs);
    assert!(g.stack.is_empty());
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bears));
    assert_eq!(g.players[0].hand.len(), hand);
}

/// Gallifrey Stands: at upkeep a Doctor comes down from hand, and thirteen
/// Doctors win the game.
#[test]
fn gallifrey_stands_wins_with_thirteen_doctors() {
    let mut g = pod(2);
    let gs = g.add_card_to_battlefield(0, catalog::gallifrey_stands());
    for _ in 0..12 {
        g.add_card_to_battlefield(0, catalog::the_fifth_doctor());
    }
    g.add_card_to_hand(0, catalog::the_first_doctor());
    let effect = g.battlefield_find(gs).unwrap().definition.triggered_abilities[1].effect.clone();
    let ctx = EffectContext::for_ability(gs, 0, None);
    let _ = g.resolve_effect(&effect, &ctx).expect("upkeep");
    assert_eq!(named(&g, "The First Doctor"), 1, "put onto the battlefield from hand");
    assert!(g.players[1].eliminated, "CR 104.2a — you win: every opponent is out");
}

/// The Fifth Doctor: at your end step, a creature that attacked this turn
/// gets nothing; one that sat out gets a counter and untaps.
#[test]
fn the_fifth_doctor_rewards_the_creatures_that_rested() {
    let mut g = pod(2);
    let doc = g.add_card_to_battlefield(0, catalog::the_fifth_doctor());
    let rested = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [doc, rested, attacker] {
        g.clear_sickness(id);
        g.battlefield_find_mut(id).unwrap().entered_turn = None;
    }
    g.battlefield_find_mut(rested).unwrap().tapped = true;
    g.battlefield_find_mut(attacker).unwrap().attacked_this_turn = true;
    fire(&mut g, doc, 0);
    assert_eq!(count(&g, rested, CounterType::PlusOnePlusOne), 1);
    assert!(!g.battlefield_find(rested).unwrap().tapped);
    assert_eq!(count(&g, attacker, CounterType::PlusOnePlusOne), 0);
}

/// The War Games I: every player gets three tapped Warriors, goaded.
#[test]
fn the_war_games_gives_everyone_goaded_warriors() {
    let mut g = pod(3);
    cast(&mut g, catalog::the_war_games());
    for p in 0..3 {
        let warriors: Vec<_> = g.battlefield.iter().filter(|c| c.controller == p && c.definition.name == "Warrior").collect();
        assert_eq!(warriors.len(), 3, "seat {p}");
        assert!(warriors.iter().all(|c| c.tapped));
    }
}

/// Trenzalore Clocktower: each tap adds a time counter; twelve pay for the
/// wheel once you control a Time Lord.
#[test]
fn trenzalore_clocktower_needs_twelve_time_counters_and_a_time_lord() {
    let mut g = pod(2);
    let tower = g.add_card_to_battlefield(0, catalog::trenzalore_clocktower());
    g.battlefield_find_mut(tower).unwrap().add_counters(CounterType::Time, 12);
    flood(&mut g, 0);
    let wheel = GameAction::ActivateAbility {
        card_id: tower,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    };
    assert!(g.perform_action(wheel.clone()).is_err(), "no Time Lord");
    g.add_card_to_battlefield(0, catalog::susan_foreman());
    g.perform_action(wheel).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(tower).is_none());
    assert_eq!(g.players[0].hand.len(), 7);
}

/// Sarah Jane Smith investigates on the first historic spell each turn only.
#[test]
fn sarah_jane_smith_investigates_once_a_turn() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::sarah_jane_smith());
    cast(&mut g, catalog::sol_ring());
    cast(&mut g, catalog::mind_stone());
    cast(&mut g, catalog::grizzly_bears());
    assert_eq!(named(&g, "Clue"), 1);
}

/// Vrestin enters with X counters and X flying Insects.
#[test]
fn vrestin_brings_x_insects() {
    let mut g = pod(2);
    let id = g.add_card_to_hand(0, catalog::vrestin_menoptra_leader());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3) })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(count(&g, id, CounterType::PlusOnePlusOne), 3);
    assert_eq!(named(&g, "Alien Insect"), 3);
}

/// Crisis of Conscience's first mode destroys only tokens.
#[test]
fn crisis_of_conscience_destroys_tokens() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g
        .resolve_effect(
            &Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: std::sync::Arc::new(crabomination_base::tokens::clue_token()),
            },
            &ctx,
        )
        .expect("clues");
    g.dispatch_triggers_for_events(&evs);
    let def = catalog::crisis_of_conscience();
    let Effect::ChooseMode(modes) = &def.effect else { panic!("modal") };
    let evs = g.resolve_effect(&modes[0], &ctx).expect("tokens");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(named(&g, "Clue"), 0);
    assert!(g.battlefield_find(bear).is_some());
    let _ = Selector::This;
}
