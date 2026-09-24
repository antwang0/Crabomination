//! Dawnbreak Reclaimer's end-step exchange: each side names a creature card in
//! the other's graveyard, then both may come back.

use super::EffectContext;
use crate::card::CardId;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// The cheapest creature card (by mana value, then id) in `seat`'s
    /// graveyard — the least a picker hands the other side.
    fn cheapest_graveyard_creature(&self, seat: usize) -> Option<(CardId, u32)> {
        self.players[seat]
            .graveyard
            .iter()
            .filter(|c| c.definition.is_creature())
            .map(|c| (c.id, c.definition.cost.cmc()))
            .min_by_key(|&(id, mv)| (mv, id.0))
    }

    /// The picks are the engine's: each side names the other's cheapest
    /// creature card, and the opponent is the first in turn order holding
    /// the cheapest one. The "may" is the controller's (`MayDo`); a bot seat
    /// returns the pair only when it gets at least as much as it gives.
    pub(super) fn choose_graveyard_creatures_each_may_return(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let live = self.opponents_of(me);
        let opps: Vec<usize> =
            self.seats_in_turn_order_from(me).into_iter().filter(|q| live.contains(q)).collect();
        let theirs = opps
            .iter()
            .filter_map(|&q| self.cheapest_graveyard_creature(q).map(|pick| (q, pick)))
            .min_by_key(|&(_, (id, mv))| (mv, id.0));
        // With no opponent left there is no "that player" to pick from yours.
        if opps.is_empty() {
            return Ok(());
        }
        let mine = self.cheapest_graveyard_creature(me);
        if theirs.is_none() && mine.is_none() {
            return Ok(());
        }
        let ids: Vec<(CardId, usize)> = theirs
            .map(|(q, (id, _))| (id, q))
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
        if self.seat_prompts(me) {
            return self.run_effect(
                &Effect::MayDo { description: "Return both creature cards to the battlefield?".into(), body: Box::new(back) },
                ctx,
                events,
            );
        }
        let gain = mine.map_or(0, |(_, mv)| mv as i64 + 1);
        let give = theirs.map_or(0, |(_, (_, mv))| mv as i64 + 1);
        if gain > 0 && gain >= give {
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
