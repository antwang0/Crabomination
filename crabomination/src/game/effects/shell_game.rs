//! "Starting with the next opponent in turn order, each player chooses a
//! [filter] … Destroy the chosen [permanents]" (Sadistic Shell Game) — every
//! seat chooses, the caster last, and the choices are destroyed together.

use super::EffectContext;
use crate::card::{CardId, SelectionRequirement};
use crate::effect::Effect;
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent, Target};

impl GameState {
    /// The filter reads "you" as the resolving controller ("a creature you
    /// don't control"); two players may choose the same permanent. Each seat's
    /// pick goes to that seat's decider on the cursor-indexed channel, like
    /// Grenzo's Rebuttal's neighbour picks.
    pub(super) fn each_player_chooses_to_destroy(
        &mut self,
        filter: &SelectionRequirement,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let source = ctx.source.unwrap_or(CardId(0));
        let mut order = self.seats_in_turn_order_from(ctx.controller);
        order.rotate_left(1);
        let legal: Vec<Target> = self
            .battlefield
            .iter()
            .filter(|c| {
                self.evaluate_requirement_static(filter, &Target::Permanent(c.id), ctx.controller, Some(source))
            })
            .map(|c| Target::Permanent(c.id))
            .collect();
        if legal.is_empty() {
            return Ok(());
        }
        let mut cursor = 0;
        let mut chosen: Vec<CardId> = Vec::new();
        for seat in order {
            let Some(picked) = self.ask_seat_target_logged(
                &mut cursor,
                seat,
                format!("P{seat}: choose a creature to destroy"),
                source,
                legal.clone(),
                effect,
            ) else {
                return Ok(());
            };
            if let Target::Permanent(id) = picked
                && !chosen.contains(&id)
            {
                chosen.push(id);
            }
        }
        self.clear_answer_log();
        for id in chosen {
            self.destroy_permanent(id, false, events);
        }
        Ok(())
    }
}
