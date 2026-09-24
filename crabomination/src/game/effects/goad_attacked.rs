//! Firkraag, Cunning Instigator — "whenever one or more Dragons you control
//! attack an opponent, goad target creature that player controls."

use super::EffectContext;
use crate::card::{CardId, SelectionRequirement};
use crate::game::GameState;
use crate::game::types::{AttackTarget, GameError};

impl GameState {
    /// `Effect::GoadACreatureOfEachOpponentAttackedBy` — CR 701.15a: once per
    /// opponent that an attacking creature you control matching `attackers`
    /// is attacking, goad one creature that opponent controls. The creature
    /// is the engine's pick — the greatest power among those you don't already
    /// goad — standing in for the printed target.
    pub(super) fn goad_a_creature_of_each_opponent_attacked_by(
        &mut self,
        attackers: &SelectionRequirement,
        ctx: &EffectContext,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let mut defenders: Vec<usize> = Vec::new();
        for a in self.attacking.iter() {
            let AttackTarget::Player(d) = a.target else { continue };
            if defenders.contains(&d) || self.same_team(me, d) {
                continue;
            }
            let matches = self.battlefield_find(a.attacker).is_some_and(|c| {
                c.controller == me && self.evaluate_requirement_on_card(attackers, c, me)
            });
            if matches {
                defenders.push(d);
            }
        }
        defenders.sort_unstable();
        for d in defenders {
            let pick: Option<CardId> = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == d && c.definition.is_creature() && !c.goaded_by.contains(&me)
                })
                .max_by_key(|c| (c.power(), std::cmp::Reverse(c.id.0)))
                .map(|c| c.id);
            if let Some(cid) = pick
                && let Some(c) = self.battlefield_find_mut(cid)
            {
                c.goaded_by.push(me);
            }
        }
        Ok(())
    }
}
