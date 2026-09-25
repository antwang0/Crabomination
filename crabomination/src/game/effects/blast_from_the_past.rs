//! Blast from the Past (WHO, The Fourth Doctor) primitives: a guess against
//! a live threshold with an else branch (The Seventh Doctor), a non-legendary
//! spell copy (The Sixth Doctor), and one added counter of a chosen kind on
//! each matching permanent (The Caves of Androzani).

use super::{EffectContext, EntityRef};
use crate::card::{CardId, CounterType, SelectionRequirement};
use crate::effect::{PlayerRef, Selector};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// "Choose a card in your hand. `who` guesses whether its mana value is
    /// greater than `threshold`. If they guessed wrong, you may cast it
    /// without paying its mana cost." True when a spell was cast this way.
    pub(super) fn guess_mana_value_then_cast(
        &mut self,
        who: &PlayerRef,
        threshold: u32,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        use crate::decision::{Decision, DecisionAnswer, OptionalKind};
        let p = ctx.controller;
        if self.resolve_players(who, ctx).is_empty() {
            return false;
        }
        // Auto-pick the hand card whose mana value the guesser is least
        // likely to call (the extreme farthest from the line).
        let Some((chosen, mv)) =
            self.players[p].hand.iter().map(|c| (c.id, c.definition.cost.cmc())).max_by_key(|(_, mv)| mv.abs_diff(threshold))
        else {
            return false;
        };
        let decision = Decision::OptionalTrigger {
            source: ctx.source.unwrap_or(CardId(0)),
            description: format!("Is the chosen card's mana value greater than {threshold}?"),
            kind: OptionalKind::Neutral,
        };
        let guess = matches!(self.decider.decide(&decision), DecisionAnswer::Bool(true));
        if guess == (mv > threshold) {
            return false;
        }
        let auto_target = self
            .players[p]
            .hand
            .iter()
            .find(|c| c.id == chosen)
            .map(|c| c.definition.effect.clone())
            .and_then(|e| self.auto_target_for_effect_avoiding(&e, p, Some(chosen)));
        match self.cast_card_for_free(p, chosen, crate::card::Zone::Hand, auto_target, vec![], None, None, false) {
            Ok(evs) => {
                events.extend(evs);
                true
            }
            Err(_) => false,
        }
    }

    /// CR 707.9b — copy the spell `what` names once, "except the copy isn't
    /// legendary". The copy is pushed by the ordinary copier, then has
    /// Legendary stripped from its own definition.
    pub(super) fn copy_spell_non_legendary(&mut self, what: &Selector, ctx: &EffectContext, events: &mut Vec<GameEvent>) {
        use crate::game::types::StackItem;
        let ids: Vec<CardId> = match what {
            Selector::TriggerSource => ctx
                .trigger_source
                .into_iter()
                .filter_map(|e| match e {
                    EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                    _ => None,
                })
                .collect(),
            _ => self
                .resolve_selector(what, ctx)
                .into_iter()
                .filter_map(|e| match e {
                    EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                    _ => None,
                })
                .collect(),
        };
        for cid in ids {
            let before = self.stack.len();
            self.copy_stack_spell_controlled(cid, 1, false, Some(ctx.controller), None, events);
            for item in self.stack.iter_mut().skip(before) {
                if let StackItem::Spell { card, .. } = item
                    && card.id != cid
                    && card.definition.supertypes.contains(&crate::card::Supertype::Legendary)
                {
                    let mut def = (*card.definition.arc()).clone();
                    def.supertypes.retain(|s| *s != crate::card::Supertype::Legendary);
                    card.set_definition(std::sync::Arc::new(def));
                }
            }
        }
    }

    /// The Caves of Androzani — on each permanent matching `filter`, one
    /// counter of a kind already on it, picked the way proliferate picks.
    pub(super) fn add_one_of_a_chosen_counter_to_each(
        &mut self,
        filter: &SelectionRequirement,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        if self.counters_locked() {
            return Ok(());
        }
        let me = ctx.controller;
        let picks: Vec<(CardId, CounterType)> = self
            .battlefield
            .iter()
            .filter(|c| self.evaluate_requirement_static_on(filter, c, me, ctx.source))
            .filter_map(|c| {
                let friendly = c.controller == me;
                let mut kinds: Vec<CounterType> = c
                    .counters
                    .iter()
                    .filter(|(k, n)| **n > 0 && super::proliferate_wants(**k, friendly))
                    .map(|(k, _)| *k)
                    .collect();
                kinds.sort_by_key(|k| format!("{k:?}"));
                kinds.first().map(|k| (c.id, *k))
            })
            .collect();
        for (cid, k) in picks {
            let n = self.scaled_counter_count_on(cid, k, 1);
            if n == 0 {
                continue;
            }
            let mut before = 0;
            if let Some(c) = self.battlefield_find_mut(cid) {
                before = c.counter_count(k);
                c.add_counters(k, n);
                events.push(GameEvent::CounterAdded { card_id: cid, counter_type: k, count: n, placer: Some(me) });
            }
            if k == CounterType::Lore {
                self.saga_chapters_crossed(cid, before, before + n);
            }
        }
        Ok(())
    }
}
