//! Commander: the Invent Superiority precon (C16, Breya,
//! `decks::cmdr_breya`) and the primitives it needed.

use crabomination::card::{
    CardDefinition, CardType, CounterType, EnchantmentSubtype, EventKind, EventScope, EventSpec, Subtypes,
    TriggeredAbility, Value,
};
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

/// CR 800.4a — "when enchanted player loses the game": an Aura on the
/// departing player triggers as they leave, reading its counters; an Aura on
/// a player still in the game doesn't.
#[test]
fn cr_800_4a_an_aura_on_a_departing_player_triggers_as_they_leave() {
    let curse = || CardDefinition {
        name: "Test Curse",
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EnchantedPlayerLeftGame, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..Default::default()
    };
    let mut g = pod(3);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let on_one = g.add_card_to_battlefield(0, curse());
    let on_two = g.add_card_to_battlefield(0, curse());
    g.battlefield_find_mut(on_one).unwrap().attached_to_player = Some(1);
    g.battlefield_find_mut(on_one).unwrap().counters.insert(CounterType::Spite, 2);
    g.battlefield_find_mut(on_two).unwrap().attached_to_player = Some(2);
    g.battlefield_find_mut(on_two).unwrap().counters.insert(CounterType::Spite, 3);
    let hand = g.players[0].hand.len();
    g.players[1].life = 0;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.players[1].eliminated);
    assert_eq!(g.players[0].hand.len(), hand + 2, "only the departed player's curse, at its 2 counters");
}
