//! CR 614 — draw replacements that only apply to a draw from an empty
//! library (Out of the Tombs). Laboratory Maniac's win is read at the loss
//! site instead (`lose_to_empty_draw`).

use super::GameState;
use super::types::GameEvent;
use crate::effect::{Effect, PlayerRef, Selector, StaticEffect, ZoneDest};

impl GameState {
    /// Out of the Tombs — "If you would draw a card while your library has no
    /// cards in it, instead return a creature card from your graveyard to the
    /// battlefield." `true` when a static applied and a creature came back;
    /// `false` leaves the empty draw to CR 104.3c ("if you can't, you lose").
    pub(crate) fn reanimate_instead_of_empty_draw(&mut self, p: usize, events: &mut Vec<GameEvent>) -> bool {
        let Some(source) = self.battlefield.iter().find_map(|c| {
            (c.controller == p
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(self.active_static(&sa.effect, c), Some(StaticEffect::ReanimateInsteadOfDrawFromEmpty))
                }))
            .then_some(c.id)
        }) else {
            return false;
        };
        // The pick is automatic (greatest mana value, first in graveyard order
        // on a tie): the draw funnel can't suspend for an ask, and an ask
        // left pending here was owed by a seat the failed draw then decked
        // (seed 17703, four-seat pod).
        let Some(pick) = self.players[p]
            .graveyard
            .iter()
            .filter(|c| c.definition.is_creature())
            .rev()
            .max_by_key(|c| c.definition.cost.cmc())
            .map(|c| c.id)
        else {
            return false;
        };
        let ctx = super::effects::EffectContext::for_ability(source, p, None);
        let back = Effect::Move {
            what: Selector::ExactObjects(vec![pick]),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        };
        let _ = self.run_effect(&back, &ctx, events);
        !self.players[p].graveyard.iter().any(|c| c.id == pick)
    }
}
