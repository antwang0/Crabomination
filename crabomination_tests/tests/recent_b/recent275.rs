//! Functionality tests for `catalog::sets::decks::recent275`.

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::{drain_stack, two_player_game};

/// Stargaze at X=2 digs 4, banks 2, bins 2, and costs 2 life.
#[test]
fn stargaze_digs_and_pays_life() {
    let mut g = two_player_game();
    let mut ids = vec![];
    for _ in 0..4 {
        ids.push(g.add_card_to_library(0, catalog::grizzly_bears()));
    }
    let life = g.players[0].life;
    // Pick the first two of the four looked at.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(ids[..2].to_vec())]));
    let ctx = EffectContext { x_value: 2, ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::stargaze().effect.clone(), &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), 2, "banked X=2 cards");
    assert_eq!(g.players[0].graveyard.len(), 2, "binned the other two");
    assert_eq!(g.players[0].life, life - 2, "lost X life");
}

/// Axgard Artisan mints a Treasure the first time counters land on it each turn.
#[test]
fn axgard_artisan_counter_makes_treasure() {
    let mut g = two_player_game();
    let ax = g.add_card_to_battlefield(0, catalog::axgard_artisan());
    let effect = catalog::axgard_artisan().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_ability(ax, 0, None);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"), "a Treasure entered");
}

/// Bloated Processor incubates its power when it dies.
#[test]
fn bloated_processor_death_incubates() {
    let mut g = two_player_game();
    let bp = g.add_card_to_battlefield(0, catalog::bloated_processor());
    g.battlefield_find_mut(bp).unwrap().damage = 100;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name.contains("Incubator")),
        "an Incubator token was created"
    );
}

/// Bloated Processor's activated ability sacrifices another Phyrexian.
#[test]
fn bloated_processor_sac_cost() {
    let def = catalog::bloated_processor();
    let ab = &def.activated_abilities[0];
    assert!(ab.sac_other_filter.is_some(), "sacrifices another Phyrexian");
}

/// Harvestrite Host draws only on the second resolution in a turn.
#[test]
fn harvestrite_host_second_resolution_draws() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::harvestrite_host());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let effect = catalog::harvestrite_host().triggered_abilities[0].effect.clone();
    let ctx = EffectContext { targets: vec![crabomination::game::Target::Permanent(host)], ..EffectContext::for_ability(host, 0, None) };
    let start = g.players[0].hand.len();
    g.resolve_effect(&effect, &ctx).unwrap(); // 1st: pump only
    assert_eq!(g.players[0].hand.len(), start, "no draw on the first resolution");
    g.resolve_effect(&effect, &ctx).unwrap(); // 2nd: pump + draw
    assert_eq!(g.players[0].hand.len(), start + 1, "draws on the second resolution");
}
