//! Keyword counters and keyword checks keyed off other cards (Symbiotic
//! Swarm): Kathril's graveyard-keyword counters, Majestic Myriarch's combat
//! keywords, Selective Adaptation's pick-per-keyword, and the counter moves
//! of Slippery Bogbonder and Nesting Grounds.

use crate::card::{CardId, CardInstance, CounterType, Keyword};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, Target};
use crate::game::{GameError, GameState};

/// Does `kw` answer to `wanted` in a printed keyword list? Parameterised
/// families (landwalk, protection) match by kind.
pub(crate) fn keyword_matches(kw: &Keyword, wanted: &Keyword) -> bool {
    std::mem::discriminant(kw) == std::mem::discriminant(wanted)
}

fn card_has(card: &CardInstance, wanted: &Keyword) -> bool {
    card.definition.keywords.iter().any(|k| keyword_matches(k, wanted))
}

impl GameState {
    fn run_on(&mut self, id: CardId, effect: Effect, ctx: &EffectContext, events: &mut Vec<GameEvent>) {
        let mut c = ctx.clone();
        c.targets = vec![Target::Permanent(id)];
        let _ = self.run_effect(&effect, &c, events);
    }

    /// `Effect::KeywordCountersFromGraveyard` — for each keyword a creature
    /// card in your graveyard has, a counter of it on a creature you control
    /// (one that lacks it first, then the greatest power); then a +1/+1
    /// counter on the source per counter placed (Kathril, Aspect Warper).
    pub(super) fn keyword_counters_from_graveyard(
        &mut self,
        keywords: &[Keyword],
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let p = ctx.controller;
        let mut placed = 0;
        for kw in keywords {
            let in_yard = self.players[p]
                .graveyard
                .iter()
                .any(|c| c.definition.is_creature() && card_has(c, kw));
            if !in_yard {
                continue;
            }
            let pick = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p && c.definition.is_creature())
                .max_by_key(|c| {
                    let has = self.computed_permanent(c.id).is_some_and(|cp| cp.keywords().iter().any(|k| k == kw));
                    (!has, c.power())
                })
                .map(|c| c.id);
            let Some(id) = pick else { continue };
            self.run_on(
                id,
                Effect::AddKeywordCounter { what: Selector::Target(0), keyword: kw.clone(), amount: Value::ONE },
                ctx,
                events,
            );
            placed += 1;
        }
        if placed > 0 && let Some(src) = ctx.source {
            self.run_on(
                src,
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(placed),
                },
                ctx,
                events,
            );
        }
        Ok(())
    }

    /// `Effect::GainKeywordsYourCreaturesHave` — `what` gains, until end of
    /// turn, each listed keyword a creature you control has (Majestic
    /// Myriarch).
    pub(super) fn gain_keywords_your_creatures_have(
        &mut self,
        what: &Selector,
        keywords: &[Keyword],
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let p = ctx.controller;
        let mut have: Vec<Keyword> = Vec::new();
        for c in self.battlefield.iter().filter(|c| c.controller == p && c.definition.is_creature()) {
            if let Some(cp) = self.computed_permanent(c.id) {
                for k in cp.keywords().iter() {
                    if keywords.iter().any(|w| keyword_matches(k, w)) && !have.contains(k) {
                        have.push(k.clone());
                    }
                }
            }
        }
        if have.is_empty() {
            return Ok(());
        }
        let _ = self.run_effect(
            &Effect::GrantKeywords { what: what.clone(), keywords: have, duration: crate::effect::Duration::EndOfTurn },
            ctx,
            events,
        );
        Ok(())
    }

    /// `Effect::RevealTopChooseByKeyword` — reveal the top `count`; choose a
    /// different card for each listed keyword it has; the best chosen
    /// permanent card goes onto the battlefield, the other chosen cards to
    /// hand, the rest to the graveyard (Selective Adaptation).
    pub(super) fn reveal_top_choose_by_keyword(
        &mut self,
        count: &Value,
        keywords: &[Keyword],
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let p = ctx.controller;
        let n = self.evaluate_value(count, ctx).max(0) as usize;
        let revealed: Vec<CardId> = self.players[p].library.iter().take(n).map(|c| c.id).collect();
        // Greedy: scarcer keywords claim their card first, then the priciest
        // card that has it.
        let mut chosen: Vec<CardId> = Vec::new();
        let mut order: Vec<(usize, &Keyword)> = keywords
            .iter()
            .map(|kw| {
                let n = revealed
                    .iter()
                    .filter(|id| self.players[p].library.iter().any(|c| c.id == **id && card_has(c, kw)))
                    .count();
                (n, kw)
            })
            .collect();
        order.sort_by_key(|(n, _)| *n);
        for (_, kw) in order {
            let pick = self.players[p]
                .library
                .iter()
                .filter(|c| revealed.contains(&c.id) && !chosen.contains(&c.id) && card_has(c, kw))
                .max_by_key(|c| c.definition.cost.cmc())
                .map(|c| c.id);
            if let Some(id) = pick {
                chosen.push(id);
            }
        }
        let onto = chosen
            .iter()
            .copied()
            .filter(|id| {
                self.players[p]
                    .library
                    .iter()
                    .any(|c| c.id == *id && c.definition.is_permanent() && !c.definition.is_land())
            })
            .max_by_key(|id| self.players[p].library.iter().find(|c| c.id == *id).map(|c| c.definition.cost.cmc()));
        for id in revealed {
            let dest = if Some(id) == onto {
                ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false }
            } else if chosen.contains(&id) {
                ZoneDest::Hand(PlayerRef::You)
            } else {
                ZoneDest::Graveyard
            };
            self.move_card_to(id, &dest, ctx, events);
        }
        Ok(())
    }

    /// `Effect::MoveCountersFromAmongOnto` — every counter on the other
    /// creatures you control moves onto `onto` (Slippery Bogbonder).
    pub(super) fn move_counters_from_among_onto(
        &mut self,
        onto: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(to) = self.resolve_selector(onto, ctx).into_iter().find_map(|e| e.as_permanent_id()) else {
            return Ok(());
        };
        let p = ctx.controller;
        let donors: Vec<(CardId, Vec<(CounterType, u32)>, Vec<(Keyword, u32)>)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == p && c.id != to && c.definition.is_creature())
            .map(|c| {
                let plain = c.counters.iter().map(|(k, n)| (*k, *n)).filter(|(_, n)| *n > 0).collect::<Vec<_>>();
                let kws = c.keyword_counters.iter().map(|(k, n)| (k.clone(), *n)).filter(|(_, n)| *n > 0).collect();
                (c.id, plain, kws)
            })
            .filter(|(_, a, b): &(CardId, Vec<(CounterType, u32)>, Vec<(Keyword, u32)>)| !a.is_empty() || !b.is_empty())
            .collect();
        for (from, plain, kws) in donors {
            for (kind, n) in plain {
                let mut c = ctx.clone();
                c.targets = vec![Target::Permanent(from), Target::Permanent(to)];
                let _ = self.run_effect(
                    &Effect::MoveCounter {
                        from: Selector::Target(0),
                        to: Selector::Target(1),
                        kind,
                        amount: Value::Const(n as i32),
                    },
                    &c,
                    events,
                );
            }
            for (kw, n) in kws {
                if let Some(c) = self.battlefield_find_mut(from) {
                    c.keyword_counters.remove_up_to(&kw, n);
                }
                self.run_on(
                    to,
                    Effect::AddKeywordCounter { what: Selector::Target(0), keyword: kw, amount: Value::Const(n as i32) },
                    ctx,
                    events,
                );
            }
        }
        Ok(())
    }

    /// `Effect::MoveOneCounter` — one counter (a +1/+1 counter when it has
    /// one) from the first permanent onto the second (Nesting Grounds).
    pub(super) fn move_one_counter(
        &mut self,
        from: &Selector,
        to: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(src) = self.resolve_selector(from, ctx).into_iter().find_map(|e| e.as_permanent_id()) else {
            return Ok(());
        };
        let Some(dst) = self.resolve_selector(to, ctx).into_iter().find_map(|e| e.as_permanent_id()) else {
            return Ok(());
        };
        let Some(card) = self.battlefield_find(src) else { return Ok(()) };
        let kind = if card.counter_count(CounterType::PlusOnePlusOne) > 0 {
            Some(CounterType::PlusOnePlusOne)
        } else {
            card.counters.iter().find(|(_, n)| **n > 0).map(|(k, _)| *k)
        };
        let kw_counter = card.keyword_counters.iter().find(|(_, n)| **n > 0).map(|(k, _)| k.clone());
        if let Some(kind) = kind {
            let mut c = ctx.clone();
            c.targets = vec![Target::Permanent(src), Target::Permanent(dst)];
            let _ = self.run_effect(
                &Effect::MoveCounter { from: Selector::Target(0), to: Selector::Target(1), kind, amount: Value::ONE },
                &c,
                events,
            );
        } else if let Some(kw) = kw_counter {
            if let Some(c) = self.battlefield_find_mut(src) {
                c.keyword_counters.remove_up_to(&kw, 1);
            }
            self.run_on(dst, Effect::AddKeywordCounter { what: Selector::Target(0), keyword: kw, amount: Value::ONE }, ctx, events);
        }
        Ok(())
    }
}
