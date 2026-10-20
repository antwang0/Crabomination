//! Panglacial Wurm — "While you're searching your library, you may cast this
//! card from your library." Offered by the generic library search
//! (`Effect::Search` / `SearchPickedBy` / `SearchZones`) once the search is
//! under way and before anything is found (2006-07-15 rulings: the cast Wurm
//! can't be found, and several can be cast in one search). The spell is cast
//! as the search resolves, paying its own cost.

use super::EffectContext;
use crate::card::{CardId, Zone};
use crate::effect::{Effect, Selector, StaticEffect};
use crate::game::GameState;
use crate::game::types::{GameEvent, Target};

impl GameState {
    /// Offer `p` each castable-while-searching card in their library. A seat
    /// that prompts isn't offered (the ask would need a suspend inside the
    /// search arm, ahead of its own stashed pick).
    pub(super) fn offer_casts_while_searching(&mut self, p: usize, ctx: &EffectContext, events: &mut Vec<GameEvent>) {
        let castable = |c: &crate::card::CardInstance| {
            c.definition.static_abilities.iter().any(|sa| matches!(sa.effect, StaticEffect::CastFromLibraryWhileSearching))
        };
        if !self.players[p].library.iter().any(castable) || self.seat_prompts(p) {
            return;
        }
        let ids: Vec<CardId> = self.players[p].library.iter().filter(|c| castable(c)).map(|c| c.id).collect();
        for id in ids {
            let Some(cost) = self.players[p].library.iter().find(|c| c.id == id).map(|c| c.definition.cost.clone()) else {
                continue;
            };
            if !self.could_pay_cost(p, &cost) {
                continue;
            }
            // The immediate cast asks "cast it?" itself and pays the card's cost.
            let free_cast = Effect::CastWithoutPayingImmediate {
                what: Selector::Target(0),
                source_zone: Zone::Library,
                exile_after: false,
                copy: false,
                reduce_generic: 0,
                pay_own_cost: true,
            };
            let cast_ctx = EffectContext { targets: vec![Target::Permanent(id)], ..ctx.clone() };
            let _ = self.run_effect(&free_cast, &cast_ctx, events);
        }
    }
}
