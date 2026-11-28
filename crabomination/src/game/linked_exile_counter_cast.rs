//! Dawnhand Dissident — "During your turn, you may cast creature spells from
//! among cards you own exiled with this creature by removing three counters
//! from among creatures you control in addition to paying their other
//! costs." (`StaticEffect::CastLinkedExiledCreaturesForCounters`.) The cast
//! is a paid cast from exile (`cast_from_zone_without_paying` bills the
//! card's own cost); the counters are drained after the mana is paid. A
//! prompting caster names the creature each counter comes off (CR 601.2b,
//! `ResumeContext::ActionCounterPick`); the rest drain -1/-1 counters first,
//! weakest creature first.

use super::GameState;
use super::types::GameEvent;
use crate::card::{CardInstance, CardType, CounterType};
use crate::effect::StaticEffect;

impl GameState {
    /// The counters `p` must remove to cast exiled `card` through a linked
    /// source's grant, when one covers it right now.
    pub(crate) fn linked_exile_counter_cast(&self, p: usize, card: &CardInstance) -> Option<u32> {
        let src = card.exiled_with?;
        if card.owner != p || self.active_player_idx != p || !card.definition.card_types.contains(&CardType::Creature)
        {
            return None;
        }
        let s = self.battlefield.find_by_id(src).filter(|s| s.controller == p)?;
        let need = s.definition.static_abilities.iter().find_map(|sa| match sa.effect {
            StaticEffect::CastLinkedExiledCreaturesForCounters { counters } => Some(counters),
            _ => None,
        })?;
        self.exile.iter().any(|c| c.id == card.id).then_some(need)
    }

    /// Counters on creatures `p` controls (keyword counters included,
    /// CR 122.1b).
    pub(crate) fn creature_counters_controlled(&self, p: usize) -> u32 {
        self.battlefield
            .iter()
            .filter(|c| c.controller == p && self.computed_is_creature(c))
            .map(|c| c.total_counters())
            .sum()
    }

    /// Counters on `p`'s creatures still free once `named` (one counter per
    /// entry) are spoken for: the creatures a further pick could name.
    pub(crate) fn counter_drain_holders(&self, p: usize, named: &[crate::card::CardId]) -> Vec<(crate::card::CardId, String)> {
        self.battlefield
            .iter()
            .filter(|c| c.controller == p && self.computed_is_creature(c))
            .filter(|c| c.total_counters() as usize > named.iter().filter(|&&f| f == c.id).count())
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect()
    }

    /// The counter picks named so far by `ActionCounterPick` answers.
    pub(crate) fn counter_plan_named(&self) -> Vec<crate::card::CardId> {
        self.scratch.pending_cast_counter_plan.as_ref().map(|(_, f)| f.clone()).unwrap_or_default()
    }

    /// CR 601.2b / 602.2b — pose `p`'s next "remove a counter from which
    /// permanent" pick toward `n` among `holders` (those with a counter not yet
    /// spoken for), replaying `action` with the answer. Only a `manual_mana`
    /// seat with a real choice is asked: `have` counters in all, above `n`.
    pub(crate) fn pose_counter_drain_pick(
        &mut self,
        p: usize,
        n: u32,
        have: u32,
        source: crate::card::CardId,
        holders: Vec<(crate::card::CardId, String)>,
        action: super::GameAction,
    ) -> bool {
        let named = self.counter_plan_named().len() as u32;
        if !self.players[p].manual_mana || named >= n || have <= n || holders.len() < 2 {
            return false;
        }
        let name = self.find_card_anywhere(source).map(|c| c.definition.name.to_string()).unwrap_or_default();
        let legal = holders.iter().map(|h| h.0).collect();
        self.pending_decision = Some(Box::new(super::types::PendingDecision {
            decision: crate::decision::Decision::ChooseCards {
                source,
                prompt: format!("{name}: remove a counter from ({} of {n})", named + 1),
                candidates: holders,
                min: 1,
                max: 1,
                eligible: None,
                value: crate::decision::PickValue::Cost,
            },
            resume: super::types::ResumeContext::ActionCounterPick { actor: p, action: Box::new(action), legal },
        }));
        true
    }

    /// Remove one counter from `cid` (-1/-1 first, then printed kinds, then
    /// keyword counters); whether one came off.
    fn remove_one_counter(&mut self, cid: crate::card::CardId, events: &mut Vec<GameEvent>) -> bool {
        let Some(c) = self.battlefield.find_by_id_mut(cid) else { return false };
        let mut kinds: Vec<CounterType> = c.counters.iter().filter(|(_, n)| **n > 0).map(|(k, _)| *k).collect();
        kinds.sort_by_key(|k| *k != CounterType::MinusOneMinusOne);
        if let Some(&k) = kinds.first() {
            c.remove_counters(k, 1);
            events.push(GameEvent::CounterRemoved { card_id: cid, counter_type: k, count: 1 });
            return true;
        }
        let kw = c.keyword_counters.iter().map(|(k, _)| k.clone()).next();
        kw.is_some_and(|kw| c.keyword_counters.remove_up_to(&kw, 1) > 0)
    }

    /// Remove `n` counters from among creatures `p` controls: one off each
    /// creature in `named` (the caster's picks) first, the rest automatic.
    pub(crate) fn drain_creature_counters_named(
        &mut self,
        p: usize,
        n: u32,
        named: &[crate::card::CardId],
        events: &mut Vec<GameEvent>,
    ) {
        let mut left = n;
        for &id in named {
            if left == 0 {
                break;
            }
            let mine = self.battlefield.find_by_id(id).is_some_and(|c| c.controller == p && self.computed_is_creature(c));
            if mine && self.remove_one_counter(id, events) {
                left -= 1;
            }
        }
        self.drain_creature_counters(p, left, events);
    }

    /// Remove `n` counters from among creatures `p` controls.
    pub(crate) fn drain_creature_counters(&mut self, p: usize, n: u32, events: &mut Vec<GameEvent>) {
        if n == 0 {
            return;
        }
        let mut picks: Vec<(crate::card::CardId, i32)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == p && self.computed_is_creature(c))
            .map(|c| (c.id, c.power()))
            .collect();
        picks.sort_by_key(|&(id, pw)| {
            let minus = self.battlefield.find_by_id(id).map_or(0, |c| c.counter_count(CounterType::MinusOneMinusOne));
            (minus == 0, pw)
        });
        let mut left = n;
        for (cid, _) in picks {
            if left == 0 {
                break;
            }
            let Some(c) = self.battlefield.find_by_id_mut(cid) else { continue };
            let mut kinds: Vec<CounterType> = c.counters.iter().filter(|(_, n)| **n > 0).map(|(k, _)| *k).collect();
            kinds.sort_by_key(|k| *k != CounterType::MinusOneMinusOne);
            for k in kinds {
                if left == 0 {
                    break;
                }
                let take = left.min(c.counter_count(k));
                c.remove_counters(k, take);
                events.push(GameEvent::CounterRemoved { card_id: cid, counter_type: k, count: take });
                left -= take;
            }
            if left > 0 && !c.keyword_counters.is_empty() {
                let kws: Vec<_> = c.keyword_counters.iter().map(|(k, _)| k.clone()).collect();
                for kw in kws {
                    if left == 0 {
                        break;
                    }
                    left -= c.keyword_counters.remove_up_to(&kw, left);
                }
            }
        }
    }
}
