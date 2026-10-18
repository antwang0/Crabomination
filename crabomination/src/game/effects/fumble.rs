//! Fumble — "Return target creature to its owner's hand. Gain control of all
//! Auras and Equipment that were attached to it, then attach them to another
//! creature." (`Effect::BounceStealAttachmentsReattach`.) The attachments are
//! read before the bounce (they stop being attached as it leaves, CR 704.5m
//! comes only after resolution); the new host is one creature the controller
//! chooses, all of them going to it.

use super::EffectContext;
use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    pub(super) fn bounce_steal_attachments_reattach(
        &mut self,
        what: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let Some(victim) = self.resolve_selector(what, ctx).into_iter().find_map(|e| e.as_permanent_id()) else {
            return Ok(());
        };
        let gear: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.attached_to == Some(victim))
            .filter(|c| c.definition.is_aura() || c.definition.is_equipment())
            .map(|c| c.id)
            .collect();
        self.run_effect(
            &Effect::Move { what: Selector::ExactObjects(vec![victim]), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            ctx,
            events,
        )?;
        let gear: Vec<CardId> = gear.into_iter().filter(|id| self.battlefield_find(*id).is_some()).collect();
        if gear.is_empty() {
            return Ok(());
        }
        self.run_effect(
            &Effect::GainControl { what: Selector::ExactObjects(gear.clone()), to: None, duration: Duration::Permanent },
            ctx,
            events,
        )?;
        let me = ctx.controller;
        // Another creature: headless, the controller's strongest, else anyone's.
        let mut hosts: Vec<(CardId, String, bool, i32)> = self
            .battlefield
            .iter()
            .filter(|c| c.id != victim && self.computed_is_creature(c))
            .filter_map(|c| {
                self.computed_permanent(c.id).map(|cp| (c.id, c.definition.name.to_string(), c.controller == me, cp.power))
            })
            .collect();
        if hosts.is_empty() {
            return Ok(());
        }
        hosts.sort_by_key(|&(id, _, mine, power)| (!mine, std::cmp::Reverse(power), id));
        let mut cursor = 0;
        let Some(pick) = self.ask_seat_cards_logged(
            &mut cursor,
            me,
            "Attach them to another creature".into(),
            ctx.source.unwrap_or(CardId(0)),
            hosts.iter().map(|h| (h.0, h.1.clone())).collect(),
            1,
            1,
            PickValue::Gain,
            effect,
            vec![hosts[0].0],
        ) else {
            return Ok(());
        };
        self.clear_answer_log();
        let Some(&host) = pick.first() else { return Ok(()) };
        self.run_effect(
            &Effect::Attach { what: Selector::ExactObjects(gear), to: Selector::ExactObjects(vec![host]) },
            ctx,
            events,
        )
    }
}
