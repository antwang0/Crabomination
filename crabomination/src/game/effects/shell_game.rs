//! "Starting with the next opponent in turn order, each player chooses a
//! [filter] … Destroy the chosen [permanents]" (Sadistic Shell Game) — every
//! seat chooses, the caster last, and the choices are destroyed together —
//! and its one-seat sibling, "destroy target [filter] that player controls of
//! their choice" (The Abyss).

use super::EffectContext;
use crate::card::{CardId, SelectionRequirement};
use crate::effect::{Effect, PlayerRef};
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

    /// `who` chooses one permanent they control matching `filter` (the
    /// filter's "you" is `who`) and it is destroyed. An unprompted seat gives
    /// up its weakest: the candidates go weakest first, and the ask's
    /// fallback is the first.
    pub(super) fn player_chooses_to_destroy(
        &mut self,
        who: &PlayerRef,
        filter: &SelectionRequirement,
        no_regen: bool,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(seat) = self.resolve_player(who, ctx) else { return Ok(()) };
        let source = ctx.source.unwrap_or(CardId(0));
        let mut owned: Vec<(u32, i32, CardId)> = self
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == seat
                    && self.evaluate_requirement_static(filter, &Target::Permanent(c.id), seat, Some(source))
            })
            .map(|c| (c.definition.cost.cmc(), c.definition.power, c.id))
            .collect();
        if owned.is_empty() {
            return Ok(());
        }
        owned.sort();
        let legal: Vec<Target> = owned.into_iter().map(|(_, _, id)| Target::Permanent(id)).collect();
        let mut cursor = 0;
        let picked = self.ask_seat_target_logged(
            &mut cursor,
            seat,
            format!("P{seat}: choose a creature you control to destroy"),
            source,
            legal,
            effect,
        );
        self.clear_answer_log();
        if let Some(Target::Permanent(id)) = picked {
            self.destroy_permanent(id, no_regen, events);
        }
        Ok(())
    }
}
