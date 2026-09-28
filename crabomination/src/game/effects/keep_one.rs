//! "Each player chooses a [filter] permanent they control, then sacrifices
//! (or: destroy) the rest" — Single Combat, Divine Reckoning, Deadly Vanity.

use crate::card::{CardId, SelectionRequirement};
use crate::decision::PickValue;
use crate::effect::{Effect, Selector};
use crate::game::effects::{EffectContext, EntityRef};
use crate::game::types::GameEvent;
use crate::game::{GameError, GameState};

impl GameState {
    /// `Effect::EachPlayerKeepsOneSacrificeRest` — every resolved player
    /// picks their keeper in APNAP order (CR 101.4; a bot keeps its highest
    /// mana value), then the rest go at once (CR 608.2c).
    pub(super) fn each_player_keeps_one(
        &mut self,
        who: &Selector,
        filter: &SelectionRequirement,
        destroy: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let seats: Vec<usize> = self
            .resolve_selector(who, ctx)
            .into_iter()
            .filter_map(|e| match e {
                EntityRef::Player(p) => Some(p),
                _ => None,
            })
            .collect();
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let mut doomed: Vec<(usize, CardId)> = Vec::new();
        for p in seats {
            let mine: Vec<&crate::card::CardInstance> = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p && self.evaluate_requirement_on_card(filter, c, p))
                .collect();
            if mine.is_empty() {
                continue;
            }
            let auto = mine.iter().max_by_key(|c| c.definition.cost.cmc()).map(|c| c.id);
            let candidates: Vec<(CardId, String)> =
                mine.iter().map(|c| (c.id, c.definition.name.to_string())).collect();
            let ids: Vec<CardId> = mine.iter().map(|c| c.id).collect();
            let Some(picked) = self.ask_seat_cards_logged(
                &mut cursor,
                p,
                "Choose the permanent you keep".into(),
                source,
                candidates,
                1,
                1,
                PickValue::Gain,
                effect,
                auto.into_iter().collect(),
            ) else {
                return Ok(());
            };
            // The choice is mandatory: an empty answer keeps the bot's pick.
            let keep = picked.first().copied().or(auto);
            doomed.extend(ids.into_iter().filter(|id| Some(*id) != keep).map(|id| (p, id)));
        }
        self.clear_answer_log();
        for (p, id) in doomed {
            if destroy {
                self.destroy_permanent(id, false, events);
            } else {
                self.sacrifice_one(id, p, events);
            }
        }
        Ok(())
    }
}
