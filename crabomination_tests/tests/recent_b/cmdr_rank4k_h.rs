//! Commander: one-card gaps of EDHREC ranks 3501-4500 (`decks::cmdr_rank4k_h`).

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

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: x,
        },
    );
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(
        g,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        },
    );
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne)
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
    g.step = TurnStep::PreCombatMain;
}

/// Gideon the Oathless pings an opponent whose creature enters.
#[test]
fn gideon_the_oathless_taxes_opposing_arrivals() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::gideon_the_oathless());
    let bear = g.add_card_to_hand(2, catalog::grizzly_bears());
    g.players[2].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 2;
    g.priority.player_with_priority = 2;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[2].life, 19);
    assert_eq!(g.players[1].life, 20);
}

/// Blood Cultist grows when a creature it damaged this turn dies.
#[test]
fn blood_cultist_grows_off_its_kills() {
    let mut g = pod(2);
    let cultist = ready(&mut g, 0, catalog::blood_cultist());
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    activate(&mut g, cultist, 0, Some(Target::Permanent(elf)));
    assert!(g.battlefield_find(elf).is_none());
    assert_eq!(plus(&g, cultist), 1);
}

/// CR 601.2f — Arachne's chosen card type costs {1} more for every player.
#[test]
fn arachne_taxes_the_chosen_type() {
    let mut g = pod(2);
    let arachne = g.add_card_to_hand(0, catalog::arachne_psionic_weaver());
    cast(&mut g, arachne, None, None);
    let chosen = g
        .battlefield_find(arachne)
        .unwrap()
        .chosen_card_type
        .clone()
        .expect("a type was chosen");
    assert_ne!(chosen, crabomination::card::CardType::Creature);
    let probe = match chosen {
        crabomination::card::CardType::Instant => catalog::lightning_bolt(),
        crabomination::card::CardType::Sorcery => catalog::divination(),
        crabomination::card::CardType::Artifact => catalog::sol_ring(),
        crabomination::card::CardType::Enchantment => catalog::pacifism(),
        _ => return,
    };
    let id = g.add_card_to_hand(1, probe);
    let card = g.players[1].hand.iter().find(|c| c.id == id).unwrap().clone();
    assert_eq!(crabomination::game::actions::extra_cost_for_spell(&g, 1, &card, None), 1);
}

/// Arachne looks at an opponent's hand as she enters, and the headless pick
/// names the type that hand holds most of.
#[test]
fn arachne_looks_at_a_hand_before_choosing() {
    let mut g = pod(2);
    g.add_card_to_hand(1, catalog::divination());
    g.add_card_to_hand(1, catalog::divination());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let arachne = g.add_card_to_hand(0, catalog::arachne_psionic_weaver());
    cast(&mut g, arachne, None, None);
    assert!(g.hands_revealed_to.contains(&(0, 1)), "the hand was looked at");
    assert_eq!(g.battlefield_find(arachne).unwrap().chosen_card_type, Some(crabomination::card::CardType::Sorcery));
}

/// The Arkenstone pumps your team and draws at your end step.
#[test]
fn the_arkenstone_pumps_and_draws() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::the_arkenstone());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    g.add_card_to_library(0, catalog::island());
    step(&mut g, TurnStep::End);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Captain Marvel exiles on entry (its controller gains the power) and
/// gains you life for any creature exiled from the battlefield.
#[test]
fn captain_marvel_exiles_and_both_sides_gain() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::serra_angel());
    let cm = g.add_card_to_hand(0, catalog::captain_marvel_shooting_star());
    cast(&mut g, cm, None, None);
    assert_eq!(g.players[1].life, 24, "Swords-style life for the Angel's controller");
    assert_eq!(g.players[0].life, 24, "and the exile watch for you");
}

/// Celebrate the Mountain-king holds one permanent per opponent until it
/// leaves.
#[test]
fn celebrate_the_mountain_king_holds_one_per_opponent() {
    let mut g = pod(3);
    let a = ready(&mut g, 1, catalog::sol_ring());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let c = g.add_card_to_hand(0, catalog::celebrate_the_mountain_king());
    cast(&mut g, c, None, None);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}
