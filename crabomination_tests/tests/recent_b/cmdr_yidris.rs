//! Commander: the Entropic Uprising precon (C16, Yidris,
//! `decks::cmdr_yidris`) and the primitives it needed.

use crabomination::card::{CardDefinition, CardType, CounterType, EventKind, EventScope, EventSpec, TriggeredAbility, Value};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::TurnStep;
use crabomination::game::*;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// CR 800.4a — "whenever a player loses the game": each other player's
/// permanent triggers once per departing player.
#[test]
fn cr_800_4a_a_permanent_sees_each_player_leave() {
    let watcher = || CardDefinition {
        name: "Test Watcher",
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerLeftGame, EventScope::SelfSource),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(5) },
        }],
        ..Default::default()
    };
    let mut g = pod(4);
    let mine = g.add_card_to_battlefield(0, watcher());
    let theirs = g.add_card_to_battlefield(3, watcher());
    g.players[1].life = 0;
    g.players[2].life = 0;
    g.check_state_based_actions();
    drain_stack(&mut g);
    for id in [mine, theirs] {
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 10, "two players left");
    }
}
