//! "Reveal the top X cards of your library. You may cast a spell with mana
//! value X or less from among them without paying its mana cost. Put the rest
//! on the bottom of your library in a random order." — Sunbird's Invocation.

use super::{EffectContext, PickValue};
use crate::card::{CardId, Zone};
use crate::effect::{Effect, LibraryPosition, PlayerRef, Value, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// The choice is made over the cards where they lie, BEFORE anything
    /// moves: a prompting seat suspends inside `choose_up_to_cards` and the
    /// effect re-runs on the answer, which must not reveal a second batch.
    pub(super) fn reveal_top_may_cast_one_free(
        &mut self,
        count: &Value,
        max_mv: &Value,
        filter: Option<&crate::card::SelectionRequirement>,
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
        // CR 401.4 — the rest go to the bottom; the engine's bottom order is
        // the same hidden-order approximation cascade uses.
        for cid in revealed {
            if cid != picked.first().copied().unwrap_or(CardId(u32::MAX))
                && self.players[p].library.iter().any(|c| c.id == cid)
            {
                self.move_card_to(
                    cid,
                    &ZoneDest::Library { who: PlayerRef::Seat(p), pos: LibraryPosition::Bottom },
                    ctx,
                    events,
                );
            }
        }
        Ok(())
    }
}
