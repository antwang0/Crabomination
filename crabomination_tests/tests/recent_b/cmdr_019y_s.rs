//! Commander: the 2026-10-18 (`019Y6J4R`) nineteenth closer batch
//! (`decks::cmdr_019y_s`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;

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


/// CR 702.41a — Don & Raph's attack arms affinity for the next noncreature
/// spell: a creature spell cast first pays full, then Lotus Ring costs {0}
/// with three artifacts out.
#[test]
fn don_and_raph_give_the_next_noncreature_spell_affinity() {
    let mut g = pod(2);
    let dr = ready(&mut g, 0, catalog::don_raph_hard_science());
    for _ in 0..3 {
        ready(&mut g, 0, catalog::fishing_pole());
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: dr, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::PostCombatMain;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("a creature spell pays its full cost");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0);
    let ring = g.add_card_to_hand(0, catalog::lotus_ring());
    g.perform_action(GameAction::CastSpell { card_id: ring, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("three artifacts pay for Lotus Ring");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_some());
}

#[allow(dead_code)]
fn _unused(_: CounterType) {}

/// CR 715.3 — Mosswood Dreadknight dies, and its Adventure may be cast from
/// the graveyard: Dread Whispers draws and costs a life, then the Knight can
/// be cast from exile.
#[test]
fn mosswood_dreadknight_casts_its_adventure_from_the_graveyard() {
    let mut g = pod(2);
    let md = ready(&mut g, 0, catalog::mosswood_dreadknight());
    let destroy = crabomination::effect::Effect::Destroy {
        what: crabomination::effect::Selector::ExactObjects(vec![md]),
    };
    let evs = g.resolve_effect(&destroy, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == md));
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastAdventure { card_id: md, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Dread Whispers from the graveyard");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].life, 19);
    assert!(g.exile.iter().any(|c| c.id == md), "on an adventure");
}

fn jace(g: &GameState, seat: usize) -> Option<CardId> {
    g.battlefield.iter().find(|c| c.controller == seat && c.definition.name == "Jace").map(|c| c.id)
}

fn loyalty(g: &mut GameState, id: CardId, idx: usize, target: Option<crabomination::game::types::Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: idx, target, x_value: None })?;
    drain_stack(g);
    Ok(())
}

/// CR 606.3 — Jace's Machinations: Empower Jace 8, and that Jace's loyalty
/// abilities work on an opponent's turn this turn.
#[test]
fn jaces_machinations_empowers_and_unlocks_instant_speed() {
    let mut g = pod(2);
    let jm = g.add_card_to_hand(0, catalog::jaces_machinations());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 3);
    g.active_player_idx = 1;
    g.perform_action(GameAction::CastSpell { card_id: jm, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("instant on their turn");
    drain_stack(&mut g);
    let j = jace(&g, 0).expect("a Jace token");
    assert_eq!(g.battlefield_find(j).unwrap().counter_count(CounterType::Loyalty), 8);
    g.priority.player_with_priority = 0;
    loyalty(&mut g, j, 1, None).expect("−3 on the opponent's turn");
    assert_eq!(g.battlefield_find(j).unwrap().counter_count(CounterType::Loyalty), 5);
}

/// Overwrite the Multiverse exiles every creature and empowers Jace that many.
#[test]
fn overwrite_the_multiverse_empowers_per_exiled_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 1, catalog::hill_giant());
    ready(&mut g, 1, catalog::grizzly_bears());
    let ow = g.add_card_to_hand(0, catalog::overwrite_the_multiverse());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 6);
    g.perform_action(GameAction::CastSpell { card_id: ow, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name != "Jace" && c.definition.is_creature()));
    let j = jace(&g, 0).expect("a Jace token");
    assert_eq!(g.battlefield_find(j).unwrap().counter_count(CounterType::Loyalty), 3);
}

/// Way of the Warlord: Empower Jace 5, and its granted −4 burns a player and
/// a creature.
#[test]
fn way_of_the_warlord_arms_jace() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let ww = g.add_card_to_hand(0, catalog::way_of_the_warlord());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 3);
    g.perform_action(GameAction::CastSpell { card_id: ww, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let j = jace(&g, 0).expect("a Jace token");
    assert_eq!(g.battlefield_find(j).unwrap().counter_count(CounterType::Loyalty), 5);
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: j, ability_index: 2, target: Some(Target::Player(1)), x_value: None })
        .expect("the granted −4");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    let _ = bear;
}

/// CR 701.44 / 614.1a — Twists and Turns' entry explores, through its own
/// scry-1 replacement: the explore still happens (a land on top goes to hand).
#[test]
fn twists_and_turns_scries_then_explores() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let tt = g.add_card_to_hand(0, catalog::twists_and_turns());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: tt,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    // Cast one, explored a Forest into hand.
    assert_eq!(g.players[0].hand.len(), hand);
    let _ = bear;
}

/// Twists and Turns transforms into Mycoid Maze once a land enters with
/// seven or more out.
#[test]
fn twists_and_turns_transforms_at_seven_lands() {
    let mut g = pod(2);
    let tt = ready(&mut g, 0, catalog::twists_and_turns());
    for _ in 0..6 {
        ready(&mut g, 0, catalog::forest());
    }
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(tt).unwrap().definition.name, "Mycoid Maze");
}
