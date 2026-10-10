//! "You may discard N cards. If you do, …" (`Effect::MayDiscard` /
//! `MayDiscardMatching`): a prompting seat names the cards it discards
//! (CR 701.9a — a player discards cards they choose). A headless seat keeps
//! the old pick, the highest mana values (least castable), so two-player
//! play is unchanged.

use super::GameState;
use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::Effect;

impl GameState {
    /// The `n` of `candidates` (cards in `seat`'s hand) to discard, asked on
    /// the arm's answer-log `cursor`. `None` while the ask is suspended.
    pub(crate) fn pick_discards(
        &mut self,
        cursor: &mut usize,
        seat: usize,
        source: CardId,
        candidates: Vec<CardId>,
        n: usize,
        effect: &Effect,
    ) -> Option<Vec<CardId>> {
        let hand = &self.players[seat].hand;
        let mut by_mv: Vec<(CardId, u32)> = candidates
            .iter()
            .filter_map(|id| hand.iter().find(|c| c.id == *id).map(|c| (c.id, c.definition.cost.cmc())))
            .collect();
        by_mv.sort_by_key(|(_, cmc)| std::cmp::Reverse(*cmc));
        let auto: Vec<CardId> = by_mv.iter().take(n).map(|(id, _)| *id).collect();
        let named: Vec<(CardId, String)> = by_mv
            .iter()
            .filter_map(|(id, _)| hand.iter().find(|c| c.id == *id).map(|c| (*id, c.definition.name.to_string())))
            .collect();
        let picks = self.ask_seat_cards_logged(
            cursor,
            seat,
            format!("Discard {n}"),
            source,
            named,
            n as u32,
            n as u32,
            PickValue::Cost,
            effect,
            auto.clone(),
        )?;
        // A short answer is topped up from the default, never under-paid.
        let mut picks = picks;
        for id in auto {
            if picks.len() >= n {
                break;
            }
            if !picks.contains(&id) {
                picks.push(id);
            }
        }
        Some(picks)
    }
}
