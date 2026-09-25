//! Commander: the Paradox Power precon (WHO, The Thirteenth Doctor + Yasmin
//! Khan, `decks::cmdr_thirteenth`), and its primitives: spells cast from
//! anywhere other than your hand (Paradox), drawing from the bottom, a
//! turn-gated graveyard flashback grant, two-type spend restriction, exiling
//! a permanent foretold (CR 702.143) and a granted demonstrate (CR 702.150).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.turn_number = 3;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..12 {
            g.add_card_to_library(seat, catalog::island());
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

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn flashback(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastFlashback { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn counters(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
}

/// CR 702.124m — The Thirteenth Doctor pairs with a Doctor's companion.
#[test]
fn cr_702_124m_the_doctor_and_yasmin_pair() {
    use crabomination::format::commanders_may_pair;
    assert!(commanders_may_pair(&catalog::the_thirteenth_doctor(), &catalog::yasmin_khan()));
    assert!(!commanders_may_pair(&catalog::yasmin_khan(), &catalog::dan_lewis()), "two companions");
}

/// Paradox + Return the Past — a spell flashed back from the graveyard on
/// your turn triggers The Thirteenth Doctor; on another turn there is no
/// flashback to use.
#[test]
fn return_the_past_grants_flashback_on_your_turn_only() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::return_the_past());
    let doc = g.add_card_to_battlefield(0, catalog::the_thirteenth_doctor());
    let loop_ = g.add_card_to_graveyard(0, catalog::flatline());
    flood(&mut g, 0);
    flashback(&mut g, 0, loop_, None).expect("flashback on your turn");
    assert_eq!(counters(&g, doc), 1, "paradox: a counter on the only creature");

    let other = g.add_card_to_graveyard(0, catalog::flatline());
    g.active_player_idx = 1;
    assert!(flashback(&mut g, 0, other, None).is_err(), "not during an opponent's turn");
}

/// Impending Flux — X is 1 plus the spells cast not from hand this turn.
#[test]
fn impending_flux_counts_paradox_spells() {
    let mut g = pod(2);
    g.players[0].spells_cast_this_turn = 3;
    g.players[0].spells_cast_from_hand_this_turn = 1;
    flood(&mut g, 0);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flux = g.add_card_to_hand(0, catalog::impending_flux());
    cast(&mut g, 0, flux, None).expect("Impending Flux");
    // Cast from hand: now 4 cast, 2 from hand → X = 1 + 2 = 3.
    assert_eq!(g.players[1].life, 17);
    assert!(g.battlefield_find(bear).is_none());
}

/// River Song — you draw from the bottom of your library.
#[test]
fn river_song_draws_from_the_bottom() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::river_song());
    let bottom = g.add_card_to_library(0, catalog::grizzly_bears());
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(bottom), "added at the bottom");
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&Effect::Draw { who: Selector::You, amount: crabomination::card::Value::ONE }, &ctx).expect("draw");
    assert!(g.players[0].hand.iter().any(|c| c.id == bottom));
}

/// Gallifrey Council Chamber — its colored mana pays for a Time Lord, not a
/// Bear.
#[test]
fn gallifrey_mana_is_for_time_lords_and_aliens() {
    let mut g = pod(2);
    let gcc = g.add_card_to_battlefield(0, catalog::gallifrey_council_chamber());
    activate(&mut g, 0, gcc, 1).expect("any color");
    g.players[0].mana_pool.add(Color::Green, 1);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(cast(&mut g, 0, bear, None).is_err(), "not for a Bear");
    g.players[0].mana_pool.add(Color::Blue, 1);
    let doc = g.add_card_to_hand(0, catalog::the_thirteenth_doctor());
    cast(&mut g, 0, doc, None).expect("The Thirteenth Doctor on Gallifrey mana");
}

/// CR 702.143 — The Foretold Soldier dealing damage is exiled foretold, and
/// can be cast for its foretell cost on a later turn.
#[test]
fn cr_702_143_foretold_soldier_exiles_itself_foretold() {
    let mut g = pod(2);
    let fs = g.add_card_to_battlefield(0, catalog::the_foretold_soldier());
    let ctx = EffectContext::for_ability(fs, 0, None);
    let evs = g
        .resolve_effect(
            &Effect::DealDamage {
                to: Selector::Player(crabomination::effect::PlayerRef::Seat(1)),
                amount: crabomination::card::Value::Const(6),
            },
            &ctx,
        )
        .expect("damage");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let ex = g.exile.iter().find(|c| c.id == fs).expect("exiled");
    assert!(ex.face_down, "foretold");
}

/// Clara Oswald — a Doctor's paradox trigger triggers an additional time.
#[test]
fn clara_doubles_the_doctors_triggers() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::return_the_past());
    g.add_card_to_battlefield(0, catalog::clara_oswald());
    let doc = g.add_card_to_battlefield(0, catalog::the_thirteenth_doctor());
    let fl = g.add_card_to_graveyard(0, catalog::flatline());
    flood(&mut g, 0);
    flashback(&mut g, 0, fl, None).expect("flashback");
    let total: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(total, 2, "the Doctor's paradox fired twice");
    let _ = doc;
}

/// Sisterhood of Karn — paradox doubles its counters.
#[test]
fn sisterhood_of_karn_doubles() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::return_the_past());
    flood(&mut g, 0);
    let sk = g.add_card_to_hand(0, catalog::sisterhood_of_karn());
    cast(&mut g, 0, sk, None).expect("Sisterhood");
    assert_eq!(counters(&g, sk), 1);
    let fl = g.add_card_to_graveyard(0, catalog::flatline());
    flashback(&mut g, 0, fl, None).expect("flashback");
    assert_eq!(counters(&g, sk), 2);
}

/// CR 702.150 — The Twelfth Doctor: the first not-from-hand spell is
/// demonstrated, and each copy of yours grows it.
#[test]
fn cr_702_150_twelfth_doctor_demonstrates() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::return_the_past());
    let twelve = g.add_card_to_battlefield(0, catalog::the_twelfth_doctor());
    let sob = g.add_card_to_graveyard(0, catalog::surge_of_brilliance());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    flashback(&mut g, 0, sob, None).expect("flashback Surge");
    assert!(counters(&g, twelve) >= 1, "your copy grew it");
    assert!(g.players[0].hand.len() >= hand + 2, "the original and your copy each drew");
}

/// Twice Upon a Time — can't be cast with fewer than two Doctors.
#[test]
fn twice_upon_a_time_needs_two_doctors() {
    let mut g = pod(2);
    flood(&mut g, 0);
    g.add_card_to_battlefield(0, catalog::the_thirteenth_doctor());
    let tut = g.add_card_to_hand(0, catalog::twice_upon_a_time());
    assert!(cast(&mut g, 0, tut, None).is_err(), "one Doctor");
    g.add_card_to_battlefield(0, catalog::the_twelfth_doctor());
    cast(&mut g, 0, tut, None).expect("two Doctors");
    assert!(g.exile.iter().any(|c| c.id == tut), "exiled as it resolves");
}

/// Heaven Sent III — with every opponent above 0, it exiles itself and may be
/// cast again this turn.
#[test]
fn heaven_sent_recurs_itself() {
    let mut g = pod(2);
    let hs = g.add_card_to_battlefield(0, catalog::heaven_sent());
    g.battlefield_find_mut(hs).unwrap().add_counters(CounterType::Lore, 2);
    let ctx = EffectContext::for_ability(hs, 0, None);
    let chapter = catalog::heaven_sent().saga_chapters[2].1.clone();
    g.resolve_effect(&chapter, &ctx).expect("III");
    assert_eq!(g.players[1].life, 19);
    let ex = g.exile.iter().find(|c| c.id == hs).expect("exiled");
    assert!(ex.may_play_until.is_some(), "and playable this turn");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: hs,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from exile this turn, paying {U}{R}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(hs).is_some());
    assert_eq!(g.players[0].mana_pool.total(), 0, "its own cost was paid");
}
