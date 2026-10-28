//! "Reveal the top X cards of your library. You may cast a spell with mana
//! value X or less from among them without paying its mana cost. Put the rest
//! on the bottom of your library in a random order." — Sunbird's Invocation.

use super::{EffectContext, PickValue};
use crate::card::{CardId, Zone};
use crate::effect::{Effect, Value};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// The choice is made over the cards where they lie, BEFORE anything
    /// moves: a prompting seat suspends inside `choose_up_to_cards` and the
    /// effect re-runs on the answer, which must not reveal a second batch.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn reveal_top_may_cast_one_free(
        &mut self,
        count: &Value,
        max_mv: &Value,
        filter: Option<&crate::card::SelectionRequirement>,
        keep_rest_on_top: bool,
        ctx: &EffectContext,
        effect: &Effect,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let p = ctx.controller;
        let n = (self.evaluate_value(count, ctx).max(0) as usize).min(self.players[p].library.len());
        if n == 0 {
            return Ok(());
        }
        let cap = self.evaluate_value(max_mv, ctx).max(0) as u32;
        // The card walk below is context-free, so an X-relative filter reads
        // this resolution's X now (Powerbalance's "same mana value").
        let owned = filter.filter(|f| f.names_x_or_converge()).map(|f| f.resolve_x(ctx.x_value));
        let filter = owned.as_ref().or(filter);
        let revealed: Vec<CardId> = self.players[p].library[..n].iter().map(|c| c.id).collect();
        let castable = |c: &crate::card::CardInstance| {
            !c.definition.is_land()
                && c.definition.cost.cmc() <= cap
                && filter.is_none_or(|f| self.evaluate_requirement_on_card(f, c, p))
        };
        let candidates: Vec<(CardId, String)> = self.players[p].library[..n]
            .iter()
            .filter(|c| castable(c))
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        let auto = self.players[p].library[..n]
            .iter()
            .filter(|c| castable(c))
            .max_by_key(|c| c.definition.cost.cmc())
            .map(|c| vec![c.id])
            .unwrap_or_default();
        let picked = if candidates.is_empty() {
            Vec::new()
        } else {
            let Some(p) = self.choose_up_to_cards(
                p,
                "Cast a revealed spell without paying its mana cost?".into(),
                ctx.source.unwrap_or(CardId(0)),
                candidates,
                1,
                PickValue::Gain,
                effect,
                auto,
            ) else {
                return Ok(());
            };
            p
        };
        if let Some(pick) = picked.first().copied() {
            let def = self.find_card_anywhere(pick).map(|c| c.definition.arc());
            if let Some(def) = def {
                let target = self.auto_target_for_effect_avoiding(&def.effect, p, Some(pick));
                events.extend(self.cast_card_for_free(p, pick, Zone::Library, target, vec![], None, None, false)?);
            }
        }
        if keep_rest_on_top {
            return Ok(());
        }
        // CR 401.4 — the rest go to the bottom in a random order.
        let rest: Vec<CardId> = revealed.into_iter().filter(|&cid| picked.first() != Some(&cid)).collect();
        self.bottom_in_random_order(p, &rest);
        Ok(())
    }

    /// The walk only reads the library: the cast is offered where the card
    /// lies (a re-run on a prompting seat's answer walks the same cards), then
    /// every other revealed card goes to the bottom in a random order.
    pub(super) fn reveal_until_may_cast_free(
        &mut self,
        max_mv: &Value,
        filter: Option<&crate::card::SelectionRequirement>,
        ctx: &EffectContext,
        effect: &Effect,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let p = ctx.controller;
        let cap = self.evaluate_value(max_mv, ctx).max(0) as u32;
        let lib = &self.players[p].library;
        let hit = lib.iter().position(|c| {
            !c.definition.is_land()
                && self.zone_mana_value(c) <= cap
                && filter.is_none_or(|f| self.evaluate_requirement_on_card(f, c, p))
        });
        let revealed: Vec<CardId> = lib[..hit.map_or(lib.len(), |i| i + 1)].iter().map(|c| c.id).collect();
        let mut cast = None;
        if let Some(i) = hit {
            let card = &self.players[p].library[i];
            let (id, name) = (card.id, card.definition.name.to_string());
            let Some(picked) = self.choose_up_to_cards(
                p,
                "Cast the revealed card without paying its mana cost?".into(),
                ctx.source.unwrap_or(CardId(0)),
                vec![(id, name)],
                1,
                PickValue::Gain,
                effect,
                vec![id],
            ) else {
                return Ok(());
            };
            cast = picked.first().copied();
        }
        if let Some(pick) = cast
            && let Some(def) = self.find_card_anywhere(pick).map(|c| c.definition.arc())
        {
            let target = self.auto_target_for_effect_avoiding(&def.effect, p, Some(pick));
            events.extend(self.cast_card_for_free(p, pick, Zone::Library, target, vec![], None, None, false)?);
        }
        // CR 401.4 — the rest go to the bottom in a random order.
        let rest: Vec<CardId> = revealed.into_iter().filter(|&cid| cast != Some(cid)).collect();
        self.bottom_in_random_order(p, &rest);
        Ok(())
    }
}
