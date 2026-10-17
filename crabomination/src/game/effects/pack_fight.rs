//! Master of the Wild Hunt: your untapped pack taps to maul a creature, which
//! answers with its power divided among the pack by its controller.

use super::{EffectContext, EntityRef, PendingEffectState};
use crate::card::{CardId, SelectionRequirement};
use crate::decision::{Decision, DecisionAnswer};
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent, Target};

impl GameState {
    /// `Effect::TapMatchingDealPowerAndTakeDivided`. The division is the only
    /// question, so it is asked before anything moves (a suspend's re-run then
    /// starts clean); the creature's power can't change between the two.
    pub(super) fn tap_matching_deal_power_and_take_divided(
        &mut self,
        filter: &SelectionRequirement,
        target: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let Some(prey) = self.resolve_selector(target, ctx).iter().find_map(|e| e.as_permanent_id()) else {
            return Ok(());
        };
        let Some(chooser) = self.battlefield_find(prey).map(|c| c.controller) else { return Ok(()) };
        let pack: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == me
                    && !c.tapped
                    && self.computed_is_creature(c)
                    && self.evaluate_requirement_static_on(filter, c, me, ctx.source)
            })
            .map(|c| c.id)
            .collect();
        let prey_power = self.battlefield_find(prey).map_or(0, |c| self.effective_power(c).max(0) as u32);
        let mut division = vec![0; pack.len()];
        if prey_power > 0 && !pack.is_empty() {
            let decision = Decision::DivideDamage {
                source: prey,
                total: prey_power,
                targets: pack.iter().map(|&id| Target::Permanent(id)).collect(),
                noun: "damage".into(),
            };
            let answer = match take_opt_scratch!(self.stashed_resolution_answer) {
                Some(a) => Some(a),
                None if self.seat_suspends(chooser) => {
                    self.suspend_signal = Some(Box::new((
                        decision,
                        PendingEffectState::seated(chooser, PendingEffectState::DivisionAnswerPending),
                        effect.clone(),
                    )));
                    return Ok(());
                }
                // A scripted seat answers; a headless one kills the most it can.
                None if !matches!(self.decider.kind(), crate::decision::DeciderKind::Auto) => {
                    Some(self.decider.decide(&decision))
                }
                None => None,
            };
            division = match answer {
                Some(DecisionAnswer::DamageDivision(v))
                    if v.len() == pack.len() && v.iter().sum::<u32>() == prey_power =>
                {
                    v
                }
                _ => self.lethal_first_division(&pack, prey_power),
            };
        }
        for &w in &pack {
            if let Some(c) = self.battlefield_find_mut(w) {
                c.tapped = true;
            }
        }
        for &w in &pack {
            let power = self.battlefield_find(w).map_or(0, |c| self.effective_power(c).max(0) as u32);
            if power > 0 {
                self.deal_damage_to_from(EntityRef::Permanent(prey), power, Some(w), events);
            }
        }
        for (&w, n) in pack.iter().zip(division) {
            if n > 0 {
                self.deal_damage_to_from(EntityRef::Permanent(w), n, Some(prey), events);
            }
        }
        Ok(())
    }

    /// Smallest toughness first, each taking exactly lethal, the rest on the
    /// last one hit — the division that kills the most.
    fn lethal_first_division(&self, pack: &[CardId], total: u32) -> Vec<u32> {
        let mut order: Vec<(usize, u32)> = pack
            .iter()
            .enumerate()
            .map(|(i, &id)| {
                let left = self.battlefield_find(id).map_or(1, |c| {
                    (self.effective_toughness(c) - c.damage as i32).max(1) as u32
                });
                (i, left)
            })
            .collect();
        order.sort_by_key(|&(i, left)| (left, i));
        let mut out = vec![0; pack.len()];
        let mut rest = total;
        let mut last = None;
        for (i, left) in order {
            if rest < left {
                break;
            }
            out[i] = left;
            rest -= left;
            last = Some(i);
        }
        if rest > 0 {
            out[last.unwrap_or(0)] += rest;
        }
        out
    }
}
