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
                .any(|c| self.computed_is_creature(c) && card_has(c, kw));
            if !in_yard {
                continue;
            }
            let pick = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p && self.computed_is_creature(c))
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
        for c in self.battlefield.iter().filter(|c| c.controller == p && self.computed_is_creature(c)) {
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


    /// `Effect::MoveOneCounter` — one counter (a +1/+1 counter when it has
    /// one) from the first permanent onto the second (Nesting Grounds).
    pub(super) fn move_one_counter(
        &mut self,
        from: &Selector,
        to: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let Some(src) = self.resolve_selector(from, ctx).into_iter().find_map(|e| e.as_permanent_id()) else {
            return Ok(());
        };
        let Some(dst) = self.resolve_selector(to, ctx).into_iter().find_map(|e| e.as_permanent_id()) else {
            return Ok(());
        };
        let Some(card) = self.battlefield_find(src) else { return Ok(()) };
        // "A counter of any kind": the controller picks the kind. Offered
        // +1/+1 first, then the other plain kinds, then keyword counters, so
        // a seat that takes the first offer keeps the old default.
        let mut plain: Vec<CounterType> = card.counters.iter().filter(|(_, n)| **n > 0).map(|(k, _)| *k).collect();
        plain.sort_by_key(|k| *k != CounterType::PlusOnePlusOne);
        let kws: Vec<Keyword> =
            card.keyword_counters.iter().filter(|(_, n)| **n > 0).map(|(k, _)| k.clone()).collect();
        let total = plain.len() + kws.len();
        if total == 0 {
            return Ok(());
        }
        let pick = if total == 1 {
            0
        } else {
            let options = plain
                .iter()
                .map(|k| format!("{k:?} counter"))
                .chain(kws.iter().map(|k| format!("{k:?} counter")))
                .collect();
            let mut cursor = 0;
            let Some(i) = self.ask_seat_option(
                &mut cursor,
                ctx.controller,
                "Move which counter?".into(),
                ctx.source.unwrap_or(src),
                options,
                effect,
            ) else {
                return Ok(());
            };
            self.clear_answer_log();
            i
        };
        if let Some(&kind) = plain.get(pick) {
            let mut c = ctx.clone();
            c.targets = vec![Target::Permanent(src), Target::Permanent(dst)];
            let _ = self.run_effect(
                &Effect::MoveCounter { from: Selector::Target(0), to: Selector::Target(1), kind, amount: Value::ONE },
                &c,
                events,
            );
        } else if let Some(kw) = kws.get(pick - plain.len()).cloned() {
            if let Some(c) = self.battlefield_find_mut(src) {
                c.keyword_counters.remove_up_to(&kw, 1);
            }
            self.run_on(dst, Effect::AddKeywordCounter { what: Selector::Target(0), keyword: kw, amount: Value::ONE }, ctx, events);
        }
        Ok(())
    }
}
