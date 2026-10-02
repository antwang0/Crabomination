//! `Effect::ChooseOneAmong` / `ChooseSomeAmong` — "choose a [card or
//! permanent]" (or N of them) made on resolution (CR 608.2d), not targeted:
//! the chosen run `chosen`, the rest run `other` (Deadly Vanity, Zimone's
//! Hypothesis, Author of Shadows, Haunting Voyage).

use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::types::GameEvent;
use crate::game::{GameError, GameState};

/// The pick is something its chooser wants: a friendly body on it (a +1/+1
/// counter, a keyword), a free turn-face-up, a play grant, becoming or
/// minting a copy of it, attaching it, casting it, or returning it to your
/// hand or battlefield.
fn pick_is_gain(e: &Effect) -> bool {
    match e {
        Effect::TurnFaceUpFree { .. }
        | Effect::GrantMayPlay { .. }
        | Effect::BecomeCopyOf { .. }
        | Effect::CreateTokenCopyOf { who: PlayerRef::You, .. }
        | Effect::Attach { .. }
        | Effect::AttachAnyNumberTo { .. }
        | Effect::CastWithoutPayingImmediate { .. } => true,
        Effect::Move { to, .. } => matches!(
            to,
            ZoneDest::Hand(PlayerRef::You | PlayerRef::OwnerOfMoved)
                | ZoneDest::Battlefield { controller: PlayerRef::You, .. }
                // Banked with the source to be played later (Esper Valigarmanda).
                | ZoneDest::ExileWithSourceStamp
        ),
        Effect::Seq(v) => v.iter().any(pick_is_gain),
        other => other.prefers_friendly_target(),
    }
}

impl GameState {
    /// `what`'s cards / permanents, best pick first for `seat`: for a gain the
    /// biggest body then the priciest (`seat`'s own permanents before any
    /// other's); for a harm an opponent's priciest permanent, then the least
    /// valuable of the rest.
    fn rank_picks(&self, ids: &[CardId], seat: usize, gain: bool) -> Vec<CardId> {
        let body_and_mv = |id: CardId| -> (i32, u32) {
            match (self.computed_permanent(id), self.battlefield_find(id)) {
                (Some(cp), Some(c)) => (cp.power.saturating_add(cp.toughness), c.definition.cost.cmc()),
                _ => self.find_card_anywhere(id).map_or((0, 0), |c| {
                    let d = &c.definition;
                    (d.power.max(0) + d.toughness.max(0), d.cost.cmc())
                }),
            }
        };
        let mut ranked = ids.to_vec();
        if gain {
            // Stable: equal picks keep resolution order.
            ranked.sort_by_key(|&id| {
                let foreign = self.battlefield_find(id).is_some_and(|c| c.controller != seat);
                (foreign, std::cmp::Reverse(body_and_mv(id)))
            });
        } else {
            ranked.sort_by_key(|&id| {
                let theirs = self.battlefield_find(id).filter(|c| !self.same_team(c.controller, seat));
                match theirs {
                    Some(c) => (0, std::cmp::Reverse(c.definition.cost.cmc()), (0, 0)),
                    None => {
                        let (body, mv) = body_and_mv(id);
                        (1, std::cmp::Reverse(0), (i64::from(mv), i64::from(body)))
                    }
                }
            });
        }
        ranked
    }

    /// Which way the pick cuts for its chooser. "Choose up to one …, destroy
    /// the rest" (Duneblast) keeps the pick, so it is a gain and none is legal;
    /// an imprint of your own card (Prototype Portal's from hand, Idris's
    /// artifact) banks the best one.
    fn pick_cuts_as_gain(&self, ids: &[CardId], seat: usize, chosen: &Effect) -> (bool, bool) {
        let keeps = matches!(chosen, Effect::Noop);
        let imprint = matches!(chosen, Effect::ExileTaggedWithSource { .. } | Effect::ExileUntilSourceLeaves { .. })
            && ids.iter().all(|id| match self.battlefield_find(*id) {
                Some(c) => c.controller == seat,
                None => true,
            });
        (keeps || imprint || pick_is_gain(chosen), keeps)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn choose_one_among(
        &mut self,
        what: &Selector,
        chooser: &PlayerRef,
        chosen: &Effect,
        other: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        self.choose_some_among(what, chooser, 1, false, chosen, other, ctx, events, effect)
    }

    /// The shared body: `chooser` picks `n` of `what` (any number to `n` when
    /// `up_to`, or when the pick is a survivor — `chosen` is `Noop`).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn choose_some_among(
        &mut self,
        what: &Selector,
        chooser: &PlayerRef,
        n: usize,
        up_to: bool,
        chosen: &Effect,
        other: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let ids: Vec<CardId> = self
            .resolve_selector(what, ctx)
            .into_iter()
            .filter_map(|e| e.as_card_id())
            .collect();
        let Some(seat) = self.resolve_player(chooser, ctx) else { return Ok(()) };
        if ids.is_empty() || n == 0 {
            return Ok(());
        }
        let source = ctx.source.unwrap_or(CardId(0));
        let candidates: Vec<(CardId, String)> = ids
            .iter()
            .filter_map(|id| self.find_card_anywhere(*id).map(|c| (*id, c.definition.name.to_string())))
            .collect();
        let (gain, keeps) = self.pick_cuts_as_gain(&ids, seat, chosen);
        let mut default = self.rank_picks(&ids, seat, gain);
        // A survivor pick never spares an opponent's permanent by default.
        if keeps {
            default.retain(|id| self.battlefield_find(*id).is_none_or(|c| c.controller == seat));
        }
        default.truncate(n);
        // A forced pick asks for at least one (as many as there are, to `n`),
        // so a bot answering a harm gives up its least valuable, not nothing.
        let min = if keeps || up_to { 0 } else { n.min(ids.len()) as u32 };
        let value = if gain { PickValue::Gain } else { PickValue::Cost };
        let prompt = if n == 1 { "Choose one.".to_string() } else { format!("Choose {n}.") };
        let picked = if self.seat_prompts(seat) || !matches!(self.decider.kind(), crate::decision::DeciderKind::Auto) {
            self.ask_seat_cards(seat, prompt, source, candidates, min, n as u32, value, effect)
        } else {
            Some(default.clone())
        };
        let Some(mut picked) = picked else { return Ok(()) };
        picked.dedup();
        // A short forced answer is topped up from the default pick.
        for id in default {
            if picked.len() >= min as usize {
                break;
            }
            if !picked.contains(&id) {
                picked.push(id);
            }
        }
        let rest: Vec<CardId> = ids.into_iter().filter(|id| !picked.contains(id)).collect();
        if picked.is_empty() && !keeps && !up_to {
            return Ok(());
        }
        self.separated_piles = (picked, rest);
        self.run_piles_then_clear(chosen, other, ctx, events)
    }
}
