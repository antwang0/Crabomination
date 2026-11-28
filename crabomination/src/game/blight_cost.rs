//! CR 701.68 — "Blight N" as an activation cost (Dawnhand Dissident's "{T},
//! Blight 1: Surveil 1."): put N -1/-1 counters on a creature you control.
//! Unpayable with no creature. A prompting seat with a real choice is asked
//! (CR 701.68a, `AbilityCostChoice::Blight`); otherwise the pick is
//! automatic, like Ward—Blight's: a survivor first, the weakest of those.

use super::GameState;
use super::types::GameEvent;
use crate::card::{CardId, CounterType};

impl GameState {
    /// The creatures `p` could blight.
    pub(crate) fn blight_candidates(&self, p: usize) -> Vec<CardId> {
        self.battlefield.iter().filter(|c| c.controller == p && self.computed_is_creature(c)).map(|c| c.id).collect()
    }

    /// Put `n` -1/-1 counters on `chosen`, or on the auto-picked creature `p`
    /// controls. A cost that puts counters is still a placement (CR 614.16).
    pub(crate) fn pay_blight(
        &mut self,
        p: usize,
        n: u32,
        chosen: Option<CardId>,
        events: &mut Vec<GameEvent>,
    ) -> Option<CardId> {
        let pick = match chosen {
            Some(id) => id,
            None => self
                .battlefield
                .iter()
                .filter(|c| c.controller == p && self.computed_is_creature(c))
                .filter_map(|c| self.computed_permanent(c.id).map(|cp| (c.id, cp.power, cp.toughness)))
                .min_by_key(|&(_, power, toughness)| (toughness <= n as i32, power))
                .map(|(id, _, _)| id)?,
        };
        let put = self.scaled_counter_count_on(pick, CounterType::MinusOneMinusOne, n);
        if put > 0
            && let Some(c) = self.battlefield_find_mut(pick)
        {
            c.add_counters(CounterType::MinusOneMinusOne, put);
            events.push(GameEvent::CounterAdded {
                card_id: pick,
                counter_type: CounterType::MinusOneMinusOne,
                count: put,
                placer: self.resolution_causer,
            });
        }
        Some(pick)
    }
}

impl GameState {
    /// CR 602.5 — suspend an activation for a prompting `p`'s battlefield
    /// pick among `legal` for one of its costs; `kind` names the cost. Nothing
    /// is paid yet, so the answer replays the activation from the top.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn pose_ability_cost_pick(
        &mut self,
        p: usize,
        card_id: CardId,
        ability_index: usize,
        target: Option<crate::game::types::Target>,
        additional_targets: &[crate::game::types::Target],
        x_value: Option<u32>,
        legal: Vec<CardId>,
        description: String,
        kind: crate::game::types::AbilityCostChoice,
    ) {
        let source_name = self.find_card_anywhere(card_id).map(|c| c.definition.name.to_string()).unwrap_or_default();
        self.pending_decision = Some(Box::new(crate::game::types::PendingDecision {
            decision: crate::decision::Decision::ChooseTarget {
                optional: false,
                source: card_id,
                legal: legal.into_iter().map(crate::game::types::Target::Permanent).collect(),
                source_name,
                description,
                extra_cast_slot: false,
            },
            resume: crate::game::types::ResumeContext::ActivateAbilityChoice {
                activator: p,
                card_id,
                ability_index,
                target,
                additional_targets: additional_targets.to_vec(),
                x_value,
                kind,
            },
        }));
    }
}
