//! `Effect::EachPlayerDiscardsAnyThenDrawsThatMany` — "each player discards
//! any number of cards, then draws that many cards" (Flux). Every living
//! seat chooses in APNAP order (CR 101.4), the discards happen together, then
//! each seat draws its own count — the count is per seat, never pooled. A
//! headless seat cycles away the lands in its hand once it has six on the
//! battlefield, and keeps everything otherwise.

use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::game::effects::EffectContext;
use crate::game::{GameError, GameEvent, GameState};

/// Lands on the battlefield past which a headless seat discards the lands in
/// its hand.
const HEADLESS_FLOODED: usize = 6;

impl GameState {
    pub(super) fn each_player_discards_any_then_draws(
        &mut self,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let source = ctx.source.unwrap_or(CardId(0));
        let seats = self.apnap_sort(self.living_seats().collect());
        let mut cursor = 0usize;
        let mut picks: Vec<(usize, Vec<CardId>)> = Vec::new();
        for p in seats {
            let hand: Vec<(CardId, String, bool)> = self.players[p]
                .hand
                .iter()
                .map(|c| (c.id, c.definition.name.to_string(), c.definition.is_land()))
                .collect();
            if hand.is_empty() {
                continue;
            }
            let lands = self.battlefield.iter().filter(|c| c.controller == p && c.definition.is_land()).count();
            let auto: Vec<CardId> = if lands >= HEADLESS_FLOODED {
                hand.iter().filter(|h| h.2).map(|h| h.0).collect()
            } else {
                Vec::new()
            };
            let named = hand.iter().map(|(id, n, _)| (*id, n.clone())).collect();
            let Some(ids) = self.ask_seat_cards_logged(
                &mut cursor,
                p,
                "Discard any number of cards, then draw that many".into(),
                source,
                named,
                0,
                hand.len() as u32,
                PickValue::Cost,
                effect,
                auto,
            ) else {
                return Ok(());
            };
            picks.push((p, ids));
        }
        self.clear_answer_log();
        let mut counts: Vec<(usize, i32)> = Vec::new();
        for (p, ids) in &picks {
            let n = ids.iter().filter(|id| self.discard_card(*p, **id, events)).count() as i32;
            counts.push((*p, n));
        }
        for (p, n) in counts {
            if n > 0 {
                self.run_effect(
                    &Effect::Draw { who: Selector::Player(PlayerRef::Seat(p)), amount: Value::Const(n) },
                    ctx,
                    events,
                )?;
            }
        }
        Ok(())
    }
}
