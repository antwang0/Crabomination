//! CR 614 — draw replacements that only apply to a draw from an empty
//! library (Out of the Tombs). Laboratory Maniac's win is read at the loss
//! site instead (`lose_to_empty_draw`).

use super::GameState;
use crate::card::CardId;
use super::types::GameEvent;
use crate::effect::{Effect, PlayerRef, Selector, StaticEffect, ZoneDest};

impl GameState {
    /// Out of the Tombs — "If you would draw a card while your library has no
    /// cards in it, instead return a creature card from your graveyard to the
    /// battlefield." `true` when a static applied and a creature came back;
    /// `false` leaves the empty draw to CR 104.3c ("if you can't, you lose").
    pub(crate) fn reanimate_instead_of_empty_draw(&mut self, p: usize, events: &mut Vec<GameEvent>) -> bool {
        let Some(source) = self.battlefield.iter().find_map(|c| {
            (c.controller == p
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(self.active_static(&sa.effect, c), Some(StaticEffect::ReanimateInsteadOfDrawFromEmpty))
                }))
            .then_some(c.id)
        }) else {
            return false;
        };
        // The draw funnel can't suspend for an ask (one left pending here was
        // owed by a seat the failed draw then decked — seed 17703, four-seat
        // pod), so it is asked off the stack like an as-enters choice: a
        // prompting seat's policy answers, a headless one the decider. The
        // offer runs greatest mana value first, and a non-answer takes it.
        let mut creatures: Vec<(CardId, String, u32)> = self.players[p]
            .graveyard
            .iter()
            .filter(|c| self.computed_is_creature(c))
            .map(|c| (c.id, c.definition.name.to_string(), c.definition.cost.cmc()))
            .collect();
        if creatures.is_empty() {
            return false;
        }
        creatures.reverse();
        creatures.sort_by_key(|c| std::cmp::Reverse(c.2));
        let decision = crate::decision::Decision::ChooseCards {
            source,
            prompt: "Return which creature card to the battlefield?".into(),
            candidates: creatures.iter().map(|(id, name, _)| (*id, name.clone())).collect(),
            min: 1,
            max: 1,
            eligible: None,
            value: crate::decision::PickValue::Gain,
        };
        let answer = if self.seat_prompts(p) {
            crate::server::bot::decide_pending_policy(self, p, &crate::server::bot::EvalWeights::default(), &decision, false)
        } else {
            self.decider.decide(&decision)
        };
        let picked = match answer {
            crate::decision::DecisionAnswer::Cards(ids) => ids.first().copied(),
            _ => None,
        };
        let pick = picked.filter(|id| creatures.iter().any(|c| c.0 == *id)).unwrap_or(creatures[0].0);
        let ctx = super::effects::EffectContext::for_ability(source, p, None);
        let back = Effect::Move {
            what: Selector::ExactObjects(vec![pick]),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        };
        let _ = self.run_effect(&back, &ctx, events);
        !self.players[p].graveyard.iter().any(|c| c.id == pick)
    }
}
