//! "You may cast [a spell] from your hand without paying its mana cost. If
//! you don't, …" — the shared body behind Kellan, the Kid and Baral and Kari
//! Zev.

use super::{EffectContext, PickValue};
use crate::card::{CardId, CardInstance};
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent, Target};

impl GameState {
    /// Offer the controller one hand card `eligible` accepts (a headless seat
    /// takes the highest mana value), cast it free, else run `else_`.
    pub(super) fn may_cast_from_hand_free(
        &mut self,
        eligible: impl Fn(&CardInstance) -> bool,
        prompt: &str,
        else_: &Effect,
        ctx: &EffectContext,
        effect: &Effect,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let hand = &self.players[me].hand;
        let candidates: Vec<(CardId, String)> =
            hand.iter().filter(|c| eligible(c)).map(|c| (c.id, c.definition.name.to_string())).collect();
        let auto = hand
            .iter()
            .filter(|c| eligible(c))
            .max_by_key(|c| c.definition.cost.cmc())
            .map(|c| vec![c.id])
            .unwrap_or_default();
        let picked = if candidates.is_empty() {
            Vec::new()
        } else {
            let Some(p) = self.choose_up_to_cards(
                me,
                prompt.into(),
                ctx.source.unwrap_or(CardId(0)),
                candidates,
                1,
                PickValue::Gain,
                effect,
                auto,
            ) else {
                return Ok(());
            };
            p
        };
        if let Some(pick) = picked.first().copied() {
            return self.run_effect(
                &Effect::CastWithoutPayingImmediate {
                    what: Selector::Target(0),
                    source_zone: crate::card::Zone::Hand,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                },
                &EffectContext { targets: vec![Target::Permanent(pick)], ..ctx.clone() },
                events,
            );
        }
        self.run_effect(else_, ctx, events)
    }
}
