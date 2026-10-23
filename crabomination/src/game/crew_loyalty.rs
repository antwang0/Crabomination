//! Heart of Kiran's alternative crew cost (CR 118.9): one loyalty counter from
//! a planeswalker the Vehicle's controller controls, in place of the crew.

use super::GameState;
use crate::card::{CardId, CardType, CounterType};
use crate::effect::StaticEffect;

impl GameState {
    /// The planeswalker whose loyalty counter would pay `vehicle`'s crew cost:
    /// its controller's most loyal one, the earliest on a tie. `None` when the
    /// Vehicle has no such alternative or no loyalty counter is there.
    pub(crate) fn crew_loyalty_source(&self, vehicle: CardId) -> Option<CardId> {
        let v = self.battlefield.find_by_id(vehicle)?;
        let alt = v.definition.static_abilities.iter().any(|sa| {
            matches!(self.active_static(&sa.effect, v), Some(StaticEffect::CrewByRemovingLoyaltyCounter))
        });
        if !alt {
            return None;
        }
        let mut best: Option<(u32, CardId)> = None;
        for c in self.battlefield.iter().filter(|c| c.controller == v.controller) {
            let n = c.counter_count(CounterType::Loyalty);
            if n == 0 || best.is_some_and(|(b, _)| b >= n) {
                continue;
            }
            if self.computed_permanent_on(c).is_some_and(|cp| cp.card_types().contains(&CardType::Planeswalker)) {
                best = Some((n, c.id));
            }
        }
        best.map(|(_, id)| id)
    }
}
