//! "Choose any number of [them]. [Each of those does …]" — the controller's
//! subset pick, then a `ForEach` over it (`Effect::ForEachChosen`, CR 608.2d).

use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::effects::EffectContext;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    pub(crate) fn for_each_chosen(
        &mut self,
        from: &Selector,
        body: &Effect,
        headless_takes_all: bool,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let ids: Vec<CardId> = self.resolve_selector(from, ctx).into_iter().filter_map(|e| e.as_card_id()).collect();
        if ids.is_empty() {
            return Ok(());
        }
        let candidates: Vec<(CardId, String)> = ids
            .iter()
            .filter_map(|id| self.find_card_anywhere(*id).map(|c| (*id, c.definition.name.to_string())))
            .collect();
        let Some(picked) = self.choose_up_to_cards(
            ctx.controller,
            "Choose any number".to_string(),
            ctx.source.unwrap_or(CardId(0)),
            candidates,
            ids.len() as u32,
            PickValue::Gain,
            effect,
            if headless_takes_all { ids.clone() } else { Vec::new() },
        ) else {
            return Ok(());
        };
        let picked: Vec<CardId> = picked.into_iter().filter(|id| ids.contains(id)).collect();
        self.run_effect(&Effect::ForEach { selector: Selector::ExactObjects(picked), body: Box::new(body.clone()) }, ctx, events)
    }
}
