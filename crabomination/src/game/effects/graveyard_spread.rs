//! Winter, Cynical Opportunist's delirium end step: exile cards from your
//! graveyard with `min_types` card types among them, then put a permanent
//! card from among them onto the battlefield with a finality counter.

use super::EffectContext;
use crate::card::{CardId, CardType, CounterType};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// The picks are the engine's: the permanent card is the graveyard's
    /// greatest mana value, and the exiled set is it plus the cheapest card
    /// adding each missing type, until `min_types` are covered. With no such
    /// set nothing happens. The "may" is the controller's (`MayDo`); a bot
    /// seat always takes it.
    pub(super) fn exile_type_spread_return_permanent(
        &mut self,
        min_types: u32,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let gy = &self.players[me].graveyard;
        let Some(keep) = gy
            .iter()
            .filter(|c| c.definition.is_permanent())
            .max_by_key(|c| (c.definition.cost.cmc(), std::cmp::Reverse(c.id.0)))
        else {
            return Ok(());
        };
        let mut types: Vec<CardType> = keep.definition.card_types.clone();
        let mut picks: Vec<CardId> = vec![keep.id];
        let mut rest: Vec<_> = gy.iter().filter(|c| c.id != keep.id).collect();
        rest.sort_by_key(|c| (c.definition.cost.cmc(), c.id.0));
        for c in rest {
            if types.len() as u32 >= min_types {
                break;
            }
            if c.definition.card_types.iter().any(|t| !types.contains(t)) {
                for t in &c.definition.card_types {
                    if !types.contains(t) {
                        types.push(t.clone());
                    }
                }
                picks.push(c.id);
            }
        }
        if (types.len() as u32) < min_types {
            return Ok(());
        }
        let keep = keep.id;
        let body = Effect::Seq(vec![
            Effect::Move { what: Selector::ExactObjects(picks), to: ZoneDest::Exile },
            Effect::Move {
                what: Selector::ExactObjects(vec![keep]),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddCounter {
                what: Selector::ExactObjects(vec![keep]),
                kind: CounterType::Finality,
                amount: Value::ONE,
            },
        ]);
        if self.seat_prompts(me) {
            return self.run_effect(
                &Effect::MayDo {
                    description: "Exile cards with four card types to return a permanent?".into(),
                    body: Box::new(body),
                },
                ctx,
                events,
            );
        }
        self.run_effect(&body, ctx, events)
    }
}
