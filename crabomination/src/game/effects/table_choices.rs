//! Multiplayer table effects where each player makes their own choice:
//! Archfiend of Depravity, Dredge the Mire, Explosion of Riches, Scythe
//! Specter. Each asks every seat first (log-replayed, so a prompting seat can
//! suspend), clears the answer log, then acts.

use crate::card::{CardId, SelectionRequirement};
use crate::decision::{OptionalKind, PickValue};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::effects::{EffectContext, EntityRef};
use crate::game::types::GameEvent;
use crate::game::{GameError, GameState};

impl GameState {
    /// `Effect::SacrificeAllButN` — each player sacrifices all but `keep` of
    /// their matching permanents (`Effect::Sacrifice`'s weakest-first pick, so
    /// the best are kept; a prompting seat chooses).
    pub(super) fn sacrifice_all_but_n(
        &mut self,
        who: &Selector,
        keep: &Value,
        filter: &SelectionRequirement,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let keep = self.evaluate_value(keep, ctx).max(0) as usize;
        let seats: Vec<usize> = self
            .resolve_selector(who, ctx)
            .into_iter()
            .filter_map(|e| if let EntityRef::Player(p) = e { Some(p) } else { None })
            .collect();
        let steps: Vec<Effect> = seats
            .into_iter()
            .filter_map(|p| {
                let have = self.sacrifice_candidates(p, filter, ctx.source).len();
                (have > keep).then(|| Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::Seat(p)),
                    count: Value::Const((have - keep) as i32),
                    filter: filter.clone(),
                })
            })
            .collect();
        if steps.is_empty() {
            return Ok(());
        }
        self.run_effect(&Effect::Seq(steps), ctx, events)
    }

    /// `Effect::EachOpponentChoosesFromGraveyard` — APNAP, each opponent
    /// names one matching card in their own graveyard (a bot gives up its
    /// least valuable), then all the picks move to `to` together.
    pub(super) fn each_opponent_chooses_from_graveyard(
        &mut self,
        filter: &SelectionRequirement,
        to: &ZoneDest,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        let seats = self.apnap_sort(self.opponents_of(me));
        let mut cursor = 0usize;
        let mut picks: Vec<CardId> = Vec::new();
        for q in seats {
            let mut cands: Vec<(CardId, String, u32, i32)> = self.players[q]
                .graveyard
                .iter()
                .filter(|c| self.evaluate_requirement_on_card(filter, c, me))
                .map(|c| (c.id, c.definition.name.to_string(), c.definition.cost.cmc(), c.definition.power))
                .collect();
            if cands.is_empty() {
                continue;
            }
            cands.sort_by_key(|&(id, _, mv, pw)| (mv, pw, id.0));
            let auto = vec![cands[0].0];
            let named = cands.iter().map(|(id, n, _, _)| (*id, n.clone())).collect();
            let Some(ids) = self.ask_seat_cards_logged(
                &mut cursor,
                q,
                "Choose a card in your graveyard for your opponent's spell".into(),
                source,
                named,
                1,
                1,
                PickValue::Cost,
                effect,
                auto,
            ) else {
                return Ok(());
            };
            picks.extend(ids.first().copied());
        }
        self.clear_answer_log();
        if picks.is_empty() {
            return Ok(());
        }
        self.run_effect(&Effect::Move { what: Selector::ExactObjects(picks), to: to.clone() }, ctx, events)
    }

    /// `Effect::EachOtherPlayerMayDraw` — every other player, APNAP, may draw
    /// a card; `per_draw` runs under the resolving controller for each draw.
    pub(super) fn each_other_player_may_draw(
        &mut self,
        per_draw: &Effect,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        let seats: Vec<usize> =
            self.apnap_sort(self.living_seats().filter(|&q| q != me).collect());
        let mut cursor = 0usize;
        let mut drawers = Vec::new();
        for q in seats {
            let Some(yes) = self.ask_seat_bool(
                &mut cursor,
                q,
                "Draw a card?".into(),
                source,
                effect,
                OptionalKind::MayBody,
            ) else {
                return Ok(());
            };
            if yes {
                drawers.push(q);
            }
        }
        self.clear_answer_log();
        for q in drawers {
            if self.draw_one_or_deck(q, events) {
                self.run_effect(per_draw, ctx, events)?;
            }
        }
        Ok(())
    }

    /// `Effect::GreatestDiscardersLoseLife` — of this resolution's discards,
    /// each owner of one with the greatest mana value loses that much.
    pub(super) fn greatest_discarders_lose_life(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let discards: Vec<(usize, u32)> = self
            .scratch
            .discarded_card_ids_this_resolution
            .iter()
            .filter_map(|&id| {
                let owner = self.find_card_owner(id)?;
                Some((owner, self.find_card_anywhere(id)?.definition.cost.cmc()))
            })
            .collect();
        let Some(top) = discards.iter().map(|&(_, mv)| mv).max() else { return Ok(()) };
        if top == 0 {
            return Ok(());
        }
        let mut losers: Vec<usize> = discards.iter().filter(|&&(_, mv)| mv == top).map(|&(p, _)| p).collect();
        losers.sort_unstable();
        losers.dedup();
        for p in losers {
            self.run_effect(
                &Effect::LoseLife { who: Selector::Player(PlayerRef::Seat(p)), amount: Value::Const(top as i32) },
                ctx,
                events,
            )?;
        }
        Ok(())
    }

    /// "A player chosen at random" among the living seats (`PlayerRef::RandomPlayer`).
    pub(crate) fn random_living_seat(&self) -> Option<usize> {
        use rand::RngExt;
        let seats: smallvec::SmallVec<[usize; 8]> = self.living_seats().collect();
        (!seats.is_empty()).then(|| seats[self.rng.draw().random_range(0..seats.len())])
    }
}

