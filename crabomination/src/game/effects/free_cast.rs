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

impl GameState {
    /// Anrakyr the Traveller — offer one `filter` card from the controller's
    /// hand or graveyard whose mana value their life can cover (CR 119.4),
    /// cast it without paying its mana cost and bill that much life instead.
    /// A headless seat takes the costliest card that leaves it above 5 life.
    pub(super) fn may_cast_for_life(
        &mut self,
        filter: &crate::card::SelectionRequirement,
        ctx: &EffectContext,
        effect: &Effect,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        use crate::card::Zone;
        let me = ctx.controller;
        let life = self.players[me].life;
        let pl = &self.players[me];
        let eligible: Vec<(CardId, u32, Zone)> = pl
            .hand
            .iter()
            .map(|c| (c, Zone::Hand))
            .chain(pl.graveyard.iter().map(|c| (c, Zone::Graveyard)))
            .filter(|(c, _)| !c.definition.is_land() && self.evaluate_requirement_on_card(filter, c, me))
            .map(|(c, z)| (c.id, c.definition.cost.cmc(), z))
            .filter(|(_, mv, _)| i64::from(*mv) <= i64::from(life))
            .collect();
        if eligible.is_empty() {
            return Ok(());
        }
        let candidates: Vec<(CardId, String)> = eligible
            .iter()
            .filter_map(|(id, _, _)| self.find_card_anywhere(*id).map(|c| (*id, c.definition.name.to_string())))
            .collect();
        let auto = eligible
            .iter()
            .filter(|(_, mv, _)| i64::from(life) - i64::from(*mv) > 5)
            .max_by_key(|(_, mv, _)| *mv)
            .map(|(id, _, _)| vec![*id])
            .unwrap_or_default();
        let Some(picked) = self.choose_up_to_cards(
            me,
            "Cast which spell by paying life equal to its mana value?".into(),
            ctx.source.unwrap_or(CardId(0)),
            candidates,
            1,
            PickValue::Gain,
            effect,
            auto,
        ) else {
            return Ok(());
        };
        let Some(&(id, mv, zone)) = picked.first().and_then(|p| eligible.iter().find(|(id, _, _)| id == p)) else {
            return Ok(());
        };
        let Some(def) = self.find_card_anywhere(id).map(|c| c.definition.arc()) else { return Ok(()) };
        let auto_target = self.auto_target_for_effect_avoiding(&def.effect, me, Some(id));
        let cast = self.cast_card_for_free(me, id, zone, auto_target, vec![], None, None, false)?;
        events.extend(cast);
        self.pay_life_cost(me, mv);
        Ok(())
    }
}
