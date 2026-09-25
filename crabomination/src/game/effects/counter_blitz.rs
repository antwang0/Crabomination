//! The Counter Blitz (FIC, Tidus) primitives: a copy that keeps its own name
//! (Kimahri, Valiant Guardian — CR 707.9b) and stripping the counters from
//! any number of permanents (Sin, Unending Cataclysm).

use super::EffectContext;
use crate::card::{CardId, Keyword, SelectionRequirement};
use crate::decision::PickValue;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// CR 707.9b — "becomes a copy of `source`, except its name is [its own]
    /// and it has `keywords` and this ability": the copy keeps the copier's
    /// triggered abilities, its name and the listed keywords.
    pub(super) fn become_copy_keeping_name(
        &mut self,
        what: &Selector,
        source: &Selector,
        keywords: &[Keyword],
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let names: Vec<(CardId, &'static str)> = self
            .resolve_selector(what, ctx)
            .iter()
            .filter_map(|e| e.as_permanent_id())
            .filter_map(|cid| self.battlefield_find(cid).map(|c| (cid, c.definition.name)))
            .collect();
        self.run_effect(
            &Effect::BecomeCopyOf {
                what: what.clone(),
                source: source.clone(),
                extra_creature_types: vec![],
                keep_own_triggered: true,
                keep_own_activated: false,
            },
            ctx,
            events,
        )?;
        for (cid, name) in names {
            if let Some(c) = self.battlefield.find_by_id_mut(cid) {
                let def = c.definition_make_mut();
                def.name = name;
                for k in keywords {
                    if !def.keywords.contains(k) {
                        def.keywords.push(k.clone());
                    }
                }
            }
        }
        Ok(())
    }

    /// "Remove all counters from any number of [filter]" (Sin, Unending
    /// Cataclysm): the controller picks among every matching permanent with
    /// a counter, the source excluded; the total feeds
    /// `Value::CountersRemovedThisEffect`. The headless pick is every
    /// opponent's.
    pub(super) fn remove_all_counters_from_any_number(
        &mut self,
        filter: &SelectionRequirement,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let seat = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        let candidates: Vec<(CardId, String, bool)> = self
            .battlefield
            .iter()
            .filter(|c| c.id != source && (!c.counters.is_empty() || !c.keyword_counters.is_empty()))
            .filter(|c| self.evaluate_requirement_on_card(filter, c, seat))
            .map(|c| (c.id, c.definition.name.to_string(), !self.same_team(c.controller, seat)))
            .collect();
        if candidates.is_empty() {
            return Ok(());
        }
        let auto: Vec<CardId> = candidates.iter().filter(|(_, _, theirs)| *theirs).map(|(id, _, _)| *id).collect();
        let offered: Vec<(CardId, String)> = candidates.iter().map(|(id, n, _)| (*id, n.clone())).collect();
        let n = offered.len() as u32;
        let Some(chosen) = self.choose_up_to_cards(
            seat,
            "Remove all counters from any number of permanents".to_string(),
            source,
            offered,
            n,
            PickValue::Cost,
            effect,
            auto,
        ) else {
            return Ok(());
        };
        let chosen: Vec<CardId> = chosen.into_iter().filter(|id| candidates.iter().any(|(c, _, _)| c == id)).collect();
        if chosen.is_empty() {
            return Ok(());
        }
        self.run_effect(&Effect::RemoveAllCounters { what: Selector::ExactObjects(chosen) }, ctx, events)
    }
}
