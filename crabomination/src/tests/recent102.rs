//! Functionality tests for `catalog::sets::decks::recent102` — deferred TDM/DFT
//! staples unblocked by new primitives.

use crate::card::CounterType;
use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::mana::Color;

/// Surrak draws when an opponent's spell targets a creature you control, but not
/// when you target your own creature.
#[test]
fn surrak_draws_on_opponent_targeting_your_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::surrak_elusive_hunter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let hand0 = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.cast_spell(bolt, Some(Target::Permanent(bear)), vec![], None, None)
        .expect("opponent bolts your creature");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "Surrak drew when your creature was targeted");
}

/// Effortless Master enters with two +1/+1 counters only after two spells cast.
#[test]
fn effortless_master_enters_bigger_after_two_spells() {
    let mut g = two_player_game();
    // Zero spells cast → no counters.
    let m0 = g.move_card_to_battlefield_for_test(0, catalog::effortless_master());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(m0).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    // Two spells cast this turn → enters with two counters.
    g.players[0].spells_cast_this_turn = 2;
    let m2 = g.move_card_to_battlefield_for_test(0, catalog::effortless_master());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(m2).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Stalwart Successor grafts one extra counter the first time a creature you
/// control gets counters each turn — but only once per creature per turn.
#[test]
fn stalwart_successor_first_counter_each_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::stalwart_successor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bump = |g: &mut GameState, id| {
        g.battlefield_find_mut(id).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        g.dispatch_triggers_for_events(&[GameEvent::CounterAdded {
            card_id: id,
            counter_type: CounterType::PlusOnePlusOne,
            count: 1,
        }]);
        drain_stack(g);
    };
    // First counter placement this turn → Stalwart adds one more (2 total).
    bump(&mut g, bear);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    // Second placement same turn → no bonus (2 + 1 = 3, not 4).
    bump(&mut g, bear);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}
