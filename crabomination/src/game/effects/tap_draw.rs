//! "You may tap any number of untapped [filter] you control. Draw a card for
//! each [one] tapped this way" (Guild Summit) — the draw-payout sibling of
//! `TapAnyNumberThenCounters` / `TapAnyNumberThenPumpPerTapped`.

use super::{EffectContext, GameState};
use crate::card::{CardId, SelectionRequirement};
use crate::decision::PickValue;
use crate::effect::{Effect, Selector, Value};
use crate::game::{GameError, GameEvent};

impl GameState {
    pub(super) fn resolve_tap_any_number_then_draw(
        &mut self,
        filter: &SelectionRequirement,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let seat = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        let candidates: Vec<(CardId, String)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && !c.tapped)
            .filter(|c| crate::game::layers::requirement_matches_card(filter, c, seat))
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        if candidates.is_empty() {
            return Ok(());
        }
        // Headless: every one — the cards outweigh the mana this late.
        let auto_default: Vec<CardId> = candidates.iter().map(|(id, _)| *id).collect();
        let n = candidates.len() as u32;
        let Some(chosen) = self.choose_up_to_cards(
            seat,
            "Tap any number to draw a card for each".to_string(),
            source,
            candidates.clone(),
            n,
            PickValue::Cost,
            effect,
            auto_default,
        ) else {
            return Ok(());
        };
        let mut tapped = 0i32;
        for cid in chosen {
            if !candidates.iter().any(|(id, _)| *id == cid) {
                continue;
            }
            if let Some(c) = self.battlefield_find_mut(cid).filter(|c| !c.tapped) {
                c.tapped = true;
                tapped += 1;
                events.push(GameEvent::PermanentTapped { card_id: cid, actor: Some(seat), as_attacker: false });
            }
        }
        if tapped > 0 {
            let draw = Effect::Draw { who: Selector::You, amount: Value::Const(tapped) };
            self.resolve_effect_into(&draw, ctx, events)?;
        }
        Ok(())
    }
}
