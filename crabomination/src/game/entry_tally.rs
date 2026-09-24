//! The per-turn entry tallies for a permanent reaching the battlefield
//! without a zone-move effect: a resolving permanent spell (CR 608.3) and a
//! persist / undying return. The token and effect-move paths count their own
//! entrants; these two only emitted `PermanentEntered`, whose funnel stamps
//! the creature log but not the counters, so Celebration ("two or more
//! nonland permanents entered the battlefield under your control this turn")
//! saw tokens only — two creature spells never turned Goddric on.

use super::GameState;

impl GameState {
    /// Count `card_id` (already on the battlefield) toward its controller's
    /// nonland-permanent and artifact entry tallies.
    pub(crate) fn tally_permanent_entry(&mut self, card_id: crate::card::CardId) {
        let Some(c) = self.battlefield_find(card_id) else { return };
        let (p, land, artifact) = (c.controller, c.definition.is_land(), c.definition.is_artifact());
        if artifact {
            self.players[p].artifacts_entered_this_turn += 1;
        }
        if !land {
            self.players[p].nonland_permanents_entered_this_turn += 1;
        }
    }
}
