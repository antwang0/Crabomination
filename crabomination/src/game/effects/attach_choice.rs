//! "You may attach to it any number of [Auras / Equipment]" — Bruna, Light of
//! Alabaster and Heavenly Blademaster. The controller picks the subset.

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
}
