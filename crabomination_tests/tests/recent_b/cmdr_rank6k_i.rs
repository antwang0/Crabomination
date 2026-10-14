//! Commander: one-card gaps of EDHREC ranks 5501-6500
//! (`decks::cmdr_rank6k_i`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Target, TurnStep};
use crabomination::game::*;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

/// CR 700.2d / 601.2c — Mikey & Mona's ETB with both modes: seat 0 puts two
/// +1/+1 counters on its own creature, seat 1 returns a creature card from its
/// graveyard to its hand.
#[test]
fn mikey_mona_both_modes_each_player() {
    let mut g = pod(2);
    let mm = ready(&mut g, 0, catalog::mikey_mona_mutant_sitters());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let dead = g.add_card_to_graveyard(1, catalog::hill_giant());
    let etb = catalog::mikey_mona_mutant_sitters().triggered_abilities[0].effect.clone();
    let mut ctx = EffectContext::for_spell(0, Some(Target::Player(0)), 0, 0);
    ctx.targets.push(Target::Player(1));
    ctx.source = Some(mm);
    let evs = g.resolve_effect(&etb, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let plus = |g: &GameState, id| g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne));
    assert_eq!(plus(&g, bears) + plus(&g, mm), 2, "two counters on one of seat 0's creatures");
    assert!(g.players[1].hand.iter().any(|c| c.id == dead), "seat 1 got its Hill Giant back");
}
