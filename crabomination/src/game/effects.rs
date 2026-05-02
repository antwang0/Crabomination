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
    /// For a resolving spell, the cast-face / cast-path that produced it
    /// (Front from hand, Back-face MDFC, or Flashback from graveyard).
    /// `Predicate::CastFromGraveyard` reads this to gate "if this spell
    /// was cast from your graveyard / not from hand" riders (Antiquities
    /// on the Loose, Plumb the Forbidden's flashback variants). Defaults
    /// to `CastFace::Front` for non-spell contexts (triggers, abilities).
    pub cast_face: crate::game::types::CastFace,
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
            cast_face: crate::game::types::CastFace::Front,
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
            cast_face: crate::game::types::CastFace::Front,
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
            cast_face: crate::game::types::CastFace::Front,
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
            cast_face: crate::game::types::CastFace::Front,
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
        // Reset last-created-token scratch — `Selector::LastCreatedToken`
        // only refers to a token created by *this* resolution.
        self.last_created_token = None;
        // Reset per-resolution discard tally so
        // `Value::CardsDiscardedThisResolution` only sees the count
        // of cards discarded by *this* resolving effect (Borrowed
        // Knowledge mode 1, Colossus death rider). Also reset the
        // sibling id list used by Mind Roots et al.
        self.cards_discarded_this_resolution = 0;
        self.cards_discarded_this_resolution_ids.clear();
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

            Effect::MayDo { description, body } => {
                // Yes/no decision via `Decision::OptionalTrigger`. The
                // installed `Decider` answers — `AutoDecider` defaults to
                // `Bool(false)` (skip), `ScriptedDecider` lets tests
                // inject `Bool(true)` to exercise the body. Asked of the
                // *controller* of the effect (`ctx.controller`).
                //
                // Synchronous path: we don't currently surface MayDo
                // through the wants_ui suspend flow because the decision
                // is local to one effect resolution and the wire format
                // already carries `DecisionWire::OptionalTrigger`. A
                // future refinement could plumb it through
                // `suspend_signal` for human-in-the-loop play; for now
                // wants_ui players land on the AutoDecider's `false`
                // default.
                use crate::decision::{Decision, DecisionAnswer};
                let source = ctx.source.unwrap_or(CardId(0));
                let answer = self.decider.decide(&Decision::OptionalTrigger {
                    source,
                    description: description.clone(),
                });
                let yes = matches!(answer, DecisionAnswer::Bool(true));
                if yes {
                    self.run_effect(body, ctx, events)?;
                }
                Ok(())
            }

            Effect::MayPay {
                description,
                mana_cost,
                body,
            } => {
                // Sibling to `MayDo`: ask yes/no, then *attempt* to pay
                // mana. If the controller can't afford the cost the body
                // is skipped silently (the decision is moot, no error).
                // The cost is deducted from the controller's already-
                // floated mana pool — we don't auto-tap lands inside an
                // effect (mana abilities aren't activatable mid-resolve
                // by default).
                use crate::decision::{Decision, DecisionAnswer};
                let source = ctx.source.unwrap_or(CardId(0));
                let answer = self.decider.decide(&Decision::OptionalTrigger {
                    source,
                    description: description.clone(),
                });
                if !matches!(answer, DecisionAnswer::Bool(true)) {
                    return Ok(());
                }
                // Pre-flight: try paying. On failure, treat as decline.
                let pool = &mut self.players[ctx.controller].mana_pool;
                if pool.pay(mana_cost).is_err() {
                    return Ok(());
                }
                self.run_effect(body, ctx, events)?;
                Ok(())
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

            Effect::Fight { attacker, defender } => {
                // Two creatures simultaneously deal damage equal to
                // their power to each other. Snapshot powers up-front
                // so post-damage stats don't affect the back-swing.
                // Either side resolving to no permanent (target left
                // the battlefield, defender selector matches nothing)
                // no-ops the whole fight, matching MTG's "if either
                // is no longer a creature, no damage is dealt".
                let atk_id = self
                    .resolve_selector(attacker, ctx)
                    .into_iter()
                    .find_map(|e| match e {
                        EntityRef::Permanent(c) => Some(c),
                        _ => None,
                    });
                let def_id = self
                    .resolve_selector(defender, ctx)
                    .into_iter()
                    .find_map(|e| match e {
                        EntityRef::Permanent(c) => Some(c),
                        _ => None,
                    });
                let (Some(atk_id), Some(def_id)) = (atk_id, def_id) else {
                    return Ok(());
                };
                let atk_power = self.battlefield_find(atk_id).map(|c| c.power()).unwrap_or(0);
                let def_power = self.battlefield_find(def_id).map(|c| c.power()).unwrap_or(0);
                if atk_power > 0 {
                    self.deal_damage_to(EntityRef::Permanent(def_id), atk_power as u32, events);
                }
                if def_power > 0 {
                    self.deal_damage_to(EntityRef::Permanent(atk_id), def_power as u32, events);
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
                        self.players[p].life_gained_this_turn =
                            self.players[p].life_gained_this_turn.saturating_add(amt);
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
                        self.players[p].life_gained_this_turn =
                            self.players[p].life_gained_this_turn.saturating_add(amt);
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
                use crate::decision::Decision;
                let n = self.evaluate_value(amount, ctx).max(0) as usize;
                if n == 0 { return Ok(()); }
                for ent in self.resolve_selector(who, ctx) {
                    let EntityRef::Player(p) = ent else { continue };
                    if *random {
                        // Random-discard semantics: deterministic-pick-first
                        // for the in-process tests; a real client would seed
                        // an RNG, but the bot harness doesn't care which
                        // card gets dumped.
                        for _ in 0..n {
                            if self.players[p].hand.is_empty() { break; }
                            let card = self.players[p].hand.remove(0);
                            let cid = card.id;
                            self.players[p].graveyard.push(card);
                            events.push(GameEvent::CardDiscarded { player: p, card_id: cid });
                            self.cards_discarded_this_resolution =
                                self.cards_discarded_this_resolution.saturating_add(1);
                            self.cards_discarded_this_resolution_ids.push(cid);
                        }
                        continue;
                    }
                    // Player-chosen discard: surface a `Decision::Discard` so
                    // the discarding player picks N cards from their own
                    // hand. Reuses the `DiscardChosenPending` resume context
                    // (the resume logic only cares about which player loses
                    // the chosen cards, not who picked them).
                    if self.players[p].hand.is_empty() { continue; }
                    let candidates: Vec<(crate::card::CardId, String)> = self
                        .players[p]
                        .hand
                        .iter()
                        .map(|c| (c.id, c.definition.name.to_string()))
                        .collect();
                    let count = (candidates.len().min(n)) as u32;
                    let decision = Decision::Discard {
                        player: p,
                        count,
                        hand: candidates,
                    };
                    let pending = PendingEffectState::DiscardChosenPending { target_player: p };
                    if self.players[p].wants_ui {
                        self.suspend_signal = Some((decision, pending, Effect::Noop));
                        return Ok(());
                    }
                    let answer = self.decider.decide(&decision);
                    let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                    events.append(&mut applied);
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
                let peek: Vec<(CardId, String)> = self.players[p]
                    .library
                    .iter()
                    .take(n)
                    .map(|c| (c.id, c.definition.name.to_string()))
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
                    ManaPayload::OfColor(color, v) => {
                        // Fixed-color, value-scaled mana adder. No player
                        // choice — just N pips of `color`. Used by
                        // power-scaled mana abilities (Topiary Lecturer,
                        // Rofellos when promoted to per-Forest scaling).
                        let n = self.evaluate_value(v, ctx).max(0) as u32;
                        for _ in 0..n {
                            self.players[p].mana_pool.add(*color, 1);
                            events.push(GameEvent::ManaAdded { player: p, color: *color });
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
                            // Bump the controller's per-turn exile tally
                            // for Ennis-style "if a card was put into
                            // exile this turn" payoffs.
                            if ctx.controller < self.players.len() {
                                self.players[ctx.controller].cards_exiled_this_turn =
                                    self.players[ctx.controller].cards_exiled_this_turn.saturating_add(1);
                            }
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

            Effect::Untap { what, up_to } => {
                let cap = up_to
                    .as_ref()
                    .map(|v| self.evaluate_value(v, ctx).max(0) as usize);
                let mut count = 0usize;
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(c) = cap
                        && count >= c
                    {
                        break;
                    }
                    if let EntityRef::Permanent(cid) = ent
                        && let Some(c) = self.battlefield_find_mut(cid)
                        && c.tapped {
                            c.tapped = false;
                            count += 1;
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
                    // Stash the freshly-minted id so a follow-up
                    // `Selector::LastCreatedToken` in the same resolution
                    // (e.g. a Seq's next element) can reference it. Cleared
                    // when the next resolution root starts.
                    self.last_created_token = Some(id);
                    // Tokens entering the battlefield are still permanents
                    // entering the battlefield — fire any self-source ETB
                    // triggers on the token's definition (a TokenDefinition
                    // currently doesn't carry triggered_abilities, but if
                    // one is added later it will fire correctly).
                    self.fire_self_etb_triggers(id, p);
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
                    if let StackItem::Spell { card, caster, is_copy, .. } =
                        self.stack.remove(pos)
                    {
                        if !is_copy {
                            self.players[caster].send_to_graveyard(*card);
                        }
                        // Copies cease to exist on counter — drop without
                        // zoning. Per MTG rule 707.10, a countered copy
                        // disappears from the stack as if it had resolved.
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
                // taps that player's lands. `try_pay_with_auto_tap` rolls
                // back the pool + tap state on payment failure.
                let saved_priority = self.priority.player_with_priority;
                self.priority.player_with_priority = spell_caster;
                let paid = self.try_pay_with_auto_tap(spell_caster, mana_cost).is_ok();
                self.priority.player_with_priority = saved_priority;

                if !paid
                    && let StackItem::Spell { card, caster, is_copy, .. } =
                        self.stack.remove(pos)
                {
                    if !is_copy {
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
                    // Prioritize sacrifice picks: tokens first (free), then
                    // by lowest mana value, then by lowest power. This is a
                    // simple AutoDecider heuristic — when forced to
                    // sacrifice, dump the cheapest/weakest first instead of
                    // whichever happens to be at the front of the
                    // battlefield Vec.
                    let mut candidates: Vec<&CardInstance> = self
                        .battlefield
                        .iter()
                        .filter(|c| c.controller == p)
                        .filter(|c| {
                            let t = Target::Permanent(c.id);
                            self.evaluate_requirement_static(filter, &t, p)
                        })
                        .collect();
                    candidates.sort_by_key(|c| {
                        (
                            !c.is_token, // false (=0) sorts before true (=1) → tokens first
                            c.definition.cost.cmc(),
                            c.power(),
                        )
                    });
                    let ids: Vec<CardId> =
                        candidates.into_iter().take(n).map(|c| c.id).collect();
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
                let candidates: Vec<(crate::card::CardId, String)> = self.players[p]
                    .library
                    .iter()
                    .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                    .map(|c| (c.id, c.definition.name.to_string()))
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
                let hand_snapshot: Vec<(crate::card::CardId, String)> =
                    self.players[p].hand.iter().map(|c| (c.id, c.definition.name.to_string())).collect();
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

            Effect::RevealTopCard { who } => {
                for p in self.resolve_players(who, ctx) {
                    let Some(top) = self.players[p].library.first() else { continue };
                    events.push(GameEvent::TopCardRevealed {
                        player: p,
                        card_name: top.definition.name,
                        is_land: top.definition.is_land(),
                    });
                }
                Ok(())
            }

            Effect::DiscardChosen { from, count, filter } => {
                // Resolve target player(s) — usually one opponent. For each,
                // the **caster** picks `count` cards matching `filter`
                // from that player's hand. When the caster has `wants_ui`,
                // the engine suspends with a `Decision::Discard` so the
                // human picks; otherwise the decider is invoked
                // synchronously (AutoDecider takes the first matching).
                use crate::decision::Decision;
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 { return Ok(()); }
                let picker = ctx.controller;
                for ent in self.resolve_selector(from, ctx) {
                    let EntityRef::Player(target_player) = ent else { continue };
                    let candidates: Vec<(crate::card::CardId, String)> = self
                        .players[target_player]
                        .hand
                        .iter()
                        .filter(|c| self.evaluate_requirement_on_card(filter, c, picker))
                        .map(|c| (c.id, c.definition.name.to_string()))
                        .collect();
                    if candidates.is_empty() {
                        continue;
                    }
                    let decision = Decision::Discard {
                        player: picker,
                        count: n as u32,
                        hand: candidates,
                    };
                    let pending = PendingEffectState::DiscardChosenPending { target_player };

                    if self.players[picker].wants_ui {
                        self.suspend_signal = Some((decision, pending, Effect::Noop));
                        return Ok(());
                    }
                    let answer = self.decider.decide(&decision);
                    let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                    events.append(&mut applied);
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
                // We can't go through `try_pay_with_auto_tap` directly:
                // even on a successful pay, we may need to roll back if
                // `life_cost` would lose the game. Snapshot manually,
                // commit on success, restore on either failure path.
                let snapshot = self.snapshot_payment_state(p);
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
                    self.restore_payment_state(p, snapshot);
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
            | Effect::ResetCreature { .. } => {
                // TODO: implement via layer/stack mechanics.
                Ok(())
            }

            Effect::CopySpell { what, count } => {
                // Resolve the selector to find the spell to copy. The
                // typical caller is a magecraft / Casualty-style trigger
                // that copies the just-cast spell; that spell sits on
                // the stack as a `StackItem::Spell` with the same
                // `card.id` as `ctx.trigger_source` (when fired off a
                // SpellCast event) or `Selector::Target(0)` (when
                // copying a targeted spell, e.g. Choreographed Sparks).
                //
                // Only non-permanent spells (instants, sorceries) are
                // supported in this first cut. Copies of permanent
                // spells should become tokens per MTG rule 707.10b — that
                // path is a follow-up (no permanent-copy primitive yet).
                let candidates = self.resolve_selector(what, ctx);
                let Some(target_cid) = candidates.into_iter().find_map(|e| match e {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => Some(cid),
                    _ => None,
                }) else {
                    return Ok(());
                };
                // Find the spell's stack-item template (without removing).
                let template = self.stack.iter().find_map(|si| match si {
                    StackItem::Spell {
                        card,
                        caster,
                        target,
                        mode,
                        x_value,
                        converged_value,
                        face,
                        ..
                    } if card.id == target_cid && !card.definition.is_permanent() => Some((
                        (**card).clone(),
                        *caster,
                        target.clone(),
                        *mode,
                        *x_value,
                        *converged_value,
                        *face,
                    )),
                    _ => None,
                });
                let Some((mut card_template, _original_caster, tgt, md, xv, cv, fc)) = template
                else {
                    return Ok(());
                };
                // Copies are controlled by the source's controller (the
                // listener that fired the trigger), not the original
                // caster. This matches MTG's rule for "you may copy
                // that spell" (Casualty / Storm) — the copy's
                // controller is the listener.
                let copy_controller = ctx.controller;
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                for _ in 0..n {
                    // Mint a fresh `CardInstance` id for each copy so
                    // they can be distinguished on the stack and don't
                    // collide with the original.
                    card_template.id = self.next_id();
                    self.stack.push(StackItem::Spell {
                        card: Box::new(card_template.clone()),
                        caster: copy_controller,
                        target: tgt.clone(),
                        mode: md,
                        x_value: xv,
                        converged_value: cv,
                        uncounterable: false,
                        face: fc,
                        is_copy: true,
                    });
                }
                Ok(())
            }

            Effect::NameCreatureType { what } => {
                // Cavern of Souls "as it enters, choose a creature type".
                // The chooser is the source's controller. Suspend with a
                // `ChooseCreatureType` decision so a UI player can pick;
                // bots / AutoDecider resolve synchronously.
                use crate::decision::Decision;
                let candidate = self
                    .resolve_selector(what, ctx)
                    .into_iter()
                    .find_map(|e| match e {
                        EntityRef::Permanent(c) => Some(c),
                        _ => None,
                    });
                let Some(target_id) = candidate else { return Ok(()); };
                let decision = Decision::ChooseCreatureType { source: target_id };
                let pending =
                    PendingEffectState::ChooseCreatureTypePending { target_id };
                let chooser = ctx.controller;
                if self.players[chooser].wants_ui {
                    self.suspend_signal = Some((decision, pending, Effect::Noop));
                    return Ok(());
                }
                let answer = self.decider.decide(&decision);
                let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                events.append(&mut applied);
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
            Selector::LastCreatedToken => self
                .last_created_token
                .filter(|id| self.battlefield.iter().any(|c| c.id == *id))
                .map(EntityRef::Permanent)
                .into_iter()
                .collect(),
            Selector::CastSpellSource => {
                // Walk the stack top-down for the topmost StackItem::Spell.
                // SpellCast triggers are pushed above the cast spell, so
                // the topmost remaining Spell IS the just-cast spell.
                self.stack
                    .iter()
                    .rev()
                    .find_map(|si| match si {
                        StackItem::Spell { card, .. } => Some(EntityRef::Permanent(card.id)),
                        _ => None,
                    })
                    .into_iter()
                    .collect()
            }

            Selector::CastSpellTarget(slot) => {
                // Walk the stack for the spell whose SpellCast event fired
                // this trigger — that's the topmost matching `Spell` whose
                // card id matches `ctx.trigger_source` (set by
                // `fire_spell_cast_triggers`). Pull the target slot off it.
                let cast_id = match ctx.trigger_source {
                    Some(EntityRef::Card(cid)) | Some(EntityRef::Permanent(cid)) => Some(cid),
                    _ => None,
                };
                let Some(cid) = cast_id else { return vec![]; };
                let target = self.stack.iter().rev().find_map(|si| match si {
                    StackItem::Spell { card, target, .. } if card.id == cid => Some(target.clone()),
                    _ => None,
                });
                match target {
                    Some(Some(t)) if *slot == 0 => vec![target_to_entity(&t)],
                    _ => vec![],
                }
            }

            Selector::DiscardedThisResolution(filter) => {
                // Resolve to the cards discarded so far this resolution
                // whose now-graveyard residence matches the filter.
                // Mind Roots: "Put up to one land card discarded this
                // way onto the battlefield tapped". The card is in
                // the discarder's graveyard at this point, so we walk
                // each player's graveyard to locate the instance and
                // run the card-level filter on it.
                self.cards_discarded_this_resolution_ids
                    .iter()
                    .filter_map(|cid| {
                        let inst = self
                            .players
                            .iter()
                            .find_map(|p| p.graveyard.iter().find(|c| c.id == *cid))?;
                        if self.evaluate_requirement_on_card(filter, inst, ctx.controller) {
                            Some(EntityRef::Card(*cid))
                        } else {
                            None
                        }
                    })
                    .collect()
            }

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
                // Use the multi-player resolver so EachPlayer / EachOpponent
                // aggregate cards from every matching seat (Soul-Guide
                // Lantern's mass-graveyard exile, Bojuka Bog–style effects,
                // future Windfall-shape primitives all need this).
                let players = self.resolve_players(who, ctx);
                let mut out: Vec<EntityRef> = Vec::new();
                for p in players {
                    let cards: Vec<&CardInstance> = match zone {
                        Zone::Hand => self.players[p].hand.iter().collect(),
                        Zone::Graveyard => self.players[p].graveyard.iter().collect(),
                        Zone::Library => self.players[p].library.iter().collect(),
                        Zone::Exile => self.exile.iter().filter(|c| c.owner == p).collect(),
                        Zone::Battlefield => self.battlefield.iter().filter(|c| c.controller == p).collect(),
                        Zone::Stack | Zone::Command => vec![],
                    };
                    // For battlefield-resident cards we use the
                    // permanent-state-aware `evaluate_requirement_static`
                    // (Tapped, IsAttacking, etc. resolve correctly). For
                    // hand/library/graveyard/exile we use the card-level
                    // evaluator since those zones don't have permanent
                    // state — `evaluate_requirement_static` only walks
                    // battlefield-then-graveyard-then-exile-then-stack
                    // and would silently miss hand-resident cards
                    // (push XVI fix — was breaking Embrace the Paradox's
                    // MayDo land sub).
                    let on_bf = matches!(zone, Zone::Battlefield);
                    out.extend(
                        cards
                            .into_iter()
                            .filter(|c| if on_bf {
                                self.evaluate_requirement_static(
                                    filter, &Target::Permanent(c.id), ctx.controller,
                                )
                            } else {
                                self.evaluate_requirement_on_card(filter, c, ctx.controller)
                            })
                            .map(|c| if on_bf {
                                EntityRef::Permanent(c.id)
                            } else {
                                EntityRef::Card(c.id)
                            }),
                    );
                }
                out
            }

            Selector::Player(p) => self
                .resolve_players(p, ctx)
                .into_iter()
                .map(EntityRef::Player)
                .collect(),

            Selector::Take { inner, count } => {
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 {
                    return vec![];
                }
                let mut all = self.resolve_selector(inner, ctx);
                all.truncate(n);
                all
            }
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
                        self.find_card_owner(cid)
                    }
                    _ => None,
                }),
            PlayerRef::ControllerOf(sel) => self
                .resolve_selector(sel, ctx)
                .into_iter()
                .find_map(|e| match e {
                    EntityRef::Permanent(cid) => self
                        .battlefield_find(cid)
                        .map(|c| c.controller)
                        .or_else(|| self.find_card_owner(cid)),
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
            Value::LibrarySizeOf(p) => self.resolve_player(p, ctx).map(|p| self.players[p].library.len() as i32).unwrap_or(0),
            Value::XFromCost => ctx.x_value as i32,
            Value::StormCount => self.spells_cast_this_turn.saturating_sub(1) as i32,
            Value::CountersOn { what, kind } => self
                .resolve_selector(what, ctx)
                .into_iter()
                .find_map(|e| {
                    let cid = match e {
                        EntityRef::Permanent(cid) | EntityRef::Card(cid) => cid,
                        _ => return None,
                    };
                    // Battlefield first, then graveyards (counters
                    // persist on a card after move-to-graveyard, so a
                    // death trigger reading "counters on this dying
                    // creature" still sees them — Scolding
                    // Administrator's "if it had counters on it, put
                    // those counters on up to one target creature".
                    self.battlefield_find(cid)
                        .or_else(|| {
                            self.players
                                .iter()
                                .find_map(|p| p.graveyard.iter().find(|c| c.id == cid))
                        })
                        .map(|c| c.counter_count(*kind) as i32)
                })
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
                        // Walk the stack last so a SpellCast trigger's
                        // filter predicate can read the mana value of the
                        // spell that just went on the stack but hasn't
                        // resolved yet (Up the Beanstalk, Mind's Desire,
                        // etc.).
                        .or_else(|| self.stack.iter().find_map(|si| match si {
                            StackItem::Spell { card, .. } if card.id == cid => Some(&**card),
                            _ => None,
                        }))
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
            Value::CardsDrawnThisTurn(p) => self
                .resolve_player(p, ctx)
                .map(|p| self.players[p].cards_drawn_this_turn as i32)
                .unwrap_or(0),
            Value::Pow2(inner) => {
                let exp = self.evaluate_value(inner, ctx).clamp(0, 30);
                1i32.checked_shl(exp as u32).unwrap_or(i32::MAX)
            }
            Value::HalfDown(inner) => self.evaluate_value(inner, ctx) / 2,
            Value::PermanentCountControlledBy(p) => self
                .resolve_player(p, ctx)
                .map(|seat| {
                    self.battlefield
                        .iter()
                        .filter(|c| c.controller == seat)
                        .count() as i32
                })
                .unwrap_or(0),
            Value::CardsDiscardedThisResolution => self.cards_discarded_this_resolution as i32,
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
            Predicate::LifeGainedThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].life_gained_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::CardsLeftGraveyardThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].cards_left_graveyard_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::SpellsCastThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].spells_cast_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::CreaturesDiedThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].creatures_died_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::CardsExiledThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].cards_exiled_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::InstantsOrSorceriesCastThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].instants_or_sorceries_cast_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::CreaturesCastThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].creatures_cast_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::CastSpellTargetsMatch(filter) => {
                // Find the cast spell on the stack via the trigger source.
                // `fire_spell_cast_triggers` sets `ctx.trigger_source` to
                // `EntityRef::Card(cast_card_id)` so we can locate the
                // `StackItem::Spell` that just got pushed.
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                let target = self.stack.iter().find_map(|si| match si {
                    StackItem::Spell { card, target, .. } if card.id == cid => Some(target.clone()),
                    _ => None,
                });
                match target {
                    Some(Some(t)) => self.evaluate_requirement_static(filter, &t, ctx.controller),
                    _ => false,
                }
            }
            Predicate::CastSpellHasX => {
                // Locate the just-cast spell via the trigger source and
                // peek at its printed mana cost. Used by "whenever you
                // cast a spell with {X} in its cost" Quandrix triggers.
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                self.stack.iter().any(|si| match si {
                    StackItem::Spell { card, .. } if card.id == cid => {
                        card.definition.cost.has_x()
                    }
                    _ => false,
                })
            }
            Predicate::CastFromGraveyard => {
                matches!(ctx.cast_face, crate::game::types::CastFace::Flashback)
            }
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
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.controller == controller)
                    .or_else(|| {
                        // Stack-resident spells (counter / copy targets):
                        // controller is the caster.
                        self.stack.iter().find_map(|si| match si {
                            StackItem::Spell { card, caster, .. } if card.id == *cid => {
                                Some(*caster == controller)
                            }
                            _ => None,
                        })
                    })
                    .unwrap_or(false),
                Target::Player(p) => *p == controller,
            },
            R::ControlledByOpponent => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.controller != controller)
                    .or_else(|| {
                        self.stack.iter().find_map(|si| match si {
                            StackItem::Spell { card, caster, .. } if card.id == *cid => {
                                Some(*caster != controller)
                            }
                            _ => None,
                        })
                    })
                    .unwrap_or(false),
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
                    R::Multicolored => card.definition.cost.distinct_colors() >= 2,
                    R::Monocolored => card.definition.cost.distinct_colors() == 1,
                    R::Colorless => card.definition.cost.distinct_colors() == 0,
                    R::HasXInCost => card.definition.cost.has_x(),
                    R::HasName(n) => card.definition.name == n.as_ref(),
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
            R::Multicolored => card.definition.cost.distinct_colors() >= 2,
            R::Monocolored => card.definition.cost.distinct_colors() == 1,
            R::Colorless => card.definition.cost.distinct_colors() == 0,
            R::HasXInCost => card.definition.cost.has_x(),
            R::HasName(n) => card.definition.name == n.as_ref(),
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
    ///
    /// Targets must satisfy *both* the effect's selector requirement AND
    /// targeting legality (Hexproof / Shroud / Protection / player-side
    /// Leyline of Sanctity). Without the legality gate the random bot
    /// happily picks an opponent's Hexproof creature, the cast is
    /// rejected by `cast_spell`, and (in spectate mode) the match
    /// deadlocks — see `debug/deadlock-t10-1777412787-934831200.json`,
    /// where the bot kept aiming Bone Shards at Sylvan Caryatid.
    pub fn auto_target_for_effect(&self, eff: &Effect, controller: usize) -> Option<Target> {
        self.auto_target_for_effect_avoiding(eff, controller, None)
    }

    /// Source-aware auto-target picker. When `avoid_source` is set, the
    /// returned target prefers any *other* legal candidate to the avoided
    /// permanent — falling back to the source only if no other legal pick
    /// exists. Powers Strixhaven's Magecraft/Repartee triggers where the
    /// trigger source is rarely the right pick (a 1/1 utility creature
    /// shouldn't pump itself when a 5/5 attacker is on the board).
    pub fn auto_target_for_effect_avoiding(
        &self,
        eff: &Effect,
        controller: usize,
        avoid_source: Option<crate::card::CardId>,
    ) -> Option<Target> {
        let req = eff.primary_target_filter()?;
        let opp = (controller + 1) % self.players.len();
        let prefer_friendly = eff.prefers_friendly_target();
        // `prefers_graveyard_target` is the broader classifier — it covers
        // both reanimate (friendly graveyard) and graveyard hate (Ghost
        // Vacuum exiling target card from a graveyard). We walk graveyards
        // BEFORE the battlefield when this is set, so an `Any`-filtered
        // Move-to-Exile doesn't grab a battlefield permanent.
        let prefer_graveyard = eff.prefers_graveyard_target();
        // Skip Player candidates entirely when the effect operates on
        // permanents/stack — without this, an `Any`-filtered Move (Regrowth)
        // auto-targets the caster as a player and silently fizzles since
        // `Effect::Move` only consumes Permanent / Card entity refs.
        let accepts_player = eff.accepts_player_target();
        let primary_player = if prefer_friendly { controller } else { opp };
        let secondary_player = if prefer_friendly { opp } else { controller };

        // Combined check: requirement match + targetable by `controller`.
        let is_legal = |t: &Target| -> bool {
            self.evaluate_requirement_static(req, t, controller)
                && self.check_target_legality(t, controller).is_ok()
        };

        if accepts_player {
            let player_primary = Target::Player(primary_player);
            if is_legal(&player_primary) { return Some(player_primary); }
            let player_secondary = Target::Player(secondary_player);
            if is_legal(&player_secondary) { return Some(player_secondary); }
        }

        // Graveyard-target effects: walk primary player's graveyard first,
        // then secondary's. Reanimate/Disentomb (friendly) hits the caster's
        // graveyard; Ghost Vacuum (hostile) hits the opp's. Falls through
        // to the battlefield walk below if no graveyard match.
        if prefer_graveyard {
            for &p in &[primary_player, secondary_player] {
                if let Some(c) = self.players[p]
                    .graveyard
                    .iter()
                    .map(|c| Target::Permanent(c.id))
                    .find(|t| is_legal(t))
                {
                    return Some(c);
                }
            }
        }

        // Battlefield: walk preferred-controller permanents first, then
        // any matching permanent. Without the preference, the bot would
        // happily Vines its opponent's bear instead of its own.
        //
        // Source-avoidance pass (see `auto_target_for_effect_avoiding`'s
        // doc comment): when caller asked us to avoid the trigger source,
        // skip the source on the first pass and only fall back to it if
        // no other legal candidate exists.
        let is_avoided = |cid: crate::card::CardId| -> bool {
            avoid_source.map(|s| s == cid).unwrap_or(false)
        };
        // For friendly pumps (Magecraft / Repartee +1/+1 fan-out, transient
        // PumpPT spells), prefer the highest-power friendly creature so the
        // buff lands on the bot's biggest threat — improves expected value
        // versus the prior "first-in-Vec" pick (which was deterministic but
        // typically picked a 1-drop utility creature). For hostile picks the
        // current first-match heuristic still applies.
        let collect_legal_on_player = |p: usize| -> Vec<(crate::card::CardId, i32)> {
            self.battlefield
                .iter()
                .filter(|c| c.controller == p)
                .filter(|c| !is_avoided(c.id))
                .filter(|c| is_legal(&Target::Permanent(c.id)))
                .map(|c| {
                    let power = self
                        .computed_permanent(c.id)
                        .map(|cp| cp.power)
                        .unwrap_or(c.definition.power);
                    (c.id, power)
                })
                .collect()
        };
        let mut primary_candidates = collect_legal_on_player(primary_player);
        if prefer_friendly && !primary_candidates.is_empty() {
            // Sort by descending power so the strongest creature wins.
            primary_candidates.sort_by(|a, b| b.1.cmp(&a.1));
        }
        if let Some(&(cid, _)) = primary_candidates.first() {
            return Some(Target::Permanent(cid));
        }
        if let Some(t) = self
            .battlefield
            .iter()
            .filter(|c| !is_avoided(c.id))
            .map(|c| Target::Permanent(c.id))
            .find(|t| is_legal(t))
        {
            return Some(t);
        }
        // Source-fallback: only the avoided source is a legal candidate.
        // Pick it as a last resort so the trigger doesn't fizzle entirely.
        if let Some(t) = self
            .battlefield
            .iter()
            .filter(|c| c.controller == primary_player)
            .map(|c| Target::Permanent(c.id))
            .find(|t| is_legal(t))
        {
            return Some(t);
        }
        if let Some(t) = self
            .battlefield
            .iter()
            .map(|c| Target::Permanent(c.id))
            .find(|t| is_legal(t))
        {
            return Some(t);
        }
        // Final fallback: any graveyard, then exile. Reanimate-style spells
        // (Goryo's Vengeance, Animate Dead) hit this path when their target
        // was just lifted off the prefer-graveyard branch (e.g. their
        // controller's graveyard is empty). Hexproof and friends don't
        // apply to graveyard/exile targets, but we still funnel through
        // `is_legal` so any future zone-aware legality rules pick up
        // these zones too.
        for player in &self.players {
            if let Some(c) = player
                .graveyard
                .iter()
                .map(|c| Target::Permanent(c.id))
                .find(|t| is_legal(t))
            {
                return Some(c);
            }
        }
        if let Some(c) = self
            .exile
            .iter()
            .map(|c| Target::Permanent(c.id))
            .find(|t| is_legal(t))
        {
            return Some(c);
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
        // Then graveyards. Emit `CardLeftGraveyard` so Strixhaven
        // "cards leave your graveyard" payoffs (Garrison Excavator,
        // Living History, Spirit Mascot, Hardened Academic) trigger.
        for p in 0..self.players.len() {
            if let Some(pos) = self.players[p].graveyard.iter().position(|c| c.id == cid) {
                let card = self.players[p].graveyard.remove(pos);
                self.players[p].cards_left_graveyard_this_turn =
                    self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
                events.push(GameEvent::CardLeftGraveyard { player: p, card_id: cid });
                self.place_card_in_dest(card, p, &resolved_dest, events);
                return;
            }
        }
        // Then exile.
        if let Some(pos) = self.exile.iter().position(|c| c.id == cid) {
            let card = self.exile.remove(pos);
            let owner = card.owner;
            self.place_card_in_dest(card, owner, &resolved_dest, events);
            return;
        }
        // Hands. Used by start-of-game opening-hand effects
        // (Leyline of Sanctity, Gemstone Caverns) that move a hand card
        // to the battlefield.
        for p in 0..self.players.len() {
            if let Some(pos) = self.players[p].hand.iter().position(|c| c.id == cid) {
                let card = self.players[p].hand.remove(pos);
                self.place_card_in_dest(card, p, &resolved_dest, events);
                return;
            }
        }
        // Libraries. Used by `Selector::TopOfLibrary` → `ZoneDest::Exile`
        // / `Hand` / etc. (Suspend Aggression's exile-top-of-library half,
        // Daydream's exile-then-return flicker pattern in passing).
        for p in 0..self.players.len() {
            if let Some(pos) = self.players[p].library.iter().position(|c| c.id == cid) {
                let card = self.players[p].library.remove(pos);
                self.place_card_in_dest(card, p, &resolved_dest, events);
                return;
            }
        }
    }

    /// Pre-resolve any selector-based player refs in a `ZoneDest` against
    /// the active ctx. `place_card_in_dest` constructs its own bare ctx and
    /// can't see the caster's targets, so any `PlayerRef::OwnerOf(Selector)`
    /// / `ControllerOf(Selector)` need to be flattened to a concrete
    /// `PlayerRef::Seat(n)` while the source card is still in its origin
    /// zone. Other ref kinds (You / ActivePlayer / etc.) pass through.
    fn resolve_zonedest_player(&self, dest: &ZoneDest, ctx: &EffectContext) -> ZoneDest {
        // Flatten any context-relative `PlayerRef` (You, OwnerOf, ControllerOf,
        // EachOpponent, Triggerer) to a concrete `Seat(p)` based on the
        // resolution context. This is necessary so a follow-up
        // `place_card_in_dest` call — which builds its own dummy
        // `EffectContext(default_player)` — doesn't mis-resolve `You` to
        // the wrong seat. Mind Roots's "discard from opp → land to *your*
        // bf" hit this when the card was located in the opp's graveyard
        // (default_player=opp) and `You` stayed unresolved.
        let flatten = |who: &PlayerRef| -> PlayerRef {
            match who {
                PlayerRef::Seat(_) => who.clone(),
                _ => {
                    if let Some(p) = self.resolve_player(who, ctx) {
                        PlayerRef::Seat(p)
                    } else {
                        who.clone()
                    }
                }
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
                    LibraryPosition::Bottom => self.players[p].library.push(card),
                    LibraryPosition::Shuffled => {
                        // Push the card in, then shuffle the entire library
                        // so the card lands at a random position (Chaos Warp,
                        // bottom-of-library reanimate-prevention effects, etc.).
                        // Pre-fix this fell through to `push` (effectively
                        // sending to bottom), which exposed deterministic
                        // ordering across cards that semantically should
                        // randomize.
                        use rand::seq::SliceRandom;
                        let mut rng = rand::rng();
                        self.players[p].library.push(card);
                        self.players[p].library.shuffle(&mut rng);
                    }
                }
            }
            ZoneDest::Graveyard => {
                let owner = card.owner;
                self.players[owner].send_to_graveyard(card);
            }
            ZoneDest::Exile => {
                let cid = card.id;
                self.exile.push(card);
                // Bump the controller-of-the-exile-effect's per-turn
                // exile tally for Strixhaven "if one or more cards were
                // put into exile this turn" payoffs (Ennis the Debate
                // Moderator). Reset on `do_untap`.
                if default_player < self.players.len() {
                    self.players[default_player].cards_exiled_this_turn =
                        self.players[default_player].cards_exiled_this_turn.saturating_add(1);
                }
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
        // CardDefinition.name is &'static str; tokens carry an owned
        // String (so they round-trip through serde), so we leak a copy
        // here to extend its lifetime. The leak is bounded by the
        // number of unique token names produced over a session.
        name: crate::static_str_serde::intern(token.name.clone()),
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
        activated_abilities: token.activated_abilities.clone(),
        triggered_abilities: token.triggered_abilities.clone(),
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
    state: &GameState,
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
        (EventKind::CardLeftGraveyard, GameEvent::CardLeftGraveyard { .. }) => true,
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
        ) || matches!(
            event,
            GameEvent::CounterAdded { card_id, .. } if *card_id == source.id
        ),
        EventScope::YourControl => event_actor(state, event)
            .is_some_and(|p| p == source.controller),
        EventScope::OpponentControl => event_actor(state, event)
            .is_some_and(|p| p != source.controller),
        EventScope::AnyPlayer | EventScope::ActivePlayer => true,
        EventScope::AnotherOfYours => {
            // ETB/die triggers for "another creature"
            let target = event_card(event);
            target != Some(source.id)
        }
        EventScope::FromYourGraveyard => event_actor(state, event)
            .is_some_and(|p| p == source.owner),
    };

    if !scope_ok {
        return false;
    }

    // Filter predicate evaluation is deferred to when the trigger actually
    // resolves; at this stage we just ensure the shape matches.
    true
}

/// The "actor" of an event for `EventScope::YourControl` /
/// `OpponentControl` checks: the player whose action / permanent the
/// event hangs off. For player-keyed events (CardDrawn, LifeGained, etc.)
/// this is the event's `player` field; for permanent-keyed events
/// (PermanentEntered, AttackerDeclared, CreatureDied) this is the
/// permanent's controller, looked up on the battlefield. CreatureDied
/// fires after the card has left the battlefield, so we fall back to the
/// graveyard owner — close enough for "your creature died" triggers.
pub(crate) fn event_actor(state: &GameState, event: &GameEvent) -> Option<usize> {
    if let Some(p) = event_player(event) {
        return Some(p);
    }
    let cid = event_card(event)?;
    if let Some(c) = state.battlefield_find(cid) {
        return Some(c.controller);
    }
    state
        .players
        .iter()
        .position(|p| p.graveyard.iter().any(|c| c.id == cid))
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
        | GameEvent::CardLeftGraveyard { player, .. }
        | GameEvent::TurnStarted { player, .. } => Some(*player),
        _ => None,
    }
}

/// Extract the "subject" of an event as an `EntityRef` — the entity the
/// trigger's filter predicate should treat as `Selector::TriggerSource`.
/// For card-subject events (cast spell, ETB permanent, attacker, etc.)
/// this is the card; for player-subject events (life gain/loss, draw,
/// discard) it's the player. Used by `dispatch_triggers_for_events` so
/// filters like `Predicate::ValueAtLeast(ManaValueOf(TriggerSource), 5)`
/// can pin-point the cast spell on the stack.
pub(crate) fn event_subject(event: &GameEvent) -> Option<EntityRef> {
    match event {
        GameEvent::SpellCast { card_id, .. } => Some(EntityRef::Card(*card_id)),
        GameEvent::PermanentEntered { card_id } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::CreatureDied { card_id } => Some(EntityRef::Card(*card_id)),
        GameEvent::AttackerDeclared(card_id) => Some(EntityRef::Permanent(*card_id)),
        GameEvent::BlockerDeclared { blocker, .. } => Some(EntityRef::Permanent(*blocker)),
        GameEvent::LandPlayed { card_id, .. } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::PermanentTapped { card_id } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::PermanentUntapped { card_id } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::TokenCreated { card_id } => Some(EntityRef::Permanent(*card_id)),
        GameEvent::CardDrawn { player, .. }
        | GameEvent::CardDiscarded { player, .. }
        | GameEvent::CardMilled { player, .. }
        | GameEvent::LifeGained { player, .. }
        | GameEvent::LifeLost { player, .. }
        | GameEvent::ManaAdded { player, .. }
        | GameEvent::ColorlessManaAdded { player } => Some(EntityRef::Player(*player)),
        GameEvent::CardLeftGraveyard { card_id, .. } => Some(EntityRef::Card(*card_id)),
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
        | GameEvent::CounterAdded { card_id, .. }
        | GameEvent::AttackerDeclared(card_id) => Some(*card_id),
        GameEvent::BlockerDeclared { blocker, .. } => Some(*blocker),
        _ => None,
    }
}

// ── Built-in token definitions ───────────────────────────────────────────────

#[allow(dead_code)]
pub fn food_token() -> TokenDefinition {
    TokenDefinition {
        name: "Food".into(),
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
        // {2}, {T}, Sacrifice this artifact: Gain 3 life.
        activated_abilities: vec![crate::card::ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost {
                symbols: vec![ManaSymbol::Generic(2)],
            },
            effect: Effect::GainLife {
                who: crate::card::Selector::You,
                amount: crate::card::Value::Const(3),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
        }],
        triggered_abilities: vec![],
    }
}

#[allow(dead_code)]
pub fn treasure_token() -> TokenDefinition {
    TokenDefinition {
        name: "Treasure".into(),
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
        // {T}, Sacrifice this artifact: Add one mana of any color.
        activated_abilities: vec![crate::card::ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: crate::effect::PlayerRef::You,
                pool: crate::effect::ManaPayload::AnyOneColor(crate::card::Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
        }],
        triggered_abilities: vec![],
    }
}

#[allow(dead_code)]
pub fn blood_token() -> TokenDefinition {
    TokenDefinition {
        name: "Blood".into(),
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
        // {1}, {T}, Discard a card, Sacrifice this artifact: Draw a card.
        // The discard piece isn't a discard-as-cost (no primitive yet) so we
        // fold it into the resolution sequence. AutoDecider picks the first
        // hand card to discard.
        activated_abilities: vec![crate::card::ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost {
                symbols: vec![ManaSymbol::Generic(1)],
            },
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: crate::card::Selector::You,
                    amount: crate::card::Value::Const(1),
                    random: false,
                },
                Effect::Draw {
                    who: crate::card::Selector::You,
                    amount: crate::card::Value::Const(1),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
        }],
        triggered_abilities: vec![],
    }
}

#[allow(dead_code)]
pub fn clue_token() -> TokenDefinition {
    TokenDefinition {
        name: "Clue".into(),
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
        // {2}, Sacrifice this artifact: Draw a card.
        activated_abilities: vec![crate::card::ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost {
                symbols: vec![ManaSymbol::Generic(2)],
            },
            effect: Effect::Draw {
                who: crate::card::Selector::You,
                amount: crate::card::Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
        }],
        triggered_abilities: vec![],
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
