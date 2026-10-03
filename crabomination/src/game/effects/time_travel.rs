//! Time travel and the suspend bookkeeping it reads (CR 701.56, CR 702.62).
//! The Timey-Wimey precon's primitives.

use super::{EffectContext, EntityRef};
use crate::card::{CardDefinition, CardId, CounterType};
use crate::effect::{Effect, EventKind, EventScope, PlayerRef, Selector, Value};
use crate::game::types::GameEvent;
use crate::game::{GameError, GameState, StackItem};

/// A permanent whose "whenever a time counter is removed from this" is the
/// payoff (Regenerations Restored): time travel removes rather than adds.
fn time_removal_rewarded(def: &CardDefinition) -> bool {
    def.triggered_abilities.iter().any(|t| {
        t.event.scope == EventScope::SelfSource && t.event.kind == EventKind::CounterRemoved(CounterType::Time)
    })
}

impl GameState {
    /// `Effect::TimeTravel` — CR 701.56a: for each suspended card `who` owns
    /// and each permanent they control with a time counter, they may add or
    /// remove one. The bot removes from suspended cards (cast sooner) and adds
    /// to permanents (vanishing lives longer, a counter-scaled one grows).
    pub(super) fn time_travel(
        &mut self,
        who: &PlayerRef,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(p) = self
            .resolve_selector(&Selector::Player(who.clone()), ctx)
            .into_iter()
            .find_map(|e| if let EntityRef::Player(p) = e { Some(p) } else { None })
        else {
            return Ok(());
        };
        let suspended: Vec<CardId> =
            self.exile.iter().filter(|c| c.owner == p && c.is_suspended()).map(|c| c.id).collect();
        for id in suspended {
            let mut evs = self.remove_suspend_time_counter(id);
            events.append(&mut evs);
        }
        let perms: Vec<(CardId, bool)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == p && c.counter_count(CounterType::Time) > 0)
            .map(|c| (c.id, time_removal_rewarded(&c.definition)))
            .collect();
        for (id, remove) in perms {
            let what = Selector::ExactObjects(vec![id]);
            let effect = if remove {
                Effect::RemoveCounter { what, kind: CounterType::Time, amount: Value::ONE }
            } else {
                Effect::AddCounter { what, kind: CounterType::Time, amount: Value::ONE }
            };
            self.run_effect(&effect, ctx, events)?;
        }
        Ok(())
    }

    /// `Effect::GrantSuspend` — CR 702.62: exile each selected object from
    /// wherever it is (a spell straight off the stack, Taigam) with `n` time
    /// counters; if it doesn't have suspend, it gains suspend. An object
    /// already in exile is stamped in place (Kang Prime's "exile until
    /// nonland").
    pub(super) fn grant_suspend(
        &mut self,
        what: &Selector,
        n: u32,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        let ids: Vec<CardId> = self.resolve_selector(what, ctx).iter().filter_map(|e| e.as_card_id()).collect();
        for cid in ids {
            if let Some(pos) =
                self.stack.iter().position(|si| matches!(si, StackItem::Spell { card, .. } if card.id == cid))
            {
                if let StackItem::Spell { card, .. } = self.stack.remove(pos) {
                    self.exile.push(*card);
                    events.push(GameEvent::PermanentExiled { card_id: cid });
                }
            } else if !self.exile.iter().any(|c| c.id == cid) {
                self.run_effect(&Effect::Exile { what: Selector::ExactObjects(vec![cid]) }, ctx, events).ok();
            }
            let Some(card) = self.exile.iter_mut().find(|c| c.id == cid) else { continue };
            // A token or a copy that left the stack ceases to exist (CR 111.8).
            if card.is_token {
                continue;
            }
            card.granted_suspend = true;
            card.add_counters(CounterType::Time, n);
            if n > 0 {
                events.push(GameEvent::CounterAdded {
                    card_id: cid,
                    counter_type: CounterType::Time,
                    count: n,
                    placer: self.resolution_causer,
                });
            }
        }
    }

    /// `Effect::Clockspin` — one counter on each selected permanent or
    /// suspended card: removed or doubled up. A deciding seat picks the
    /// counter (keyword counters included) and the direction; headless, the
    /// controller helps what they own (a suspended card of theirs loses a time
    /// counter, a permanent of theirs sheds a harmful counter or gains a
    /// helpful one) and hurts an opponent's (the reverse).
    pub(super) fn clockspin(
        &mut self,
        what: &Selector,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let asks = self.seat_prompts(me) || !matches!(self.decider.kind(), crate::decision::DeciderKind::Auto);
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        // Every pick first (the re-run after a suspend replays them), then
        // the counters move.
        let mut plan: Vec<Effect> = Vec::new();
        for ent in self.resolve_selector(what, ctx) {
            let Some(id) = ent.as_card_id() else { continue };
            if let Some(card) = self.exile.iter().find(|c| c.id == id && c.is_suspended()) {
                let remove = card.owner == me;
                let pick = if asks {
                    let options = vec!["Remove a time counter".to_string(), "Add a time counter".to_string()];
                    let Some(i) = self.ask_seat_option(&mut cursor, me, "Clockspinning".into(), source, options, effect)
                    else {
                        return Ok(());
                    };
                    i == 0
                } else {
                    remove
                };
                plan.push(if pick {
                    Effect::RemoveCounter { what: Selector::ExactObjects(vec![id]), kind: CounterType::Time, amount: Value::ONE }
                } else {
                    Effect::AddCounter { what: Selector::ExactObjects(vec![id]), kind: CounterType::Time, amount: Value::ONE }
                });
                continue;
            }
            let Some(c) = self.battlefield_find(id) else { continue };
            let mine = c.controller == me;
            let harmful = |k: CounterType| {
                matches!(k, CounterType::MinusOneMinusOne | CounterType::MinusZeroMinusOne | CounterType::Stun)
            };
            let removal_rewarded = time_removal_rewarded(&c.definition);
            // (label, effect, remove?, good for the permanent's controller)
            let target = || Selector::ExactObjects(vec![id]);
            let mut options: Vec<(String, Effect, bool, bool)> = Vec::new();
            for (&k, &n) in c.counters.iter() {
                if n == 0 {
                    continue;
                }
                let good = !(harmful(k) || k == CounterType::Time && removal_rewarded);
                options.push((format!("Remove a {k:?} counter"), Effect::RemoveCounter { what: target(), kind: k, amount: Value::ONE }, true, good));
                options.push((format!("Add a {k:?} counter"), Effect::AddCounter { what: target(), kind: k, amount: Value::ONE }, false, good));
            }
            for (kw, n) in &c.keyword_counters {
                if *n == 0 {
                    continue;
                }
                let kw = kw.clone();
                options.push((
                    format!("Remove a {kw:?} counter"),
                    Effect::RemoveKeywordCounter { what: target(), keyword: kw.clone(), amount: Value::ONE },
                    true,
                    true,
                ));
                options.push((
                    format!("Add a {kw:?} counter"),
                    Effect::AddKeywordCounter { what: target(), keyword: kw, amount: Value::ONE },
                    false,
                    true,
                ));
            }
            if options.is_empty() {
                continue;
            }
            // Headless: the first counter whose right move for this side is
            // an addition, else the first removal.
            let right = |o: &(String, Effect, bool, bool)| o.2 == (o.3 != mine);
            let auto = options
                .iter()
                .position(|o| right(o) && !o.2)
                .or_else(|| options.iter().position(right))
                .unwrap_or(0);
            let pick = if asks {
                let labels = options.iter().map(|o| o.0.clone()).collect();
                let Some(i) = self.ask_seat_option(&mut cursor, me, "Clockspinning".into(), source, labels, effect) else {
                    return Ok(());
                };
                i.min(options.len() - 1)
            } else {
                auto
            };
            plan.push(options.swap_remove(pick).1);
        }
        self.clear_answer_log();
        for e in plan {
            if let Effect::RemoveCounter { what: Selector::ExactObjects(ids), kind: CounterType::Time, .. } = &e
                && let Some(&id) = ids.first()
                && self.exile.iter().any(|c| c.id == id && c.is_suspended())
            {
                let mut evs = self.remove_suspend_time_counter(id);
                events.append(&mut evs);
                continue;
            }
            if let Effect::AddCounter { what: Selector::ExactObjects(ids), kind: CounterType::Time, .. } = &e
                && let Some(&id) = ids.first()
                && let Some(card) = self.exile.iter_mut().find(|c| c.id == id && c.is_suspended())
            {
                card.add_counters(CounterType::Time, 1);
                events.push(GameEvent::CounterAdded {
                    card_id: id,
                    counter_type: CounterType::Time,
                    count: 1,
                    placer: self.resolution_causer,
                });
                continue;
            }
            self.run_effect(&e, ctx, events)?;
        }
        Ok(())
    }
}
