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
use crate::game::layers::EffectDuration;
use crate::mana::Color;

/// Translate the cast-site `effect::Duration` into the runtime
/// `layers::EffectDuration` used by the continuous-effect layer system.
///
/// CR 511.2: `Duration::EndOfCombat` maps to the dedicated
/// `UntilEndOfCombat` variant (cleared in `do_combat_end`) so that
/// "until end of combat" effects don't linger into the post-combat
/// main phase.
pub(crate) fn map_effect_duration(
    duration: crate::effect::Duration,
) -> EffectDuration {
    match duration {
        crate::effect::Duration::EndOfTurn => EffectDuration::UntilEndOfTurn,
        crate::effect::Duration::EndOfCombat => EffectDuration::UntilEndOfCombat,
        crate::effect::Duration::UntilNextTurn
        | crate::effect::Duration::UntilYourNextUntap => {
            EffectDuration::UntilNextTurn
        }
        crate::effect::Duration::Permanent => EffectDuration::Indefinite,
    }
}

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
    /// True if the resolving spell was cast from its caster's hand.
    /// False for flashback / cast-from-graveyard / cast-from-exile
    /// paths. Stamped by `for_spell_with_source` from the resolving
    /// `CardInstance.cast_from_hand` flag, read by
    /// `Predicate::CastFromGraveyard` (Increasing Vengeance "if cast
    /// from graveyard, copy twice instead"). Defaults to `true` for
    /// non-spell contexts (triggers, activated abilities) since those
    /// don't have a "cast zone" concept.
    pub cast_from_hand: bool,
    /// Per-event amount of the firing event (life gained, life lost,
    /// damage dealt, cards drawn, …). Set on trigger resolutions from
    /// the event payload (`StackItem::Trigger.event_amount`) so trigger
    /// bodies can read it via `Value::TriggerEventAmount`. Used by
    /// Light of Promise's "Whenever you gain life, put that many
    /// +1/+1 counters on target creature you control." Defaults to 0
    /// for non-trigger contexts (spells, activated abilities).
    pub event_amount: u32,
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
            cast_from_hand: true,
            event_amount: 0,
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
        Self::for_spell_with_source_and_origin(
            spell_card,
            spell_name,
            caster,
            target,
            additional_targets,
            mode,
            x_value,
            converged_value,
            mana_spent,
            true,
        )
    }

    /// Variant of `for_spell_with_source` that also stamps the
    /// `cast_from_hand` flag so the resolution-time predicate
    /// `Predicate::CastFromGraveyard` (Increasing Vengeance) can read
    /// whether the spell came from hand vs graveyard / flashback. The
    /// no-origin sibling defaults to `cast_from_hand = true` since the
    /// vast majority of spell resolutions are hand casts.
    #[allow(clippy::too_many_arguments)]
    pub fn for_spell_with_source_and_origin(
        spell_card: CardId,
        spell_name: &'static str,
        caster: usize,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: usize,
        x_value: u32,
        converged_value: u32,
        mana_spent: u32,
        cast_from_hand: bool,
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
            cast_from_hand,
            event_amount: 0,
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
            cast_from_hand: true,
            event_amount: 0,
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
            cast_from_hand: true,
            event_amount: 0,
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
        // Reset sacrificed-power / sacrificed-toughness scratch for this
        // independent resolution.
        self.sacrificed_power = None;
        self.sacrificed_toughness = None;
        // Reset last-created-token scratch — `Selector::LastCreatedToken`
        // (singular) and `Selector::LastCreatedTokens` (plural) only refer
        // to tokens created by *this* resolution.
        self.last_created_token = None;
        self.last_created_tokens.clear();
        // Reset last-moved-cards scratch — `Selector::LastMoved` only
        // refers to cards moved by *this* resolution (Practiced
        // Scrollsmith's ETB chains Move → GrantMayPlay on the same
        // moved card via this scratch).
        self.last_moved_cards.clear();
        // Reset cards-discarded scratch — `Value::CardsDiscardedThisEffect`
        // only counts discards from *this* resolution (Borrowed Knowledge
        // mode 1's "draw cards equal to the number discarded this way").
        self.cards_discarded_this_resolution = 0;
        self.creature_cards_discarded_this_resolution = 0;
        self.discarded_card_ids_this_resolution.clear();
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

            // CR 705 — Flip a coin `count` times; for each flip ask the
            // controller's decider for heads/tails and dispatch to
            // `on_heads` / `on_tails`. AutoDecider always picks heads;
            // ScriptedDecider can override per-flip via DecisionAnswer::
            // Bool(false) for tails.
            Effect::FlipCoin { count, on_heads, on_tails } => {
                let n = self.evaluate_value(count, ctx).max(0);
                for _ in 0..n {
                    let answer = self.decider.decide(&crate::decision::Decision::CoinFlip {
                        player: ctx.controller,
                    });
                    let heads = matches!(answer, crate::decision::DecisionAnswer::Bool(true));
                    if heads {
                        self.run_effect(on_heads, ctx, events)?;
                    } else {
                        self.run_effect(on_tails, ctx, events)?;
                    }
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
                        self.adjust_life(p, amt as i32);
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
                        self.adjust_life(p, -(amt as i32));
                        events.push(GameEvent::LifeLost { player: p, amount: amt });
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::SetLifeTotal { who, amount } => {
                let new_total = self.evaluate_value(amount, ctx);
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        let delta = new_total - self.effective_life(p);
                        self.set_life(p, new_total);
                        if delta > 0 {
                            let amt = delta as u32;
                            self.players[p].life_gained_this_turn =
                                self.players[p].life_gained_this_turn.saturating_add(amt);
                            events.push(GameEvent::LifeGained { player: p, amount: amt });
                        } else if delta < 0 {
                            let amt = (-delta) as u32;
                            events.push(GameEvent::LifeLost { player: p, amount: amt });
                        }
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
                        self.adjust_life(p, -(amt as i32));
                        events.push(GameEvent::LifeLost { player: p, amount: amt });
                    }
                }
                for ent in self.resolve_selector(to, ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.adjust_life(p, amt as i32);
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
                            let was_creature = card
                                .definition
                                .card_types
                                .contains(&crate::card::CardType::Creature);
                            self.players[p].graveyard.push(card);
                            events.push(GameEvent::CardDiscarded { player: p, card_id: cid });
                            self.cards_discarded_this_resolution += 1;
                            self.discarded_card_ids_this_resolution.push(cid);
                            if was_creature {
                                self.creature_cards_discarded_this_resolution += 1;
                            }
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
                            // Stash the milled id so a follow-up
                            // `Selector::LastMoved` in the same Seq can
                            // target the milled card (Tablet of
                            // Discovery, Ark of Hunger's "you may play
                            // that card this turn" rider).
                            self.last_moved_cards.push(cid);
                        }
                    }
                }
                Ok(())
            }

            Effect::SetNoMaxHandSize { who } => {
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].no_maximum_hand_size = true;
                    }
                }
                Ok(())
            }

            Effect::DiscardAnyNumber { who } => {
                use crate::decision::Decision;
                for ent in self.resolve_selector(who, ctx) {
                    let EntityRef::Player(p) = ent else { continue };
                    if self.players[p].hand.is_empty() { continue; }
                    let candidates: Vec<(crate::card::CardId, String)> = self
                        .players[p]
                        .hand
                        .iter()
                        .map(|c| (c.id, c.definition.name.to_string()))
                        .collect();
                    // "Any number" — count = hand size; the decider's
                    // `Discard(picked_ids)` answer can return 0..=hand.len()
                    // entries. AutoDecider picks 0 by default (it returns
                    // `iter().take(count).take(0)` semantics — but our
                    // AutoDecider uses `count` directly so we tell it 0 by
                    // surfacing `count: 0` with the full hand). For UI seats
                    // the full hand is surfaced so the player can pick any
                    // subset.
                    let count = if self.players[p].wants_ui {
                        candidates.len() as u32
                    } else {
                        0
                    };
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
                                // Cache the dying card's snapshot for
                                // AnotherOfYours-scope triggers and type
                                // filter predicates (token deaths in
                                // particular vanish before dispatch).
                                if let Some(c) = self.battlefield_find(cid) {
                                    self.died_card_snapshots.insert(cid, c.clone());
                                }
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

            Effect::LoseAllAbilities { what, duration } => {
                // Layer-6 strip-abilities continuous effect (CR 113.10b).
                // Installs `Modification::RemoveAllAbilities` against each
                // resolved permanent so the trigger dispatcher and
                // activated-ability resolver skip the target's printed
                // abilities while the effect is in scope. Used by Mercurial
                // Transformation / Turn to Frog / Lignify "becomes 1/1
                // creature and loses all abilities" patterns.
                use crate::game::layers::{
                    AffectedPermanents, ContinuousEffect, Layer, Modification,
                };
                let duration_kind = map_effect_duration(*duration);
                let source = ctx.source.unwrap_or(CardId(0));
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id() {
                        let ts = self.next_timestamp();
                        self.add_continuous_effect(ContinuousEffect {
                            timestamp: ts,
                            source,
                            affected: AffectedPermanents::Specific(vec![cid]),
                            layer: Layer::L6Ability,
                            sublayer: None,
                            duration: duration_kind.clone(),
                            modification: Modification::RemoveAllAbilities,
                        });
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
                    AffectedPermanents, ContinuousEffect, Layer, Modification,
                    PtSublayer,
                };
                let p = self.evaluate_value(power, ctx);
                let t = self.evaluate_value(toughness, ctx);
                let duration_kind = map_effect_duration(*duration);
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

            Effect::GrantTriggeredAbility { what, trigger, duration: _ } => {
                // Currently only EOT-duration grants are honored; the
                // entry is cleared in `do_cleanup`. Permanent-duration
                // grants would need a separate map keyed off the
                // granting permanent (so the grant retires when the
                // granter leaves) — tracked as future engine work.
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id() {
                        self.granted_triggers_eot
                            .entry(cid)
                            .or_default()
                            .push((**trigger).clone());
                    }
                }
                Ok(())
            }

            Effect::GrantKeyword { what, keyword, duration } => {
                // Per-instance granted keyword. Previously mutated
                // `definition.keywords` directly with no cleanup, so an
                // "EOT haste" grant would persist forever. Now distinguishes
                // EOT vs Permanent: EOT grants enter the `granted_keywords_eot`
                // bag (cleared at Cleanup along with `power_bonus`), while
                // Permanent grants still mutate the printed keyword list
                // (a leak-free no-op since indefinite grants don't expire).
                use crate::effect::Duration as EffectDur;
                let is_eot = matches!(
                    duration,
                    EffectDur::EndOfTurn | EffectDur::EndOfCombat
                );
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id()
                        && let Some(c) = self.battlefield_find_mut(cid)
                    {
                        if is_eot {
                            if !c.granted_keywords_eot.contains(keyword)
                                && !c.definition.keywords.contains(keyword)
                            {
                                c.granted_keywords_eot.push(keyword.clone());
                            }
                        } else if !c.definition.keywords.contains(keyword) {
                            c.definition.keywords.push(keyword.clone());
                        }
                    }
                }
                Ok(())
            }

            Effect::AddCounter { what, kind, amount } => {
                let base = self.evaluate_value(amount, ctx).max(0) as u32;
                if base == 0 { return Ok(()); }
                for ent in self.resolve_selector(what, ctx) {
                    match ent {
                        EntityRef::Permanent(cid) => {
                            // CR 614.16 counter-doubling replacement: each
                            // `StaticEffect::DoubleCounters` permanent the
                            // affected permanent's *controller* has on the
                            // battlefield doubles the count. Stacking
                            // doublers multiply (2^k where k is the number
                            // of active doublers). Looked up per-target
                            // since a fan-out (`ForEach`) could span
                            // controllers.
                            let target_ctrl = self.battlefield_find(cid).map(|c| c.controller);
                            let n = if let Some(ctrl) = target_ctrl {
                                let doublers = self.counter_doublers_for(ctrl);
                                let mut scaled = base;
                                for _ in 0..doublers {
                                    scaled = scaled.saturating_mul(2);
                                }
                                scaled
                            } else {
                                base
                            };
                            if let Some(c) = self.battlefield_find_mut(cid) {
                                c.add_counters(*kind, n);
                                events.push(GameEvent::CounterAdded { card_id: cid, counter_type: *kind, count: n });
                            }
                            // Track per-turn "this permanent gained counters"
                            // for Fractal Tender's end-step trigger and any
                            // future "if you put a counter on this creature
                            // this turn" payoff.
                            self.permanents_gained_counter_this_turn.insert(cid);
                        }
                        EntityRef::Player(p) if *kind == CounterType::Poison => {
                            // Poison counters on players also scale per
                            // CR 614.16 (Doubling Season / Vorinclex would
                            // double poison too); use the affected player's
                            // own counter-doubler count.
                            let doublers = self.counter_doublers_for(p);
                            let mut n = base;
                            for _ in 0..doublers {
                                n = n.saturating_mul(2);
                            }
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

            Effect::MoveCounter { from, to, kind, amount } => {
                // CR 122.5: moving counters is a single zone-internal
                // transfer, not a remove-then-add (DoubleCounters does
                // NOT apply). The actual move is clamped at the source's
                // current counter pool.
                let request = self.evaluate_value(amount, ctx).max(0) as u32;
                if request == 0 { return Ok(()); }
                // Pick the source (singular — moves typically target one
                // permanent; if multiple, take the first).
                let source_cids: Vec<_> = self.resolve_selector(from, ctx)
                    .into_iter()
                    .filter_map(|e| e.as_permanent_id())
                    .collect();
                let Some(src_cid) = source_cids.first().copied() else { return Ok(()); };
                let removed = if let Some(s) = self.battlefield_find_mut(src_cid) {
                    s.remove_counters(*kind, request)
                } else {
                    0
                };
                if removed == 0 { return Ok(()); }
                events.push(GameEvent::CounterRemoved {
                    card_id: src_cid, counter_type: *kind, count: removed,
                });
                // Pick the first destination and add the removed counter
                // count (no doubling per CR 122.5 — moves preserve the
                // counter identity, they're not "put counters").
                for ent in self.resolve_selector(to, ctx) {
                    if let Some(cid) = ent.as_permanent_id()
                        && let Some(d) = self.battlefield_find_mut(cid) {
                            d.add_counters(*kind, removed);
                            events.push(GameEvent::CounterAdded {
                                card_id: cid, counter_type: *kind, count: removed,
                            });
                            break;
                        }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::Proliferate => {
                // CR 701.34a — "To proliferate means to choose any number
                // of permanents and/or players that have a counter, then
                // give each one additional counter of each kind that
                // permanent or player already has." The proliferating
                // player picks which permanents and which players. The
                // engine's auto-decider implements a strategic baseline:
                //  - Pick each friendly permanent for every counter kind
                //    that benefits the controller (+1/+1, Charge, Loyalty,
                //    Page, Stun on enemy-bound permanents). Skip
                //    MinusOneMinusOne on friendlies (would shrink them).
                //  - Pick each enemy permanent for MinusOneMinusOne only
                //    (would shrink them); skip +1/+1 on enemies (would
                //    pump them).
                //  - For poison: each opponent gets +1 poison; the
                //    proliferating player declines their own poison
                //    counter.
                let proliferating = ctx.controller;
                let updates: Vec<(CardId, Vec<CounterType>)> = self
                    .battlefield
                    .iter()
                    .map(|c| {
                        let friendly = c.controller == proliferating;
                        let kinds: Vec<CounterType> = c.counters.iter()
                            .filter(|(_, n)| **n > 0)
                            .filter(|(k, _)| match **k {
                                // Bad-for-friendly: skip on your stuff,
                                // proliferate on enemy stuff.
                                CounterType::MinusOneMinusOne => !friendly,
                                CounterType::Stun => !friendly,
                                // Good-for-friendly: proliferate yours,
                                // skip opponent's.
                                CounterType::PlusOnePlusOne
                                | CounterType::Loyalty
                                | CounterType::Charge
                                | CounterType::Page => friendly,
                                // Other kinds: proliferate by default
                                // (the controller can always elect to
                                // proliferate any counter under the
                                // printed rule).
                                _ => true,
                            })
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
                    if self.players[i].poison_counters > 0 && i != proliferating {
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
                let mut n = self.evaluate_value(count, ctx).max(0) as u32;
                // CR 614.13 token-doubling replacement: each
                // `StaticEffect::DoubleTokens` permanent the controller has on
                // the battlefield (Adrix and Nev, Twincasters; Doubling Season
                // for the token half) doubles the count. Stacking doublers
                // multiply (2^k where k is the number of active doublers).
                let doublers = self.token_doublers_for(p);
                for _ in 0..doublers {
                    n = n.saturating_mul(2);
                }
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
                    // Plural variant: track every token minted this
                    // resolution. Read by `Selector::LastCreatedTokens`
                    // (Fractal Spawning, multi-mint cards). Cleared at
                    // resolution root start alongside the singular slot.
                    self.last_created_tokens.push(id);
                    // Tokens entering the battlefield are still permanents
                    // entering the battlefield — fire any self-source ETB
                    // triggers on the token's definition (a TokenDefinition
                    // currently doesn't carry triggered_abilities, but if
                    // one is added later it will fire correctly).
                    self.fire_self_etb_triggers(id, p);
                }
                Ok(())
            }

            Effect::CreateTokenCopyOf {
                who,
                count,
                source,
                extra_creature_types,
                override_pt,
            } => {
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let mut n = self.evaluate_value(count, ctx).max(0) as u32;
                let doublers = self.token_doublers_for(p);
                for _ in 0..doublers {
                    n = n.saturating_mul(2);
                }
                // Resolve the source permanent. Walk battlefield first;
                // fall back to graveyard / hand / exile via the same
                // sequence `move_card_to` uses, so a fresh copy can be
                // minted off a card that left the bf mid-resolution.
                let source_id = self
                    .resolve_selector(source, ctx)
                    .into_iter()
                    .find_map(|e| match e {
                        EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                        _ => None,
                    });
                let Some(src_id) = source_id else { return Ok(()); };
                let source_def = self
                    .battlefield
                    .iter()
                    .find(|c| c.id == src_id)
                    .map(|c| c.definition.clone());
                let Some(mut def) = source_def else { return Ok(()); };
                // Apply extra creature types & P/T override.
                let mut extra_types = def.subtypes.creature_types.clone();
                for t in extra_creature_types.iter() {
                    if !extra_types.contains(t) {
                        extra_types.push(*t);
                    }
                }
                def.subtypes.creature_types = extra_types;
                if let Some((p_o, t_o)) = override_pt {
                    def.power = *p_o;
                    def.toughness = *t_o;
                }
                for _ in 0..n {
                    let id = self.next_id();
                    let mut inst = CardInstance::new(id, def.clone(), p);
                    inst.controller = p;
                    inst.is_token = true;
                    self.battlefield.push(inst);
                    events.push(GameEvent::TokenCreated { card_id: id });
                    events.push(GameEvent::PermanentEntered { card_id: id });
                    self.last_created_token = Some(id);
                    self.last_created_tokens.push(id);
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

            Effect::CounterUnless { what, cost } => {
                // CR 702.21 — Ward body. Resolve `what` to a card-id, walk
                // the stack for the topmost matching `Spell` (by `card.id`)
                // or `Trigger` (by `source`), and try to auto-pay `cost`
                // on the affected controller's behalf. If they can't pay,
                // the stack item is removed (spells fall into their
                // owner's graveyard; abilities just vanish).
                use crate::card::WardCost;

                let targets = self.resolve_selector(what, ctx);
                let target_id = targets.into_iter().find_map(|t| match t {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => Some(cid),
                    _ => None,
                });
                let Some(cid) = target_id else { return Ok(()); };

                // Find the topmost matching stack item — prefer Spell, then
                // Trigger. CR 702.21a: Ward fires on "spell or ability";
                // the stack carries spells as `StackItem::Spell` and
                // activated/triggered abilities as `StackItem::Trigger`.
                let mut spell_pos: Option<usize> = None;
                let mut trigger_pos: Option<usize> = None;
                for (i, si) in self.stack.iter().enumerate().rev() {
                    match si {
                        StackItem::Spell { card, uncounterable: false, .. }
                            if card.id == cid && spell_pos.is_none() =>
                        {
                            spell_pos = Some(i);
                        }
                        StackItem::Trigger { source, .. }
                            if *source == cid && trigger_pos.is_none() =>
                        {
                            trigger_pos = Some(i);
                        }
                        _ => {}
                    }
                    if spell_pos.is_some() && trigger_pos.is_some() {
                        break;
                    }
                }
                // Prefer Spell if both exist — a card on the stack as a
                // spell can't simultaneously be the source of an ability
                // (the permanent doesn't exist yet).
                let (pos, affected_controller, is_spell) = match (spell_pos, trigger_pos) {
                    (Some(p), _) => {
                        let StackItem::Spell { caster, .. } = &self.stack[p] else {
                            unreachable!()
                        };
                        (p, *caster, true)
                    }
                    (None, Some(p)) => {
                        let StackItem::Trigger { controller, .. } = &self.stack[p] else {
                            unreachable!()
                        };
                        (p, *controller, false)
                    }
                    (None, None) => return Ok(()),
                };

                // Attempt auto-pay on the affected controller's behalf.
                let paid = match cost {
                    WardCost::Mana(mc) => {
                        let saved_priority = self.priority.player_with_priority;
                        self.priority.player_with_priority = affected_controller;
                        let ok = self.try_pay_with_auto_tap(affected_controller, mc).is_ok();
                        self.priority.player_with_priority = saved_priority;
                        ok
                    }
                    WardCost::Life(n) => {
                        // Ward—Pay N life. CR 119.4 forbids paying more
                        // life than you have, so insufficient life means
                        // payment fails.
                        let n = *n as i32;
                        if self.effective_life(affected_controller) >= n {
                            self.adjust_life(affected_controller, -n);
                            true
                        } else {
                            false
                        }
                    }
                    WardCost::Discard(n) => {
                        // Ward—Discard N cards. Payable only if the
                        // controller has ≥ N cards in hand. Auto-pay
                        // picks the first N cards. An interactive
                        // surface should prompt.
                        let n = *n as usize;
                        if self.players[affected_controller].hand.len() >= n {
                            for _ in 0..n {
                                let card = self.players[affected_controller].hand.remove(0);
                                let card_id = card.id;
                                self.players[affected_controller].graveyard.push(card);
                                self.cards_discarded_this_resolution =
                                    self.cards_discarded_this_resolution.saturating_add(1);
                                self.discarded_card_ids_this_resolution.push(card_id);
                            }
                            true
                        } else {
                            false
                        }
                    }
                    WardCost::SacrificeCreature => {
                        let pick = self
                            .battlefield
                            .iter()
                            .find(|c| {
                                c.controller == affected_controller && c.definition.is_creature()
                            })
                            .map(|c| c.id);
                        if let Some(sac_id) = pick {
                            let _ = self.remove_to_graveyard_with_triggers(sac_id);
                            true
                        } else {
                            false
                        }
                    }
                };

                if !paid {
                    let removed = self.stack.remove(pos);
                    if is_spell
                        && let StackItem::Spell { card, caster, .. } = removed
                    {
                        self.players[caster].send_to_graveyard(*card);
                    }
                    // Trigger items just drop off — nothing else to clean up.
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
                        if is_creature {
                            // Cache snapshot for AnotherOfYours triggers.
                            if let Some(c) = self.battlefield_find(id) {
                                self.died_card_snapshots.insert(id, c.clone());
                            }
                            // Emit CreatureSacrificed before CreatureDied
                            // so order-sensitive triggers (CR 701.16) see
                            // the sacrifice-specific event first.
                            events.push(GameEvent::CreatureSacrificed { card_id: id, who: p });
                            events.push(GameEvent::CreatureDied { card_id: id });
                        }
                        let mut die_evs = self.remove_to_graveyard_with_triggers(id);
                        events.append(&mut die_evs);
                    }
                }
                Ok(())
            }

            Effect::SacrificeGreatestMV { who, count, filter } => {
                // Same picker as `Effect::Sacrifice` but reverses the CMC
                // sort to pick the most-expensive match. Used by Soul
                // Shatter ("Each opponent sacrifices a creature or
                // planeswalker with the greatest mana value among
                // permanents that player controls"). When ties exist
                // (multiple matches at the same CMC), the auto-picker
                // breaks them by lowest power (matching Sacrifice's
                // secondary key).
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let source_id = ctx.source;
                for ent in self.resolve_selector(who, ctx) {
                    let EntityRef::Player(p) = ent else { continue; };
                    let mut candidates: Vec<&CardInstance> = self
                        .battlefield
                        .iter()
                        .filter(|c| c.controller == p)
                        .filter(|c| {
                            let t = Target::Permanent(c.id);
                            self.evaluate_requirement_static(filter, &t, p, ctx.source)
                        })
                        .collect();
                    // Sort key:
                    // 1. Source last (preserves "another creature" intent).
                    // 2. Descending CMC (highest first via i32 negation).
                    // 3. Ascending power (lowest first as tiebreaker).
                    candidates.sort_by_key(|c| {
                        (
                            Some(c.id) == source_id,
                            -(c.definition.cost.cmc() as i32),
                            c.power(),
                        )
                    });
                    let ids: Vec<CardId> =
                        candidates.into_iter().take(n).map(|c| c.id).collect();
                    for id in ids {
                        let is_creature = self
                            .battlefield_find(id)
                            .map(|c| c.definition.is_creature())
                            .unwrap_or(false);
                        if is_creature {
                            if let Some(c) = self.battlefield_find(id) {
                                self.died_card_snapshots.insert(id, c.clone());
                            }
                            events.push(GameEvent::CreatureSacrificed { card_id: id, who: p });
                            events.push(GameEvent::CreatureDied { card_id: id });
                        }
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
                    // Stash the moved id so a downstream
                    // `Selector::LastMoved` in the same Seq can target
                    // it (Practiced Scrollsmith's Move → GrantMayPlay
                    // chain).
                    self.last_moved_cards.push(cid);
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
                // record its power + toughness on `state.sacrificed_power`
                // / `state.sacrificed_toughness` so a subsequent
                // `Value::SacrificedPower` / `Value::SacrificedToughness`
                // can reference it (Thud, Tribute to Hunger).
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let candidate = self
                    .battlefield
                    .iter()
                    .find(|c| {
                        c.controller == p
                            && self.evaluate_requirement_static(filter, &Target::Permanent(c.id), p, ctx.source)
                    })
                    .map(|c| (c.id, c.power(), c.toughness()));
                if let Some((cid, power, toughness)) = candidate {
                    self.sacrificed_power = Some(power);
                    self.sacrificed_toughness = Some(toughness);
                    let is_creature = self
                        .battlefield_find(cid)
                        .map(|c| c.definition.is_creature())
                        .unwrap_or(false);
                    if is_creature {
                        if let Some(c) = self.battlefield_find(cid) {
                            self.died_card_snapshots.insert(cid, c.clone());
                        }
                        events.push(GameEvent::CreatureSacrificed { card_id: cid, who: p });
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
                //
                // Fall back to the cast spell's slot-0 target when our own
                // resolution context has no target — that's the Repartee /
                // triggered-ability shape used by Conciliator's Duelist
                // ("Repartee → exile the cast spell's target, return at
                // next end step"). The trigger resolves with empty
                // `ctx.targets` but the cast-spell `StackItem::Spell` is
                // still below us on the stack, so we can pull its target.
                let target = ctx.targets.first().cloned().or_else(|| {
                    let cid = match ctx.trigger_source {
                        Some(EntityRef::Card(c)) | Some(EntityRef::Permanent(c)) => c,
                        _ => return None,
                    };
                    self.stack.iter().rev().find_map(|si| match si {
                        StackItem::Spell { card, target, .. } if card.id == cid => {
                            target.clone()
                        }
                        _ => None,
                    })
                });
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
                        self.adjust_life(p, -(*life_cost as i32));
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
                // and place it via the requested destination. Push the
                // matched id onto `last_moved_cards` so a downstream
                // `Selector::LastMoved` in the same Seq can target it
                // — used by Velomachus Lorehold's
                // `RevealUntilFind + GrantMayPlay` chain.
                if found_idx.is_some() && !self.players[p].library.is_empty() {
                    let card = self.players[p].library.remove(0);
                    let cid = card.id;
                    self.place_card_in_dest(card, p, &resolved_dest, events);
                    self.last_moved_cards.push(cid);
                }
                // Lose 1 life per revealed card (Spoils of the Vault rider).
                let life = (revealed as u32).saturating_mul(*life_per_revealed);
                if life > 0 {
                    self.adjust_life(p, -(life as i32));
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

            Effect::CopySpellUnlessPaid { what, mana_cost, count } => {
                // Wandering Archaic shape: the *caster of the spell being
                // copied* may pay `mana_cost` to avoid being copied. We:
                // (1) locate the matching `StackItem::Spell`; (2) ask the
                // *caster* yes/no via `Decision::OptionalTrigger`; (3) on
                // yes + affordable pool, deduct + skip copy; (4) on no
                // or unaffordable, fall through to the same copy path as
                // `Effect::CopySpell`.
                use crate::decision::{Decision, DecisionAnswer};
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 {
                    return Ok(());
                }
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
                    let stack_idx = self.stack.iter().rposition(|s| {
                        matches!(s, crate::game::types::StackItem::Spell { card, .. }
                            if card.id == cid)
                    });
                    let Some(idx) = stack_idx else { continue; };
                    // Snapshot the caster (the affected payer for the
                    // optional pay) up-front. The spell snapshot for the
                    // copy is taken inside the "unpaid" branch.
                    let caster_for_pay = if let crate::game::types::StackItem::Spell {
                        caster, ..
                    } = &self.stack[idx]
                    {
                        *caster
                    } else {
                        continue;
                    };
                    // Ask the *caster* of the spell whether they want to
                    // pay the tax. Bot's AutoDecider defaults to false
                    // (let the copy happen — saves the {2}).
                    let answer = self.decider.decide(&Decision::OptionalTrigger {
                        source: ctx.source.unwrap_or(CardId(0)),
                        description: "Pay {2} to prevent Wandering Archaic's copy?"
                            .to_string(),
                    });
                    if matches!(answer, DecisionAnswer::Bool(true)) {
                        // Try to deduct from the payer's pool.
                        let pool = &mut self.players[caster_for_pay].mana_pool;
                        if pool.pay(mana_cost).is_ok() {
                            // Paid — skip the copy.
                            continue;
                        }
                        // Couldn't afford; fall through to copy.
                    }
                    // Unpaid (declined or unaffordable) → copy `n` times.
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
                            uncounterable: true,
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

            Effect::PreventAllCombatDamageThisTurn => {
                // CR 615.1 — set the engine-wide flag the combat damage
                // resolver consults. Cleared in `do_cleanup` (CR 514.2).
                self.prevent_combat_damage_this_turn = true;
                Ok(())
            }

            Effect::DiminishCreaturesExceptChosenType { power, toughness } => {
                // Crippling Fear-style "Choose a creature type. Creatures
                // other than creatures of the chosen type get -P/-T EOT."
                // Synchronously decides via `self.decider` (AutoDecider
                // picks Demon, ScriptedDecider can override) so the
                // effect resolves in a single pass without
                // suspend/resume. The pump is applied per-creature via
                // the standard `power_bonus` / `toughness_bonus` mutation
                // path (same code shape as Effect::PumpPT).
                use crate::decision::{Decision, DecisionAnswer};
                let p = self.evaluate_value(power, ctx);
                let t = self.evaluate_value(toughness, ctx);
                let source_id = ctx.source.unwrap_or(CardId(0));
                let decision = Decision::ChooseCreatureType { source: source_id };
                let answer = self.decider.decide(&decision);
                let DecisionAnswer::CreatureType(ct) = answer else {
                    return Err(crate::game::GameError::DecisionAnswerMismatch);
                };
                let card_ids: Vec<CardId> = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.definition.is_creature()
                            && !c.definition.subtypes.creature_types.contains(&ct)
                    })
                    .map(|c| c.id)
                    .collect();
                for cid in card_ids {
                    if let Some(c) = self.battlefield_find_mut(cid) {
                        c.power_bonus += p;
                        c.toughness_bonus += t;
                        events.push(GameEvent::PumpApplied {
                            card_id: cid,
                            power: p,
                            toughness: t,
                        });
                    }
                }
                Ok(())
            }

            Effect::GrantMayPlay {
                what,
                duration,
                to_owner,
                exile_after,
            } => {
                // Resolve `what` to a set of cards and stamp each with a
                // `MayPlayPermission`. The selector can match cards in
                // any zone — graveyard (Practiced Scrollsmith's target
                // from your graveyard), exile (Suspend Aggression's
                // exiled cards), or even hand. The cast-from-zone game
                // action consults `may_play_until` regardless of zone.
                let entities = self.resolve_selector(what, ctx);
                let granter_player = ctx.controller;
                let granted_turn = self.turn_number;
                for ent in entities {
                    let cid = match ent {
                        EntityRef::Card(id) => id,
                        _ => continue,
                    };
                    // Determine recipient before we take a mut borrow.
                    let recipient = if *to_owner {
                        self.find_card_owner(cid).unwrap_or(granter_player)
                    } else {
                        granter_player
                    };
                    if let Some(card) = self.find_card_anywhere_mut(cid) {
                        card.may_play_until = Some(crate::card::MayPlayPermission {
                            player: recipient,
                            granted_turn,
                            duration: *duration,
                            exile_after: *exile_after,
                        });
                    }
                }
                Ok(())
            }

            Effect::CastWithoutPayingImmediate {
                what,
                source_zone,
                exile_after,
            } => {
                // Resolve `what` to a single card in `source_zone`, ask
                // the controller via OptionalTrigger, and on yes hand
                // off to the free-cast helper. The helper auto-targets /
                // auto-modes the card; ScriptedDecider can override.
                let entities = self.resolve_selector(what, ctx);
                let card_id = entities.into_iter().find_map(|e| match e {
                    EntityRef::Card(id) => Some(id),
                    _ => None,
                });
                let Some(card_id) = card_id else { return Ok(()); };
                // Confirm the card is actually in the named zone — the
                // selector may have read a stale target.
                if self.find_card_zone(card_id) != Some(*source_zone) {
                    return Ok(());
                }
                // Lands are played, not cast — skip them silently. This
                // makes `ForEach LastMoved → CastWithoutPayingImmediate`
                // safe to use over a mixed top-of-library exile (e.g.
                // Improvisation Capstone exiles whatever's on top).
                let is_land = self
                    .find_card_anywhere(card_id)
                    .map(|c| c.definition.card_types.contains(&crate::card::CardType::Land))
                    .unwrap_or(false);
                if is_land { return Ok(()); }
                use crate::decision::{Decision, DecisionAnswer};
                let source_for_ask = ctx.source.unwrap_or(CardId(0));
                let answer = self.decider.decide(&Decision::OptionalTrigger {
                    source: source_for_ask,
                    description: "Cast without paying?".to_string(),
                });
                let yes = matches!(answer, DecisionAnswer::Bool(true));
                if !yes {
                    return Ok(());
                }
                // Auto-pick a target for the freshly-cast spell. Targets
                // are picked from the controller's perspective (avoiding
                // the cast card itself).
                let card_def = self
                    .find_card_anywhere(card_id)
                    .map(|c| c.definition.clone());
                let Some(card_def) = card_def else { return Ok(()); };
                let auto_target = self.auto_target_for_effect_avoiding(
                    &card_def.effect,
                    ctx.controller,
                    Some(card_id),
                );
                let cast_events = self.cast_card_for_free(
                    ctx.controller,
                    card_id,
                    *source_zone,
                    auto_target,
                    vec![],
                    None,
                    None,
                    *exile_after,
                )?;
                events.extend(cast_events);
                Ok(())
            }

            Effect::RegisterParadigm => {
                // Register a recurring `YourNextMainPhase` delayed
                // trigger whose body is `Effect::CastFreeParadigmCopy`.
                // The trigger's `source` is the resolving Paradigm
                // card's id; combined with `CardDefinition.exile_on_resolve
                // = true`, the card lands in exile and stays reachable
                // for the recurrence. `fires_once: false` so the trigger
                // survives each firing and fans out again next turn.
                use crate::game::types::{DelayedKind, DelayedTrigger};
                let Some(source) = ctx.source else { return Ok(()); };
                self.delayed_triggers.push(DelayedTrigger {
                    controller: ctx.controller,
                    source,
                    kind: DelayedKind::YourNextMainPhase,
                    effect: Effect::CastFreeParadigmCopy,
                    target: None,
                    fires_once: false,
                });
                Ok(())
            }

            Effect::CastFreeParadigmCopy => {
                // Find the original Paradigm-exiled card by id (the
                // trigger's `source`). If it's missing from exile (e.g.
                // someone Bojuka-Bogged the exile zone — unlikely in
                // SOS but defensive), drop the trigger.
                use crate::card::Zone;
                let source = ctx.source.unwrap_or(CardId(0));
                let original_def = self
                    .exile
                    .iter()
                    .find(|c| c.id == source)
                    .map(|c| c.definition.clone());
                let Some(def) = original_def else { return Ok(()); };
                // Ask the controller "cast a copy?"
                use crate::decision::{Decision, DecisionAnswer};
                let answer = self.decider.decide(&Decision::OptionalTrigger {
                    source,
                    description: format!("Paradigm: cast a copy of {}?", def.name),
                });
                if !matches!(answer, DecisionAnswer::Bool(true)) {
                    return Ok(());
                }
                // Mint a tokenized copy of the exiled card in exile.
                let new_id = self.next_id();
                let mut copy = crate::card::CardInstance::new(new_id, def.clone(), ctx.controller);
                copy.is_token = true;
                self.exile.push(copy);
                // Auto-target for the copy.
                let auto_target = self.auto_target_for_effect_avoiding(
                    &def.effect,
                    ctx.controller,
                    Some(new_id),
                );
                // Free-cast from exile. Drop the result events into our
                // surrounding events buffer.
                let cast_events = self.cast_card_for_free(
                    ctx.controller,
                    new_id,
                    Zone::Exile,
                    auto_target,
                    vec![],
                    None,
                    None,
                    false,
                )?;
                events.extend(cast_events);
                Ok(())
            }

            Effect::ActivateDellianEmblem { who } => {
                for ent in self.resolve_selector(&Selector::Player(who.clone()), ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].dellian_fel_emblem = true;
                    }
                }
                Ok(())
            }

            Effect::SkipTurns { who, count } => {
                let n = self.evaluate_value(count, ctx).max(0) as u32;
                if n == 0 { return Ok(()); }
                for ent in self.resolve_selector(&Selector::Player(who.clone()), ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].skip_turns =
                            self.players[p].skip_turns.saturating_add(n);
                    }
                }
                Ok(())
            }

            Effect::WinGame { who } => {
                // CR 104.2a — "you win the game". Resolve `who` to a single
                // player and eliminate every other (non-eliminated) player.
                // The SBA pass after this resolution will pick up the
                // 1-alive-player state and promote it to
                // `game_over = Some(winner)`. We don't directly set
                // `game_over` here so that anything resolving after this
                // step (in the same Seq) can still observe normal state;
                // the SBA loop is the canonical "the game ends" gate.
                let winner = self
                    .resolve_selector(&Selector::Player(who.clone()), ctx)
                    .into_iter()
                    .find_map(|e| match e {
                        EntityRef::Player(p) => Some(p),
                        _ => None,
                    });
                if let Some(w) = winner {
                    for (idx, pl) in self.players.iter_mut().enumerate() {
                        if idx != w {
                            pl.eliminated = true;
                        }
                    }
                }
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
            Selector::LastCreatedTokens => self
                .last_created_tokens
                .iter()
                .copied()
                .filter(|id| self.battlefield.iter().any(|c| c.id == *id))
                .map(EntityRef::Permanent)
                .collect(),
            Selector::LastMoved => self
                .last_moved_cards
                .iter()
                .copied()
                .map(EntityRef::Card)
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
            Selector::TopOfLibraryUntilMvAtLeast { who, threshold } => {
                let Some(p) = self.resolve_player(who, ctx) else { return vec![]; };
                let cap = self.evaluate_value(threshold, ctx).max(0);
                let mut sum: i32 = 0;
                let mut out: Vec<EntityRef> = Vec::new();
                for c in self.players[p].library.iter() {
                    out.push(EntityRef::Card(c.id));
                    sum += c.definition.cost.cmc() as i32;
                    if sum >= cap {
                        break;
                    }
                }
                out
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

            Selector::DiscardedThisResolution { filter } => {
                // Walk the IDs captured in `discarded_card_ids_this_resolution`
                // and look them up in their owner's graveyard (where
                // `Effect::Discard` moves them). Filter via the card-level
                // evaluator since the discarded cards aren't on the
                // battlefield.
                let ids = self.discarded_card_ids_this_resolution.clone();
                let mut out: Vec<EntityRef> = Vec::new();
                for cid in ids {
                    let card = self.players.iter()
                        .find_map(|p| p.graveyard.iter().find(|c| c.id == cid));
                    if let Some(c) = card
                        && self.evaluate_requirement_on_card(filter, c, ctx.controller)
                    {
                        out.push(EntityRef::Card(cid));
                    }
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

            Selector::TakeWithSumCap { inner, cap, value_of_each } => {
                let cap_n = self.evaluate_value(cap, ctx).max(0);
                if cap_n == 0 {
                    return vec![];
                }
                let candidates = self.resolve_selector(inner, ctx);
                let mut running_total: i32 = 0;
                let mut kept: Vec<EntityRef> = Vec::new();
                for ent in candidates {
                    // Bind the candidate to `ctx.trigger_source` so that
                    // `value_of_each` can reference it via
                    // `Selector::TriggerSource` (mirrors `Effect::ForEach`'s
                    // binding convention). Per-iteration sub-ctx clone keeps
                    // outer ctx untouched after evaluation.
                    let mut sub_ctx = ctx.clone();
                    sub_ctx.trigger_source = Some(ent);
                    let v = self.evaluate_value(value_of_each, &sub_ctx).max(0);
                    if running_total + v <= cap_n {
                        running_total += v;
                        kept.push(ent);
                    }
                    // Otherwise skip this candidate; iteration continues so
                    // smaller items can still fit. Greedy walk gives the
                    // AutoDecider a deterministic pick.
                }
                kept
            }
        }
    }

    /// Multi-player resolver. `EachPlayer` and `EachOpponent` return all
    /// matching alive seats so effects like Wheel of Fortune actually hit
    /// every player. Non-collective `PlayerRef` variants resolve to a single
    /// seat (or empty if the reference can't be resolved).
    pub(crate) fn resolve_players(&self, pref: &PlayerRef, ctx: &EffectContext) -> Vec<usize> {
        match pref {
            PlayerRef::EachOpponent => self
                .opponents_of(ctx.controller)
                .into_iter()
                .filter(|i| self.players[*i].is_alive())
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
                self.opponents_of(ctx.controller)
                    .into_iter()
                    .find(|i| self.players[*i].is_alive())
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

