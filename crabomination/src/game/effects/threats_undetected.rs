//! Threats Undetected — "Search your library for up to four creature cards
//! with different powers and reveal them. An opponent chooses two of those
//! cards. Shuffle the chosen cards into your library and put the rest into
//! your hand." (`Effect::SearchDistinctPowersOpponentReturnsTwo`.) The
//! caster's pick is held to distinct powers (the first card of each power,
//! in pick order); the choosing opponent is the next one in turn order.

use super::EffectContext;
use crate::card::{CardId, CardType};
use crate::decision::PickValue;
use crate::effect::Effect;
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    pub(super) fn search_distinct_powers_opponent_returns_two(
        &mut self,
        max: u32,
        back: u32,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        // CR 701.19 — a search: the locks and Aven Mindcensor's window apply.
        if self.no_search_this_turn || self.player_search_locked_by_opponent(me) {
            return Ok(());
        }
        self.note_library_searched(me, ctx, events);
        let limit = self.search_top_limit_for(me).unwrap_or(usize::MAX);
        let mut cands: Vec<(CardId, String, i32, u32)> = self.players[me]
            .library
            .iter()
            .take(limit)
            .filter(|c| c.definition.card_types.contains(&CardType::Creature))
            .map(|c| (c.id, c.definition.name.to_string(), c.definition.power, c.definition.cost.cmc()))
            .collect();
        cands.sort_by_key(|c| (std::cmp::Reverse(c.3), c.0));
        let distinct = |ids: &[CardId], cands: &[(CardId, String, i32, u32)]| -> Vec<CardId> {
            let mut powers: Vec<i32> = Vec::new();
            let mut out = Vec::new();
            for id in ids {
                if let Some(c) = cands.iter().find(|c| c.0 == *id)
                    && !powers.contains(&c.2)
                    && out.len() < max as usize
                {
                    powers.push(c.2);
                    out.push(c.0);
                }
            }
            out
        };
        let mut cursor = 0;
        let found: Vec<CardId> = if cands.is_empty() {
            Vec::new()
        } else {
            let all: Vec<CardId> = cands.iter().map(|c| c.0).collect();
            let auto = distinct(&all, &cands);
            let Some(pick) = self.ask_seat_cards_logged(
                &mut cursor,
                me,
                format!("Search for up to {max} creature cards with different powers"),
                source,
                cands.iter().map(|c| (c.0, c.1.clone())).collect(),
                0,
                max,
                PickValue::Gain,
                effect,
                auto,
            ) else {
                return Ok(());
            };
            distinct(&pick, &cands)
        };
        // An opponent chooses which `back` go home: the strongest by default.
        let back_ids: Vec<CardId> = match self.opponents_of(me).into_iter().find(|_| !found.is_empty()) {
            Some(opp) if found.len() > back as usize => {
                let mut shown: Vec<(CardId, String, u32)> = found
                    .iter()
                    .filter_map(|id| cands.iter().find(|c| c.0 == *id))
                    .map(|c| (c.0, c.1.clone(), c.3))
                    .collect();
                shown.sort_by_key(|c| (std::cmp::Reverse(c.2), c.0));
                let auto: Vec<CardId> = shown.iter().take(back as usize).map(|c| c.0).collect();
                let Some(pick) = self.ask_seat_cards_logged(
                    &mut cursor,
                    opp,
                    format!("Choose {back} of these cards to shuffle back into their library"),
                    source,
                    shown.iter().map(|c| (c.0, c.1.clone())).collect(),
                    back,
                    back,
                    PickValue::Gain,
                    effect,
                    auto.clone(),
                ) else {
                    return Ok(());
                };
                let mut v: Vec<CardId> = pick.into_iter().filter(|id| found.contains(id)).take(back as usize).collect();
                for id in auto {
                    if v.len() < back as usize && !v.contains(&id) {
                        v.push(id);
                    }
                }
                v
            }
            _ => found.clone(),
        };
        self.clear_answer_log();
        for id in found.iter().filter(|id| !back_ids.contains(id)) {
            if let Some(pos) = self.players[me].library.iter().position(|c| c.id == *id) {
                let card = self.players[me].library.remove(pos);
                self.players[me].hand.push(card);
            }
        }
        self.shuffle_library(me, events);
        Ok(())
    }
}
