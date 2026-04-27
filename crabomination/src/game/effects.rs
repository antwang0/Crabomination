//! Resolver for the unified `Effect` tree.
//!
//! A single entry point — [`GameState::resolve_effect`] — walks the effect
//! tree against an [`EffectContext`] describing the casting/activating player,
//! chosen target(s), etc. Combinators (`Seq`, `If`, `ForEach`, `Repeat`,
//! `ChooseMode`) recurse; leaf mutations perform game-state changes and emit
//! [`GameEvent`]s.

use super::*;
use crate::card::{
    ArtifactSubtype, CardDefinition, CardId, CardInstance, CardType, CounterType,
    Keyword, SelectionRequirement, Subtypes, Supertype, TokenDefinition, Zone,
};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, LibraryPosition, ManaPayload, PlayerRef, Predicate,
    Selector, Value, ZoneDest, ZoneRef,
};
use crate::mana::{Color, ManaCost, ManaSymbol};

/// Runtime context threaded through effect resolution.
#[derive(Debug, Clone)]
pub struct EffectContext {
    pub controller: usize,
    pub source: Option<CardId>,
    /// Targets chosen at cast/activation time (typically 0 or 1 entries).
    pub targets: Vec<Target>,
    /// The entity that caused the current trigger to fire (`Selector::TriggerSource`).
    pub trigger_source: Option<EntityRef>,
    /// Modal choice index (for `Effect::ChooseMode`).
    pub mode: usize,
    pub x_value: u32,
    /// Number of distinct colors of mana spent on the spell's cost
    /// (Pest Control, Prismatic Ending). Computed at cast time and
    /// threaded through `StackItem::Spell`.
    pub converged_value: u32,
}

impl EffectContext {
    pub fn for_spell(caster: usize, target: Option<Target>, mode: usize, x_value: u32) -> Self {
        Self {
            controller: caster,
            source: None,
            targets: target.into_iter().collect(),
            trigger_source: None,
            mode,
            x_value,
            converged_value: 0,
        }
    }
    pub fn for_spell_full(
        caster: usize,
        target: Option<Target>,
        mode: usize,
        x_value: u32,
        converged_value: u32,
    ) -> Self {
        Self {
            controller: caster,
            source: None,
            targets: target.into_iter().collect(),
            trigger_source: None,
            mode,
            x_value,
            converged_value,
        }
    }
    pub fn for_trigger(
        source: CardId,
        controller: usize,
        target: Option<Target>,
        mode: usize,
    ) -> Self {
        Self {
            controller,
            source: Some(source),
            targets: target.into_iter().collect(),
            trigger_source: Some(EntityRef::Permanent(source)),
            mode,
            x_value: 0,
            converged_value: 0,
        }
    }
    pub fn for_ability(
        source: CardId,
        controller: usize,
        target: Option<Target>,
    ) -> Self {
        Self {
            controller,
            source: Some(source),
            targets: target.into_iter().collect(),
            trigger_source: Some(EntityRef::Permanent(source)),
            mode: 0,
            x_value: 0,
            converged_value: 0,
        }
    }
}

/// A resolved reference to something in the game (used internally for selector
/// resolution and `ForEach` iteration).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityRef {
    Player(usize),
    Permanent(CardId),
    /// A card in a non-battlefield zone (library/graveyard/exile/hand).
    Card(CardId),
}

impl EntityRef {
    #[allow(dead_code)]
    pub fn as_target(&self) -> Target {
        match *self {
            EntityRef::Player(p) => Target::Player(p),
            EntityRef::Permanent(c) | EntityRef::Card(c) => Target::Permanent(c),
        }
    }
}

impl GameState {
    // ── Entry points ─────────────────────────────────────────────────────────

    pub(crate) fn resolve_effect(
        &mut self,
        effect: &Effect,
        ctx: &EffectContext,
    ) -> Result<Vec<GameEvent>, GameError> {
        // Reset sacrificed-power scratch for this independent resolution.
        self.sacrificed_power = None;
        let mut events = vec![];
        self.run_effect(effect, ctx, &mut events)?;
        Ok(events)
    }

    fn run_effect(
        &mut self,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        match effect {
            Effect::Noop => Ok(()),

            Effect::Seq(steps) => {
                for (idx, step) in steps.iter().enumerate() {
                    self.run_effect(step, ctx, events)?;
                    // A child effect signalled suspension — prepend the rest of
                    // this Seq onto whatever remaining effects it already saved.
                    if let Some((_, _, remaining)) = self.suspend_signal.as_mut() {
                        let tail: Vec<Effect> = steps[idx + 1..].to_vec();
                        if !tail.is_empty() {
                            let carried = std::mem::replace(remaining, Effect::Noop);
                            let mut combined = Vec::with_capacity(tail.len() + 1);
                            combined.extend(tail);
                            if !matches!(carried, Effect::Noop) {
                                combined.push(carried);
                            }
                            *remaining = Effect::seq(combined);
                        }
                        return Ok(());
                    }
                }
                Ok(())
            }

            Effect::If { cond, then, else_ } => {
                if self.evaluate_predicate(cond, ctx) {
                    self.run_effect(then, ctx, events)
                } else {
                    self.run_effect(else_, ctx, events)
                }
            }

            Effect::ForEach { selector, body } => {
                let entities = self.resolve_selector(selector, ctx);
                for ent in entities {
                    let mut sub_ctx = ctx.clone();
                    sub_ctx.trigger_source = Some(ent);
                    self.run_effect(body, &sub_ctx, events)?;
                }
                Ok(())
            }

            Effect::Repeat { count, body } => {
                let n = self.evaluate_value(count, ctx).max(0);
                for _ in 0..n {
                    self.run_effect(body, ctx, events)?;
                }
                Ok(())
            }

            Effect::ChooseMode(modes) => {
                let idx = ctx.mode;
                if let Some(m) = modes.get(idx) {
                    self.run_effect(m, ctx, events)
                } else {
                    Err(GameError::ModeOutOfBounds(idx))
                }
            }

            Effect::DealDamage { to, amount } => {
                let amt = self.evaluate_value(amount, ctx).max(0) as u32;
                if amt == 0 { return Ok(()); }
                for ent in self.resolve_selector(to, ctx) {
                    self.deal_damage_to(ent, amt, events);
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::GainLife { who, amount } => {
                let amt = self.evaluate_value(amount, ctx).max(0) as u32;
                if amt == 0 { return Ok(()); }
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].life += amt as i32;
                        events.push(GameEvent::LifeGained { player: p, amount: amt });
                    }
                }
                Ok(())
            }

            Effect::LoseLife { who, amount } => {
                let amt = self.evaluate_value(amount, ctx).max(0) as u32;
                if amt == 0 { return Ok(()); }
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].life -= amt as i32;
                        events.push(GameEvent::LifeLost { player: p, amount: amt });
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::Drain { from, to, amount } => {
                let amt = self.evaluate_value(amount, ctx).max(0) as u32;
                if amt == 0 { return Ok(()); }
                for ent in self.resolve_selector(from, ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].life -= amt as i32;
                        events.push(GameEvent::LifeLost { player: p, amount: amt });
                    }
                }
                for ent in self.resolve_selector(to, ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].life += amt as i32;
                        events.push(GameEvent::LifeGained { player: p, amount: amt });
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::Draw { who, amount } => {
                let n = self.evaluate_value(amount, ctx).max(0) as usize;
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        for _ in 0..n {
                            match self.players[p].draw_top() {
                                Some(id) => events.push(GameEvent::CardDrawn { player: p, card_id: id }),
                                None => {
                                    // Drawing from empty library eliminates p
                                    // (SBA at the end of the call decides
                                    // game-over).
                                    self.players[p].eliminated = true;
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                Ok(())
            }

            Effect::Discard { who, amount, random } => {
                let n = self.evaluate_value(amount, ctx).max(0) as usize;
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        for _ in 0..n {
                            let idx = if *random {
                                if self.players[p].hand.is_empty() { break; }
                                // AutoDecider picks index 0 deterministically.
                                0usize
                            } else {
                                0usize
                            };
                            if idx >= self.players[p].hand.len() { break; }
                            let card = self.players[p].hand.remove(idx);
                            let cid = card.id;
                            self.players[p].graveyard.push(card);
                            events.push(GameEvent::CardDiscarded { player: p, card_id: cid });
                        }
                    }
                }
                Ok(())
            }

            Effect::Mill { who, amount } => {
                let n = self.evaluate_value(amount, ctx).max(0) as usize;
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        for _ in 0..n {
                            if self.players[p].library.is_empty() { break; }
                            let card = self.players[p].library.remove(0);
                            let cid = card.id;
                            self.players[p].graveyard.push(card);
                            events.push(GameEvent::CardMilled { player: p, card_id: cid });
                        }
                    }
                }
                Ok(())
            }

            Effect::Scry { who, amount } | Effect::Surveil { who, amount } | Effect::LookAtTop { who, amount } => {
                use crate::decision::Decision;
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let n = self.evaluate_value(amount, ctx).max(0) as usize;
                let peek: Vec<(CardId, &'static str)> = self.players[p]
                    .library
                    .iter()
                    .take(n)
                    .map(|c| (c.id, c.definition.name))
                    .collect();
                let actual = peek.len();
                if actual == 0 {
                    return Ok(());
                }

                let decision = Decision::Scry { player: p, cards: peek.clone() };
                let is_surveil = matches!(effect, Effect::Surveil { .. });
                let pending_state = if is_surveil {
                    PendingEffectState::SurveilPeeked { count: actual, player: p }
                } else {
                    PendingEffectState::ScryPeeked { count: actual, player: p }
                };

                // If the acting player wants UI input, suspend — the outer
                // resolver will convert `suspend_signal` into `pending_decision`
                // and `submit_decision` will apply the answer + run any
                // remaining Seq effects.
                if self.players[p].wants_ui {
                    self.suspend_signal = Some((decision, pending_state, Effect::Noop));
                    return Ok(());
                }

                // Otherwise resolve synchronously via the decider (bot / tests).
                let answer = self.decider.decide(&decision);
                let mut applied = self.apply_pending_effect_answer(pending_state, &answer)?;
                events.append(&mut applied);
                Ok(())
            }

            Effect::AddMana { who, pool } => {
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                match pool {
                    ManaPayload::Colors(colors) => {
                        for c in colors {
                            self.players[p].mana_pool.add(*c, 1);
                            events.push(GameEvent::ManaAdded { player: p, color: *c });
                        }
                    }
                    ManaPayload::Colorless(v) => {
                        let n = self.evaluate_value(v, ctx).max(0) as u32;
                        for _ in 0..n {
                            self.players[p].mana_pool.add_colorless(1);
                            events.push(GameEvent::ColorlessManaAdded { player: p });
                        }
                    }
                    ManaPayload::AnyOneColor(v) => {
                        // ONE color choice; add `n` mana of that color (Black
                        // Lotus, Birds of Paradise, Mox Diamond, etc.).
                        let n = self.evaluate_value(v, ctx).max(0) as u32;
                        if n == 0 { return Ok(()); }
                        let source = ctx.source.unwrap_or(CardId(0));
                        let legal = vec![
                            Color::White, Color::Blue, Color::Black, Color::Red, Color::Green,
                        ];
                        if self.players[p].wants_ui {
                            // Surface a `ChooseColor` decision to the UI.
                            // After the player answers, `apply_pending_effect_answer`
                            // adds `n` mana of the chosen color.
                            self.suspend_signal = Some((
                                crate::decision::Decision::ChooseColor {
                                    source,
                                    legal,
                                },
                                PendingEffectState::AnyOneColorPending { player: p, count: n },
                                Effect::Noop,
                            ));
                            return Ok(());
                        }
                        let answer = self.decider.decide(
                            &crate::decision::Decision::ChooseColor {
                                source,
                                legal,
                            },
                        );
                        let color = match answer {
                            crate::decision::DecisionAnswer::Color(c) => c,
                            _ => Color::White,
                        };
                        for _ in 0..n {
                            self.players[p].mana_pool.add(color, 1);
                            events.push(GameEvent::ManaAdded { player: p, color });
                        }
                    }
                    ManaPayload::AnyColors(v) => {
                        // N independent color choices (one per pip). Currently
                        // resolves synchronously via the installed decider — a
                        // UI prompt per pip would require a multi-step pending
                        // state and isn't needed by any catalog card today.
                        let n = self.evaluate_value(v, ctx).max(0) as u32;
                        let source = ctx.source.unwrap_or(CardId(0));
                        let legal = vec![
                            Color::White, Color::Blue, Color::Black, Color::Red, Color::Green,
                        ];
                        for _ in 0..n {
                            let answer = self.decider.decide(&crate::decision::Decision::ChooseColor {
                                source,
                                legal: legal.clone(),
                            });
                            let color = match answer {
                                crate::decision::DecisionAnswer::Color(c) => c,
                                _ => Color::White,
                            };
                            self.players[p].mana_pool.add(color, 1);
                            events.push(GameEvent::ManaAdded { player: p, color });
                        }
                    }
                }
                Ok(())
            }

            Effect::Destroy { what } => {
                let entities = self.resolve_selector(what, ctx);
                for ent in entities {
                    if let EntityRef::Permanent(cid) = ent {
                        let indestructible = self.battlefield_find(cid)
                            .map(|c| c.has_keyword(&Keyword::Indestructible))
                            .unwrap_or(true);
                        if !indestructible {
                            let is_creature = self.battlefield_find(cid)
                                .map(|c| c.definition.is_creature())
                                .unwrap_or(false);
                            if is_creature {
                                events.push(GameEvent::CreatureDied { card_id: cid });
                            }
                            let mut dies = self.remove_to_graveyard_with_triggers(cid);
                            events.append(&mut dies);
                        }
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::Exile { what } => {
                // Exile accepts both `EntityRef::Permanent` (battlefield)
                // and `EntityRef::Card` (any other zone). Battlefield exits
                // emit `PermanentExiled` and walk through the standard
                // remove-from-battlefield path so leaves-the-battlefield
                // hooks fire; non-battlefield zones (graveyards, hand,
                // exile→exile re-routes) just relocate via `move_card_to`.
                for ent in self.resolve_selector(what, ctx) {
                    match ent {
                        EntityRef::Permanent(cid) => {
                            self.remove_from_battlefield_to_exile(cid);
                            events.push(GameEvent::PermanentExiled { card_id: cid });
                        }
                        EntityRef::Card(cid) => {
                            self.move_card_to(cid, &ZoneDest::Exile, ctx, events);
                        }
                        _ => {}
                    }
                }
                Ok(())
            }

            Effect::Tap { what } => {
                for ent in self.resolve_selector(what, ctx) {
                    if let EntityRef::Permanent(cid) = ent
                        && let Some(c) = self.battlefield_find_mut(cid)
                        && !c.tapped {
                            c.tapped = true;
                            events.push(GameEvent::PermanentTapped { card_id: cid });
                        }
                }
                Ok(())
            }

            Effect::Untap { what } => {
                for ent in self.resolve_selector(what, ctx) {
                    if let EntityRef::Permanent(cid) = ent
                        && let Some(c) = self.battlefield_find_mut(cid)
                        && c.tapped {
                            c.tapped = false;
                            events.push(GameEvent::PermanentUntapped { card_id: cid });
                        }
                }
                Ok(())
            }

            Effect::PumpPT { what, power, toughness, duration: _ } => {
                let p = self.evaluate_value(power, ctx);
                let t = self.evaluate_value(toughness, ctx);
                for ent in self.resolve_selector(what, ctx) {
                    if let EntityRef::Permanent(cid) = ent
                        && let Some(c) = self.battlefield_find_mut(cid) {
                            c.power_bonus += p;
                            c.toughness_bonus += t;
                            events.push(GameEvent::PumpApplied { card_id: cid, power: p, toughness: t });
                        }
                }
                Ok(())
            }

            Effect::GrantKeyword { what, keyword, duration: _ } => {
                for ent in self.resolve_selector(what, ctx) {
                    if let EntityRef::Permanent(cid) = ent
                        && let Some(c) = self.battlefield_find_mut(cid)
                        && !c.definition.keywords.contains(keyword) {
                            c.definition.keywords.push(keyword.clone());
                        }
                }
                Ok(())
            }

            Effect::AddCounter { what, kind, amount } => {
                let n = self.evaluate_value(amount, ctx).max(0) as u32;
                if n == 0 { return Ok(()); }
                for ent in self.resolve_selector(what, ctx) {
                    match ent {
                        EntityRef::Permanent(cid) => {
                            if let Some(c) = self.battlefield_find_mut(cid) {
                                c.add_counters(*kind, n);
                                events.push(GameEvent::CounterAdded { card_id: cid, counter_type: *kind, count: n });
                            }
                        }
                        EntityRef::Player(p) if *kind == CounterType::Poison => {
                            self.players[p].poison_counters += n;
                            events.push(GameEvent::PoisonAdded { player: p, amount: n });
                        }
                        _ => {}
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::RemoveCounter { what, kind, amount } => {
                let n = self.evaluate_value(amount, ctx).max(0) as u32;
                if n == 0 { return Ok(()); }
                for ent in self.resolve_selector(what, ctx) {
                    if let EntityRef::Permanent(cid) = ent
                        && let Some(c) = self.battlefield_find_mut(cid) {
                            let removed = c.remove_counters(*kind, n);
                            if removed > 0 {
                                events.push(GameEvent::CounterRemoved { card_id: cid, counter_type: *kind, count: removed });
                            }
                        }
                }
                Ok(())
            }

            Effect::Proliferate => {
                // Add one counter of each existing type on any permanent/player.
                // Simplified: only handles permanents, each counter type present.
                let updates: Vec<(CardId, Vec<CounterType>)> = self
                    .battlefield
                    .iter()
                    .map(|c| {
                        let kinds: Vec<CounterType> = c.counters.iter()
                            .filter(|(_, n)| **n > 0)
                            .map(|(k, _)| *k)
                            .collect();
                        (c.id, kinds)
                    })
                    .filter(|(_, kinds)| !kinds.is_empty())
                    .collect();
                for (cid, kinds) in updates {
                    if let Some(c) = self.battlefield_find_mut(cid) {
                        for k in kinds {
                            c.add_counters(k, 1);
                            events.push(GameEvent::CounterAdded { card_id: cid, counter_type: k, count: 1 });
                        }
                    }
                }
                for i in 0..self.players.len() {
                    if self.players[i].poison_counters > 0 {
                        self.players[i].poison_counters += 1;
                        events.push(GameEvent::PoisonAdded { player: i, amount: 1 });
                    }
                }
                Ok(())
            }

            Effect::GainControl { what, duration: _ } => {
                let new_ctrl = ctx.controller;
                for ent in self.resolve_selector(what, ctx) {
                    if let EntityRef::Permanent(cid) = ent
                        && let Some(c) = self.battlefield_find_mut(cid) {
                            c.controller = new_ctrl;
                        }
                }
                Ok(())
            }

            Effect::CreateToken { who, count, definition } => {
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let n = self.evaluate_value(count, ctx).max(0) as u32;
                for _ in 0..n {
                    let id = self.next_id();
                    let def = token_to_card_definition(definition);
                    let mut inst = CardInstance::new_token(id, def, p);
                    inst.controller = p;
                    self.battlefield.push(inst);
                    events.push(GameEvent::TokenCreated { card_id: id });
                    events.push(GameEvent::PermanentEntered { card_id: id });
                }
                Ok(())
            }

            Effect::CounterSpell { what } => {
                // With only a single stack target, we pop the top of the
                // stack if it's a spell (matching by target id when
                // available). Spells flagged `uncounterable` (Cavern of
                // Souls) are skipped — the counter has no effect on them.
                let targets = self.resolve_selector(what, ctx);
                let mut to_remove: Vec<usize> = Vec::new();
                for t in &targets {
                    if let EntityRef::Permanent(cid) = t
                        && let Some(pos) = self.stack.iter().position(|si| matches!(
                            si,
                            StackItem::Spell { card, uncounterable: false, .. }
                                if card.id == *cid
                        ))
                    {
                        to_remove.push(pos);
                    }
                }
                to_remove.sort_unstable_by(|a, b| b.cmp(a));
                for pos in to_remove {
                    if let StackItem::Spell { card, caster, .. } = self.stack.remove(pos) {
                        self.players[caster].send_to_graveyard(*card);
                    }
                }
                Ok(())
            }

            Effect::CounterUnlessPaid { what, mana_cost } => {
                // Counter target spell unless its controller pays `mana_cost`.
                // Auto-pays on behalf of the spell's controller via the
                // existing `auto_tap_for_cost` + `mana_pool.pay` path: if
                // affordable, the spell stays; otherwise it's countered.
                let targets = self.resolve_selector(what, ctx);
                let target_id = targets.into_iter().find_map(|t| match t {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => Some(cid),
                    _ => None,
                });
                let Some(cid) = target_id else { return Ok(()); };
                let pos = self.stack.iter().position(|si| matches!(
                    si,
                    StackItem::Spell { card, uncounterable: false, .. } if card.id == cid
                ));
                let Some(pos) = pos else { return Ok(()); };
                let StackItem::Spell { caster: spell_caster, .. } = &self.stack[pos]
                    else { unreachable!("filtered above") };
                let spell_caster = *spell_caster;

                // Try to auto-pay on behalf of the spell's controller. We
                // override priority temporarily so `auto_tap_for_cost`
                // taps that player's lands.
                let saved_priority = self.priority.player_with_priority;
                self.priority.player_with_priority = spell_caster;
                let pool_before = self.players[spell_caster].mana_pool.clone();
                let tapped_before: Vec<(CardId, bool)> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.owner == spell_caster)
                    .map(|c| (c.id, c.tapped))
                    .collect();
                self.auto_tap_for_cost(spell_caster, mana_cost);
                let paid = self.players[spell_caster].mana_pool.pay(mana_cost).is_ok();
                if !paid {
                    // Roll back any tap side-effects.
                    self.players[spell_caster].mana_pool = pool_before;
                    for (id, was_tapped) in tapped_before {
                        if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == id) {
                            c.tapped = was_tapped;
                        }
                    }
                }
                self.priority.player_with_priority = saved_priority;

                if !paid {
                    if let StackItem::Spell { card, caster, .. } = self.stack.remove(pos) {
                        self.players[caster].send_to_graveyard(*card);
                    }
                }
                Ok(())
            }

            Effect::CounterAbility { what } => {
                // Counter target activated/triggered ability. The selector
                // resolves to a permanent (the ability's source); we remove
                // the topmost `StackItem::Trigger` whose `source` matches.
                // Used by Consign to Memory.
                let targets = self.resolve_selector(what, ctx);
                let mut to_remove: Vec<usize> = Vec::new();
                for t in &targets {
                    if let EntityRef::Permanent(cid) = t {
                        // Walk top-down so we counter the most recent
                        // matching trigger (the one the player most likely
                        // intends to cancel).
                        if let Some(pos) = self
                            .stack
                            .iter()
                            .enumerate()
                            .rev()
                            .find_map(|(i, si)| match si {
                                StackItem::Trigger { source, .. } if source == cid => Some(i),
                                _ => None,
                            })
                        {
                            to_remove.push(pos);
                        }
                    }
                }
                to_remove.sort_unstable_by(|a, b| b.cmp(a));
                for pos in to_remove {
                    self.stack.remove(pos);
                }
                Ok(())
            }

            Effect::Sacrifice { who, count, filter } => {
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                for ent in self.resolve_selector(who, ctx) {
                    let EntityRef::Player(p) = ent else { continue; };
                    let ids: Vec<CardId> = self.battlefield.iter()
                        .filter(|c| c.controller == p)
                        .filter(|c| {
                            let t = Target::Permanent(c.id);
                            self.evaluate_requirement_static(filter, &t, p)
                        })
                        .take(n)
                        .map(|c| c.id)
                        .collect();
                    for id in ids {
                        let is_creature = self.battlefield_find(id).map(|c| c.definition.is_creature()).unwrap_or(false);
                        if is_creature { events.push(GameEvent::CreatureDied { card_id: id }); }
                        let mut die_evs = self.remove_to_graveyard_with_triggers(id);
                        events.append(&mut die_evs);
                    }
                }
                Ok(())
            }

            Effect::AddPoison { who, amount } => {
                let n = self.evaluate_value(amount, ctx).max(0) as u32;
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].poison_counters += n;
                        events.push(GameEvent::PoisonAdded { player: p, amount: n });
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::Move { what, to } => {
                for ent in self.resolve_selector(what, ctx) {
                    let cid = match ent {
                        EntityRef::Permanent(c) | EntityRef::Card(c) => c,
                        _ => continue,
                    };
                    self.move_card_to(cid, to, ctx, events);
                }
                Ok(())
            }

            Effect::Search { who, filter, to } => {
                use crate::decision::Decision;
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };

                // Collect candidates from the library using definition-level evaluation
                // (cards are not on the battlefield so battlefield_find would fail).
                let candidates: Vec<(crate::card::CardId, &'static str)> = self.players[p]
                    .library
                    .iter()
                    .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                    .map(|c| (c.id, c.definition.name))
                    .collect();

                let decision = Decision::SearchLibrary { player: p, candidates };
                let pending = PendingEffectState::SearchPending { player: p, to: to.clone() };

                if self.players[p].wants_ui {
                    self.suspend_signal = Some((decision, pending, Effect::Noop));
                    return Ok(());
                }

                let answer = self.decider.decide(&decision);
                let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                events.append(&mut applied);
                Ok(())
            }

            Effect::ShuffleGraveyardIntoLibrary { who } => {
                if let Some(p) = self.resolve_player(who, ctx) {
                    let cards = std::mem::take(&mut self.players[p].graveyard);
                    self.players[p].library.extend(cards);
                }
                Ok(())
            }

            Effect::Attach { what, to } => {
                let anchor = self.resolve_selector(to, ctx).into_iter().find_map(|e| {
                    if let EntityRef::Permanent(cid) = e { Some(cid) } else { None }
                });
                for ent in self.resolve_selector(what, ctx) {
                    if let EntityRef::Permanent(cid) = ent
                        && let Some(c) = self.battlefield_find_mut(cid) {
                            c.attached_to = anchor;
                            events.push(GameEvent::AttachmentMoved { attachment: cid, attached_to: anchor });
                        }
                }
                Ok(())
            }

            Effect::PutOnLibraryFromHand { who, count } => {
                use crate::decision::Decision;
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let hand_snapshot: Vec<(crate::card::CardId, &'static str)> =
                    self.players[p].hand.iter().map(|c| (c.id, c.definition.name)).collect();
                let actual = n.min(hand_snapshot.len());
                if actual == 0 { return Ok(()); }

                let decision = Decision::PutOnLibrary { player: p, count: actual, hand: hand_snapshot.clone() };
                let pending = PendingEffectState::PutOnLibraryPending { player: p, count: actual };

                if self.players[p].wants_ui {
                    self.suspend_signal = Some((decision, pending, Effect::Noop));
                    return Ok(());
                }
                // Bot: auto-pick first N cards.
                let chosen: Vec<crate::card::CardId> =
                    hand_snapshot.iter().take(actual).map(|(id, _)| *id).collect();
                self.execute_put_on_library(p, &chosen, events);
                Ok(())
            }

            Effect::RevealTopAndDrawIf { who, reveal_filter } => {
                // Each resolved player reveals the top card of their library;
                // if it matches `reveal_filter`, that player puts it into
                // their hand (otherwise it stays on top).
                for p in self.resolve_players(who, ctx) {
                    let Some(top) = self.players[p].library.first() else {
                        continue;
                    };
                    let card_name = top.definition.name;
                    let is_land = top.definition.is_land();
                    // `evaluate_requirement_on_card` works on any
                    // `CardInstance` (the battlefield-only variant would fail
                    // here since the card is in the library).
                    let matches =
                        self.evaluate_requirement_on_card(reveal_filter, top, ctx.controller);
                    events.push(GameEvent::TopCardRevealed {
                        player: p,
                        card_name,
                        is_land,
                    });
                    if matches {
                        let card = self.players[p].library.remove(0);
                        let cid = card.id;
                        self.players[p].hand.push(card);
                        events.push(GameEvent::CardDrawn { player: p, card_id: cid });
                    }
                }
                Ok(())
            }

            Effect::DiscardChosen { from, count, filter } => {
                // Resolve target player(s) — usually one opponent. For each,
                // discard `count` cards matching `filter` from their hand,
                // picking each one via the simplest legal choice.
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 { return Ok(()); }
                for ent in self.resolve_selector(from, ctx) {
                    let EntityRef::Player(p) = ent else { continue };
                    for _ in 0..n {
                        let pick = self.players[p].hand.iter().position(|c| {
                            self.evaluate_requirement_on_card(filter, c, ctx.controller)
                        });
                        let Some(idx) = pick else { break };
                        let card = self.players[p].hand.remove(idx);
                        let cid = card.id;
                        self.players[p].graveyard.push(card);
                        events.push(GameEvent::CardDiscarded { player: p, card_id: cid });
                    }
                }
                Ok(())
            }

            Effect::SacrificeAndRemember { who, filter } => {
                // Resolve `who` to a single player; pick one of their
                // controlled permanents matching `filter`; sacrifice it and
                // record its power on `state.sacrificed_power` so a
                // subsequent `Value::SacrificedPower` can reference it.
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let candidate = self
                    .battlefield
                    .iter()
                    .find(|c| {
                        c.controller == p
                            && self.evaluate_requirement_static(filter, &Target::Permanent(c.id), p)
                    })
                    .map(|c| (c.id, c.power()));
                if let Some((cid, power)) = candidate {
                    self.sacrificed_power = Some(power);
                    let is_creature = self
                        .battlefield_find(cid)
                        .map(|c| c.definition.is_creature())
                        .unwrap_or(false);
                    if is_creature {
                        events.push(GameEvent::CreatureDied { card_id: cid });
                    }
                    self.remove_from_battlefield_to_graveyard(cid);
                }
                Ok(())
            }

            Effect::DelayUntil { kind, body } => {
                // Capture the current target slot so the delayed body can
                // reference it via `Selector::Target(0)` later (e.g. Goryo's
                // wants to exile the same creature it reanimated).
                let target = ctx.targets.first().cloned();
                let source = ctx.source.unwrap_or(crate::card::CardId(0));
                self.delayed_triggers.push(DelayedTrigger {
                    controller: ctx.controller,
                    source,
                    kind: delayed_kind_from_effect(*kind),
                    effect: (**body).clone(),
                    target,
                    fires_once: true,
                });
                Ok(())
            }

            Effect::PayOrLoseGame { mana_cost, life_cost } => {
                let p = ctx.controller;
                // Try to pay mana via auto-tap, then deduct life. If any of
                // those fail, the controller loses the game. Roll back any
                // partial payment on failure.
                let cost_subbed = if mana_cost.has_x() {
                    mana_cost.with_x_value(0)
                } else {
                    mana_cost.clone()
                };
                let pool_before = self.players[p].mana_pool.clone();
                let tapped_before: Vec<(CardId, bool)> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.owner == p)
                    .map(|c| (c.id, c.tapped))
                    .collect();
                let mut paid_events = self.auto_tap_for_cost(p, &cost_subbed);
                let mana_paid = self.players[p].mana_pool.pay(&cost_subbed).is_ok();
                let life_ok = self.players[p].life > *life_cost as i32;
                if mana_paid && life_ok {
                    if *life_cost > 0 {
                        self.players[p].life -= *life_cost as i32;
                        paid_events.push(GameEvent::LifeLost { player: p, amount: *life_cost });
                    }
                    events.append(&mut paid_events);
                } else {
                    // Roll back the auto-tap and pool change.
                    self.players[p].mana_pool = pool_before;
                    for (id, was_tapped) in tapped_before {
                        if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == id) {
                            c.tapped = was_tapped;
                        }
                    }
                    self.players[p].eliminated = true;
                    let mut sba = self.check_state_based_actions();
                    events.append(&mut sba);
                }
                Ok(())
            }

            Effect::AddFirstSpellTax { who, count } => {
                let n = self.evaluate_value(count, ctx).max(0) as u32;
                if n == 0 {
                    return Ok(());
                }
                for p in self.resolve_players(who, ctx) {
                    self.players[p].first_spell_tax_charges =
                        self.players[p].first_spell_tax_charges.saturating_add(n);
                }
                Ok(())
            }

            Effect::GrantSorceriesAsFlash { who } => {
                for p in self.resolve_players(who, ctx) {
                    self.players[p].sorceries_as_flash = true;
                }
                Ok(())
            }

            Effect::RevealUntilFind {
                who,
                find,
                to,
                cap,
                life_per_revealed,
            } => {
                // Walk the top of `who`'s library until we either find a
                // matching card or hit the cap. Mill the misses.
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let cap_n = self.evaluate_value(cap, ctx).max(0) as usize;
                if cap_n == 0 {
                    return Ok(());
                }
                let resolved_dest = self.resolve_zonedest_player(to, ctx);
                let mut revealed = 0usize;
                let mut found_idx: Option<usize> = None;
                for i in 0..cap_n.min(self.players[p].library.len()) {
                    revealed += 1;
                    let card = &self.players[p].library[i];
                    if self.evaluate_requirement_on_card(find, card, ctx.controller) {
                        found_idx = Some(i);
                        break;
                    }
                }
                // Move the misses (everything before `found_idx`, or
                // everything if no match) into the graveyard.
                let mill_count = found_idx.unwrap_or(revealed);
                for _ in 0..mill_count {
                    if self.players[p].library.is_empty() {
                        break;
                    }
                    let card = self.players[p].library.remove(0);
                    let cid = card.id;
                    self.players[p].graveyard.push(card);
                    events.push(GameEvent::CardMilled { player: p, card_id: cid });
                }
                // If we found a match, take it off the (now-shifted) top
                // and place it via the requested destination.
                if found_idx.is_some() && !self.players[p].library.is_empty() {
                    let card = self.players[p].library.remove(0);
                    self.place_card_in_dest(card, p, &resolved_dest, events);
                }
                // Lose 1 life per revealed card (Spoils of the Vault rider).
                let life = (revealed as u32).saturating_mul(*life_per_revealed);
                if life > 0 {
                    self.players[p].life -= life as i32;
                    events.push(GameEvent::LifeLost { player: p, amount: life });
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::BecomeBasicLand { .. }
            | Effect::ResetCreature { .. }
            | Effect::CopySpell { .. } => {
                // TODO: implement via layer/stack mechanics.
                Ok(())
            }
        }
    }

    pub(crate) fn execute_put_on_library(
        &mut self,
        player: usize,
        chosen: &[crate::card::CardId],
        _events: &mut Vec<crate::game::GameEvent>,
    ) {
        // Remove chosen cards from hand (in reverse order to preserve indices).
        let mut cards_to_insert: Vec<crate::card::CardInstance> = Vec::new();
        for &id in chosen {
            if let Some(pos) = self.players[player].hand.iter().position(|c| c.id == id) {
                cards_to_insert.push(self.players[player].hand.remove(pos));
            }
        }
        // Insert in reverse so that chosen[0] ends up on top.
        for card in cards_to_insert.into_iter().rev() {
            self.players[player].library.insert(0, card);
        }
    }

    // ── Selector / Value / Predicate resolution ─────────────────────────────

    pub(crate) fn resolve_selector(&self, sel: &Selector, ctx: &EffectContext) -> Vec<EntityRef> {
        match sel {
            Selector::None => vec![],
            Selector::This => ctx.source.map(EntityRef::Permanent).into_iter().collect(),
            Selector::You => vec![EntityRef::Player(ctx.controller)],
            Selector::Target(idx) | Selector::TargetFiltered { slot: idx, .. } => ctx
                .targets
                .get(*idx as usize)
                .map(target_to_entity)
                .into_iter()
                .collect(),
            Selector::TriggerSource => ctx.trigger_source.into_iter().collect(),
            Selector::ChoiceResult(_) => vec![], // TODO when decision loop lands

            Selector::EachMatching { zone, filter } => self.entities_in_zone(zone, filter, ctx),
            Selector::EachPermanent(filter) => self
                .battlefield
                .iter()
                .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), ctx.controller))
                .map(|c| EntityRef::Permanent(c.id))
                .collect(),

            Selector::AttachedTo(inner) => self
                .resolve_selector(inner, ctx)
                .into_iter()
                .filter_map(|e| {
                    let EntityRef::Permanent(cid) = e else { return None; };
                    self.battlefield_find(cid)
                        .and_then(|c| c.attached_to)
                        .map(EntityRef::Permanent)
                })
                .collect(),

            Selector::AttachedToMe(inner) => {
                let anchors: Vec<CardId> = self
                    .resolve_selector(inner, ctx)
                    .into_iter()
                    .filter_map(|e| if let EntityRef::Permanent(c) = e { Some(c) } else { None })
                    .collect();
                self.battlefield
                    .iter()
                    .filter(|c| c.attached_to.is_some_and(|a| anchors.contains(&a)))
                    .map(|c| EntityRef::Permanent(c.id))
                    .collect()
            }

            Selector::TopOfLibrary { who, count } => {
                let Some(p) = self.resolve_player(who, ctx) else { return vec![]; };
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                self.players[p]
                    .library
                    .iter()
                    .take(n)
                    .map(|c| EntityRef::Card(c.id))
                    .collect()
            }
            Selector::BottomOfLibrary { who, count } => {
                let Some(p) = self.resolve_player(who, ctx) else { return vec![]; };
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let lib = &self.players[p].library;
                let total = lib.len();
                if n >= total {
                    lib.iter().map(|c| EntityRef::Card(c.id)).collect()
                } else {
                    lib.iter().skip(total - n).map(|c| EntityRef::Card(c.id)).collect()
                }
            }
            Selector::CardsInZone { who, zone, filter } => {
                let Some(p) = self.resolve_player(who, ctx) else { return vec![]; };
                let cards: Vec<&CardInstance> = match zone {
                    Zone::Hand => self.players[p].hand.iter().collect(),
                    Zone::Graveyard => self.players[p].graveyard.iter().collect(),
                    Zone::Library => self.players[p].library.iter().collect(),
                    Zone::Exile => self.exile.iter().filter(|c| c.owner == p).collect(),
                    Zone::Battlefield => self.battlefield.iter().filter(|c| c.controller == p).collect(),
                    Zone::Stack | Zone::Command => vec![],
                };
                cards
                    .into_iter()
                    .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), ctx.controller))
                    .map(|c| if matches!(zone, Zone::Battlefield) { EntityRef::Permanent(c.id) } else { EntityRef::Card(c.id) })
                    .collect()
            }

            Selector::Player(p) => self
                .resolve_players(p, ctx)
                .into_iter()
                .map(EntityRef::Player)
                .collect(),
        }
    }

    /// Multi-player resolver. `EachPlayer` and `EachOpponent` return all
    /// matching alive seats so effects like Wheel of Fortune actually hit
    /// every player. Non-collective `PlayerRef` variants resolve to a single
    /// seat (or empty if the reference can't be resolved).
    pub(crate) fn resolve_players(&self, pref: &PlayerRef, ctx: &EffectContext) -> Vec<usize> {
        match pref {
            PlayerRef::EachOpponent => (0..self.players.len())
                .filter(|i| *i != ctx.controller && self.players[*i].is_alive())
                .collect(),
            PlayerRef::EachPlayer => (0..self.players.len())
                .filter(|i| self.players[*i].is_alive())
                .collect(),
            _ => self.resolve_player(pref, ctx).into_iter().collect(),
        }
    }

    pub(crate) fn resolve_player(&self, pref: &PlayerRef, ctx: &EffectContext) -> Option<usize> {
        match pref {
            PlayerRef::You => Some(ctx.controller),
            PlayerRef::Seat(p) => Some(*p),
            PlayerRef::ActivePlayer => Some(self.active_player_idx),
            PlayerRef::Triggerer => ctx.trigger_source.and_then(|e| match e {
                EntityRef::Player(p) => Some(p),
                _ => None,
            }),
            PlayerRef::Target(idx) => ctx.targets.get(*idx as usize).and_then(|t| match t {
                Target::Player(p) => Some(*p),
                _ => None,
            }),
            PlayerRef::EachOpponent => {
                // Singular fallback — `resolve_players` returns the full set.
                (0..self.players.len())
                    .find(|i| *i != ctx.controller && self.players[*i].is_alive())
            }
            PlayerRef::EachPlayer => (0..self.players.len()).find(|i| self.players[*i].is_alive()),
            PlayerRef::DefendingPlayer => ctx
                .source
                .and_then(|src| self.attack_for(src).map(|a| a.target))
                .and_then(|target| self.defender_for(target)),
            PlayerRef::OwnerOf(sel) => self
                .resolve_selector(sel, ctx)
                .into_iter()
                .find_map(|e| match e {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => {
                        self.battlefield_find(cid).map(|c| c.owner)
                    }
                    _ => None,
                }),
            PlayerRef::ControllerOf(sel) => self
                .resolve_selector(sel, ctx)
                .into_iter()
                .find_map(|e| match e {
                    EntityRef::Permanent(cid) => self.battlefield_find(cid).map(|c| c.controller),
                    _ => None,
                }),
        }
    }

    fn entities_in_zone(
        &self,
        zone: &ZoneRef,
        filter: &SelectionRequirement,
        ctx: &EffectContext,
    ) -> Vec<EntityRef> {
        match zone {
            ZoneRef::Battlefield => self
                .battlefield
                .iter()
                .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), ctx.controller))
                .map(|c| EntityRef::Permanent(c.id))
                .collect(),
            ZoneRef::Stack => self
                .stack
                .iter()
                .filter_map(|si| match si {
                    StackItem::Spell { card, .. } => Some(EntityRef::Permanent(card.id)),
                    _ => None,
                })
                .collect(),
            ZoneRef::Library(who) | ZoneRef::Hand(who) | ZoneRef::Graveyard(who) => {
                let Some(p) = self.resolve_player(who, ctx) else { return vec![]; };
                let cards: Vec<&CardInstance> = match zone {
                    ZoneRef::Library(_) => self.players[p].library.iter().collect(),
                    ZoneRef::Hand(_) => self.players[p].hand.iter().collect(),
                    ZoneRef::Graveyard(_) => self.players[p].graveyard.iter().collect(),
                    _ => vec![],
                };
                cards
                    .into_iter()
                    .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), ctx.controller))
                    .map(|c| EntityRef::Card(c.id))
                    .collect()
            }
            ZoneRef::Exile => self
                .exile
                .iter()
                .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), ctx.controller))
                .map(|c| EntityRef::Card(c.id))
                .collect(),
            ZoneRef::Command => vec![],
        }
    }

    pub(crate) fn evaluate_value(&self, v: &Value, ctx: &EffectContext) -> i32 {
        match v {
            Value::Const(n) => *n,
            Value::CountOf(s) => self.resolve_selector(s, ctx).len() as i32,
            Value::PowerOf(s) => self.resolve_selector(s, ctx).iter().find_map(|e| {
                if let EntityRef::Permanent(cid) = e { self.battlefield_find(*cid).map(|c| c.power()) } else { None }
            }).unwrap_or(0),
            Value::ToughnessOf(s) => self.resolve_selector(s, ctx).iter().find_map(|e| {
                if let EntityRef::Permanent(cid) = e { self.battlefield_find(*cid).map(|c| c.toughness()) } else { None }
            }).unwrap_or(0),
            Value::LifeOf(p) => self.resolve_player(p, ctx).map(|p| self.players[p].life).unwrap_or(0),
            Value::HandSizeOf(p) => self.resolve_player(p, ctx).map(|p| self.players[p].hand.len() as i32).unwrap_or(0),
            Value::GraveyardSizeOf(p) => self.resolve_player(p, ctx).map(|p| self.players[p].graveyard.len() as i32).unwrap_or(0),
            Value::XFromCost => ctx.x_value as i32,
            Value::StormCount => self.spells_cast_this_turn.saturating_sub(1) as i32,
            Value::CountersOn { what, kind } => self
                .resolve_selector(what, ctx)
                .into_iter()
                .find_map(|e| if let EntityRef::Permanent(cid) = e { self.battlefield_find(cid).map(|c| c.counter_count(*kind) as i32) } else { None })
                .unwrap_or(0),
            Value::Sum(vs) => vs.iter().map(|v| self.evaluate_value(v, ctx)).sum(),
            Value::Diff(a, b) => self.evaluate_value(a, ctx) - self.evaluate_value(b, ctx),
            Value::Times(a, b) => self.evaluate_value(a, ctx) * self.evaluate_value(b, ctx),
            Value::Min(a, b) => self.evaluate_value(a, ctx).min(self.evaluate_value(b, ctx)),
            Value::Max(a, b) => self.evaluate_value(a, ctx).max(self.evaluate_value(b, ctx)),
            Value::NonNeg(v) => self.evaluate_value(v, ctx).max(0),
            Value::SacrificedPower => self.sacrificed_power.unwrap_or(0),
            Value::ConvergedValue => ctx.converged_value as i32,
            Value::ManaValueOf(s) => self
                .resolve_selector(s, ctx)
                .into_iter()
                .find_map(|e| match e {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => self
                        .battlefield_find(cid)
                        .or_else(|| {
                            self.players.iter().find_map(|p| {
                                p.graveyard
                                    .iter()
                                    .find(|c| c.id == cid)
                                    .or_else(|| p.hand.iter().find(|c| c.id == cid))
                                    .or_else(|| p.library.iter().find(|c| c.id == cid))
                            })
                        })
                        .or_else(|| self.exile.iter().find(|c| c.id == cid))
                        .map(|c| c.definition.cost.cmc() as i32),
                    EntityRef::Player(_) => None,
                })
                .unwrap_or(0),
            Value::DistinctTypesInTopOfLibrary { who, count } => {
                let Some(p) = self.resolve_player(who, ctx) else { return 0; };
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let mut seen: std::collections::HashSet<CardType> =
                    std::collections::HashSet::new();
                for card in self.players[p].library.iter().take(n) {
                    for t in &card.definition.card_types {
                        seen.insert(t.clone());
                    }
                }
                seen.len() as i32
            }
        }
    }

    pub(crate) fn evaluate_predicate(&self, p: &Predicate, ctx: &EffectContext) -> bool {
        match p {
            Predicate::True => true,
            Predicate::False => false,
            Predicate::Not(q) => !self.evaluate_predicate(q, ctx),
            Predicate::All(qs) => qs.iter().all(|q| self.evaluate_predicate(q, ctx)),
            Predicate::Any(qs) => qs.iter().any(|q| self.evaluate_predicate(q, ctx)),
            Predicate::SelectorExists(s) => !self.resolve_selector(s, ctx).is_empty(),
            Predicate::SelectorCountAtLeast { sel, n } => {
                self.resolve_selector(sel, ctx).len() as i32 >= self.evaluate_value(n, ctx)
            }
            Predicate::ValueAtLeast(a, b) => self.evaluate_value(a, ctx) >= self.evaluate_value(b, ctx),
            Predicate::ValueAtMost(a, b) => self.evaluate_value(a, ctx) <= self.evaluate_value(b, ctx),
            Predicate::IsTurnOf(pref) => self.resolve_player(pref, ctx) == Some(self.active_player_idx),
            Predicate::EntityMatches { what, filter } => self
                .resolve_selector(what, ctx)
                .into_iter()
                .all(|e| match e {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => {
                        self.evaluate_requirement_static(filter, &Target::Permanent(cid), ctx.controller)
                    }
                    EntityRef::Player(_) => matches!(filter, SelectionRequirement::Player),
                }),
        }
    }

    // ── Requirement evaluation (unchanged API) ──────────────────────────────

    pub(crate) fn evaluate_requirement_static(
        &self,
        req: &SelectionRequirement,
        target: &Target,
        controller: usize,
    ) -> bool {
        use SelectionRequirement as R;
        match req {
            R::Any => true,
            R::Player => matches!(target, Target::Player(_)),
            R::And(a, b) => self.evaluate_requirement_static(a, target, controller)
                && self.evaluate_requirement_static(b, target, controller),
            R::Or(a, b) => self.evaluate_requirement_static(a, target, controller)
                || self.evaluate_requirement_static(b, target, controller),
            R::Not(inner) => !self.evaluate_requirement_static(inner, target, controller),
            R::ControlledByYou => match target {
                Target::Permanent(cid) => self.battlefield_find(*cid).map(|c| c.controller == controller).unwrap_or(false),
                Target::Player(p) => *p == controller,
            },
            R::ControlledByOpponent => match target {
                Target::Permanent(cid) => self.battlefield_find(*cid).map(|c| c.controller != controller).unwrap_or(false),
                Target::Player(p) => *p != controller,
            },
            _ => {
                let Target::Permanent(cid) = target else { return false; };
                // Look on the battlefield first; fall through to graveyards,
                // exile, and the stack so reanimate-style spells (Goryo's
                // Vengeance, Reanimate, Animate Dead) can validate their
                // targets, and so counter-style spells (Mystical Dispute,
                // Force of Negation) can read the colors of a target stack
                // spell.
                let stack_card = self.stack.iter().find_map(|si| match si {
                    StackItem::Spell { card, .. } if card.id == *cid => Some(&**card),
                    _ => None,
                });
                let card = self
                    .battlefield_find(*cid)
                    .or_else(|| self.players.iter().find_map(|p| p.graveyard.iter().find(|c| c.id == *cid)))
                    .or_else(|| self.exile.iter().find(|c| c.id == *cid))
                    .or(stack_card);
                let Some(card) = card else { return false; };
                match req {
                    R::Creature => card.definition.is_creature(),
                    R::Artifact => card.definition.is_artifact(),
                    R::Enchantment => card.definition.is_enchantment(),
                    R::Planeswalker => card.definition.is_planeswalker(),
                    R::Permanent => card.definition.is_permanent(),
                    R::Land => card.definition.is_land(),
                    R::Nonland => !card.definition.is_land(),
                    R::Noncreature => !card.definition.is_creature(),
                    R::Tapped => card.tapped,
                    R::Untapped => !card.tapped,
                    R::HasColor(c) => card
                        .definition
                        .cost
                        .symbols
                        .iter()
                        .any(|s| matches!(s, ManaSymbol::Colored(cc) if cc == c)),
                    R::HasKeyword(kw) => card.has_keyword(kw),
                    R::PowerAtMost(n) => card.definition.is_creature() && card.power() <= *n,
                    R::ToughnessAtMost(n) => card.definition.is_creature() && card.toughness() <= *n,
                    R::PowerAtLeast(n) => card.definition.is_creature() && card.power() >= *n,
                    R::ToughnessAtLeast(n) => card.definition.is_creature() && card.toughness() >= *n,
                    R::WithCounter(k) => card.counter_count(*k) > 0,
                    R::HasSupertype(st) => card.definition.supertypes.contains(st),
                    R::HasCreatureType(ct) => card.definition.subtypes.creature_types.contains(ct),
                    R::HasLandType(lt) => card.definition.subtypes.land_types.contains(lt),
                    R::HasArtifactSubtype(a) => card.definition.subtypes.artifact_subtypes.contains(a),
                    R::HasEnchantmentSubtype(e) => card.definition.subtypes.enchantment_subtypes.contains(e),
                    R::IsToken => card.is_token,
                    R::NotToken => !card.is_token,
                    R::IsBasicLand => card.definition.is_land() && card.definition.supertypes.contains(&Supertype::Basic),
                    R::IsAttacking => self.attacking.iter().any(|a| a.attacker == card.id),
                    R::IsBlocking => self.block_map.contains_key(&card.id),
                    R::IsSpellOnStack => self.stack.iter().any(|si| matches!(si, StackItem::Spell { card: c, .. } if c.id == card.id)),
                    R::ManaValueAtMost(n) => card.definition.cost.cmc() <= *n,
                    R::ManaValueAtLeast(n) => card.definition.cost.cmc() >= *n,
                    R::HasCardType(ct) => card.definition.card_types.contains(ct),
                    _ => unreachable!("handled above"),
                }
            }
        }
    }

    /// Evaluate a `SelectionRequirement` directly against a `CardInstance`
    /// without requiring it to be on the battlefield. Used for library searches.
    /// Battlefield-only predicates (Tapped, IsAttacking, etc.) return false.
    pub(crate) fn evaluate_requirement_on_card(
        &self,
        req: &SelectionRequirement,
        card: &CardInstance,
        controller: usize,
    ) -> bool {
        use SelectionRequirement as R;
        match req {
            R::Any => true,
            R::Player => false,
            R::And(a, b) => {
                self.evaluate_requirement_on_card(a, card, controller)
                    && self.evaluate_requirement_on_card(b, card, controller)
            }
            R::Or(a, b) => {
                self.evaluate_requirement_on_card(a, card, controller)
                    || self.evaluate_requirement_on_card(b, card, controller)
            }
            R::Not(inner) => !self.evaluate_requirement_on_card(inner, card, controller),
            R::ControlledByYou => card.controller == controller,
            R::ControlledByOpponent => card.controller != controller,
            R::Creature => card.definition.is_creature(),
            R::Artifact => card.definition.is_artifact(),
            R::Enchantment => card.definition.is_enchantment(),
            R::Planeswalker => card.definition.is_planeswalker(),
            R::Permanent => card.definition.is_permanent(),
            R::Land => card.definition.is_land(),
            R::Nonland => !card.definition.is_land(),
            R::Noncreature => !card.definition.is_creature(),
            R::HasColor(c) => card.definition.cost.symbols.iter().any(|s| {
                matches!(s, crate::mana::ManaSymbol::Colored(cc) if cc == c)
            }),
            R::HasKeyword(kw) => card.has_keyword(kw),
            R::PowerAtMost(n) => card.definition.is_creature() && card.power() <= *n,
            R::PowerAtLeast(n) => card.definition.is_creature() && card.power() >= *n,
            R::ToughnessAtMost(n) => card.definition.is_creature() && card.toughness() <= *n,
            R::ToughnessAtLeast(n) => card.definition.is_creature() && card.toughness() >= *n,
            R::HasSupertype(st) => card.definition.supertypes.contains(st),
            R::HasCreatureType(ct) => card.definition.subtypes.creature_types.contains(ct),
            R::HasLandType(lt) => card.definition.subtypes.land_types.contains(lt),
            R::HasArtifactSubtype(a) => card.definition.subtypes.artifact_subtypes.contains(a),
            R::HasEnchantmentSubtype(e) => card.definition.subtypes.enchantment_subtypes.contains(e),
            R::IsToken => card.is_token,
            R::NotToken => !card.is_token,
            R::IsBasicLand => card.definition.is_land() && card.definition.supertypes.contains(&Supertype::Basic),
            R::ManaValueAtMost(n) => card.definition.cost.cmc() <= *n,
            R::ManaValueAtLeast(n) => card.definition.cost.cmc() >= *n,
            R::HasCardType(ct) => card.definition.card_types.contains(ct),
            // Battlefield-state predicates can't be evaluated for library cards.
            R::Tapped | R::Untapped | R::WithCounter(_)
            | R::IsAttacking | R::IsBlocking | R::IsSpellOnStack => false,
        }
    }

    // ── Auto-target heuristic for simple triggers ────────────────────────────

    /// Pick a legal target for an effect that requires one, used when the
    /// engine fires a trigger without explicit user input (ETB, attack trigger,
    /// etc.). Returns `None` if the effect requires no target or no legal
    /// target exists.
    pub fn auto_target_for_effect(&self, eff: &Effect, controller: usize) -> Option<Target> {
        let req = eff.primary_target_filter()?;
        // Opponent first; fall back to controller for "any".
        let opp = (controller + 1) % self.players.len();
        if self.evaluate_requirement_static(req, &Target::Player(opp), controller) {
            return Some(Target::Player(opp));
        }
        if self.evaluate_requirement_static(req, &Target::Player(controller), controller) {
            return Some(Target::Player(controller));
        }
        // Battlefield first.
        if let Some(t) = self
            .battlefield
            .iter()
            .find(|c| self.evaluate_requirement_static(req, &Target::Permanent(c.id), controller))
            .map(|c| Target::Permanent(c.id))
        {
            return Some(t);
        }
        // Then graveyards (so reanimate-style spells like Goryo's Vengeance,
        // Animate Dead, Reanimate auto-pick a legal graveyard target when
        // no manual target is supplied). `evaluate_requirement_static`
        // already walks graveyard/exile so this just makes the auto-target
        // path consult those zones too.
        for player in &self.players {
            if let Some(c) = player.graveyard.iter().find(|c| {
                self.evaluate_requirement_static(req, &Target::Permanent(c.id), controller)
            }) {
                return Some(Target::Permanent(c.id));
            }
        }
        // Then exile (cards that target exiled cards — Misthollow Griffin,
        // some Ixalan stuff).
        if let Some(c) = self.exile.iter().find(|c| {
            self.evaluate_requirement_static(req, &Target::Permanent(c.id), controller)
        }) {
            return Some(Target::Permanent(c.id));
        }
        None
    }

    // ── Zone move helpers ────────────────────────────────────────────────────

    fn deal_damage_to(&mut self, ent: EntityRef, amount: u32, events: &mut Vec<GameEvent>) {
        match ent {
            EntityRef::Player(p) => {
                self.players[p].life -= amount as i32;
                events.push(GameEvent::DamageDealt { amount, to_player: Some(p), to_card: None });
                events.push(GameEvent::LifeLost { player: p, amount });
            }
            EntityRef::Permanent(cid) => {
                if let Some(c) = self.battlefield_find_mut(cid) {
                    c.damage += amount;
                    events.push(GameEvent::DamageDealt { amount, to_player: None, to_card: Some(cid) });
                }
            }
            EntityRef::Card(_) => {}
        }
    }

    fn move_card_to(
        &mut self,
        cid: CardId,
        dest: &ZoneDest,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        // Resolve any selector-based player refs in the destination *now*,
        // while the card is still findable in its source zone — otherwise
        // `PlayerRef::OwnerOf(Target(0))` can't see the card after we remove
        // it. The resolved dest uses concrete `PlayerRef::You`-anchored refs.
        let resolved_dest = self.resolve_zonedest_player(dest, ctx);

        // Try battlefield first.
        if let Some(pos) = self.battlefield.iter().position(|c| c.id == cid) {
            let mut card = self.battlefield.remove(pos);
            self.remove_effects_from_source(cid);
            card.damage = 0;
            card.tapped = false;
            card.attached_to = None;
            self.place_card_in_dest(card, ctx.controller, &resolved_dest, events);
            return;
        }
        // Then graveyards.
        for p in 0..self.players.len() {
            if let Some(pos) = self.players[p].graveyard.iter().position(|c| c.id == cid) {
                let card = self.players[p].graveyard.remove(pos);
                self.place_card_in_dest(card, p, &resolved_dest, events);
                return;
            }
        }
        // Then exile.
        if let Some(pos) = self.exile.iter().position(|c| c.id == cid) {
            let card = self.exile.remove(pos);
            let owner = card.owner;
            self.place_card_in_dest(card, owner, &resolved_dest, events);
        }
    }

    /// Pre-resolve any selector-based player refs in a `ZoneDest` against
    /// the active ctx. `place_card_in_dest` constructs its own bare ctx and
    /// can't see the caster's targets, so any `PlayerRef::OwnerOf(Selector)`
    /// / `ControllerOf(Selector)` need to be flattened to a concrete
    /// `PlayerRef::Seat(n)` while the source card is still in its origin
    /// zone. Other ref kinds (You / ActivePlayer / etc.) pass through.
    fn resolve_zonedest_player(&self, dest: &ZoneDest, ctx: &EffectContext) -> ZoneDest {
        let flatten = |who: &PlayerRef| -> PlayerRef {
            match who {
                PlayerRef::OwnerOf(_) | PlayerRef::ControllerOf(_) => {
                    if let Some(p) = self.resolve_player(who, ctx) {
                        PlayerRef::Seat(p)
                    } else {
                        who.clone()
                    }
                }
                _ => who.clone(),
            }
        };
        match dest {
            ZoneDest::Hand(who) => ZoneDest::Hand(flatten(who)),
            ZoneDest::Library { who, pos } => ZoneDest::Library {
                who: flatten(who),
                pos: *pos,
            },
            ZoneDest::Battlefield { controller, tapped } => ZoneDest::Battlefield {
                controller: flatten(controller),
                tapped: *tapped,
            },
            ZoneDest::Graveyard | ZoneDest::Exile => dest.clone(),
        }
    }

    pub(crate) fn place_card_in_dest(
        &mut self,
        mut card: CardInstance,
        default_player: usize,
        dest: &ZoneDest,
        events: &mut Vec<GameEvent>,
    ) {
        match dest {
            ZoneDest::Hand(who) => {
                let ctx = EffectContext::for_spell(default_player, None, 0, 0);
                let p = self.resolve_player(who, &ctx).unwrap_or(default_player);
                card.controller = p;
                self.players[p].hand.push(card);
            }
            ZoneDest::Library { who, pos } => {
                let ctx = EffectContext::for_spell(default_player, None, 0, 0);
                let p = self.resolve_player(who, &ctx).unwrap_or(default_player);
                match pos {
                    LibraryPosition::Top => self.players[p].library.insert(0, card),
                    LibraryPosition::Bottom | LibraryPosition::Shuffled => self.players[p].library.push(card),
                }
            }
            ZoneDest::Graveyard => {
                let owner = card.owner;
                self.players[owner].send_to_graveyard(card);
            }
            ZoneDest::Exile => {
                let cid = card.id;
                self.exile.push(card);
                events.push(GameEvent::PermanentExiled { card_id: cid });
            }
            ZoneDest::Battlefield { controller, tapped } => {
                let ctx = EffectContext::for_spell(default_player, None, 0, 0);
                let p = self.resolve_player(controller, &ctx).unwrap_or(default_player);
                card.controller = p;
                card.tapped = *tapped;
                card.summoning_sick = card.definition.is_creature();
                // A permanent entering the battlefield from another zone is
                // a brand-new object (rule 400.7) — clear residual damage,
                // pump bonuses, and attachment.
                card.damage = 0;
                card.power_bonus = 0;
                card.toughness_bonus = 0;
                card.attached_to = None;
                let cid = card.id;
                self.battlefield.push(card);
                events.push(GameEvent::PermanentEntered { card_id: cid });
                // Fire self-source ETB triggers so reanimate / flicker /
                // search-to-battlefield paths trigger creature ETBs the same
                // way casting does.
                self.fire_self_etb_triggers(cid, p);
            }
        }
    }
}

fn target_to_entity(t: &Target) -> EntityRef {
    match t {
        Target::Player(p) => EntityRef::Player(*p),
        Target::Permanent(c) => EntityRef::Permanent(*c),
    }
}

// ── Token → CardDefinition ──────────────────────────────────────────────────

pub fn token_to_card_definition(token: &TokenDefinition) -> CardDefinition {
    CardDefinition {
        name: token.name,
        cost: ManaCost::default(),
        supertypes: token.supertypes.clone(),
        card_types: token.card_types.clone(),
        subtypes: token.subtypes.clone(),
        power: token.power,
        toughness: token.toughness,
        base_loyalty: 0,
        keywords: token.keywords.clone(),
        static_abilities: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Event matching for triggers ─────────────────────────────────────────────

/// Returns true if `event` matches the `EventSpec` on `source` (a permanent
/// on the battlefield). Used by `fire_triggers_for_event` to decide whether a
/// triggered ability should be pushed onto the stack.
pub(crate) fn event_matches_spec(
    event: &GameEvent,
    spec: &EventSpec,
    source: &CardInstance,
) -> bool {
    let kind_ok = match (&spec.kind, event) {
        (EventKind::EntersBattlefield, GameEvent::PermanentEntered { .. }) => true,
        (EventKind::CreatureDied, GameEvent::CreatureDied { .. }) => true,
        (EventKind::PermanentLeavesBattlefield, GameEvent::CreatureDied { .. }) => true,
        (EventKind::CardDrawn, GameEvent::CardDrawn { .. }) => true,
        (EventKind::CardDiscarded, GameEvent::CardDiscarded { .. }) => true,
        (EventKind::LandPlayed, GameEvent::LandPlayed { .. }) => true,
        (EventKind::SpellCast, GameEvent::SpellCast { .. }) => true,
        (EventKind::Attacks, GameEvent::AttackerDeclared(_)) => true,
        (EventKind::BecomesBlocked, GameEvent::BlockerDeclared { .. }) => true,
        (EventKind::LifeGained, GameEvent::LifeGained { .. }) => true,
        (EventKind::LifeLost, GameEvent::LifeLost { .. }) => true,
        (EventKind::StepBegins(s), GameEvent::StepChanged(got)) => s == got,
        (EventKind::TurnBegins, GameEvent::TurnStarted { .. }) => true,
        (EventKind::CounterAdded(k), GameEvent::CounterAdded { counter_type, .. }) => counter_type == k,
        (EventKind::AbilityActivated, GameEvent::AbilityActivated { .. }) => true,
        _ => false,
    };
    if !kind_ok {
        return false;
    }

    let scope_ok = match spec.scope {
        EventScope::SelfSource => matches!(
            event,
            GameEvent::PermanentEntered { card_id } if *card_id == source.id
        ) || matches!(
            event,
            GameEvent::AttackerDeclared(id) if *id == source.id
        ) || matches!(
            event,
            GameEvent::CreatureDied { card_id } if *card_id == source.id
        ) || matches!(
            event,
            GameEvent::BlockerDeclared { attacker, .. } if *attacker == source.id
        ),
        EventScope::YourControl => event_player(event).is_some_and(|p| p == source.controller),
        EventScope::OpponentControl => event_player(event).is_some_and(|p| p != source.controller),
        EventScope::AnyPlayer | EventScope::ActivePlayer => true,
        EventScope::AnotherOfYours => {
            // ETB/die triggers for "another creature"
            let target = event_card(event);
            target != Some(source.id)
        }
    };

    if !scope_ok {
        return false;
    }

    // Filter predicate evaluation is deferred to when the trigger actually
    // resolves; at this stage we just ensure the shape matches.
    true
}

fn event_player(event: &GameEvent) -> Option<usize> {
    match event {
        GameEvent::CardDrawn { player, .. }
        | GameEvent::CardDiscarded { player, .. }
        | GameEvent::LandPlayed { player, .. }
        | GameEvent::SpellCast { player, .. }
        | GameEvent::LifeGained { player, .. }
        | GameEvent::LifeLost { player, .. }
        | GameEvent::PoisonAdded { player, .. }
        | GameEvent::CardMilled { player, .. }
        | GameEvent::ManaAdded { player, .. }
        | GameEvent::ColorlessManaAdded { player }
        | GameEvent::TurnStarted { player, .. } => Some(*player),
        _ => None,
    }
}

fn event_card(event: &GameEvent) -> Option<CardId> {
    match event {
        GameEvent::PermanentEntered { card_id }
        | GameEvent::PermanentExiled { card_id }
        | GameEvent::CreatureDied { card_id }
        | GameEvent::PermanentTapped { card_id }
        | GameEvent::PermanentUntapped { card_id }
        | GameEvent::TokenCreated { card_id }
        | GameEvent::AttackerDeclared(card_id) => Some(*card_id),
        GameEvent::BlockerDeclared { blocker, .. } => Some(*blocker),
        _ => None,
    }
}

// ── Built-in token definitions ───────────────────────────────────────────────

#[allow(dead_code)]
pub fn food_token() -> TokenDefinition {
    TokenDefinition {
        name: "Food",
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Artifact],
        colors: vec![],
        supertypes: vec![],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Food],
            ..Default::default()
        },
    }
}

#[allow(dead_code)]
pub fn treasure_token() -> TokenDefinition {
    TokenDefinition {
        name: "Treasure",
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Artifact],
        colors: vec![],
        supertypes: vec![],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Treasure],
            ..Default::default()
        },
    }
}

#[allow(dead_code)]
pub fn blood_token() -> TokenDefinition {
    TokenDefinition {
        name: "Blood",
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Artifact],
        colors: vec![],
        supertypes: vec![],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Blood],
            ..Default::default()
        },
    }
}

#[allow(dead_code)]
pub fn clue_token() -> TokenDefinition {
    TokenDefinition {
        name: "Clue",
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Artifact],
        colors: vec![],
        supertypes: vec![],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Clue],
            ..Default::default()
        },
    }
}

/// Translate an `Effect`-side `DelayedTriggerKind` to its game-state mirror
/// `DelayedKind`. Centralized so adding a new delayed-trigger kind requires
/// only this one pattern match update.
pub(crate) fn delayed_kind_from_effect(
    k: crate::effect::DelayedTriggerKind,
) -> super::DelayedKind {
    use crate::effect::DelayedTriggerKind;
    use super::DelayedKind;
    match k {
        DelayedTriggerKind::YourNextUpkeep => DelayedKind::YourNextUpkeep,
        DelayedTriggerKind::NextEndStep => DelayedKind::NextEndStep,
        DelayedTriggerKind::YourNextMainPhase => DelayedKind::YourNextMainPhase,
    }
}
