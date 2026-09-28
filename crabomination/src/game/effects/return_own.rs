//! "Return a [permanent] you control to its owner's hand" chosen on
//! resolution (`Effect::ReturnOneYouControl`) — Whitemane Lion, Kor
//! Skyfisher, Guildless Commons, Time Wipe. CR 608.2d: a choice made as the
//! effect resolves, so no target, no hexproof bar and no fizzle.

use crate::card::{CardId, SelectionRequirement};
use crate::decision::PickValue;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::{GameError, GameEvent, GameState};

impl GameState {
    pub(super) fn return_one_you_control(
        &mut self,
        filter: &SelectionRequirement,
        keep_best: bool,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let seat = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        // (is the source, mana value, untapped) per candidate, battlefield order.
        let mut ranked: Vec<(CardId, bool, u32, bool)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && self.evaluate_requirement_on_card(filter, c, seat))
            .map(|c| (c.id, c.id == source, c.definition.cost.cmc(), !c.tapped))
            .collect();
        if ranked.is_empty() {
            return Ok(());
        }
        // Stable sorts: the earliest wins a tie either way. The source itself
        // comes last — Guildless Commons enters tapped and bouncing it wastes
        // the land drop; Whitemane Lion alone still returns itself.
        if keep_best {
            ranked.sort_by_key(|&(_, _, mv, _)| std::cmp::Reverse(mv));
        } else {
            ranked.sort_by_key(|&(_, own, mv, untapped)| (own, mv, untapped));
        }
        let default = ranked[0].0;
        let candidates: Vec<(CardId, String)> = ranked
            .iter()
            .filter_map(|&(id, ..)| self.battlefield_find(id).map(|c| (id, c.definition.name.to_string())))
            .collect();
        let value = if keep_best { PickValue::Gain } else { PickValue::Cost };
        let Some(picked) = self.choose_up_to_cards(
            seat,
            "Return a permanent you control to its owner's hand.".into(),
            source,
            candidates,
            1,
            value,
            effect,
            vec![default],
        ) else {
            return Ok(());
        };
        // Mandatory: an empty answer takes the default.
        let one = picked.first().copied().unwrap_or(default);
        self.separated_piles = (vec![one], Vec::new());
        let bounce = Effect::Move {
            what: Selector::SeparatedPile { chosen: true },
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        };
        let run = self.run_effect(&bounce, ctx, events);
        self.separated_piles = (Vec::new(), Vec::new());
        run
    }
}
