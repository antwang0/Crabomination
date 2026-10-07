//! CR 122.6 — "[filter] you control enter with a [keyword] counter on them"
//! (`StaticEffect::MatchingEntersWithKeywordCounter`): the counter is on the
//! permanent as it enters, so nothing sees it arrive without one.

use crate::card::{CardId, Keyword};
use crate::effect::{Effect, Selector, StaticEffect, Value};
use crate::game::GameState;
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, Target};

impl GameState {
    /// Called beside `apply_etb_type_riders` at the spell-resolve, move and
    /// token-mint entry sites, before the entry's triggers are gathered.
    pub(crate) fn apply_etb_keyword_counters(&mut self, entering: CardId, controller: usize, events: &mut Vec<GameEvent>) {
        // CR 122.2 — a card that keeps its counters off the battlefield (Me,
        // the Immortal, Skullbriar) can enter already carrying a keyword
        // counter; the whole-board keyword gate must hear of it.
        if !self.board_instance_keywords
            && self.battlefield.find_by_id(entering).is_some_and(|c| c.cold_any(|k| !k.keyword_counters.is_empty()))
        {
            self.board_instance_keywords = true;
        }
        if !self.battlefield.has_etb_counter_static() || self.counters_locked() {
            return;
        }
        let mut riders: Vec<(CardId, Keyword)> = Vec::new();
        for src in self.battlefield.iter().filter(|s| s.controller == controller && s.id != entering) {
            for sa in &src.definition.static_abilities {
                if let StaticEffect::MatchingEntersWithKeywordCounter { filter, keyword } = sa.effect.ungated()
                    && self.active_static(&sa.effect, src).is_some()
                    && self.evaluate_requirement_static(filter, &Target::Permanent(entering), controller, Some(src.id))
                {
                    riders.push((src.id, keyword.clone()));
                }
            }
        }
        for (src, keyword) in riders {
            let ctx = EffectContext::for_ability(src, controller, None);
            let add = Effect::AddKeywordCounter { what: Selector::ExactObjects(vec![entering]), keyword, amount: Value::ONE };
            let _ = self.run_effect(&add, &ctx, events);
        }
    }
}
