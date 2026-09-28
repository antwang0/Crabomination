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
/// counter, a keyword), a free turn-face-up, a play grant, becoming a copy
/// of it, or returning it to your hand or battlefield.
fn pick_is_gain(e: &Effect) -> bool {
    match e {
        Effect::TurnFaceUpFree { .. } | Effect::GrantMayPlay { .. } | Effect::BecomeCopyOf { .. } => true,
        Effect::Move { to, .. } => matches!(
            to,
            ZoneDest::Hand(PlayerRef::You | PlayerRef::OwnerOfMoved)
                | ZoneDest::Battlefield { controller: PlayerRef::You, .. }
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
        let gain = keeps || pick_is_gain(chosen);
        let default = if gain {
            ids.iter()
                .copied()
                .filter(|id| self.battlefield_find(*id).is_some_and(|c| c.controller == seat))
                .max_by_key(|id| self.computed_permanent(*id).map_or(0, |cp| cp.power.saturating_add(cp.toughness)))
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
            vec![ids[0]]
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
