//! CR 702.63a — Vanishing N is three abilities: it enters with N time
//! counters, the upkeep removes one if there is one, and "when the last is
//! removed, sacrifice it". The last is a trigger on the removal itself, so
//! time travel or Clockspinning taking the last counter sacrifices it too, and
//! a permanent's own "whenever a time counter is removed" trigger resolves
//! first (Regenerations Restored).

use super::GameState;
use super::types::{GameEvent, TriggerCandidate};
use crate::card::{CounterType, Keyword};
use crate::effect::Effect;

impl GameState {
    /// One sacrifice trigger per vanishing permanent whose last time counter
    /// came off in `events`. Empty unless the batch removed a time counter.
    pub(crate) fn vanishing_sacrifice_candidates(&self, events: &[GameEvent]) -> Vec<TriggerCandidate> {
        let mut out: Vec<TriggerCandidate> = Vec::new();
        for ev in events {
            let GameEvent::CounterRemoved { card_id, counter_type: CounterType::Time, .. } = ev else { continue };
            let Some(c) = self.battlefield_find(*card_id) else { continue };
            if c.counter_count(CounterType::Time) > 0
                || !c.definition.keywords.iter().any(|k| matches!(k, Keyword::Vanishing(_)))
                || out.iter().any(|t| t.source == c.id)
            {
                continue;
            }
            out.push(TriggerCandidate {
                actor: None,
                source: c.id,
                effect: Effect::SacrificeSource,
                controller: c.controller,
                filter: None,
                subject: None,
                event_amount: 0,
                triggered_by_etb: false,
                triggered_by_death: false,
                triggered_by_attack: false,
                triggered_by_land_entry: false,
                triggered_by_face_up: false,
                triggered_by_draw: false,
                damaged_creature_controller: None,
                from_mana_ability: false,
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::card::CounterType;
    use crate::catalog;
    use crate::effect::{Effect, Selector, Value};
    use crate::game::types::TurnStep;
    use crate::game::*;

    fn upkeep(g: &mut GameState) {
        g.active_player_idx = 0;
        g.step = TurnStep::Upkeep;
        let ev = g.process_fading_vanishing();
        g.dispatch_triggers_for_events(&ev);
        drain_stack(g);
    }

    /// CR 702.63a — the upkeep removes a counter; the permanent is sacrificed
    /// only when the last one comes off, by a trigger (not on the spot).
    #[test]
    fn cr_702_63a_vanishing_sacrifices_on_the_last_counter() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::deep_forest_hermit());
        g.battlefield_find_mut(id).unwrap().add_counters(CounterType::Time, 2);
        upkeep(&mut g);
        assert_eq!(g.battlefield_find(id).map(|c| c.counter_count(CounterType::Time)), Some(1));
        upkeep(&mut g);
        assert!(g.battlefield_find(id).is_none(), "the last counter's removal sacrifices it");
    }

    /// CR 702.63a — "if there is a time counter on it": with none, the upkeep
    /// removes nothing and nothing is sacrificed (Out of Time with no creature
    /// phased out stays).
    #[test]
    fn cr_702_63a_vanishing_without_counters_is_not_sacrificed() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::deep_forest_hermit());
        g.battlefield_find_mut(id).unwrap().counters = Default::default();
        upkeep(&mut g);
        assert!(g.battlefield_find(id).is_some());
    }

    /// CR 702.63a — any removal of the last counter triggers the sacrifice,
    /// not only the upkeep's.
    #[test]
    fn cr_702_63a_an_effect_removing_the_last_counter_sacrifices_it() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::deep_forest_hermit());
        let c = g.battlefield_find_mut(id).unwrap();
        c.counters = Default::default();
        c.add_counters(CounterType::Time, 1);
        let ctx = EffectContext::for_ability(id, 0, None);
        let remove = Effect::RemoveCounter {
            what: Selector::ExactObjects(vec![id]),
            kind: CounterType::Time,
            amount: Value::ONE,
        };
        let ev = g.resolve_effect(&remove, &ctx).expect("remove");
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
        assert!(g.battlefield_find(id).is_none());
    }
}
