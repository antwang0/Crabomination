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
}
