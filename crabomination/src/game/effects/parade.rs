//! Parade! (Starlight Spectacular): the controller lines up every creature
//! they control one at a time; each gets +1/+1 until end of turn for each
//! creature chosen before it.

use super::{EffectContext, PickValue};
use crate::card::CardId;
use crate::effect::{Duration, Effect, Selector, Value};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// `Effect::Parade` — one ask, its pick order the line (a re-asked
    /// pick per creature replayed wrong after a seat suspended); creatures
    /// left unpicked join the end. Headless, the smallest goes first so the
    /// biggest bonus lands on the biggest body.
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
        if left.len() < 2 {
            return Ok(());
        }
        left.sort_by_key(|(id, p, _)| (*p, *id));
        let by_power: Vec<CardId> = left.iter().map(|(id, ..)| *id).collect();
        let Some(picked) = self.choose_up_to_cards(
            me,
            "Parade: line your creatures up, first to last".to_string(),
            ctx.source.unwrap_or(CardId(0)),
            left.iter().map(|(id, _, n)| (*id, n.clone())).collect(),
            left.len() as u32,
            PickValue::Gain,
            effect,
            by_power.clone(),
        ) else {
            return Ok(());
        };
        let mut line: Vec<CardId> = Vec::with_capacity(by_power.len());
        for id in picked.into_iter().chain(by_power) {
            if left.iter().any(|(c, ..)| *c == id) && !line.contains(&id) {
                line.push(id);
            }
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
