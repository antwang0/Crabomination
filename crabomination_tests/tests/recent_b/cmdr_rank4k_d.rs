//! Commander: one-card gaps of EDHREC ranks 3501-4500 (`decks::cmdr_rank4k_d`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
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

fn loyalty(g: &mut GameState, id: CardId, index: usize) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: id,
        ability_index: index,
        target: None,
        x_value: None,
    })
    .expect("loyalty");
    drain_stack(g);
}

/// CR 614.1a — Goblin Goliath doubles damage to opponents (players only)
/// for the turn, and makes a Goblin per opponent.
#[test]
fn goblin_goliath_doubles_damage_to_opponent_players() {
    let mut g = pod(4);
    let goliath = g.add_card_to_hand(0, catalog::goblin_goliath());
    cast(&mut g, goliath, None, None);
    let goblins = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Goblin")
        .count();
    assert_eq!(goblins, 3, "one per opponent");
    g.clear_sickness(goliath);
    activate(&mut g, goliath, 0, None);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(2)), None);
    assert_eq!(g.players[2].life, 14, "3 doubled to 6");
    let ogre = ready(&mut g, 2, catalog::serra_angel());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(ogre)), None);
    assert!(g.battlefield_find(ogre).is_some(), "a permanent takes the printed 3");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(0)), None);
    assert_eq!(g.players[0].life, 17, "not doubled to its own controller");
}

/// CR 701.21 — By Invitation Only: every player sacrifices the chosen number.
#[test]
fn by_invitation_only_takes_the_chosen_count_from_everyone() {
    let mut g = pod(3);
    for seat in 0..3 {
        for _ in 0..3 {
            ready(&mut g, seat, catalog::grizzly_bears());
        }
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    let s = g.add_card_to_hand(0, catalog::by_invitation_only());
    cast(&mut g, s, None, None);
    for seat in 0..3 {
        let left = g.battlefield.iter().filter(|c| c.controller == seat).count();
        assert_eq!(left, 1, "seat {seat}");
    }
}

/// CR 122.1 — Dramatist's Puppet adds a helpful kind to its controller's own
/// permanent and strips it from an opponent's.
#[test]
fn dramatists_puppet_adds_to_ours_and_strips_theirs() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(mine).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let p = g.add_card_to_hand(0, catalog::dramatists_puppet());
    cast(&mut g, p, None, None);
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(theirs).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    for id in [mine, theirs] {
        let e = Effect::AddOrRemoveCounterOfEachKindOn { what: Selector::ExactObjects(vec![id]) };
        g.resolve_effect(&e, &ctx).unwrap();
    }
    assert!(plus(&g, mine) >= 2);
    assert_eq!(plus(&g, theirs), 1);
}

/// Way of the Pyromancer: an empowered Jace token has "[+1]: Add {R}."
#[test]
fn way_of_the_pyromancer_grants_a_mana_plus_one() {
    let mut g = pod(2);
    let way = g.add_card_to_hand(0, catalog::way_of_the_pyromancer());
    cast(&mut g, way, None, None);
    let jace = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.name.contains("Jace"))
        .map(|c| c.id)
        .expect("Jace token");
    g.players[0].mana_pool.empty();
    let n = g.battlefield_find(jace).unwrap().definition.loyalty_abilities.len();
    loyalty(&mut g, jace, n);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
}
