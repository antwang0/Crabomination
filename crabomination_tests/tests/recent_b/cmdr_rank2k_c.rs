//! Commander: EDHREC ranks 1501-2000, two-card gaps (`decks::cmdr_rank2k_c`).

use crabomination::card::{CardId, CounterType, Keyword};
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

fn cast_by(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, ability_index: usize, target: Option<Target>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Primalcrux is as big as your devotion to green, both ways.
#[test]
fn primalcrux_counts_green_pips() {
    let mut g = pod(2);
    let crux = ready(&mut g, 0, catalog::primalcrux());
    ready(&mut g, 0, catalog::steward_of_valeron());
    let cp = g.computed_permanent(crux).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7), "six from itself, one from the Steward");
}

/// Ivorytusk Fortress untaps its controller's countered creatures during
/// another player's untap step, and only those.
#[test]
fn ivorytusk_fortress_untaps_countered_creatures_off_turn() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::ivorytusk_fortress());
    let grown = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(grown).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let plain = ready(&mut g, 0, catalog::grizzly_bears());
    for id in [grown, plain] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    g.active_player_idx = 2;
    g.do_untap();
    assert!(!g.battlefield_find(grown).unwrap().tapped);
    assert!(g.battlefield_find(plain).unwrap().tapped);
}

/// Soot Imp drains whoever cast the nonblack spell — at four seats, a third
/// player, not "the opponent".
#[test]
fn soot_imp_taxes_the_caster() {
    let mut g = pod(4);
    ready(&mut g, 0, catalog::soot_imp());
    let life = g.players[2].life;
    let bears = g.add_card_to_hand(2, catalog::grizzly_bears());
    g.active_player_idx = 2;
    cast_by(&mut g, 2, bears, None);
    assert_eq!(g.players[2].life, life - 1);
    assert_eq!(g.players[1].life, life);
}

/// Owlbear Shepherd draws at your end step only from 8 total power.
#[test]
fn owlbear_shepherd_needs_eight_power() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    ready(&mut g, 0, catalog::owlbear_shepherd());
    ready(&mut g, 0, catalog::ant_queen());
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "1 + 5 power");
    ready(&mut g, 0, catalog::grizzly_bears());
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "8 power");
}

/// Thran Temporal Gateway puts a historic permanent from hand onto the
/// battlefield, and nothing else.
#[test]
fn thran_temporal_gateway_cheats_historic_permanents() {
    let mut g = pod(2);
    let gate = ready(&mut g, 0, catalog::thran_temporal_gateway());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let devil = g.add_card_to_hand(0, catalog::devil_dinosaur());
    activate(&mut g, gate, 0, None);
    assert!(g.battlefield_find(devil).is_some(), "a legendary creature card is historic");
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Grizzly Bears"));
}

/// Loch Mare enters shrunk; each counter removed buys a card and grows it.
#[test]
fn loch_mare_spends_its_counters() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let mare = g.add_card_to_hand(0, catalog::loch_mare());
    cast_by(&mut g, 0, mare, None);
    assert_eq!(g.computed_permanent(mare).unwrap().power, 1);
    let hand = g.players[0].hand.len();
    activate(&mut g, mare, 0, None);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.computed_permanent(mare).unwrap().power, 2);
}

/// Return to the Light Realms returns every nonland permanent card, leaving
/// lands and spells.
#[test]
fn return_to_the_light_realms_skips_lands_and_spells() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::sol_ring());
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let spell = g.add_card_to_hand(0, catalog::return_to_the_light_realms());
    cast_by(&mut g, 0, spell, None);
    let names: Vec<_> = g.players[0].graveyard.iter().map(|c| c.definition.name).collect();
    assert!(names.contains(&"Forest") && names.contains(&"Lightning Bolt"), "{names:?}");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Sol Ring"));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Generous Revival's creature comes back with a +1/+1 counter.
#[test]
fn generous_revival_adds_a_counter() {
    let mut g = pod(2);
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::generous_revival());
    cast_by(&mut g, 0, spell, Some(Target::Permanent(bears)));
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Traitorous Greed steals and pays two mana of one color.
#[test]
fn traitorous_greed_steals_and_pays() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let greed = g.add_card_to_hand(0, catalog::traitorous_greed());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: greed,
        target: Some(Target::Permanent(angel)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!(cp.controller, 0);
    assert!(cp.keywords().contains(&Keyword::Haste));
    let pool: u32 = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green]
        .into_iter()
        .map(|c| g.players[0].mana_pool.amount(c))
        .sum();
    assert_eq!(pool, 2);
}
