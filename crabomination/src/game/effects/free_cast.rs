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

impl GameState {
    /// `Effect::CastExiledFreeAsking` — the logged yes of a suspended exiled
    /// free-cast offer, then the cast with slot 0's target named by `player`
    /// (CR 601.2c), the engine's pick leading the ballot. A logged no (the
    /// decline already ran) or a card no longer in exile does nothing.
    pub(super) fn cast_exiled_free_asking(
        &mut self,
        card: CardId,
        player: usize,
        effect: &Effect,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        use crate::decision::DecisionAnswer;
        let yes = matches!(self.scratch.resolution_answer_log.first(), Some(DecisionAnswer::Bool(true)));
        if !yes || !self.exile.iter().any(|c| c.id == card) {
            self.clear_answer_log();
            return Ok(());
        }
        let Some(def) = self.find_card_anywhere(card).map(|c| c.definition.arc()) else {
            self.clear_answer_log();
            return Ok(());
        };
        let auto = self.auto_target_for_effect_avoiding(&def.effect, player, Some(card));
        let target = if def.effect.requires_target() {
            let filter = def.effect.target_filter_for_slot_in_mode(0, None).cloned();
            let mut legal: Vec<Target> = match &filter {
                Some(f) => self.with_frozen_layers(|s| {
                    s.battlefield
                        .iter()
                        .map(|c| Target::Permanent(c.id))
                        .chain((0..s.players.len()).map(Target::Player))
                        .filter(|t| {
                            s.evaluate_requirement_static(f, t, player, Some(card))
                                && s.check_target_legality(t, player).is_ok()
                        })
                        .collect()
                }),
                None => Vec::new(),
            };
            if let Some(a) = auto.clone() {
                legal.retain(|t| *t != a);
                legal.insert(0, a);
            }
            if legal.is_empty() {
                auto
            } else {
                let mut cursor = 1;
                let name = def.name.to_string();
                match self.ask_seat_target_logged(&mut cursor, player, format!("Choose a target for {name}"), card, legal, effect) {
                    Some(t) => Some(t),
                    None => return Ok(()),
                }
            }
        } else {
            None
        };
        self.clear_answer_log();
        let cast = self.cast_card_for_free(player, card, crate::card::Zone::Exile, target, vec![], None, None, false)?;
        events.extend(cast);
        Ok(())
    }
}
