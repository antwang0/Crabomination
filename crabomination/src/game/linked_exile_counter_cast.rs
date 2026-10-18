//! Dawnhand Dissident — "During your turn, you may cast creature spells from
//! among cards you own exiled with this creature by removing three counters
//! from among creatures you control in addition to paying their other
//! costs." (`StaticEffect::CastLinkedExiledCreaturesForCounters`.) The cast
//! is a paid cast from exile (`cast_from_zone_without_paying` bills the
//! card's own cost); the counters are drained after the mana is paid,
//! -1/-1 counters first, weakest creature first.

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

    /// Remove `n` counters from among creatures `p` controls.
    pub(crate) fn drain_creature_counters(&mut self, p: usize, n: u32, events: &mut Vec<GameEvent>) {
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
