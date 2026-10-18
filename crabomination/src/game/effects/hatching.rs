//! The Dragon-Kami Reborn I, II — "Look at the top three cards of your
//! library. Exile one of them face down with a hatching counter on it, then
//! put the rest on the bottom of your library in any order."
//! (`Effect::LookTopExileOneFaceDownWithCounter`.) The auto pick is the
//! priciest creature card (the Egg casts creature spells from among them),
//! else the priciest card; the rest go to the bottom in the order seen.

use super::EffectContext;
use crate::card::{CardId, CardType, CounterType};
use crate::decision::PickValue;
use crate::effect::{Effect, PlayerRef, Value};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    pub(super) fn look_top_exile_one_face_down_with_counter(
        &mut self,
        count: &Value,
        kind: CounterType,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let Some(p) = self.resolve_player(&PlayerRef::You, ctx) else { return Ok(()) };
        let n = (self.evaluate_value(count, ctx).max(0) as usize).min(self.players[p].library.len());
        if n == 0 {
            return Ok(());
        }
        let mut cands: Vec<(CardId, String, bool, u32)> = self.players[p].library[..n]
            .iter()
            .map(|c| {
                let creature = c.definition.card_types.contains(&CardType::Creature);
                (c.id, c.definition.name.to_string(), creature, c.definition.cost.cmc())
            })
            .collect();
        cands.sort_by_key(|c| (!c.2, std::cmp::Reverse(c.3), c.0));
        let mut cursor = 0;
        let Some(pick) = self.ask_seat_cards_logged(
            &mut cursor,
            p,
            "Exile one face down with a hatching counter".into(),
            ctx.source.unwrap_or(CardId(0)),
            cands.iter().map(|c| (c.0, c.1.clone())).collect(),
            1,
            1,
            PickValue::Gain,
            effect,
            vec![cands[0].0],
        ) else {
            return Ok(());
        };
        self.clear_answer_log();
        let chosen = pick.first().copied().filter(|id| cands.iter().any(|c| c.0 == *id)).unwrap_or(cands[0].0);
        let top: Vec<_> = self.players[p].library.drain(..n).collect();
        let mut rest = Vec::new();
        for mut card in top {
            if card.id == chosen {
                card.face_down = true;
                card.add_counters(kind, 1);
                events.push(GameEvent::CounterAdded { card_id: card.id, counter_type: kind, count: 1, placer: None });
                self.exile.push(card);
            } else {
                rest.push(card);
            }
        }
        self.players[p].library.extend(rest);
        Ok(())
    }
}
