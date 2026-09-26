//! Bot answers for Cosima, God of the Voyage's two asks. The generic "may"
//! screen reads "exile this" as a self-cost and would never set sail, and it
//! takes every free upside, so a voyage counter would be added for ever.

use crate::card::{CardId, CounterType};
use crate::effect::{Effect, Selector, TriggerZone, ZoneDest};
use crate::game::GameState;

/// Voyage counters the bot collects before bringing Cosima home.
const VOYAGE_TARGET: u32 = 3;

/// `Some(answer)` when `source`'s ask is one of the voyage's: set sail at
/// upkeep with a land in hand to make the trip pay; on the voyage, add a
/// counter until [`VOYAGE_TARGET`], then come home.
pub(super) fn voyage_answer(state: &GameState, source: CardId) -> Option<bool> {
    if let Some(c) = state.exile.iter().find(|c| c.id == source && c.exiled_with == Some(source)) {
        let voyaging = c.definition.triggered_abilities.iter().any(|t| t.event.zone == TriggerZone::WhileSelfExiled);
        return voyaging.then(|| c.counter_count(CounterType::Voyage) < VOYAGE_TARGET);
    }
    let c = state.battlefield.find_by_id(source)?;
    let sets_sail = c.definition.triggered_abilities.iter().any(|t| {
        matches!(&t.effect, Effect::MayDo { body, .. }
            if matches!(**body, Effect::Move { what: Selector::This, to: ZoneDest::ExileWithSourceStamp }))
    }) && c.definition.triggered_abilities.iter().any(|t| t.event.zone == TriggerZone::WhileSelfExiled);
    sets_sail.then(|| state.players[c.owner].hand.iter().any(|h| h.definition.is_land()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sets sail only with a land to play; on the voyage, counters to three.
    #[test]
    fn the_bot_sails_with_a_land_and_comes_home_at_three() {
        let mut g = crate::game::two_player_game();
        let cosima = g.add_card_to_battlefield(0, crate::catalog::cosima_god_of_the_voyage());
        assert_eq!(voyage_answer(&g, cosima), Some(false), "no land in hand");
        g.add_card_to_hand(0, crate::catalog::island());
        assert_eq!(voyage_answer(&g, cosima), Some(true));
        let bear = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        assert_eq!(voyage_answer(&g, bear), None, "not a voyage");
        g.remove_from_battlefield_to_exile(cosima);
        let c = g.exile.iter_mut().find(|c| c.id == cosima).unwrap();
        c.exiled_with = Some(cosima);
        c.add_counters(CounterType::Voyage, 2);
        assert_eq!(voyage_answer(&g, cosima), Some(true), "two: one more");
        g.exile.iter_mut().find(|c| c.id == cosima).unwrap().add_counters(CounterType::Voyage, 1);
        assert_eq!(voyage_answer(&g, cosima), Some(false), "three: home");
    }
}
