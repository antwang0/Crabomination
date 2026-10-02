//! Wakanda Forever! — "reveal the top six cards of your library. You may put a
//! permanent card from among them onto the battlefield with an indestructible
//! counter on it. You may put a permanent card from among them into your hand.
//! Put the rest into your graveyard."

use super::EffectContext;
use crate::card::{CardId, Keyword};
use crate::decision::PickValue;
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameEvent, Target};

impl GameState {
    /// `Effect::RevealDeployOneTakeOne` — among the top `count`, the caster
    /// may put a permanent card onto the battlefield (with an indestructible
    /// counter when `indestructible`) and may put another into hand; the rest
    /// go to the graveyard. Both picks are asked before anything moves; a
    /// headless seat deploys the priciest and takes the next.
    pub(super) fn reveal_deploy_one_take_one(
        &mut self,
        count: &Value,
        indestructible: bool,
        ctx: &EffectContext,
        effect: &Effect,
        events: &mut Vec<GameEvent>,
    ) {
        let p = ctx.controller;
        let n = self.evaluate_value(count, ctx).max(0) as usize;
        let top: Vec<CardId> = self.players[p].library.iter().take(n).map(|c| c.id).collect();
        let mut permanents: Vec<(u32, CardId, String)> = self.players[p]
            .library
            .iter()
            .take(n)
            .filter(|c| c.definition.is_permanent())
            .map(|c| (c.definition.cost.cmc(), c.id, c.definition.name.to_string()))
            .collect();
        permanents.sort_by_key(|p| std::cmp::Reverse(p.0));
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let offer = |except: Option<CardId>| -> Vec<(CardId, String)> {
            permanents.iter().filter(|(_, id, _)| Some(*id) != except).map(|(_, id, name)| (*id, name.clone())).collect()
        };
        let first = offer(None);
        let deploy = if first.is_empty() {
            None
        } else {
            let auto = vec![first[0].0];
            let Some(v) = self.ask_seat_cards_logged(
                &mut cursor,
                p,
                "Put a permanent card onto the battlefield?".into(),
                source,
                first,
                0,
                1,
                PickValue::Gain,
                effect,
                auto,
            ) else {
                return;
            };
            v.first().copied()
        };
        let second = offer(deploy);
        let take = if second.is_empty() {
            None
        } else {
            let auto = vec![second[0].0];
            let Some(v) = self.ask_seat_cards_logged(
                &mut cursor,
                p,
                "Put a permanent card into your hand?".into(),
                source,
                second,
                0,
                1,
                PickValue::Gain,
                effect,
                auto,
            ) else {
                return;
            };
            v.first().copied()
        };
        self.clear_answer_log();
        if let Some(id) = deploy {
            self.move_card_to(id, &ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false }, ctx, events);
            if indestructible && self.battlefield_find(id).is_some() {
                let mut c = ctx.clone();
                c.targets = vec![Target::Permanent(id)];
                let _ = self.run_effect(
                    &Effect::AddKeywordCounter { what: Selector::Target(0), keyword: Keyword::Indestructible, amount: Value::ONE },
                    &c,
                    events,
                );
            }
        }
        if let Some(id) = take {
            self.move_card_to(id, &ZoneDest::Hand(PlayerRef::You), ctx, events);
        }
        for id in top.into_iter().filter(|id| Some(*id) != deploy && Some(*id) != take) {
            if self.players[p].library.iter().any(|c| c.id == id) {
                self.move_card_to(id, &ZoneDest::Graveyard, ctx, events);
            }
        }
    }
}
