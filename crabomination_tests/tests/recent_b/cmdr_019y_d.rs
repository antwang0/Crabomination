//! Commander: the 2026-10-17 (`019Y6J4R`) fourth closer batch
//! (`decks::cmdr_019y_d`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::card::Value;
use crabomination::effect::{Effect, Selector};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(6);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    cast_mode(g, id, target, None)
}

fn cast_mode(
    g: &mut GameState,
    id: CardId,
    target: Option<Target>,
    mode: Option<usize>,
) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode,
        x_value: None,
    })?;
    drain_stack(g);
    Ok(())
}










fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

/// Uldaros Theorix, cast, exiles a creature and an instant from your graveyard
/// and casts copies of both (total mana value 3 ≤ 6); the creature copy is a
/// token and the cards stay exiled.
#[test]
fn uldaros_theorix_casts_copies_of_one_card_per_type() {
    let mut g = pod(2);
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let ut = g.add_card_to_hand(0, catalog::uldaros_theorix());
    flood(&mut g, 0);
    cast(&mut g, ut, None).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == bears) && g.exile.iter().any(|c| c.id == bolt));
    let copy = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Grizzly Bears");
    assert!(copy.is_some_and(|c| c.is_token), "a token copy of the Bears");
}

/// Uldaros put onto the battlefield without being cast does nothing.
#[test]
fn uldaros_theorix_needs_to_be_cast() {
    let mut g = pod(2);
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ut = g.add_card_to_graveyard(0, catalog::uldaros_theorix());
    let raise = g.add_card_to_hand(0, catalog::zombify());
    flood(&mut g, 0);
    cast(&mut g, raise, Some(Target::Permanent(ut))).expect("reanimate");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bears));
}

/// Beast flies once a +1/+1 counter went on him this turn.
#[test]
fn beast_flies_after_a_counter() {
    let mut g = pod(2);
    let beast = ready(&mut g, 0, catalog::beast_erudite_aerialist());
    assert!(!g.computed_permanent(beast).unwrap().keywords().contains(&Keyword::Flying));
    let add = Effect::AddCounter {
        what: Selector::ExactObjects(vec![beast]),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    };
    g.resolve_effect(&add, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert!(g.computed_permanent(beast).unwrap().keywords().contains(&Keyword::Flying));
}

/// Hired Hexblade draws only when Treasure mana paid for it.
#[test]
fn hired_hexblade_draws_off_treasure_mana() {
    let mut g = pod(2);
    let t = crabomination_base::tokens::treasure_token();
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Color(Color::Black),
        DecisionAnswer::Color(Color::Black),
    ]));
    for _ in 0..2 {
        let id = g.add_token_to_battlefield(0, &t);
        activate(&mut g, id, 0, None).expect("Treasure for mana");
    }
    let hh = g.add_card_to_hand(0, catalog::hired_hexblade());
    let hand = g.players[0].hand.len();
    cast(&mut g, hh, None).expect("two Treasures pay {1}{B}");
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    let hh2 = g.add_card_to_hand(0, catalog::hired_hexblade());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    cast(&mut g, hh2, None).expect("cast from the pool");
    assert_eq!(g.players[0].hand.len(), hand - 1, "no Treasure, no card");
}

/// Ultron's Auxiliary grows when an artifact card hits your graveyard, from
/// the battlefield or from your hand.
#[test]
fn ultrons_auxiliary_grows_off_artifacts_to_your_graveyard() {
    let mut g = pod(2);
    let ua = ready(&mut g, 0, catalog::ultrons_auxiliary());
    let rock = ready(&mut g, 0, catalog::sol_ring());
    let shatter = g.add_card_to_hand(0, catalog::shatter());
    flood(&mut g, 0);
    cast(&mut g, shatter, Some(Target::Permanent(rock))).expect("shatter");
    assert_eq!(g.battlefield_find(ua).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Ragamuffin Raptor returns a creature card from your graveyard.
#[test]
fn ragamuffin_raptor_regrows_a_creature() {
    let mut g = pod(2);
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let rr = g.add_card_to_hand(0, catalog::ragamuffin_raptor());
    flood(&mut g, 0);
    cast(&mut g, rr, None).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == bears));
}

/// Urban Retreat comes down from hand for {2} by returning a tapped creature,
/// entering tapped.
#[test]
fn urban_retreat_swaps_in_for_a_tapped_creature() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let ur = g.add_card_to_hand(0, catalog::urban_retreat());
    flood(&mut g, 0);
    activate(&mut g, ur, 1, None).expect("from hand");
    assert!(g.battlefield_find(ur).is_some_and(|c| c.tapped));
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Soulbright Seeker's third resolution in a turn adds {R}{R}{R}{R}.
#[test]
fn soulbright_seeker_pays_out_on_the_third_activation() {
    let mut g = pod(2);
    let ss = ready(&mut g, 0, catalog::soulbright_seeker());
    g.players[0].mana_pool.add(Color::Red, 3);
    for _ in 0..2 {
        activate(&mut g, ss, 0, Some(Target::Permanent(ss))).expect("activate");
    }
    assert_eq!(g.players[0].mana_pool.total(), 1);
    activate(&mut g, ss, 0, Some(Target::Permanent(ss))).expect("third");
    assert_eq!(g.players[0].mana_pool.total(), 4, "four red");
}

/// Ith untaps an attacker and fogs it both ways.
#[test]
fn ith_high_arcanist_mazes_an_attacker() {
    let mut g = pod(2);
    let ith = ready(&mut g, 0, catalog::ith_high_arcanist());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    activate(&mut g, ith, 0, Some(Target::Permanent(bear))).expect("maze");
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    let life = g.players[0].life;
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, life);
}
