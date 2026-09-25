//! Timey-Wimey's time-counter primitives (the Doctor Who Tenth Doctor deck):
//!
//! - cards that gain suspend with their own mana value in time counters
//!   (The Parting of the Ways, The Eleventh Doctor, The Wedding of River
//!   Song — CR 702.62e);
//! - removing several time counters from one suspended card through the
//!   suspend funnel, so the last one casts it (Amy Pond, CR 702.62);
//! - damage to a creature and everything sharing a creature type with it
//!   (Killer), and a board exile that spares the convokers' kin (Everything
//!   Comes to Dust).

use super::{EffectContext, EntityRef};
use crate::card::{CardId, CounterType, SelectionRequirement};
use crate::decision::PickValue;
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// `Effect::GrantSuspendManaValueCounters` — each card `what` names is
    /// exiled (if it isn't already) with time counters equal to its mana
    /// value and gains suspend.
    pub(super) fn grant_suspend_mana_value_counters(
        &mut self,
        what: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let ids: Vec<CardId> = self.resolve_selector(what, ctx).iter().filter_map(|e| e.as_card_id()).collect();
        for cid in ids {
            let Some(mv) = self.find_card_anywhere(cid).map(|c| c.definition.cost.cmc()) else { continue };
            self.run_effect(
                &Effect::GrantSuspend { what: Selector::ExactObjects(vec![cid]), time_counters: mv },
                ctx,
                events,
            )?;
        }
        Ok(())
    }

    /// `Effect::MayExileFromHandSuspended` — `who` may exile one matching card
    /// from their hand; it gets its mana value in time counters and suspend.
    /// A headless seat takes the highest mana value it holds (the longest
    /// wait buys the biggest free cast).
    pub(super) fn may_exile_from_hand_suspended(
        &mut self,
        who: &PlayerRef,
        filter: &SelectionRequirement,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(p) = self.resolve_player(who, ctx) else { return Ok(()) };
        if !self.players[p].is_alive() {
            return Ok(());
        }
        let mut candidates: Vec<(CardId, String, u32)> = self.players[p]
            .hand
            .iter()
            .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
            .map(|c| (c.id, c.definition.name.to_string(), c.definition.cost.cmc()))
            .collect();
        if candidates.is_empty() {
            return Ok(());
        }
        candidates.sort_by_key(|(id, _, mv)| (std::cmp::Reverse(*mv), *id));
        let auto = vec![candidates[0].0];
        let Some(picked) = self.choose_up_to_cards(
            p,
            format!("P{p}: exile a card from your hand with time counters equal to its mana value?"),
            ctx.source.unwrap_or(CardId(0)),
            candidates.iter().map(|(id, n, _)| (*id, n.clone())).collect(),
            1,
            PickValue::Gain,
            effect,
            auto,
        ) else {
            return Ok(());
        };
        let Some(&cid) = picked.iter().find(|id| candidates.iter().any(|(c, _, _)| c == *id)) else {
            return Ok(());
        };
        let Some(card) = self.players[p].remove_from_hand(cid) else { return Ok(()) };
        self.exile.push(card);
        events.push(GameEvent::PermanentExiled { card_id: cid });
        self.grant_suspend_mana_value_counters(&Selector::ExactObjects(vec![cid]), ctx, events)
    }

    /// `Effect::RemoveTimeCountersFromSuspended` — the controller picks one
    /// suspended card they own and takes `amount` time counters off it, one
    /// at a time through the suspend funnel (the last one casts it). A
    /// headless seat takes the biggest card the removal finishes, else the
    /// one closest to done.
    pub(super) fn remove_time_counters_from_suspended(
        &mut self,
        amount: &Value,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let p = ctx.controller;
        let n = self.evaluate_value(amount, ctx).max(0) as u32;
        if n == 0 {
            return Ok(());
        }
        let mut candidates: Vec<(CardId, String, u32, u32)> = self
            .exile
            .iter()
            .filter(|c| c.owner == p && c.has_suspend() && c.counter_count(CounterType::Time) > 0)
            .map(|c| (c.id, c.definition.name.to_string(), c.counter_count(CounterType::Time), c.definition.cost.cmc()))
            .collect();
        if candidates.is_empty() {
            return Ok(());
        }
        candidates.sort_by_key(|(id, _, left, mv)| {
            let finishes = *left <= n;
            (!finishes, if finishes { std::cmp::Reverse(*mv) } else { std::cmp::Reverse(0) }, *left, *id)
        });
        let Some(picked) = self.ask_seat_cards(
            p,
            format!("P{p}: remove {n} time counters from which suspended card?"),
            ctx.source.unwrap_or(CardId(0)),
            candidates.iter().map(|(id, name, _, _)| (*id, name.clone())).collect(),
            1,
            1,
            PickValue::Gain,
            effect,
        ) else {
            return Ok(());
        };
        let cid = picked
            .iter()
            .copied()
            .find(|id| candidates.iter().any(|(c, ..)| c == id))
            .unwrap_or(candidates[0].0);
        for _ in 0..n {
            if !self.exile.iter().any(|c| c.id == cid && c.counter_count(CounterType::Time) > 0) {
                break;
            }
            events.append(&mut self.remove_suspend_time_counter(cid));
        }
        Ok(())
    }

    /// Do two permanents share a creature type (CR 205.3, changeling
    /// included)? Reads the computed types.
    fn permanents_share_creature_type(&self, a: CardId, b: CardId) -> bool {
        let (Some(ca), Some(cb)) = (self.computed_permanent(a), self.computed_permanent(b)) else { return false };
        let wild = |cp: &crate::game::layers::ComputedPermanent| cp.keywords().contains(&crate::card::Keyword::Changeling);
        let a_types = &ca.subtypes().creature_types;
        let b_types = &cb.subtypes().creature_types;
        if a_types.is_empty() && !wild(&ca) || b_types.is_empty() && !wild(&cb) {
            return false;
        }
        wild(&ca) || wild(&cb) || a_types.iter().any(|t| b_types.contains(t))
    }

    /// `Effect::DealDamageToTargetAndTypeSharers` (Killer).
    pub(super) fn deal_damage_to_target_and_type_sharers(
        &mut self,
        what: &Selector,
        amount: &Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(first) = self.resolve_selector(what, ctx).into_iter().find_map(|e| match e {
            EntityRef::Permanent(id) => Some(id),
            _ => None,
        }) else {
            return Ok(());
        };
        let mut hit = vec![first];
        let others: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.id != first && c.definition.is_creature())
            .map(|c| c.id)
            .collect();
        hit.extend(others.into_iter().filter(|&id| {
            self.computed_permanent(id).is_some_and(|cp| cp.card_types().contains(&crate::card::CardType::Creature))
                && self.permanents_share_creature_type(first, id)
        }));
        self.run_effect(&Effect::DealDamage { to: Selector::ExactObjects(hit), amount: amount.clone() }, ctx, events)
    }

    /// `Effect::ExileAllButConvokerKin` (Everything Comes to Dust).
    pub(super) fn exile_all_but_convoker_kin(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let convokers: Vec<CardId> = self
            .resolve_selector(&Selector::CreaturesThatConvokedSource, ctx)
            .into_iter()
            .filter_map(|e| e.as_permanent_id())
            .collect();
        let doomed: Vec<CardId> = self
            .battlefield
            .iter()
            .map(|c| c.id)
            .collect::<Vec<_>>()
            .into_iter()
            .filter(|&id| {
                let Some(cp) = self.computed_permanent(id) else { return false };
                let types = cp.card_types();
                if types.contains(&crate::card::CardType::Artifact) || types.contains(&crate::card::CardType::Enchantment) {
                    return true;
                }
                types.contains(&crate::card::CardType::Creature)
                    && !convokers.iter().any(|&k| self.permanents_share_creature_type(k, id))
            })
            .collect();
        for id in doomed {
            self.move_card_to(id, &ZoneDest::Exile, ctx, events);
        }
        Ok(())
    }
}

impl GameState {
    /// `Effect::CastFromHandPayingSuspendCost` (The Face of Boe) — the
    /// controller may pick a card with printed suspend from their hand, pay
    /// its suspend cost, and cast it without paying its mana cost. A headless
    /// seat takes the costliest card whose suspend cost it can pay.
    pub(super) fn cast_from_hand_paying_suspend_cost(
        &mut self,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let suspend_cost = |c: &crate::card::CardInstance| {
            c.definition.keywords.iter().find_map(|k| match k {
                crate::card::Keyword::Suspend(_, cost) => Some(cost.clone()),
                _ => None,
            })
        };
        let mut candidates: Vec<(CardId, String, u32)> = self.players[me]
            .hand
            .iter()
            .filter(|c| suspend_cost(c).is_some())
            .map(|c| (c.id, c.definition.name.to_string(), c.definition.cost.cmc()))
            .collect();
        if candidates.is_empty() {
            return Ok(());
        }
        candidates.sort_by_key(|(id, _, mv)| (std::cmp::Reverse(*mv), *id));
        let auto = vec![candidates[0].0];
        let Some(picked) = self.choose_up_to_cards(
            me,
            format!("P{me}: cast a suspend card from your hand for its suspend cost?"),
            ctx.source.unwrap_or(CardId(0)),
            candidates.iter().map(|(id, n, _)| (*id, n.clone())).collect(),
            1,
            PickValue::Gain,
            effect,
            auto,
        ) else {
            return Ok(());
        };
        let Some(&pick) = picked.iter().find(|id| candidates.iter().any(|(c, ..)| c == *id)) else {
            return Ok(());
        };
        let Some(cost) = self.players[me].hand.iter().find(|c| c.id == pick).and_then(suspend_cost) else {
            return Ok(());
        };
        if !self.pay_mana_cost_with_picks(me, &cost, None, events) {
            return Ok(());
        }
        self.run_effect(
            &Effect::CastWithoutPayingImmediate {
                what: Selector::Target(0),
                source_zone: crate::card::Zone::Hand,
                exile_after: false,
                copy: false,
                reduce_generic: 0,
                pay_own_cost: false,
            },
            &EffectContext { targets: vec![crate::game::types::Target::Permanent(pick)], ..ctx.clone() },
            events,
        )
    }

    /// `Effect::AcquireAbilitiesOfExiledWithSource` — the activated and
    /// triggered abilities of each card exiled with the source join its own
    /// (Idris). Stamped as the card is exiled, so a later layer effect that
    /// removes abilities removes these too.
    pub(super) fn acquire_abilities_of_exiled_with_source(&mut self, ctx: &EffectContext) {
        let Some(src) = ctx.source else { return };
        let (mut activated, mut triggered) = (Vec::new(), Vec::new());
        for c in self
            .exile
            .iter()
            .filter(|c| c.exiled_with == Some(src) || c.exiled_by.as_ref().is_some_and(|l| l.source == src))
        {
            activated.extend(c.definition.activated_abilities.iter().cloned());
            triggered.extend(c.definition.triggered_abilities.iter().cloned());
        }
        if activated.is_empty() && triggered.is_empty() {
            return;
        }
        if let Some(me) = self.battlefield_find_mut(src) {
            let def = me.definition_make_mut();
            def.activated_abilities.extend(activated);
            def.triggered_abilities.extend(triggered);
        }
    }

    /// `Effect::ExileOtherCreaturesKeepingUpTo` — keep up to `max` of the
    /// controller's creatures matching `keep` (greatest power first) and
    /// exile every other creature.
    pub(super) fn exile_other_creatures_keeping(
        &mut self,
        keep: &SelectionRequirement,
        max: u32,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let mut kept: Vec<(CardId, i32)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == me && c.definition.is_creature())
            .filter(|c| self.evaluate_requirement_on_card(keep, c, me))
            .map(|c| (c.id, self.computed_permanent(c.id).map_or(0, |cp| cp.power)))
            .collect();
        kept.sort_by_key(|&(id, power)| (std::cmp::Reverse(power), id));
        kept.truncate(max as usize);
        let others: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.definition.is_creature() && !kept.iter().any(|(k, _)| *k == c.id))
            .map(|c| c.id)
            .collect();
        self.run_effect(&Effect::Exile { what: Selector::ExactObjects(others) }, ctx, events)
    }
}
