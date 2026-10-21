//! `Effect::EachPlayerKeepsUpToPermanentsRestPhaseOut` — "starting with you,
//! each player chooses up to N permanents they control. All permanents other
//! than this one that weren't chosen phase out" (Disciple of Caelus Nin).
//! Seats choose in turn order from the controller (CR 101.4); the phase-out
//! is one event after every choice. A headless seat keeps its priciest
//! nontoken permanents, lands last.

use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::Effect;
use crate::game::effects::EffectContext;
use crate::game::{GameError, GameEvent, GameState};

impl GameState {
    pub(super) fn each_player_keeps_permanents_rest_phase_out(
        &mut self,
        keep: u32,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0usize;
        let mut kept: Vec<CardId> = Vec::new();
        for p in self.seats_in_turn_order_from(ctx.controller) {
            let mut mine: Vec<(CardId, String, (bool, bool, u32))> = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p && c.id != source)
                .map(|c| (c.id, c.definition.name.to_string(), (!c.definition.is_land(), !c.is_token, c.definition.cost.cmc())))
                .collect();
            if mine.len() <= keep as usize {
                kept.extend(mine.iter().map(|m| m.0));
                continue;
            }
            mine.sort_by_key(|m| std::cmp::Reverse(m.2));
            let auto = mine.iter().take(keep as usize).map(|m| m.0).collect();
            let named = mine.iter().map(|(id, n, _)| (*id, n.clone())).collect();
            let Some(ids) = self.ask_seat_cards_logged(
                &mut cursor,
                p,
                format!("Choose up to {keep} permanents to keep; the rest phase out"),
                source,
                named,
                0,
                keep,
                PickValue::Gain,
                effect,
                auto,
            ) else {
                return Ok(());
            };
            kept.extend(ids);
        }
        self.clear_answer_log();
        let out = self
            .battlefield
            .iter()
            .filter(|c| c.id != source && !kept.contains(&c.id))
            .map(|c| c.id)
            .collect();
        self.phase_out_permanents(out, None, events);
        Ok(())
    }
}
