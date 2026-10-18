//! CR 701.44 — explore. Each resolved permanent explores: reveal the top card
//! of its controller's library; a land goes to hand, otherwise the permanent
//! gets a +1/+1 counter (the revealed nonland card stays on top — the
//! optional graveyard choice is collapsed). An empty library still explores
//! (counter, no card). Emits `Explored` so payoff triggers can fire.
//!
//! Two "would explore" replacements apply, both counted per permanent its
//! controller controls: Topography Tracker's "instead it explores, then it
//! explores again" (`StaticEffect::ExploresTwice`) and Twists and Turns'
//! "instead you scry 1, then that creature explores"
//! (`StaticEffect::ScryBeforeExploring`). With a scry in play the explores run
//! as a `Seq` of scries and `Effect::ExploreAfterScry`, so a prompting seat's
//! scry suspends and resumes like any sequence step.

use super::GameState;
use super::effects::EffectContext;
use super::types::{GameError, GameEvent};
use crate::card::{CardId, CounterType};
use crate::effect::{Effect, PlayerRef, Selector, StaticEffect, Value};

impl GameState {
    fn explore_static_count(&self, controller: usize, want: fn(&StaticEffect) -> bool) -> usize {
        self.battlefield
            .iter()
            .filter(|c| c.controller == controller)
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter(|sa| want(&sa.effect))
            .count()
    }

    pub(crate) fn resolve_explore(
        &mut self,
        who: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let mut deferred: Vec<Effect> = Vec::new();
        for ent in self.resolve_selector(who, ctx) {
            // `as_card_id` so a reanimated explorer reached via
            // `Selector::LastMoved` (an `EntityRef::Card`) is honored; the
            // `battlefield_find` below still gates to permanents.
            let Some(cid) = ent.as_card_id() else { continue };
            let Some(controller) = self.battlefield_find(cid).map(|c| c.controller) else { continue };
            let reps = 1 + self.explore_static_count(controller, |e| matches!(e, StaticEffect::ExploresTwice));
            let scries = self.explore_static_count(controller, |e| matches!(e, StaticEffect::ScryBeforeExploring));
            if scries == 0 {
                for _ in 0..reps {
                    self.explore_once(cid, controller, events);
                }
                continue;
            }
            for _ in 0..reps {
                for _ in 0..scries {
                    deferred.push(Effect::Scry { who: PlayerRef::Seat(controller), amount: Value::ONE });
                }
                deferred.push(Effect::ExploreAfterScry { who: Selector::ExactObjects(vec![cid]) });
            }
        }
        if deferred.is_empty() {
            self.check_state_based_actions_mid_resolution(events);
            return Ok(());
        }
        self.run_effect(&Effect::Seq(deferred), ctx, events)
    }

    /// One explore by `cid` (controlled by `controller`), no replacement.
    pub(crate) fn explore_once(&mut self, cid: CardId, controller: usize, events: &mut Vec<GameEvent>) {
        let top = self.players[controller].library.first();
        let is_land = top.map(|c| c.definition.is_land());
        if let Some(name) = top.map(|c| c.definition.name) {
            events.push(GameEvent::TopCardRevealed {
                player: controller,
                card_name: name,
                is_land: is_land.unwrap_or(false),
            });
        }
        if is_land == Some(true) {
            let card = self.players[controller].library.remove(0);
            self.players[controller].hand.push(card);
        } else {
            // Nonland revealed (or empty library): +1/+1 counter.
            // CR 614.16 — counter replacement effects apply.
            let n = self.scaled_counter_count_on(cid, CounterType::PlusOnePlusOne, 1);
            if let Some(c) = self.battlefield_find_mut(cid) {
                c.add_counters(CounterType::PlusOnePlusOne, n);
                events.push(GameEvent::CounterAdded {
                    card_id: cid,
                    counter_type: CounterType::PlusOnePlusOne,
                    count: n,
                    placer: self.resolution_causer,
                });
                self.turn.permanents_gained_counter_this_turn.insert(cid);
            }
        }
        events.push(GameEvent::Explored { card_id: cid, controller, explored_land: is_land == Some(true) });
    }
}
