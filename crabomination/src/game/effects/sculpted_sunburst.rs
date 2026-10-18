//! Sculpted Sunburst — "Choose a creature you control, then each opponent
//! chooses a creature they control with equal or lesser power. If you chose
//! a creature this way, exile each creature not chosen by any player this
//! way." (`Effect::ChooseCreaturesByPowerExileRest`.) The choices are made in
//! APNAP order after the caster's (CR 101.4); the exile is simultaneous.

use super::EffectContext;
use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::{Effect, Selector, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// Creatures `seat` controls as `(id, name, power)`, strongest first.
    fn sunburst_creatures(&self, seat: usize) -> Vec<(CardId, String, i32)> {
        let mut out: Vec<(CardId, String, i32)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && self.computed_is_creature(c))
            .filter_map(|c| self.computed_permanent(c.id).map(|cp| (c.id, c.definition.name.to_string(), cp.power)))
            .collect();
        out.sort_by_key(|&(id, _, power)| (std::cmp::Reverse(power), id));
        out
    }

    pub(super) fn choose_creatures_by_power_exile_rest(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let mine = self.sunburst_creatures(me);
        if mine.is_empty() {
            return Ok(());
        }
        let Some(pick) = self.ask_seat_cards_logged(
            &mut cursor,
            me,
            "Choose a creature you control".into(),
            source,
            mine.iter().map(|c| (c.0, c.1.clone())).collect(),
            1,
            1,
            PickValue::Gain,
            effect,
            vec![mine[0].0],
        ) else {
            return Ok(());
        };
        let Some(&(chosen, _, power)) = pick.first().and_then(|id| mine.iter().find(|c| c.0 == *id)) else {
            return Ok(());
        };
        let mut kept = vec![chosen];
        for opp in self.apnap_sort(self.opponents_of(me)) {
            let theirs: Vec<(CardId, String, i32)> =
                self.sunburst_creatures(opp).into_iter().filter(|c| c.2 <= power).collect();
            if theirs.is_empty() {
                continue;
            }
            let Some(p) = self.ask_seat_cards_logged(
                &mut cursor,
                opp,
                format!("Choose a creature you control with power {power} or less"),
                source,
                theirs.iter().map(|c| (c.0, c.1.clone())).collect(),
                1,
                1,
                PickValue::Gain,
                effect,
                vec![theirs[0].0],
            ) else {
                return Ok(());
            };
            if let Some(id) = p.first().filter(|id| theirs.iter().any(|c| c.0 == **id)) {
                kept.push(*id);
            }
        }
        self.clear_answer_log();
        let doomed: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| self.computed_is_creature(c) && !kept.contains(&c.id))
            .map(|c| c.id)
            .collect();
        if doomed.is_empty() {
            return Ok(());
        }
        self.run_effect(&Effect::Move { what: Selector::ExactObjects(doomed), to: ZoneDest::Exile }, ctx, events)
    }
}
