//! Sudden Demise — "choose a color; deal X damage to each creature of the
//! chosen color".

use super::EffectContext;
use crate::card::CardId;
use crate::effect::{Selector, Value};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};
use crate::mana::Color;

impl GameState {
    /// `Effect::DamageEachCreatureOfChosenColor`. The caster chooses the
    /// color as it resolves; the colors are offered best first — the most
    /// opposing mana value `amount` would kill, net of the caster's own, ties
    /// in WUBRG order — so a headless seat's option 0 is that pick.
    pub(super) fn damage_each_creature_of_chosen_color(
        &mut self,
        amount: &Value,
        ctx: &EffectContext,
        effect: &crate::effect::Effect,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let n = self.evaluate_value(amount, ctx).max(0);
        let me = ctx.controller;
        let of_color = |g: &GameState, color: Color| -> Vec<CardId> {
            g.battlefield
                .iter()
                .filter(|c| {
                    g.computed_permanent(c.id).is_some_and(|cp| {
                        cp.card_types().contains(&crate::card::CardType::Creature) && cp.colors.contains(color)
                    })
                })
                .map(|c| c.id)
                .collect()
        };
        let score = |color: Color| -> i64 {
            of_color(self, color)
                .into_iter()
                .filter_map(|id| {
                    let c = self.battlefield_find(id)?;
                    let dies = self.computed_permanent(id).is_some_and(|cp| cp.toughness <= n);
                    let v = i64::from(c.definition.cost.cmc()) + 1;
                    dies.then_some(if c.controller == me { -v } else { v })
                })
                .sum()
        };
        let mut order: Vec<Color> = Color::ALL.to_vec();
        order.sort_by_key(|&c| (std::cmp::Reverse(score(c)), c as u8));
        let mut cursor = 0;
        let Some(i) = self.ask_seat_option(
            &mut cursor,
            me,
            "Choose a color".into(),
            ctx.source.unwrap_or(CardId(0)),
            order.iter().map(|c| format!("{c:?}")).collect(),
            effect,
        ) else {
            return Ok(());
        };
        self.clear_answer_log();
        let color = order.get(i).copied().unwrap_or(Color::White);
        let victims = of_color(self, color);
        self.run_effect(
            &crate::effect::Effect::DealDamage { to: Selector::ExactObjects(victims), amount: Value::Const(n) },
            ctx,
            events,
        )
    }
}
