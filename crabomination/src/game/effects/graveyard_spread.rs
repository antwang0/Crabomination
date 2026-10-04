//! Winter, Cynical Opportunist's delirium end step: exile cards from your
//! graveyard with `min_types` card types among them, then put a permanent
//! card from among them onto the battlefield with a finality counter.

use super::EffectContext;
use crate::card::{CardId, CardType, CounterType};
use crate::decision::PickValue;
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// The controller picks the exiled set (any number of graveyard cards;
    /// fewer than `min_types` card types among them is a decline) and then
    /// the permanent card among them to return. A headless seat takes
    /// [`Self::type_spread_auto_pick`].
    pub(super) fn exile_type_spread_return_permanent(
        &mut self,
        min_types: u32,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        let (auto_set, auto_keep) = self.type_spread_auto_pick(me, min_types).unwrap_or_default();
        let candidates: Vec<(CardId, String)> =
            self.players[me].graveyard.iter().map(|c| (c.id, c.definition.name.to_string())).collect();
        let n = candidates.len() as u32;
        let mut cursor = 0;
        let Some(picks) = self.ask_seat_cards_logged(
            &mut cursor,
            me,
            format!("Exile cards with {min_types} or more card types among them?"),
            source,
            candidates,
            0,
            n,
            PickValue::Gain,
            effect,
            auto_set,
        ) else {
            return Ok(());
        };
        let gy = &self.players[me].graveyard;
        let cards: Vec<_> = gy.iter().filter(|c| picks.contains(&c.id)).collect();
        let mut types: Vec<CardType> = Vec::new();
        for c in &cards {
            for t in &c.definition.card_types {
                if !types.contains(t) {
                    types.push(t.clone());
                }
            }
        }
        if (types.len() as u32) < min_types {
            self.clear_answer_log();
            return Ok(());
        }
        let permanents: Vec<(CardId, String)> = cards
            .iter()
            .filter(|c| c.definition.is_permanent())
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        let best = cards
            .iter()
            .filter(|c| c.definition.is_permanent())
            .max_by_key(|c| (c.definition.cost.cmc(), std::cmp::Reverse(c.id.0)))
            .map(|c| c.id);
        let default_keep: Vec<CardId> = auto_keep.filter(|k| picks.contains(k)).or(best).into_iter().collect();
        let keep = if permanents.is_empty() {
            Vec::new()
        } else {
            let Some(k) = self.ask_seat_cards_logged(
                &mut cursor,
                me,
                "Put which permanent card onto the battlefield?".into(),
                source,
                permanents,
                1,
                1,
                PickValue::Gain,
                effect,
                default_keep,
            ) else {
                return Ok(());
            };
            k
        };
        self.clear_answer_log();
        // CR 122.6 — "onto the battlefield with a finality counter": it
        // enters with it (`SpellEntersWithCounters` stamps the card first).
        let mut body = vec![Effect::Move { what: Selector::ExactObjects(picks), to: ZoneDest::Exile }];
        if !keep.is_empty() {
            body.push(Effect::SpellEntersWithCounters {
                what: Selector::ExactObjects(keep.clone()),
                kind: CounterType::Finality,
                amount: Value::ONE,
            });
            body.push(Effect::Move {
                what: Selector::ExactObjects(keep),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            });
        }
        self.run_effect(&Effect::Seq(body), ctx, events)
    }

    /// The headless pick: the graveyard's greatest-mana-value permanent card
    /// and the cheapest cards adding each missing type, until `min_types`
    /// are covered. `None` when no such set exists.
    fn type_spread_auto_pick(&self, me: usize, min_types: u32) -> Option<(Vec<CardId>, Option<CardId>)> {
        let gy = &self.players[me].graveyard;
        let keep = gy
            .iter()
            .filter(|c| c.definition.is_permanent())
            .max_by_key(|c| (c.definition.cost.cmc(), std::cmp::Reverse(c.id.0)))?;
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
        ((types.len() as u32) >= min_types).then_some((picks, Some(keep.id)))
    }
}
