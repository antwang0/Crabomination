//! Heart of Kiran's loyalty crew (CR 118.9): the bot pays a Vehicle's crew with
//! a loyalty counter when a planeswalker can spare one (two or more on it).

use crate::card::{CardId, CounterType};
use crate::game::GameState;
use crate::game::types::GameAction;

/// The no-creature `Crew` of `vehicle`, when a planeswalker with loyalty to
/// spare pays it and the engine accepts it.
pub(super) fn loyalty_crew(state: &GameState, vehicle: CardId) -> Option<GameAction> {
    let pw = state.crew_loyalty_source(vehicle)?;
    if state.battlefield.find_by_id(pw)?.counter_count(CounterType::Loyalty) < 2 {
        return None;
    }
    let action = GameAction::Crew { vehicle, crew_creatures: Vec::new() };
    state.would_accept(action.clone()).then_some(action)
}
