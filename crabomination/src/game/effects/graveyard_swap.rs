//! Dawnbreak Reclaimer's end-step exchange: each side names a creature card in
//! the other's graveyard, then both may come back.

use super::EffectContext;
use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// "Choose a creature card in an opponent's graveyard, then that player
    /// chooses a creature card in your graveyard. You may return those cards"
    /// — the controller picks first, then the owner of that pick picks from
    /// the controller's graveyard (both asked; the cheapest card, and the
    /// first opponent in turn order holding it, are the headless default),
    /// then the controller's "may". A bot seat returns the pair only when it
    /// gets at least as much as it gives.
    pub(super) fn choose_graveyard_creatures_each_may_return(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let live = self.opponents_of(me);
        let opps: Vec<usize> =
            self.seats_in_turn_order_from(me).into_iter().filter(|q| live.contains(q)).collect();
        // With no opponent left there is no "that player" to pick from yours.
        if opps.is_empty() {
            self.clear_answer_log();
            return Ok(());
        }
        let source = ctx.source.unwrap_or(CardId(0));
        let creatures_of = |g: &Self, seat: usize| -> Vec<(CardId, u32, String)> {
            let mut v: Vec<(CardId, u32, String)> = g.players[seat]
                .graveyard
                .iter()
                .filter(|c| g.computed_is_creature(c))
                .map(|c| (c.id, c.definition.cost.cmc(), c.definition.name.to_string()))
                .collect();
            v.sort_by_key(|c| (c.1, c.0.0));
            v
        };
        let mut cursor = 0;
        // Cheapest first: a stable sort keeps turn order among equals, so the
        // head is the old pick.
        let mut theirs_cands: Vec<(CardId, u32, String, usize)> = opps
            .iter()
            .flat_map(|&q| creatures_of(self, q).into_iter().map(move |(id, mv, n)| (id, mv, n, q)))
            .collect();
        theirs_cands.sort_by_key(|c| (c.1, c.0.0));
        let theirs: Option<(CardId, u32, usize)> = match theirs_cands.len() {
            0 => None,
            1 => Some((theirs_cands[0].0, theirs_cands[0].1, theirs_cands[0].3)),
            _ => {
                let auto = theirs_cands[0].0;
                let Some(picked) = self.ask_seat_cards_logged(
                    &mut cursor,
                    me,
                    "Choose a creature card in an opponent's graveyard".into(),
                    source,
                    theirs_cands.iter().map(|c| (c.0, c.2.clone())).collect(),
                    1,
                    1,
                    PickValue::Cost,
                    effect,
                    vec![auto],
                ) else {
                    return Ok(());
                };
                let id = picked.first().copied().unwrap_or(auto);
                theirs_cands.iter().find(|c| c.0 == id).map(|c| (c.0, c.1, c.3))
            }
        };
        let mine_cands = creatures_of(self, me);
        let mine: Option<(CardId, u32)> = match (theirs, mine_cands.len()) {
            (_, 0) => None,
            (Some((_, _, q)), n) if n > 1 => {
                let auto = mine_cands[0].0;
                let Some(picked) = self.ask_seat_cards_logged(
                    &mut cursor,
                    q,
                    "Choose a creature card in that player's graveyard".into(),
                    source,
                    mine_cands.iter().map(|c| (c.0, c.2.clone())).collect(),
                    1,
                    1,
                    PickValue::Cost,
                    effect,
                    vec![auto],
                ) else {
                    return Ok(());
                };
                let id = picked.first().copied().unwrap_or(auto);
                mine_cands.iter().find(|c| c.0 == id).map(|c| (c.0, c.1))
            }
            _ => Some((mine_cands[0].0, mine_cands[0].1)),
        };
        if theirs.is_none() && mine.is_none() {
            self.clear_answer_log();
            return Ok(());
        }
        let ids: Vec<(CardId, usize)> = theirs
            .map(|(id, _, q)| (id, q))
            .into_iter()
            .chain(mine.map(|(id, _)| (id, me)))
            .collect();
        let back = Effect::Seq(
            ids.iter()
                .map(|&(id, owner)| Effect::Move {
                    what: Selector::ExactObjects(vec![id]),
                    to: ZoneDest::Battlefield { controller: PlayerRef::Seat(owner), tapped: false },
                })
                .collect(),
        );
        let go = if self.seat_prompts(me) {
            let Some(yes) = self.ask_seat_bool(
                &mut cursor,
                me,
                "Return both creature cards to the battlefield?".into(),
                source,
                effect,
                crate::decision::OptionalKind::MayBody,
            ) else {
                return Ok(());
            };
            yes
        } else {
            let gain = mine.map_or(0, |(_, mv)| mv as i64 + 1);
            let give = theirs.map_or(0, |(_, mv, _)| mv as i64 + 1);
            gain > 0 && gain >= give
        };
        self.clear_answer_log();
        if go {
            self.run_effect(&back, ctx, events)?;
        }
        Ok(())
    }

    /// Sinister Waltz's tail: `count` of the targets still in the controller's
    /// graveyard, picked at random off the game RNG, enter under them; the
    /// rest go to the bottom of their library.
    pub(super) fn return_target_cards_at_random(
        &mut self,
        count: &crate::effect::Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        use rand::seq::SliceRandom;
        let me = ctx.controller;
        let mut ids: Vec<CardId> = ctx
            .targets
            .iter()
            .filter_map(|t| match t {
                crate::game::types::Target::Permanent(id) => Some(*id),
                _ => None,
            })
            .filter(|id| self.players[me].graveyard.iter().any(|c| c.id == *id))
            .collect();
        ids.dedup();
        let n = (self.evaluate_value(count, ctx).max(0) as usize).min(ids.len());
        ids.shuffle(&mut self.rng.draw());
        let (back, rest) = ids.split_at(n);
        for &id in rest {
            let to_bottom = ZoneDest::Library { who: PlayerRef::Seat(me), pos: crate::effect::LibraryPosition::Bottom };
            self.run_effect(&Effect::Move { what: Selector::ExactObjects(vec![id]), to: to_bottom }, ctx, events)?;
        }
        for &id in back {
            let dest = ZoneDest::Battlefield { controller: PlayerRef::Seat(me), tapped: false };
            self.run_effect(&Effect::Move { what: Selector::ExactObjects(vec![id]), to: dest }, ctx, events)?;
        }
        Ok(())
    }

    /// Footbottom Feast — the controller picks any number of `filter` cards
    /// in their graveyard (a bot takes what `PickValue::Gain` takes); they go
    /// on top of the library in mana-value order, the greatest ending on top.
    pub(super) fn put_any_number_from_graveyard_on_top(
        &mut self,
        filter: &crate::card::SelectionRequirement,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let seat = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        let candidates: Vec<(CardId, String)> = self.players[seat]
            .graveyard
            .iter()
            .filter(|c| self.evaluate_requirement_on_card(filter, c, seat))
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        if candidates.is_empty() {
            return Ok(());
        }
        // Auto default: the single greatest card, the one the draw takes.
        let auto_default: Vec<CardId> = self.players[seat]
            .graveyard
            .iter()
            .filter(|c| candidates.iter().any(|(id, _)| *id == c.id))
            .max_by_key(|c| (c.definition.cost.cmc(), c.id.0))
            .map(|c| vec![c.id])
            .unwrap_or_default();
        let n = candidates.len() as u32;
        let Some(chosen) = self.choose_up_to_cards(
            seat,
            "Put any number of cards from your graveyard on top of your library".to_string(),
            source,
            candidates.clone(),
            n,
            crate::decision::PickValue::Gain,
            effect,
            auto_default,
        ) else {
            return Ok(());
        };
        let mut picks: Vec<(u32, CardId)> = chosen
            .into_iter()
            .filter(|cid| candidates.iter().any(|(id, _)| id == cid))
            .filter_map(|cid| self.players[seat].graveyard.iter().find(|c| c.id == cid))
            .map(|c| (c.definition.cost.cmc(), c.id))
            .collect();
        picks.sort_by_key(|&(mv, id)| (mv, id.0));
        for (_, cid) in picks {
            let dest = ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top };
            self.move_card_to(cid, &dest, ctx, events);
        }
        Ok(())
    }
}
