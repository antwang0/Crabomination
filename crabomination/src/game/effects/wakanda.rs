//! Wakanda Forever! — "reveal the top six cards of your library. You may put a
//! permanent card from among them onto the battlefield with an indestructible
//! counter on it. You may put a permanent card from among them into your hand.
//! Put the rest into your graveyard."

use super::EffectContext;
use crate::card::{CardId, Keyword};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameEvent, Target};

impl GameState {
    /// `Effect::RevealDeployOneTakeOne` — the highest-mana-value permanent card
    /// among the top `count` enters (with an indestructible counter when
    /// `indestructible`), the next goes to hand, the rest to the graveyard.
    pub(super) fn reveal_deploy_one_take_one(
        &mut self,
        count: &Value,
        indestructible: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        let p = ctx.controller;
        let n = self.evaluate_value(count, ctx).max(0) as usize;
        let top: Vec<CardId> = self.players[p].library.iter().take(n).map(|c| c.id).collect();
        let mut permanents: Vec<(u32, CardId)> = self.players[p]
            .library
            .iter()
            .take(n)
            .filter(|c| c.definition.is_permanent())
            .map(|c| (c.definition.cost.cmc(), c.id))
            .collect();
        permanents.sort_by_key(|p| std::cmp::Reverse(p.0));
        let deploy = permanents.first().map(|(_, id)| *id);
        let take = permanents.get(1).map(|(_, id)| *id);
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
