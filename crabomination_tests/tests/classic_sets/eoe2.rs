//! Edge of Eternities (EOE) gap closure.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

/// Every EOE gap factory is registered under its printed name.
#[test]
fn eoe2_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::chorale_of_the_void as fn() -> crabomination::card::CardDefinition,
        catalog::famished_worldsire,
        catalog::lightstall_inquisitor,
        catalog::requiem_monolith,
        catalog::sothera_the_supervoid,
        catalog::the_dominion_bracelet,
        catalog::moonlit_meditation,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

/// The Aura's attack trigger reanimates out of the *defending* player's
/// graveyard, tapped and attacking.
#[test]
fn chorale_of_the_void_reanimates_from_the_defender() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::chorale_of_the_void());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(host);
    let corpse = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.battlefield_find_mut(host).unwrap().summoning_sick = false;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: host,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let raised = g.battlefield_find(corpse).expect("reanimated under your control");
    assert_eq!(raised.controller, 0);
    assert!(raised.tapped, "enters tapped");
    assert!(g.attack_for(corpse).is_some(), "and attacking");
}

/// Void off → the Aura eats itself at your end step.
#[test]
fn chorale_of_the_void_sacrifices_itself_without_void() {
    let mut g = two_player_game();
    let aura = g.add_card_to_battlefield(0, catalog::chorale_of_the_void());
    advance_to(&mut g, TurnStep::End);
    assert!(g.battlefield_find(aura).is_none(), "Void was off");
}

/// Devour land 3 eats lands, not creatures, and drops three counters each.
#[test]
fn famished_worldsire_devours_lands() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    let sire = g.move_card_to_battlefield_for_test(0, catalog::famished_worldsire());
    drain_stack(&mut g);
    let c = g.battlefield_find(sire).expect("Worldsire");
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied(), Some(6));
    assert!(g.battlefield_find(bear).is_some(), "creatures aren't devoured");
}

/// Ward {3} is printed on the Worldsire.
#[test]
fn famished_worldsire_has_ward_three() {
    let d = catalog::famished_worldsire();
    assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Ward(_))));
    assert_eq!((d.power, d.toughness), (0, 0));
}

/// The Inquisitor exiles an opponent's hand card and leaves it playable for its
/// own cost plus {1}.
#[test]
fn lightstall_inquisitor_exiles_and_taxes() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.move_card_to_battlefield_for_test(0, catalog::lightstall_inquisitor());
    drain_stack(&mut g);
    let exiled = g.exile.iter().find(|c| c.id == bolt).expect("Bolt exiled");
    assert_eq!(exiled.may_play_until.map(|p| p.player), Some(1));
    // {R} plus the {1} surcharge.
    assert_eq!(exiled.granted_alt_cast_cost_eot.as_ref().map(|c| c.cmc()), Some(2));
}

/// Sothera eats an opponent's creature per death of yours, then cashes in.
#[test]
fn sothera_exiles_and_cashes_in() {
    let mut g = two_player_game();
    let sothera = g.add_card_to_battlefield(0, catalog::sothera_the_supervoid());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(mine)));
    assert!(g.exile.iter().any(|c| c.id == theirs), "opponent exiled a creature");
    // Player 1 now controls no creatures, so the end step cashes Sothera in.
    advance_to(&mut g, TurnStep::End);
    assert!(g.battlefield_find(sothera).is_none(), "Sothera sacrificed itself");
    let back = g.battlefield_find(theirs).expect("returned under your control");
    assert_eq!(back.controller, 0);
    assert_eq!(back.counters.get(&CounterType::PlusOnePlusOne).copied(), Some(2));
}

/// The Monolith's grant makes damage draw-and-drain the creature's controller.
#[test]
fn requiem_monolith_grants_the_damage_trigger() {
    let mut g = two_player_game();
    let mono = g.add_card_to_battlefield(0, catalog::requiem_monolith());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for i in 0..4 {
        g.players[0].library.push(CardInstance::new(CardId(700 + i), catalog::forest(), 0));
    }
    g.decider = Box::new(ScriptedDecider::new(std::iter::repeat_n(
        DecisionAnswer::Bool(true),
        4,
    )));
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: mono,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the self-ping drew a card");
    assert_eq!(g.players[0].life, life - 1);
}

/// The Bracelet's granted ability is exile-the-Equipment, discounted by the
/// bearer's power.
#[test]
fn dominion_bracelet_discounts_by_power_and_exiles_itself() {
    let mut g = two_player_game();
    let bracelet = g.add_card_to_battlefield(0, catalog::the_dominion_bracelet());
    // A 12/12 bearer (+1/+1 from the Bracelet) drops {15} to {2}.
    let bearer = g.add_card_to_battlefield(0, catalog::gigantosaurus());
    g.battlefield_find_mut(bracelet).unwrap().attached_to = Some(bearer);
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bearer,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate the granted ability");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bracelet), "the Bracelet exiled itself");
}

/// Moonlit Meditation turns the turn's first token batch into copies of its
/// host — and only that batch.
#[test]
fn moonlit_meditation_replaces_the_first_token_batch() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::moonlit_meditation());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(host);
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::plains());
    }
    let first = g.add_card_to_hand(0, catalog::raise_the_alarm());
    let second = g.add_card_to_hand(0, catalog::raise_the_alarm());
    g.players[0].mana_pool.add(Color::White, 4);
    cast(&mut g, first, None);
    let bear_copies = g
        .battlefield
        .iter()
        .filter(|c| c.is_token && c.definition.name == "Grizzly Bears")
        .count();
    assert_eq!(bear_copies, 2, "both Soldiers came in as Bear copies");
    cast(&mut g, second, None);
    assert!(
        g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Soldier"),
        "the second batch is normal — the replacement is once per turn"
    );
}
