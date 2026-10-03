//! "You may choose new targets for any number of other spells and/or
//! abilities" (Boltbender, CR 115.7d). Nothing is targeted, so which items
//! are re-aimed is the controller's choice; the policy picks every spell or
//! ability an opponent controls that is aimed at the controller's side, and
//! each is repointed by the same slot walk Redirect uses.

use super::GameState;
use super::effects::EffectContext;
use super::types::{GameEvent, StackItem, Target};
use crate::card::CardId;
use crate::effect::{Effect, Selector};
use crate::game::GameError;

impl GameState {
    pub(crate) fn choose_new_targets_for_any_number(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let ours = |t: &Target| match t {
            Target::Player(p) => self.same_team(*p, me),
            Target::Permanent(id) => self.battlefield_find(*id).is_some_and(|c| self.same_team(c.controller, me)),
        };
        let ids: Vec<CardId> = self
            .stack
            .iter()
            .filter_map(|si| match si {
                StackItem::Spell { card, caster, target, additional_targets, .. }
                    if !self.same_team(*caster, me) && target.iter().chain(additional_targets).any(ours) =>
                {
                    Some(card.id)
                }
                StackItem::Trigger { ability_id, controller, target, additional_targets, .. }
                    if *ability_id != 0
                        && !self.same_team(*controller, me)
                        && target.iter().chain(additional_targets).any(ours) =>
                {
                    Some(CardId(*ability_id))
                }
                _ => None,
            })
            .collect();
        if ids.is_empty() {
            return Ok(());
        }
        self.run_effect(&Effect::ChooseNewTargetsForSpell { what: Selector::ExactObjects(ids) }, ctx, events)
    }
}
