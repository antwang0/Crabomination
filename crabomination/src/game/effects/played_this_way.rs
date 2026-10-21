//! `Effect::WhenPlayedThisWayThisTurn` — "when you play a card this way,
//! [body]" (Fires of Mount Doom). The grant's cards are registered on
//! `TurnRegistries::played_this_way_triggers`; the play (a cast or a land
//! drop through the card's may-play permission) spends the registration and
//! puts `body` on the stack as a trigger of the granting source (CR 603.2),
//! with the played card as its subject.

use super::{EffectContext, EntityRef};
use crate::effect::{Effect, Selector};
use crate::game::GameState;

impl GameState {
    pub(super) fn register_played_this_way(&mut self, what: &Selector, body: &Effect, ctx: &EffectContext) {
        let Some(src) = ctx.source else { return };
        for ent in self.resolve_selector(what, ctx) {
            let Some(cid) = ent.as_card_id() else { continue };
            self.turn.played_this_way_triggers.push((cid, src, ctx.controller, body.clone()));
        }
    }

    /// `card` was just played through its permission.
    pub(crate) fn fire_played_this_way(&mut self, card: crate::card::CardId) {
        if !self.turn.played_this_way_triggers.iter().any(|(c, ..)| *c == card) {
            return;
        }
        let mut fired = Vec::new();
        self.turn.played_this_way_triggers.retain(|(c, src, ctrl, body)| {
            if *c == card {
                fired.push((*src, *ctrl, body.clone()));
                false
            } else {
                true
            }
        });
        for (src, ctrl, body) in fired {
            let ctx = EffectContext::for_ability(src, ctrl, None);
            self.push_reflexive_trigger(&body, &ctx, EntityRef::Card(card), 0);
        }
    }
}
