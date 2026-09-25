//! Kinds of counters (CR 122.1): the Bedecked Brokers effects that read or
//! copy each *kind* of counter — a keyword counter is a kind of its own
//! (CR 122.1b), so every walk here reads `keyword_counters` beside `counters`.

use super::{EffectContext, EntityRef};
use crate::card::{CardId, CounterType, Keyword, SelectionRequirement};
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::game::GameState;
use crate::game::types::{GameEvent, Target};

/// One kind of counter: a plain [`CounterType`] or a keyword counter.
#[derive(Clone, PartialEq)]
pub(crate) enum CounterKind {
    Plain(CounterType),
    Keyword(Keyword),
}

impl CounterKind {
    /// A counter nobody wants on their own creature (a -1/-1, a stun).
    fn is_harmful(&self) -> bool {
        matches!(
            self,
            CounterKind::Plain(
                CounterType::MinusOneMinusOne | CounterType::MinusZeroMinusOne | CounterType::Stun
            )
        )
    }

    /// The order a player choosing a kind for their own creature prefers:
    /// keyword counters, then shield, then +1/+1, then the rest.
    fn preference(&self) -> u8 {
        match self {
            CounterKind::Keyword(_) => 3,
            CounterKind::Plain(CounterType::Shield) => 2,
            CounterKind::Plain(CounterType::PlusOnePlusOne) => 1,
            CounterKind::Plain(_) => 0,
        }
    }
}

impl GameState {
    /// The kinds of counters on permanent `id`, plain ones first.
    fn counter_kinds_on(&self, id: CardId) -> Vec<CounterKind> {
        let Some(c) = self.battlefield_find(id) else { return Vec::new() };
        let mut kinds: Vec<CounterKind> =
            c.counters.iter().filter(|(_, n)| **n > 0).map(|(k, _)| CounterKind::Plain(*k)).collect();
        kinds.extend(c.keyword_counters.iter().filter(|(_, n)| **n > 0).map(|(k, _)| CounterKind::Keyword(k.clone())));
        kinds
    }

    /// The distinct kinds among `p`'s permanents matching `filter`, each
    /// paired with the first permanent carrying it.
    fn counter_kinds_among(&self, p: usize, filter: &SelectionRequirement) -> Vec<(CounterKind, CardId)> {
        let mut out: Vec<(CounterKind, CardId)> = Vec::new();
        for c in self.battlefield.iter().filter(|c| c.controller == p) {
            if !self.evaluate_requirement_on_card(filter, c, p) {
                continue;
            }
            for k in self.counter_kinds_on(c.id) {
                if !out.iter().any(|(o, _)| *o == k) {
                    out.push((k, c.id));
                }
            }
        }
        out
    }

    /// `Value::CounterKindsAmong`.
    pub(crate) fn count_counter_kinds_among(&self, who: &PlayerRef, filter: &SelectionRequirement, ctx: &EffectContext) -> i32 {
        self.resolve_player(who, ctx).map_or(0, |p| self.counter_kinds_among(p, filter).len() as i32)
    }

    /// Put one counter of `kind` on `id`, through the ordinary counter
    /// effects so doublers and "whenever a counter is put" watchers see it.
    fn put_counter_kind(&mut self, id: CardId, kind: &CounterKind, n: u32, ctx: &EffectContext, events: &mut Vec<GameEvent>) {
        let mut c = ctx.clone();
        c.targets = vec![Target::Permanent(id)];
        let amount = Value::Const(n as i32);
        let effect = match kind {
            CounterKind::Plain(k) => Effect::AddCounter { what: Selector::Target(0), kind: *k, amount },
            CounterKind::Keyword(k) => Effect::AddKeywordCounter { what: Selector::Target(0), keyword: k.clone(), amount },
        };
        let _ = self.run_effect(&effect, &c, events);
    }

    fn permanents_of(&self, sel: &Selector, ctx: &EffectContext) -> Vec<CardId> {
        self.resolve_selector(sel, ctx).into_iter().filter_map(|e| e.as_permanent_id()).collect()
    }

    /// `Effect::AddCounterOfEachKindAmong`.
    pub(super) fn add_counter_of_each_kind_among(
        &mut self,
        onto: &Selector,
        who: &PlayerRef,
        filter: &SelectionRequirement,
        or_plus_one: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        let Some(p) = self.resolve_player(who, ctx) else { return };
        let recipients = self.permanents_of(onto, ctx);
        if recipients.is_empty() {
            return;
        }
        let kinds = self.counter_kinds_among(p, filter);
        for (i, (kind, _)) in kinds.into_iter().enumerate() {
            let kind = if or_plus_one && kind.preference() < 2 {
                CounterKind::Plain(CounterType::PlusOnePlusOne)
            } else {
                kind
            };
            self.put_counter_kind(recipients[i % recipients.len()], &kind, 1, ctx, events);
        }
    }

    /// `Effect::AddCounterOfEachKindOn` — a permanent gets one more of each
    /// kind it has; a player one more energy, experience and poison counter
    /// for each they have.
    pub(super) fn add_counter_of_each_kind_on(&mut self, what: &Selector, ctx: &EffectContext, events: &mut Vec<GameEvent>) {
        for e in self.resolve_selector(what, ctx) {
            match e {
                EntityRef::Permanent(id) => {
                    for kind in self.counter_kinds_on(id) {
                        self.put_counter_kind(id, &kind, 1, ctx, events);
                    }
                }
                EntityRef::Player(p) => {
                    let pl = &mut self.players[p];
                    if pl.energy > 0 {
                        pl.energy += 1;
                        events.push(GameEvent::EnergyGained { player: p, amount: 1 });
                    }
                    let pl = &mut self.players[p];
                    if pl.experience > 0 {
                        pl.experience += 1;
                    }
                    if pl.poison_counters > 0 {
                        pl.poison_counters += 1;
                    }
                }
                EntityRef::Card(_) => {}
            }
        }
    }

    /// `Effect::AddMissingCounterKindFromYours`.
    pub(super) fn add_missing_counter_kind_from_yours(&mut self, what: &Selector, ctx: &EffectContext, events: &mut Vec<GameEvent>) {
        let Some(id) = self.permanents_of(what, ctx).into_iter().next() else { return };
        let present = self.counter_kinds_on(id);
        let pick = self
            .counter_kinds_among(ctx.controller, &SelectionRequirement::Any)
            .into_iter()
            .map(|(k, _)| k)
            .filter(|k| !k.is_harmful() && !present.contains(k))
            .max_by_key(CounterKind::preference);
        if let Some(kind) = pick {
            self.put_counter_kind(id, &kind, 1, ctx, events);
        }
    }

    /// `Effect::SpreadCounterKindToOthers`.
    pub(super) fn spread_counter_kind_to_others(&mut self, shield_first: bool, ctx: &EffectContext, events: &mut Vec<GameEvent>) {
        let p = ctx.controller;
        let creature = SelectionRequirement::Creature;
        let mine: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == p && self.evaluate_requirement_on_card(&creature, c, p))
            .map(|c| c.id)
            .collect();
        if shield_first {
            let best = mine.iter().copied().max_by_key(|id| self.computed_permanent(*id).map_or(0, |cp| cp.power));
            if let Some(best) = best {
                self.put_counter_kind(best, &CounterKind::Plain(CounterType::Shield), 1, ctx, events);
            }
        }
        let pick = self
            .counter_kinds_among(p, &creature)
            .into_iter()
            .filter(|(k, _)| !k.is_harmful())
            .max_by_key(|(k, _)| k.preference());
        let Some((kind, holder)) = pick else { return };
        for id in mine.into_iter().filter(|id| *id != holder) {
            if self.battlefield_find(id).is_some() {
                self.put_counter_kind(id, &kind, 1, ctx, events);
            }
        }
    }

    /// `Effect::CopyCountersOnto`.
    pub(super) fn copy_counters_onto(&mut self, from: &Selector, to: &Selector, ctx: &EffectContext, events: &mut Vec<GameEvent>) {
        let Some(src) = self.permanents_of(from, ctx).into_iter().next() else { return };
        let Some(c) = self.battlefield_find(src) else { return };
        let mut counts: Vec<(CounterKind, u32)> =
            c.counters.iter().filter(|(_, n)| **n > 0).map(|(k, n)| (CounterKind::Plain(*k), *n)).collect();
        counts.extend(c.keyword_counters.iter().filter(|(_, n)| **n > 0).map(|(k, n)| (CounterKind::Keyword(k.clone()), *n)));
        for dst in self.permanents_of(to, ctx) {
            for (kind, n) in &counts {
                self.put_counter_kind(dst, kind, *n, ctx, events);
            }
        }
    }

    /// `Effect::ExileSameNameInvestigate`.
    pub(super) fn exile_same_name_investigate(&mut self, what: &Selector, ctx: &EffectContext, events: &mut Vec<GameEvent>) {
        let Some(target) = self.permanents_of(what, ctx).into_iter().next() else { return };
        let Some(t) = self.battlefield_find(target) else { return };
        let (owner_seat, name, face_down) = (t.controller, t.definition.name, t.face_down);
        let creature = SelectionRequirement::Creature;
        let mut victims = vec![target];
        if !face_down {
            victims.extend(
                self.battlefield
                    .iter()
                    .filter(|c| {
                        c.id != target
                            && c.controller == owner_seat
                            && !c.face_down
                            && c.definition.name == name
                            && self.evaluate_requirement_on_card(&creature, c, owner_seat)
                    })
                    .map(|c| c.id),
            );
        }
        let mut nontoken = 0;
        for id in victims {
            let is_token = self.battlefield_find(id).is_some_and(|c| c.is_token);
            let mut c = ctx.clone();
            c.targets = vec![Target::Permanent(id)];
            let _ = self.run_effect(&Effect::Exile { what: Selector::Target(0) }, &c, events);
            if !is_token && self.battlefield_find(id).is_none() {
                nontoken += 1;
            }
        }
        if nontoken > 0 {
            let _ = self.run_effect(
                &Effect::CreateToken {
                    who: PlayerRef::Seat(owner_seat),
                    count: Value::Const(nontoken),
                    definition: std::sync::Arc::new(crabomination_base::tokens::clue_token()),
                },
                ctx,
                events,
            );
        }
    }
}
