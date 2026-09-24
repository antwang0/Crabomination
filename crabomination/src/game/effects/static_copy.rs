//! CR 707.2 / 614.1c — "creatures you control enter as a copy of X", the
//! static form of `enters_as_copy` (Infinite Reflection).

use crate::card::CardId;
use crate::effect::{Effect, Selector, StaticEffect};
use crate::game::effects::EffectContext;
use crate::game::types::GameEvent;
use crate::game::{GameState, Target};

impl GameState {
    /// CR 707.2 — every card in every graveyard matching `filter`, as
    /// `(id, name, printed power)`: the copy sources of an `EntersAsCopy`
    /// with `from_graveyards` (Body Double's "any creature card in a
    /// graveyard"). Evaluated against the card, so a type filter reads its
    /// printed characteristics.
    pub(crate) fn graveyard_copy_candidates(
        &self,
        filter: &crate::card::SelectionRequirement,
        controller: usize,
    ) -> Vec<(CardId, String, i32)> {
        self.players
            .iter()
            .flat_map(|p| p.graveyard.iter())
            .filter(|c| self.evaluate_requirement_on_card(filter, c, controller))
            .map(|c| (c.id, c.definition.name.to_string(), c.definition.power))
            .collect()
    }

    /// Apply a `StaticEffect::CreaturesEnterAsCopyOf` that `controller`'s
    /// permanents hold over the entering `card_id`. With several, the last one
    /// found is applied last and wins (CR 616.1 — the controller's order).
    pub(crate) fn apply_static_enters_as_copy(
        &mut self,
        card_id: CardId,
        controller: usize,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        let mut specs: Vec<(CardId, Selector)> = Vec::new();
        for src in self.battlefield.iter() {
            if src.controller != controller || src.id == card_id {
                continue;
            }
            for sa in src.definition.static_abilities.iter() {
                if let StaticEffect::CreaturesEnterAsCopyOf { filter, of } = &sa.effect
                    && self.evaluate_requirement_static(
                        filter,
                        &Target::Permanent(card_id),
                        controller,
                        Some(src.id),
                    )
                {
                    specs.push((src.id, of.clone()));
                }
            }
        }
        let mut model = None;
        for (src, of) in specs {
            let ctx = EffectContext::for_trigger(src, controller, None, 0);
            if let Some(id) = self
                .resolve_selector(&of, &ctx)
                .into_iter()
                .filter_map(|e| e.as_permanent_id())
                .find(|&id| id != card_id)
            {
                model = Some(id);
            }
        }
        let Some(model) = model else { return false };
        let ctx = EffectContext::for_trigger(card_id, controller, Some(Target::Permanent(model)), 0);
        if let Ok(evs) = self.resolve_effect_driven(
            &Effect::BecomeCopyOf {
                what: Selector::This,
                source: Selector::Target(0),
                extra_creature_types: vec![],
                keep_own_triggered: false,
                keep_own_activated: false,
            },
            &ctx,
        ) {
            events.extend(evs);
        }
        true
    }

    /// CR 606.3 override — `seat` holds an emblem letting them activate
    /// loyalty abilities at instant speed (Teferi, Temporal Archmage).
    pub(crate) fn loyalty_at_instant_speed(&self, seat: usize) -> bool {
        self.players.get(seat).is_some_and(|p| {
            p.emblems.iter().any(|e| {
                e.statics
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::LoyaltyAbilitiesAtInstantSpeed))
            })
        })
    }
}
