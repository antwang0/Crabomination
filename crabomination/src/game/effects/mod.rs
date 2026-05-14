//! Resolver for the unified `Effect` tree.
//!
//! A single entry point — [`GameState::resolve_effect`] — walks the effect
//! tree against an [`EffectContext`] describing the casting/activating player,
//! chosen target(s), etc. Combinators (`Seq`, `If`, `ForEach`, `Repeat`,
//! `ChooseMode`) recurse; leaf mutations perform game-state changes and emit
//! [`GameEvent`]s.

mod delayed;
mod eval;
mod events;
mod movement;
mod targeting;
mod tokens;

pub use tokens::{blood_token, clue_token, food_token, token_to_card_definition, treasure_token};
pub(crate) use delayed::delayed_kind_from_effect;
pub(crate) use events::{event_matches_spec, event_subject};

use super::*;
use crate::card::{
    CardId, CardInstance, CounterType,
    Keyword, SelectionRequirement, Zone,
};
use crate::effect::{
    Effect, ManaPayload, PlayerRef,
    Selector, ZoneDest, ZoneRef,
};
use crate::mana::Color;

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
    /// Total mana spent paying the originating spell's cost
    /// (Increment / Opus payoffs). Computed at cast time and
    /// threaded through `StackItem::Spell` / `StackItem::Trigger`.
    pub mana_spent: u32,
    /// The resolving spell's printed name. Stamped by
    /// `for_spell_with_source` so predicates that need to introspect the
    /// spell's name (e.g. `Predicate::SameNamedInZoneAtLeast` for
    /// Dragon's Approach's "same name in your graveyard" rider) can
    /// read it directly — the card itself is in transient ownership
    /// during spell resolution and isn't present in any visible zone,
    /// so a zone-walking lookup wouldn't find it.
    pub source_name: Option<&'static str>,
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
            mana_spent: 0,
            source_name: None,
        }
    }
    /// Spell-resolution context with the resolving spell's
    /// `CardId` + printed name stamped onto `ctx.source` /
    /// `ctx.source_name` and all cast-time scalars (X, Converge, total
    /// mana spent) threaded in. Lets predicates that introspect the
    /// resolving spell (e.g. `Predicate::SameNamedInZoneAtLeast` for
    /// Dragon's Approach's "same name in your graveyard" rider) read
    /// the spell's identity without needing to find the card in any
    /// game zone — during spell resolution the card is in transient
    /// ownership (popped from the stack, not yet placed in graveyard),
    /// so a zone-walking lookup would fail.
    #[allow(clippy::too_many_arguments)]
    pub fn for_spell_with_source(
        spell_card: CardId,
        spell_name: &'static str,
        caster: usize,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: usize,
        x_value: u32,
        converged_value: u32,
        mana_spent: u32,
    ) -> Self {
        // Merge slot 0 (`target`) + slots 1+ (`additional_targets`) into
        // a single `targets` Vec so `Selector::Target(n)` reads slot n
        // for any n. Single-target spells leave `additional_targets`
        // empty.
        let mut targets: Vec<Target> = target.into_iter().collect();
        targets.extend(additional_targets);
        Self {
            controller: caster,
            source: Some(spell_card),
            targets,
            trigger_source: None,
            mode,
            x_value,
            converged_value,
            mana_spent,
            source_name: Some(spell_name),
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
            mana_spent: 0,
            source_name: None,
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
            mana_spent: 0,
            source_name: None,
        }
    }
}

/// A resolved reference to something in the game (used internally for selector
/// resolution and `ForEach` iteration).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EntityRef {
    Player(usize),
    Permanent(CardId),
    /// A card in a non-battlefield zone (library/graveyard/exile/hand).
    Card(CardId),
}

impl EntityRef {
    /// Extract the `CardId` if this entity is a battlefield permanent.
    pub fn as_permanent_id(&self) -> Option<CardId> {
        match *self {
            EntityRef::Permanent(c) => Some(c),
            _ => None,
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
        // Reset cards-discarded scratch — `Value::CardsDiscardedThisEffect`
        // only counts discards from *this* resolution (Borrowed Knowledge
        // mode 1's "draw cards equal to the number discarded this way").
        self.cards_discarded_this_resolution = 0;
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

            Effect::ChooseN { picks, modes } => {
                // Run each picked mode in `picks` order. Out-of-range
                // indices are silently skipped (defensive; the card
                // factories build the list explicitly).
                for &i in picks {
                    if let Some(m) = modes.get(i as usize) {
                        self.run_effect(m, ctx, events)?;
                    }
                }
                Ok(())
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

            Effect::IfRevealFromHand { filter, then, else_ } => {
                // Peek at the controller's hand for a card matching `filter`.
                // If any match exists, run `then` (the implicit "yes, I
                // reveal" branch — AutoDecider always accepts since the
                // alternative is the printed downside, e.g. enters-tapped
                // for STX Snarl lands). If no match, run `else_`. The
                // reveal itself is information-only and isn't modeled as
                // a separate game event today; a future enhancement could
                // emit `GameEvent::CardRevealed { player, card_id }` for
                // replay/log purposes.
                let has_match = self.players[ctx.controller]
                    .hand
                    .iter()
                    .any(|c| self.evaluate_requirement_on_card(filter, c, ctx.controller));
                if has_match {
                    self.run_effect(then, ctx, events)?;
                } else {
                    self.run_effect(else_, ctx, events)?;
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
                    // CR 702.90b — pass the source so infect routes
                    // player-target damage to poison counters.
                    self.deal_damage_to_from(ent, amt, ctx.source, events);
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
                            self.cards_discarded_this_resolution += 1;
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
                // CR 701.22b: "If a player is instructed to scry 0, no
                // scry event occurs. Abilities that trigger whenever a
                // player scries won't trigger." (Same wording covers
                // Surveil 0 via 701.42 by reference.) An instruction
                // of `n == 0` short-circuits at the top so no decision
                // / event flows downstream.
                if n == 0 {
                    return Ok(());
                }
                let peek: Vec<(CardId, String)> = self.players[p]
                    .library
                    .iter()
                    .take(n)
                    .map(|c| (c.id, c.definition.name.to_string()))
                    .collect();
                let actual = peek.len();
                // Per CR 701.22a, if the library has fewer cards than
                // requested, the player looks at and may rearrange the
                // available cards — the scry instruction still
                // executes. Only return early when there are literally
                // no cards to peek at (e.g. empty library + Scry N>0
                // is still a vacuous-but-real scry; we model that by
                // proceeding to emit the bookkeeping path). For
                // simplicity if `actual == 0` we still skip the
                // decision (no cards to reorder) but acknowledge the
                // event happened; future scry-counting payoffs would
                // need an explicit event emission here.
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
                    if let Some(cid) = ent.as_permanent_id() {
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
                    if let Some(cid) = ent.as_permanent_id()
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
                    if let Some(cid) = ent.as_permanent_id()
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
                    if let Some(cid) = ent.as_permanent_id()
                        && let Some(c) = self.battlefield_find_mut(cid) {
                            c.power_bonus += p;
                            c.toughness_bonus += t;
                            events.push(GameEvent::PumpApplied { card_id: cid, power: p, toughness: t });
                        }
                }
                Ok(())
            }

            Effect::SetBasePT { what, power, toughness, duration } => {
                // Layer-7b SetPT continuous effect — installs a real
                // `Modification::SetPowerToughness(p, t)` against the
                // resolved target permanent, with the given duration
                // mapped onto `EffectDuration`. The layer system
                // applies the set *before* counters / +N/+M bonuses
                // (CR 613.7b vs c/f), so a +1/+1 counter on top of
                // Square Up's 0/4 yields 1/5 — matching the printed
                // rules exactly.
                use crate::game::layers::{
                    AffectedPermanents, ContinuousEffect, EffectDuration, Layer, Modification,
                    PtSublayer,
                };
                let p = self.evaluate_value(power, ctx);
                let t = self.evaluate_value(toughness, ctx);
                let duration_kind = match duration {
                    crate::effect::Duration::EndOfTurn
                    | crate::effect::Duration::EndOfCombat => EffectDuration::UntilEndOfTurn,
                    crate::effect::Duration::UntilNextTurn
                    | crate::effect::Duration::UntilYourNextUntap => {
                        EffectDuration::UntilNextTurn
                    }
                    crate::effect::Duration::Permanent => EffectDuration::Indefinite,
                };
                let source = ctx.source.unwrap_or(CardId(0));
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id() {
                        let ts = self.next_timestamp();
                        self.add_continuous_effect(ContinuousEffect {
                            timestamp: ts,
                            source,
                            affected: AffectedPermanents::Specific(vec![cid]),
                            layer: Layer::L7PowerTough,
                            sublayer: Some(PtSublayer::SetValue),
                            duration: duration_kind.clone(),
                            modification: Modification::SetPowerToughness(p, t),
                        });
                        events.push(GameEvent::PumpApplied {
                            card_id: cid,
                            power: p,
                            toughness: t,
                        });
                    }
                }
                Ok(())
            }

            Effect::GrantKeyword { what, keyword, duration: _ } => {
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id()
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
                    if let Some(cid) = ent.as_permanent_id()
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
                    if let Some(cid) = ent.as_permanent_id()
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
                    if let Some(cid) = t.as_permanent_id()
                        && let Some(pos) = self.stack.iter().position(|si| matches!(
                            si,
                            StackItem::Spell { card, uncounterable: false, .. }
                                if card.id == cid
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

            Effect::CounterSpellToZone { what, zone } => {
                // Counter target spell and route the lifted card to a
                // non-graveyard zone. Overrides CR 701.6a's default
                // (countered spell -> owner's graveyard) via the spell's
                // printed "instead" clause (CR 608.2c — later text on a
                // card may modify earlier text). Memory Lapse routes to
                // top of owner's library; Spell Crumple to exile; Remand
                // to owner's hand. Spells flagged `uncounterable` (Cavern
                // of Souls) are skipped — the counter does nothing.
                use crate::effect::CounteredSpellZone;
                let targets = self.resolve_selector(what, ctx);
                let mut to_remove: Vec<usize> = Vec::new();
                for t in &targets {
                    if let Some(cid) = t.as_permanent_id()
                        && let Some(pos) = self.stack.iter().position(|si| matches!(
                            si,
                            StackItem::Spell { card, uncounterable: false, .. }
                                if card.id == cid
                        ))
                    {
                        to_remove.push(pos);
                    }
                }
                to_remove.sort_unstable_by(|a, b| b.cmp(a));
                for pos in to_remove {
                    if let StackItem::Spell { card, .. } = self.stack.remove(pos) {
                        let owner = card.owner;
                        match zone {
                            CounteredSpellZone::OwnerLibraryTop => {
                                self.players[owner].library.push(*card);
                            }
                            CounteredSpellZone::OwnerLibraryBottom => {
                                self.players[owner].library.insert(0, *card);
                            }
                            CounteredSpellZone::OwnerHand => {
                                self.players[owner].hand.push(*card);
                            }
                            CounteredSpellZone::Exile => {
                                self.exile.push(*card);
                            }
                        }
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
                    && let StackItem::Spell { card, caster, .. } = self.stack.remove(pos)
                {
                    self.players[caster].send_to_graveyard(*card);
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
                    if let Some(cid) = t.as_permanent_id() {
                        // Walk top-down so we counter the most recent
                        // matching trigger (the one the player most likely
                        // intends to cancel).
                        if let Some(pos) = self
                            .stack
                            .iter()
                            .enumerate()
                            .rev()
                            .find_map(|(i, si)| match si {
                                StackItem::Trigger { source, .. } if *source == cid => Some(i),
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
                // The source permanent (if any) — Daemogoth Titan-style
                // "When this attacks, sacrifice another creature" triggers
                // bind `ctx.source` to themselves. Prefer NOT picking the
                // source when other legal candidates exist, so the printed
                // "another" intent is honored even though the filter
                // doesn't carry the source exclusion explicitly.
                let source_id = ctx.source;
                for ent in self.resolve_selector(who, ctx) {
                    let EntityRef::Player(p) = ent else { continue; };
                    // Prioritize sacrifice picks: non-source first
                    // (deprioritizes self-sacrifice for "another creature"
                    // semantics), then tokens (free), then by lowest mana
                    // value, then by lowest power. Simple AutoDecider
                    // heuristic — when forced to sacrifice, dump the
                    // cheapest/weakest non-source candidate.
                    let mut candidates: Vec<&CardInstance> = self
                        .battlefield
                        .iter()
                        .filter(|c| c.controller == p)
                        .filter(|c| {
                            let t = Target::Permanent(c.id);
                            self.evaluate_requirement_static(filter, &t, p, ctx.source)
                        })
                        .collect();
                    candidates.sort_by_key(|c| {
                        (
                            // Source last (true sorts after false), so any
                            // non-source candidate is picked first.
                            Some(c.id) == source_id,
                            !c.is_token, // false (=0) sorts before true → tokens first
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
                let anchor = self.resolve_selector(to, ctx)
                    .into_iter()
                    .find_map(|e| e.as_permanent_id());
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id()
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
                //
                // CR 121.5 compliance — "If an effect moves cards from a
                // player's library to that player's hand without using
                // the word 'draw,' the player has not drawn those cards.
                // This makes a difference for abilities that trigger on
                // drawing and effects that count cards drawn." Goblin
                // Guide and its kin say "puts it into their hand", not
                // "draws"; so we do NOT emit a `CardDrawn` event here
                // and do NOT increment `cards_drawn_this_turn`. A
                // dedicated `CardPutIntoHand` event would let cards
                // listen to *all* library→hand moves, but no current
                // card needs that; if/when one lands, add the event
                // here in front of the silent move.
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
                        self.players[p].hand.push(card);
                        // Intentionally no CardDrawn event (CR 121.5).
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
                            && self.evaluate_requirement_static(filter, &Target::Permanent(c.id), p, ctx.source)
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
                miss_dest,
            } => {
                // Walk the top of `who`'s library until we either find a
                // matching card or hit the cap. Route misses according to
                // `miss_dest` (graveyard by default; bottom-of-library
                // for SOS Strixhaven "rest on the bottom in a random
                // order" cards).
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
                // everything if no match) into the configured miss zone.
                let miss_count = found_idx.unwrap_or(revealed);
                for _ in 0..miss_count {
                    if self.players[p].library.is_empty() {
                        break;
                    }
                    let card = self.players[p].library.remove(0);
                    let cid = card.id;
                    match miss_dest {
                        crate::effect::RevealMissDest::Graveyard => {
                            self.players[p].graveyard.push(card);
                            events.push(GameEvent::CardMilled { player: p, card_id: cid });
                        }
                        crate::effect::RevealMissDest::BottomRandom => {
                            // No RNG hook in the engine yet — push to
                            // the back of the library deterministically.
                            // From a gameplay standpoint this is
                            // indistinguishable from a "random bottom"
                            // since no card knows the bottom ordering
                            // before the next shuffle / reveal.
                            self.players[p].library.push(card);
                        }
                    }
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
                // Resolve which spell to copy. We support two main patterns:
                // 1. `Selector::TriggerSource` — the spell that fired this
                //    trigger (Magecraft / Storm / Aziza-style "whenever you
                //    cast X, copy that spell"). The trigger_source carries
                //    the cast spell's CardId.
                // 2. `Selector::Target(n)` / `CastSpellTarget(n)` — a
                //    specifically-targeted spell on the stack
                //    (Reverberate / Twincast / Choreographed Sparks).
                //
                // For each, we locate the matching `StackItem::Spell` and
                // clone it `count` times with a fresh CardId per copy. The
                // copies inherit the original's target / mode / x_value /
                // converged_value. Auto-retargeting for friendly-fire
                // (choose new targets for the copy) is left to the
                // existing auto_target_for_effect picker, which runs at
                // resolution time. The original spell still resolves
                // afterward — copies always end up above it on the stack.
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 {
                    return Ok(());
                }
                // Find candidate stack indices to copy.
                let candidate_ids: Vec<CardId> = match what {
                    Selector::TriggerSource => ctx
                        .trigger_source
                        .into_iter()
                        .filter_map(|e| match e {
                            EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                            _ => None,
                        })
                        .collect(),
                    Selector::This => ctx.source.into_iter().collect(),
                    _ => self
                        .resolve_selector(what, ctx)
                        .into_iter()
                        .filter_map(|e| match e {
                            EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                            _ => None,
                        })
                        .collect(),
                };
                for cid in candidate_ids {
                    // Locate the matching Spell on the stack (topmost wins).
                    let stack_idx = self.stack.iter().rposition(|s| {
                        matches!(s, crate::game::types::StackItem::Spell { card, .. }
                            if card.id == cid)
                    });
                    let Some(idx) = stack_idx else { continue; };
                    // Skip if the spell is flagged uncounterable AND we are
                    // copying via an effect that says "this spell can't be
                    // copied". Today we don't carry a per-spell "can't be
                    // copied" flag; this branch is left for future
                    // refinement.
                    // Snapshot the spell, then push `n` copies above it.
                    let (orig_card_def, caster, target, additional_targets, mode, x_value, converged_value)
                        = if let crate::game::types::StackItem::Spell {
                            card, caster, target, additional_targets, mode, x_value, converged_value, ..
                        } = &self.stack[idx] {
                            (
                                card.definition.clone(),
                                *caster,
                                target.clone(),
                                additional_targets.clone(),
                                *mode,
                                *x_value,
                                *converged_value,
                            )
                        } else {
                            continue;
                        };
                    for _ in 0..n {
                        let new_id = self.next_id();
                        let mut copy_inst =
                            crate::card::CardInstance::new(new_id, orig_card_def.clone(), caster);
                        // Mark as token so CR 707.10a applies: a copy of a
                        // spell ceases to exist in any zone other than the
                        // stack. Token-cleanup SBAs handle the removal when
                        // the resolved copy lands in graveyard / hand /
                        // library after resolution.
                        copy_inst.is_token = true;
                        self.stack.push(crate::game::types::StackItem::Spell {
                            card: Box::new(copy_inst),
                            caster,
                            target: target.clone(),
                            additional_targets: additional_targets.clone(),
                            mode,
                            x_value,
                            converged_value,
                            mana_spent: 0,
                            uncounterable: true, // copies can't be countered
                        });
                    }
                    events.push(GameEvent::SpellsCopied {
                        original: cid,
                        count: n as u32,
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

            Selector::EachMatching { zone, filter } => self.entities_in_zone(zone, filter, ctx),
            Selector::EachPermanent(filter) => self
                .battlefield
                .iter()
                .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), ctx.controller, ctx.source))
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
                    .filter_map(|e| e.as_permanent_id())
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
                                    filter, &Target::Permanent(c.id), ctx.controller, ctx.source,
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
                .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), ctx.controller, ctx.source))
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
                // Multi-player aware: an `EachPlayer` / `EachOpponent` ref
                // expands to every matching player's zone, not just the
                // first. Previously this resolved to a single player only,
                // silently dropping the remaining graveyards / hands.
                // Powers Devious Cover-Up's "exile any number of target
                // cards from graveyards" (across all gys), Necrogenesis's
                // total-creature-count payoff, and any other cross-player
                // zone iteration.
                let players = self.resolve_players(who, ctx);
                let mut out = Vec::new();
                for p in players {
                    let cards: Vec<&CardInstance> = match zone {
                        ZoneRef::Library(_) => self.players[p].library.iter().collect(),
                        ZoneRef::Hand(_) => self.players[p].hand.iter().collect(),
                        ZoneRef::Graveyard(_) => self.players[p].graveyard.iter().collect(),
                        _ => vec![],
                    };
                    out.extend(
                        cards
                            .into_iter()
                            .filter(|c| {
                                self.evaluate_requirement_static(
                                    filter,
                                    &Target::Permanent(c.id),
                                    ctx.controller,
                                    ctx.source,
                                )
                            })
                            .map(|c| EntityRef::Card(c.id)),
                    );
                }
                out
            }
            ZoneRef::Exile => self
                .exile
                .iter()
                .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), ctx.controller, ctx.source))
                .map(|c| EntityRef::Card(c.id))
                .collect(),
            ZoneRef::Command => vec![],
        }
    }

}

fn target_to_entity(t: &Target) -> EntityRef {
    match t {
        Target::Player(p) => EntityRef::Player(*p),
        Target::Permanent(c) => EntityRef::Permanent(*c),
    }
}

