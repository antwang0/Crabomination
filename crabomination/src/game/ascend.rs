//! CR 702.131b — Ascend on a permanent is a static ability: "any time you
//! control ten or more permanents and you don't have the city's blessing, you
//! get the city's blessing." The count can only reach ten as a permanent
//! enters, so the entry funnel asks here; seats with no ascend permanent pay
//! one flag read.

use super::GameState;
use crate::card::CardId;
use crate::effect::StaticEffect;

impl GameState {
    /// A permanent entered under some player's control: arm its controller if
    /// it has ascend, then grant the blessing if an armed seat reached ten.
    pub(super) fn apply_permanent_ascend(&mut self, entered: CardId) {
        let Some(c) = self.battlefield_find(entered) else { return };
        let p = c.controller;
        if self.players[p].city_blessing {
            return;
        }
        let has_ascend = |c: &crate::card::CardInstance| {
            c.definition.static_abilities.iter().any(|sa| matches!(sa.effect, StaticEffect::Ascend))
        };
        if !self.players[p].ascend_armed {
            if !has_ascend(c) {
                return;
            }
            self.players[p].ascend_armed = true;
        }
        let mut count = 0;
        let mut ascend = false;
        for c in self.battlefield.iter().filter(|c| c.controller == p) {
            count += 1;
            ascend |= has_ascend(c);
        }
        if ascend && count >= 10 {
            self.players[p].city_blessing = true;
        }
    }
}
