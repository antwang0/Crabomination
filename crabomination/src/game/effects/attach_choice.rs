//! "You may attach to it any number of [Auras / Equipment]" — Bruna, Light of
//! Alabaster and Heavenly Blademaster. The controller picks the subset. And
//! its mirror, "for each one, you may attach it to a creature you control"
//! (Inventory Management): a host per attachment.

use super::{EffectContext, PickValue};
use crate::card::CardId;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// `Effect::AttachAnyNumberTo` — offer every permanent `what` resolves to
    /// (other than the host) and attach the chosen ones to `to`. A headless
    /// controller takes its own attachments that aren't on an opponent's
    /// permanent, so a Curse or a Pacifism it aimed elsewhere stays put.
    pub(super) fn attach_any_number_to(
        &mut self,
        what: &Selector,
        to: &Selector,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let Some(host) = self.resolve_selector(to, ctx).iter().find_map(|e| e.as_permanent_id()) else {
            return Ok(());
        };
        let candidates: Vec<(CardId, String)> = self
            .resolve_selector(what, ctx)
            .into_iter()
            .filter_map(|e| e.as_permanent_id())
            .filter(|&id| id != host)
            .filter_map(|id| self.battlefield_find(id).map(|c| (id, c.definition.name.to_string())))
            .collect();
        if candidates.is_empty() {
            return Ok(());
        }
        let hostile_host = |g: &GameState, id: CardId| {
            g.battlefield_find(id)
                .and_then(|c| c.attached_to)
                .and_then(|h| g.battlefield_find(h))
                .is_some_and(|h| h.controller != me)
        };
        let auto: Vec<CardId> = candidates
            .iter()
            .map(|(id, _)| *id)
            .filter(|&id| self.battlefield_find(id).is_some_and(|c| c.controller == me) && !hostile_host(self, id))
            .collect();
        let max = candidates.len() as u32;
        let Some(picked) = self.choose_up_to_cards(
            me,
            "Choose any number to attach".into(),
            ctx.source.unwrap_or(CardId(0)),
            candidates,
            max,
            PickValue::Gain,
            effect,
            auto,
        ) else {
            return Ok(());
        };
        for id in picked {
            if let Some(c) = self.battlefield_find_mut(id) {
                c.attached_to = Some(host);
                c.attached_to_player = None;
                events.push(GameEvent::AttachmentMoved { attachment: id, attached_to: Some(host) });
            }
        }
        Ok(())
    }

    /// `Effect::AttachEachToCreatureYouControl` — for each permanent `what`
    /// resolves to, its controller may move it onto a creature they control
    /// that it can legally be attached to (CR 301.5c / 303.4). Every host is
    /// asked first, then they move; headless, each goes to the greatest-power
    /// one (where it already is, nothing moves).
    pub(super) fn attach_each_to_creature_you_control(
        &mut self,
        what: &Selector,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let attachments: Vec<CardId> = self
            .resolve_selector(what, ctx)
            .into_iter()
            .filter_map(|e| e.as_permanent_id())
            .filter(|id| self.battlefield_find(*id).is_some_and(|c| c.controller == me))
            .collect();
        let mut creatures: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == me && self.computed_is_creature(c))
            .map(|c| c.id)
            .collect();
        creatures.sort_by_key(|&id| std::cmp::Reverse(self.battlefield_find(id).map_or(0, |c| self.effective_power_on(c))));
        let asks = self.seat_prompts(me) || !matches!(self.decider.kind(), crate::decision::DeciderKind::Auto);
        let mut cursor = 0;
        let mut moves: Vec<(CardId, CardId)> = Vec::new();
        for att in attachments {
            let Some(def) = self.battlefield_find(att).map(|c| c.definition.clone()) else { continue };
            let legal: Vec<CardId> = creatures
                .iter()
                .copied()
                .filter(|&h| h != att)
                .filter(|&h| match def.aura_enchant_filter() {
                    Some(f) if def.is_aura() => {
                        self.evaluate_requirement_static(f, &crate::game::types::Target::Permanent(h), me, Some(att))
                    }
                    _ => true,
                })
                .collect();
            let Some(&best) = legal.first() else { continue };
            let pick = if asks {
                let name = def.name.to_string();
                let targets = legal.iter().map(|&h| crate::game::types::Target::Permanent(h)).collect();
                match self.ask_seat_target_maybe_logged(&mut cursor, me, format!("Attach {name} to"), att, targets, effect, true) {
                    None => return Ok(()),
                    Some(Some(crate::game::types::Target::Permanent(h))) => Some(h),
                    Some(_) => None,
                }
            } else {
                Some(best)
            };
            if let Some(h) = pick {
                moves.push((att, h));
            }
        }
        self.clear_answer_log();
        for (att, h) in moves {
            if let Some(c) = self.battlefield_find_mut(att)
                && c.attached_to != Some(h)
            {
                c.attached_to = Some(h);
                c.attached_to_player = None;
                events.push(GameEvent::AttachmentMoved { attachment: att, attached_to: Some(h) });
            }
        }
        Ok(())
    }
}
