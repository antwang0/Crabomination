//! Promise of Loyalty's vow counters: "each of those creatures can't attack
//! you or planeswalkers you control for as long as it has a vow counter on
//! it". The restriction rides the creature as a `CantAttackPlayer` keyword;
//! this ends it once the last vow counter comes off (CR 611.2b — an effect
//! bounded by a condition stops when the condition does).

use super::GameState;
use super::types::GameEvent;
use crate::card::{CounterType, Keyword};

impl GameState {
    /// Drop the vow restriction from every creature in `events` that lost
    /// its last vow counter.
    pub(crate) fn release_spent_vows(&mut self, events: &[GameEvent]) {
        for ev in events {
            let GameEvent::CounterRemoved { card_id, counter_type: CounterType::Vow, .. } = ev else { continue };
            let Some(c) = self.battlefield_find_mut(*card_id) else { continue };
            if c.counter_count(CounterType::Vow) > 0
                || !c.definition.keywords.iter().any(|k| matches!(k, Keyword::CantAttackPlayer(_)))
            {
                continue;
            }
            c.definition_make_mut().keywords.retain(|k| !matches!(k, Keyword::CantAttackPlayer(_)));
        }
    }
}
