//! Parade! (Starlight Spectacular): the controller lines up every creature
//! they control one at a time; each gets +1/+1 until end of turn for each
//! creature chosen before it.

use super::{EffectContext, PickValue};
use crate::card::CardId;
use crate::effect::{Duration, Effect, Selector, Value};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// `Effect::Parade` — every pick is asked first (a suspended seat replays
    /// them), then the line is pumped. Headless, the smallest creature goes
    /// first so the biggest bonus lands on the biggest body.
    pub(super) fn parade(&mut self, effect: &Effect, ctx: &EffectContext, events: &mut Vec<GameEvent>) -> Result<(), GameError> {
        let me = ctx.controller;
        let mut left: Vec<(CardId, i32, String)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == me && self.computed_is_creature(c))
            .map(|c| {
                let power = self.computed_permanent(c.id).map_or(c.definition.power, |cp| cp.power);
                (c.id, power, c.definition.name.to_string())
            })
            .collect();
        left.sort_by_key(|(id, p, _)| (*p, *id));
        let mut line = Vec::with_capacity(left.len());
        while let Some(&(first, ..)) = left.first() {
            let Some(picked) = self.choose_up_to_cards(
                me,
                format!("Parade: which creature is number {}?", line.len() + 1),
                ctx.source.unwrap_or(CardId(0)),
                left.iter().map(|(id, _, n)| (*id, n.clone())).collect(),
                1,
                PickValue::Gain,
                effect,
                vec![first],
            ) else {
                return Ok(());
            };
            let pick = picked.into_iter().find(|id| left.iter().any(|(c, ..)| c == id)).unwrap_or(first);
            left.retain(|(c, ..)| *c != pick);
            line.push(pick);
        }
        for (before, id) in line.into_iter().enumerate().skip(1) {
            let n = Value::Const(before as i32);
            self.run_effect(
                &Effect::PumpPT { what: Selector::ExactObjects(vec![id]), power: n.clone(), toughness: n, duration: Duration::EndOfTurn },
                ctx,
                events,
            )?;
        }
        Ok(())
    }
}
