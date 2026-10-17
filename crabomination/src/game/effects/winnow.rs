//! Winnowing: you choose a creature for each player; each sacrifices every
//! other creature that shares no creature type with that choice.

use super::EffectContext;
use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::Effect;
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// `Effect::EachPlayerSacrificesUnlikeChosen`. Every pick is the
    /// controller's, asked seat by seat in turn order before anything is
    /// sacrificed; headless, an opponent's pick spares the fewest and yours the
    /// most. Sacrifices are simultaneous (CR 701.21a — each player's own).
    pub(super) fn each_player_sacrifices_unlike_chosen(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let mut doomed: Vec<(CardId, usize)> = Vec::new();
        for seat in self.seats_in_turn_order_from(me) {
            let creatures: Vec<(CardId, String)> = self
                .battlefield
                .iter()
                .filter(|c| c.controller == seat && self.computed_is_creature(c))
                .map(|c| (c.id, c.definition.name.to_string()))
                .collect();
            if creatures.is_empty() {
                continue;
            }
            let spared = |keep: CardId| {
                creatures.iter().filter(|c| c.0 != keep && self.permanents_share_creature_type(keep, c.0)).count()
            };
            let mine = seat == me;
            let auto = creatures
                .iter()
                .max_by_key(|c| {
                    let n = spared(c.0) as i64;
                    (if mine { n } else { -n }, std::cmp::Reverse(c.0.0))
                })
                .map(|c| c.0)
                .unwrap_or(creatures[0].0);
            let Some(picked) = self.ask_seat_cards_logged(
                &mut cursor,
                me,
                format!("Choose a creature P{seat} controls"),
                source,
                creatures.iter().map(|c| (c.0, c.1.clone())).collect(),
                1,
                1,
                if mine { PickValue::Gain } else { PickValue::Cost },
                effect,
                vec![auto],
            ) else {
                return Ok(());
            };
            let chosen = picked.first().copied().unwrap_or(auto);
            if seat != me && self.player_cant_be_made_to_sacrifice(seat) {
                continue;
            }
            doomed.extend(
                creatures
                    .iter()
                    .filter(|c| c.0 != chosen && !self.permanents_share_creature_type(chosen, c.0))
                    .map(|c| (c.0, seat)),
            );
        }
        self.clear_answer_log();
        for (id, seat) in doomed {
            self.sacrifice_one(id, seat, events);
        }
        Ok(())
    }
}
