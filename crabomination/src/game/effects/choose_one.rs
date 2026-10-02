//! `Effect::ChooseOneAmong` — "choose a [card or permanent]" made on
//! resolution (CR 608.2d), not targeted: the chosen one runs `chosen`, the
//! rest run `other` (Deadly Vanity, Zimone's Hypothesis, Author of Shadows).

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
        let ids: Vec<CardId> = self
            .resolve_selector(what, ctx)
            .into_iter()
            .filter_map(|e| e.as_card_id())
            .collect();
        let Some(seat) = self.resolve_player(chooser, ctx) else { return Ok(()) };
        if ids.is_empty() {
            return Ok(());
        }
        let source = ctx.source.unwrap_or(CardId(0));
        let candidates: Vec<(CardId, String)> = ids
            .iter()
            .filter_map(|id| self.find_card_anywhere(*id).map(|c| (*id, c.definition.name.to_string())))
            .collect();
        // "Choose up to one …, destroy the rest" (Duneblast): the pick is the
        // survivor, so it is a gain — ours first, the biggest — and choosing
        // none is legal.
        let keeps = matches!(chosen, Effect::Noop);
        // An imprint of your own card (Prototype Portal's from hand, Idris's
        // artifact) banks the best one.
        let imprint = matches!(chosen, Effect::ExileTaggedWithSource { .. } | Effect::ExileUntilSourceLeaves { .. })
            && ids.iter().all(|id| match self.battlefield_find(*id) {
                Some(c) => c.controller == seat,
                None => true,
            });
        let gain = keeps || imprint || pick_is_gain(chosen);
        let default = if gain {
            ids.iter()
                .copied()
                .filter(|id| self.battlefield_find(*id).is_some_and(|c| c.controller == seat))
                // The biggest body; among bodiless ones (Equipment) the priciest.
                .max_by_key(|id| {
                    let body = self.computed_permanent(*id).map_or(0, |cp| cp.power.saturating_add(cp.toughness));
                    (body, self.battlefield_find(*id).map_or(0, |c| c.definition.cost.cmc()))
                })
                // Cards off the battlefield: the biggest body, else the priciest.
                .or_else(|| {
                    ids.iter().copied().filter(|id| self.battlefield_find(*id).is_none()).max_by_key(|id| {
                        self.find_card_anywhere(*id).map_or((0, 0), |c| {
                            let d = &c.definition;
                            (d.power.max(0) + d.toughness.max(0), d.cost.cmc())
                        })
                    })
                })
                .into_iter()
                .collect()
        } else {
            // A harm on the pick (a doom counter, an exile, a steal) lands on
            // an opponent's priciest permanent when there is one (Eye of Doom);
            // on your own things, the least of them (Kefnet's land, The War
            // Games' exile).
            ids.iter()
                .copied()
                .filter(|id| self.battlefield_find(*id).is_some_and(|c| !self.same_team(c.controller, seat)))
                .max_by_key(|id| self.battlefield_find(*id).map_or(0, |c| c.definition.cost.cmc()))
                .or_else(|| {
                    ids.iter().copied().min_by_key(|id| match self.computed_permanent(*id) {
                        Some(cp) => (self.battlefield_find(*id).map_or(0, |c| c.definition.cost.cmc()), cp.power.saturating_add(cp.toughness)),
                        None => self.find_card_anywhere(*id).map_or((0, 0), |c| {
                            (c.definition.cost.cmc(), c.definition.power.max(0) + c.definition.toughness.max(0))
                        }),
                    })
                })
                .into_iter()
                .collect()
        };
        let Some(picked) = self.choose_up_to_cards(
            seat,
            "Choose one.".into(),
            source,
            candidates,
            1,
            if gain { PickValue::Gain } else { PickValue::Cost },
            effect,
            default,
        ) else {
            return Ok(());
        };
        let one = match picked.first() {
            Some(id) => *id,
            None if keeps => {
                self.separated_piles = (Vec::new(), ids);
                return self.run_piles_then_clear(chosen, other, ctx, events);
            }
            None => ids[0],
        };
        self.separated_piles = (vec![one], ids.into_iter().filter(|id| *id != one).collect());
        self.run_piles_then_clear(chosen, other, ctx, events)
    }
}
