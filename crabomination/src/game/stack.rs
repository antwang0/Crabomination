use super::*;
use crate::card::{Keyword, Supertype};
use crate::decision::{Decision, DecisionAnswer};
use crate::effect::{Effect, EventKind, EventScope};
use crate::game::types::{DelayedKind, DelayedTrigger};

impl GameState {
    /// CR 700.2b — "The controller of a modal triggered ability chooses
    /// the mode(s) as part of putting that ability on the stack."
    ///
    /// Inspect the trigger's top-level effect: if it's `Effect::ChooseMode`,
    /// ask the controller (via the installed `Decider`) which mode to pick.
    /// Returns `Some(idx)` for modal triggers and `None` for non-modal ones
    /// (which keeps the existing `mode.unwrap_or(0)` resolution path
    /// behaving correctly for the simple case). The `AutoDecider` picks
    /// mode 0 (the leftmost printed mode), preserving prior behaviour;
    /// `ScriptedDecider::new([DecisionAnswer::Mode(idx)])` lets tests
    /// inject alternative picks for cards like Prismari Apprentice
    /// (modal Magecraft: Scry 1 / +1/+0 EOT).
    ///
    /// The picked index is clamped to `modes.len() - 1` to guard against
    /// a misbehaving decider returning an out-of-range mode. Effects that
    /// nest `ChooseMode` inside `Seq`/`If`/`ForEach` are not addressed
    /// here — those would need a recursive walk and an N-tuple of picks;
    /// the printed Magic cards in scope today (Prismari Apprentice,
    /// future Tempted by the Oriq Magecraft rider) all have a top-level
    /// `ChooseMode` so the simple walk is sufficient.
    pub(crate) fn pick_trigger_mode(&mut self, effect: &Effect, source: CardId) -> Option<usize> {
        if let Effect::ChooseMode(modes) = effect {
            if modes.is_empty() {
                return None;
            }
            let answer = self.decider.decide(&Decision::ChooseMode {
                source,
                num_modes: modes.len(),
            });
            if let DecisionAnswer::Mode(idx) = answer {
                return Some(idx.min(modes.len() - 1));
            }
        }
        None
    }
}

impl GameState {
    // ── Pass priority ─────────────────────────────────────────────────────────

    pub(crate) fn pass_priority(&mut self) -> Result<Vec<GameEvent>, GameError> {
        let alive = self.alive_count();
        self.priority.consecutive_passes += 1;

        if self.priority.consecutive_passes < alive {
            // Move priority to the next non-eliminated player.
            self.priority.player_with_priority =
                self.next_alive_seat(self.priority.player_with_priority);
            return Ok(vec![]);
        }

        // All players passed — either resolve top of stack or advance the step.
        self.priority.consecutive_passes = 0;

        if !self.stack.is_empty() {
            let events = self.resolve_top_of_stack()?;
            // After resolution, active player gets priority again.
            self.give_priority_to_active();
            return Ok(events);
        }

        // Stack is empty — advance to next step.

        // MTG rule 500.4: mana pools empty at the end of each step and phase.
        for player in &mut self.players {
            player.mana_pool.empty();
        }

        // Auto-declare empty blockers if no one blocked.
        if self.step == TurnStep::DeclareBlockers
            && !self.attacking.is_empty()
            && !self.blockers_declared
        {
            self.blockers_declared = true;
        }

        let events = vec![];

        if self.step == TurnStep::Cleanup && self.do_cleanup() {
            // Suspended on the cleanup discard decision: a UI player must
            // choose which cards to pitch. `submit_decision` resumes
            // `finish_cleanup` + the step advance once it's answered.
            return Ok(events);
        }

        self.advance_step(events)
    }

    /// Compute and enter the step following the current one, running its
    /// turn-based entry actions (untap, draw, combat resolution, step
    /// triggers, …). Split out of `pass_priority` so the cleanup discard
    /// resume path can re-run it after a suspended discard is answered.
    pub(crate) fn advance_step(
        &mut self,
        mut events: Vec<GameEvent>,
    ) -> Result<Vec<GameEvent>, GameError> {
        // Skip FirstStrikeDamage if no first/double-strike creatures are in combat.
        let mut next = self.step.next();
        if next == TurnStep::FirstStrikeDamage && !self.has_first_strikers() {
            next = next.next(); // skip directly to CombatDamage
        }
        // CR 506.1 — "The declare blockers and combat damage steps are
        // skipped if no creatures are declared as attackers or put onto
        // the battlefield attacking." When the DeclareAttackers step
        // ends with no attackers, advance straight past DeclareBlockers /
        // FirstStrikeDamage / CombatDamage to EndCombat. Trigger windows
        // for "at the beginning of combat" still fire at BeginCombat
        // since that step is unaffected.
        if self.step == TurnStep::DeclareAttackers && self.attacking.is_empty() {
            next = TurnStep::EndCombat;
        }

        // CR 505.1b — additional combat phase. When the active player leaves
        // End of Combat with a banked extra phase, loop back to Begin Combat
        // (a fresh combat) instead of advancing to the postcombat main.
        if self.step == TurnStep::EndCombat && self.additional_combat_phases > 0 {
            self.additional_combat_phases -= 1;
            next = TurnStep::BeginCombat;
        }

        // CR 511.2 — "Effects that last 'until end of combat' expire at the
        // end of the combat phase." Sweep `UntilEndOfCombat` continuous
        // effects whenever we leave EndCombat — including into an additional
        // combat phase, since each combat phase has its own end.
        if self.step == TurnStep::EndCombat {
            self.expire_end_of_combat_effects();
            self.revert_temporary_control(&[crate::effect::Duration::EndOfCombat]);
            self.revert_temporary_copies(&[crate::effect::Duration::EndOfCombat]);
            let mut cleanup = self.process_attacking_token_cleanup();
            events.append(&mut cleanup);
        }

        self.step = next;
        // Per-step draw tallies reset at every step boundary (Orcish
        // Bowmasters' "first card drawn in the draw step" exemption).
        for pl in &mut self.players {
            pl.cards_drawn_this_step = 0;
        }
        events.push(GameEvent::StepChanged(next));

        match next {
            // Untap has no priority window — auto-execute and move on.
            TurnStep::Untap => {
                self.do_untap();
                // CR 502.2 — day/night turn-based check (doesn't use the stack).
                self.check_day_night_transition(&mut events);
                events.push(GameEvent::TurnStarted {
                    player: self.active_player_idx,
                    turn: self.turn_number,
                });
                // No priority in Untap — immediately advance to Upkeep.
                self.priority.player_with_priority = self.active_player_idx;
                let mut upkeep_events = self.pass_priority()?;
                events.append(&mut upkeep_events);
                return Ok(events);
            }
            TurnStep::Draw => {
                if self.skip_first_draw {
                    self.skip_first_draw = false;
                } else {
                    let p = self.active_player_idx;
                    if !self.draw_one(p, &mut events) {
                        // Drawing from an empty library eliminates `p`.
                        // Game-over check happens inside SBA and may end
                        // the game if only one player remains.
                        self.players[p].eliminated = true;
                        let mut sba = self.check_state_based_actions();
                        events.append(&mut sba);
                        if self.is_game_over() {
                            return Ok(events);
                        }
                    }
                }
                self.give_priority_to_active();
            }
            TurnStep::Upkeep => {
                // CR 702.32 / 702.62 — Fading / Vanishing tick down as a
                // turn-based action at upkeep, before step triggers.
                let mut fv = self.process_fading_vanishing();
                events.append(&mut fv);
                // CR 702.62d/e — Suspend time counters tick at the owner's
                // upkeep; the spell is cast for free when the last comes off.
                let mut susp = self.process_suspend();
                events.append(&mut susp);
                // Uvilda — hone counters tick down at the owner's upkeep.
                let mut hone = self.process_hone();
                events.append(&mut hone);
                // CR 702.24 — cumulative upkeep: age counter + pay-or-sacrifice.
                let mut cu = self.process_cumulative_upkeep();
                events.append(&mut cu);
                self.fire_step_triggers(TurnStep::Upkeep);
                self.give_priority_to_active();
            }
            TurnStep::PreCombatMain => {
                // CR 728.2 — rad-counter mill is a turn-based action that
                // happens as the precombat main phase begins, before
                // players receive priority (and thus before step triggers).
                let mut rad = self.do_rad_counters();
                events.append(&mut rad);
                // CR 714.2b — as the active player's precombat main phase
                // begins, put a lore counter on each Saga they control,
                // firing that chapter's ability.
                let sagas: Vec<CardId> = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == self.active_player_idx
                            && !c.definition.saga_chapters.is_empty()
                    })
                    .map(|c| c.id)
                    .collect();
                for id in sagas {
                    self.saga_advance(id);
                }
                self.fire_step_triggers(TurnStep::PreCombatMain);
                self.give_priority_to_active();
            }
            TurnStep::BeginCombat => {
                self.fire_step_triggers(TurnStep::BeginCombat);
                self.give_priority_to_active();
            }
            TurnStep::FirstStrikeDamage => {
                let mut fs_events = self.resolve_first_strike_damage()?;
                events.append(&mut fs_events);
                // Combat damage may suspend on a `wants_ui` player's ordering /
                // assignment choice; leave priority alone until it's answered.
                if self.pending_decision.is_none() {
                    self.give_priority_to_active();
                }
            }
            TurnStep::CombatDamage => {
                let mut combat_events = self.resolve_combat()?;
                events.append(&mut combat_events);
                if self.pending_decision.is_none() {
                    self.give_priority_to_active();
                }
            }
            TurnStep::End => {
                // CR 724 — the monarch draws a card at the beginning of
                // their end step (a turn-based action).
                if self.monarch == Some(self.active_player_idx) {
                    self.draw_one(self.active_player_idx, &mut events);
                }
                // CR 702.183 — Impending time counters tick at the beginning of
                // the controller's end step; the permanent becomes a creature
                // when the last is gone.
                let mut imp = self.process_impending();
                events.append(&mut imp);
                self.fire_step_triggers(TurnStep::End);
                self.give_priority_to_active();
            }
            TurnStep::Cleanup => {
                // Reset per-turn spell counter and the Gravestorm
                // permanents-died tally. Snapshot the turn's total first so
                // the classic werewolf "no spells cast last turn" check (read
                // at the next upkeep) sees it.
                self.spells_cast_last_turn = self.spells_cast_this_turn;
                self.spells_cast_this_turn = 0;
                self.mana_spent_on_spells_this_turn = 0;
                self.permanents_to_graveyard_this_turn = 0;
                self.give_priority_to_active();
            }
            _ => {
                self.give_priority_to_active();
            }
        }

        Ok(events)
    }

    /// Push step-based triggers onto the stack for the given step.
    /// Fires `EventKind::StepBegins(step)` triggers. Scope controls which
    /// players' permanents' triggers fire: `ActivePlayer` is default for
    /// "at the beginning of your upkeep"; `AnyPlayer` fires for everyone.
    /// Also processes any `delayed_triggers` whose kind matches this step
    /// (e.g. Pact upkeep cost, Goryo's exile-at-end-step).
    pub(crate) fn fire_step_triggers(&mut self, step: TurnStep) {
        let active = self.active_player_idx;
        let kind = EventKind::StepBegins(step);
        // Collect candidate (source, effect, controller, filter) tuples for
        // each battlefield permanent's matching trigger. We snapshot the
        // optional `event.filter` predicate alongside the effect so we can
        // re-check it after gathering — predicate evaluation needs
        // `&self.evaluate_predicate(...)` which can't run inside the inner
        // closure due to the `iter` borrow.
        let mut candidates: Vec<(CardId, Effect, usize, Option<crate::card::Predicate>)> = self
            .battlefield
            .iter()
            .flat_map(|c| {
                c.definition
                    .triggered_abilities
                    .iter()
                    .filter(|t| t.event.kind == kind)
                    .filter(|t| match t.event.scope {
                        EventScope::AnyPlayer => true,
                        EventScope::ActivePlayer | EventScope::YourControl | EventScope::SelfSource => {
                            c.controller == active
                        }
                        EventScope::OpponentControl => c.controller != active,
                        EventScope::AnotherOfYours => false,
                        EventScope::FromYourGraveyard => false, // walked separately below
                        EventScope::YourPermanentTargetedByOpponent => false, // event-based
                        EventScope::ControllerAttackedByOpponent => false, // combat-based
                    })
                    .map(|t| (c.id, t.effect.clone(), c.controller, t.event.filter.clone()))
            })
            .collect();
        // Walk the active player's graveyard for `FromYourGraveyard`
        // step triggers (Ichorid's "at the beginning of your upkeep").
        if let Some(player) = self.players.get(active) {
            for c in &player.graveyard {
                for t in &c.definition.triggered_abilities {
                    if t.event.kind == kind
                        && matches!(t.event.scope, EventScope::FromYourGraveyard)
                    {
                        candidates.push((c.id, t.effect.clone(), c.owner, t.event.filter.clone()));
                    }
                }
            }
        }
        // CR 114 — step-keyed emblem triggers ("at the beginning of your
        // upkeep, draw a card"). "Your" scope fires only on the emblem
        // owner's step; `AnyPlayer` fires for every player's step.
        for (seat, player) in self.players.iter().enumerate() {
            for em in &player.emblems {
                for t in &em.triggered {
                    let scoped_to_owner = matches!(
                        t.event.scope,
                        EventScope::YourControl | EventScope::ActivePlayer | EventScope::SelfSource
                    );
                    if t.event.kind == kind
                        && (matches!(t.event.scope, EventScope::AnyPlayer)
                            || (scoped_to_owner && seat == active))
                    {
                        candidates.push((CardId(0), t.effect.clone(), seat, t.event.filter.clone()));
                    }
                }
            }
        }
        // CR 603.4 — Intervening 'if' clause: "When the trigger event
        // occurs, the ability checks whether the stated condition is
        // true. The ability triggers only if it is; otherwise it does
        // nothing." Evaluate each trigger's optional `event.filter`
        // predicate now, before pushing to the stack. Triggers whose
        // filter fails are dropped (Triskaidekaphile's "if you have
        // exactly 13 cards in your hand", Felidar Sovereign's "if you
        // have 40 or more life", Pact-style "if it's your turn", etc.).
        // The second-half of CR 603.4 — re-check the condition as the
        // ability resolves — is now also wired (see
        // `triggers_with_filter` below + the resolver's `intervening_if`
        // branch).
        // Single filter pass that keeps both halves of CR 603.4 alive: drop
        // triggers whose intervening-if predicate is false right now (the
        // trigger-time check), and preserve the predicate on the survivors
        // so the resolver can re-check at resolution time.
        let triggers_with_filter: Vec<(CardId, Effect, usize, Option<crate::card::Predicate>)> =
            candidates
                .into_iter()
                .filter(|(src, _eff, ctrl, filter)| {
                    let Some(pred) = filter else { return true };
                    let ctx = crate::game::effects::EffectContext::for_trigger(
                        *src, *ctrl, None, 0,
                    );
                    self.evaluate_predicate(pred, &ctx)
                })
                .collect();

        // Drain matching delayed triggers off the queue and queue them up
        // alongside the regular battlefield triggers. Fires-once triggers
        // are removed; this keeps `pact_of_negation`-style "next upkeep"
        // logic correct without leaking back into the next turn.
        let mut delayed_to_fire: Vec<(CardId, Effect, usize, Option<Target>)> = Vec::new();
        let mut keep: Vec<DelayedTrigger> = Vec::new();
        for dt in std::mem::take(&mut self.delayed_triggers) {
            let matches = match (dt.kind, step) {
                (DelayedKind::YourNextUpkeep, TurnStep::Upkeep) => dt.controller == active,
                (DelayedKind::YourNextMainPhase, TurnStep::PreCombatMain) => {
                    dt.controller == active
                }
                (DelayedKind::NextEndStep, TurnStep::End) => true,
                _ => false,
            };
            if matches {
                delayed_to_fire.push((dt.source, dt.effect.clone(), dt.controller, dt.target.clone()));
                if !dt.fires_once {
                    keep.push(dt);
                }
            } else {
                keep.push(dt);
            }
        }
        self.delayed_triggers = keep;

        // Build a single APNAP-ordered queue (delayed triggers first,
        // then step triggers) so `drain_trigger_queue` can surface
        // `Decision::ChooseTarget` for wants_ui controllers instead of
        // silently auto-targeting them.
        let mut queue: Vec<PendingTriggerPush> = Vec::new();
        for (source, effect, controller, captured_target) in delayed_to_fire {
            let mode = self.pick_trigger_mode(&effect, source);
            // Delayed triggers may have captured a target at registration
            // time (e.g. Pact's "lose the game"). If so, push immediately
            // with that target — we already passed the targeting moment.
            if captured_target.is_some() {
                self.push_pending_trigger(
                    PendingTriggerPush {
                        source,
                        controller,
                        effect,
                        subject: None,
                        event_amount: 0,
                        mode,
                        intervening_if: None,
                    },
                    captured_target,
                );
                continue;
            }
            queue.push(PendingTriggerPush {
                source,
                controller,
                effect,
                subject: None,
                event_amount: 0,
                mode,
                intervening_if: None,
            });
        }
        for (source, effect, controller, intervening_if) in triggers_with_filter {
            let mode = self.pick_trigger_mode(&effect, source);
            queue.push(PendingTriggerPush {
                source,
                controller,
                effect,
                subject: None,
                event_amount: 0,
                mode,
                intervening_if,
            });
        }
        self.drain_trigger_queue(queue);
    }

    /// CR 714 — put a lore counter on a Saga and fire the chapter ability/ies
    /// for the new lore-counter total. Called on ETB (chapter I) and as a
    /// turn-based action at the start of the controller's precombat main.
    pub(crate) fn saga_advance(&mut self, card_id: CardId) {
        let Some(card) = self.battlefield.iter_mut().find(|c| c.id == card_id) else {
            return;
        };
        if card.definition.saga_chapters.is_empty() {
            return;
        }
        card.add_counters(crate::card::CounterType::Lore, 1);
        let chapter = card.counter_count(crate::card::CounterType::Lore);
        let controller = card.controller;
        let effects: Vec<Effect> = card
            .definition
            .saga_chapters
            .iter()
            .filter(|(n, _)| *n == chapter)
            .map(|(_, e)| e.clone())
            .collect();
        let mut queue: Vec<PendingTriggerPush> = Vec::new();
        for effect in effects {
            let mode = self.pick_trigger_mode(&effect, card_id);
            queue.push(PendingTriggerPush {
                source: card_id,
                controller,
                effect,
                subject: None,
                event_amount: 0,
                mode,
                intervening_if: None,
            });
        }
        self.drain_trigger_queue(queue);
    }

    // ── Stack resolution ──────────────────────────────────────────────────────

    pub(crate) fn resolve_top_of_stack(&mut self) -> Result<Vec<GameEvent>, GameError> {
        let Some(item) = self.stack.pop() else {
            return Ok(vec![]);
        };
        let mut events = vec![];

        match item {
            StackItem::Spell {
                card,
                caster,
                target,
                additional_targets,
                mode,
                x_value,
                converged_value,
                mana_spent,
                uncounterable: _,
            } => {
                let card = *card;
                let card_id = card.id;
                // CR 715 — while on an adventure the card is its instant/
                // sorcery half, so it resolves down the spell path (and is
                // exiled, not put onto the battlefield) regardless of its
                // creature card type.
                let is_noncreature = card.adventuring || !card.definition.is_creature();

                if card.definition.is_permanent() && !card.adventuring {
                    // Collect ETB triggers before moving card into battlefield.
                    // `mut` so the enters-as-copy path can swap in the
                    // copied object's ETB triggers (CR 707.5).
                    let mut etb_triggers: Vec<Effect> = card
                        .definition
                        .triggered_abilities
                        .iter()
                        .filter(|t| t.event.kind == EventKind::EntersBattlefield
                            && matches!(t.event.scope, EventScope::SelfSource))
                        .map(|t| t.effect.clone())
                        .collect();
                    let evoked = card.evoked;
                    let dashed = card.dashed;
                    // CR 614.12 — capture the "enters with N counters"
                    // replacement before the card moves to the battlefield;
                    // we apply the counters immediately after pushing,
                    // BEFORE the next state-based-action sweep, so a printed
                    // 0/0 body (Pterafractyl, Symmathematics) survives ETB.
                    let enters_spec = card.definition.enters_with_counters.clone();
                    let mut card = card;
                    card.controller = self.apply_etb_control_replacement(&card, card.controller);
                    self.battlefield.push(card);
                    // Collect the printed `enters_with_counters` spec and
                    // any active `ExtraEtbCountersForCreatureCasts` static
                    // effects controlled by the caster. The static fires
                    // only for creature spells (we gate on the resolving
                    // card's type).
                    let is_creature_resolve = self
                        .battlefield
                        .iter()
                        .find(|c| c.id == card_id)
                        .map(|c| c.definition.is_creature())
                        .unwrap_or(false);
                    let mut counter_specs: Vec<(crate::card::CounterType, crate::effect::Value)> =
                        Vec::new();
                    if let Some(spec) = enters_spec {
                        counter_specs.push(spec);
                    }
                    if is_creature_resolve {
                        for src in &self.battlefield {
                            if src.controller != caster {
                                continue;
                            }
                            for sa in &src.definition.static_abilities {
                                if let crate::effect::StaticEffect::ExtraEtbCountersForCreatureCasts {
                                    kind,
                                    value,
                                } = &sa.effect
                                {
                                    counter_specs.push((*kind, value.clone()));
                                }
                            }
                        }
                    }
                    // Metallic Mimic-style chosen-type ETB counters (any matching
                    // creature entry the caster controls).
                    if is_creature_resolve {
                        for kind in self.chosen_type_etb_counter_specs(card_id, caster) {
                            counter_specs.push((kind, crate::effect::Value::Const(1)));
                        }
                    }
                    // CR 122.1 — Solemnity drops enters-with-counters.
                    if self.counters_locked() { counter_specs.clear(); }
                    for (kind, value) in counter_specs {
                        let etb_ctx = crate::game::effects::EffectContext::for_spell_with_source(
                            card_id,
                            self.battlefield
                                .iter()
                                .find(|c| c.id == card_id)
                                .map(|c| c.definition.name)
                                .unwrap_or(""),
                            caster,
                            target.clone(),
                            additional_targets.clone(),
                            mode.unwrap_or(0),
                            x_value,
                            converged_value,
                            mana_spent,
                        );
                        let base = self.evaluate_value(&value, &etb_ctx);
                        if base > 0 {
                            // CR 614.16: counter-doubling replacement effects
                            // also apply to the "enters with N counters"
                            // replacement (Pestseed / Doubling Season / etc.).
                            let target_ctrl = self
                                .battlefield
                                .iter()
                                .find(|c| c.id == card_id)
                                .map(|c| c.controller);
                            let mut n = base as u32;
                            if let Some(ctrl) = target_ctrl {
                                let doublers = self.counter_doublers_for(ctrl);
                                for _ in 0..doublers {
                                    n = n.saturating_mul(2);
                                }
                            }
                            if let Some(card_mut) =
                                self.battlefield.iter_mut().find(|c| c.id == card_id)
                            {
                                card_mut.add_counters(kind, n);
                            }
                            events.push(GameEvent::CounterAdded {
                                card_id,
                                counter_type: kind,
                                count: n,
                            });
                        }
                    }
                    // CR 702.32 / 702.62 — Fading / Vanishing enter-with-counters.
                    self.apply_fading_vanishing_etb(card_id, &mut events);
                    // CR 702.183 — Impending: a permanent cast for its impending
                    // cost enters with N time counters (and isn't a creature
                    // until they tick off).
                    self.apply_impending_etb(card_id, &mut events);
                    // CR 707 — "enters as a copy of [filter]" replacement.
                    // Applied here, before the first SBA sweep, so a 0/0
                    // copier (Clone, Phantasmal Image) never dies as a 0/0.
                    if self.apply_enters_as_copy(card_id, caster, &mut events) {
                        // CR 707.5 — the copy's own ETB triggers fire. The
                        // list collected above was the copier's (usually
                        // empty); re-read it from the post-copy definition.
                        etb_triggers = self
                            .battlefield
                            .iter()
                            .find(|c| c.id == card_id)
                            .map(|c| {
                                c.definition
                                    .triggered_abilities
                                    .iter()
                                    .filter(|t| t.event.kind == EventKind::EntersBattlefield
                                        && matches!(t.event.scope, EventScope::SelfSource))
                                    .map(|t| t.effect.clone())
                                    .collect()
                            })
                            .unwrap_or_default();
                    }

                    events.push(GameEvent::PermanentEntered { card_id });

                    // CR 702.146e — a daybound permanent entering while it's
                    // neither day nor night makes it day.
                    if self.day_night.is_none()
                        && self
                            .battlefield_find(card_id)
                            .is_some_and(|c| c.definition.keywords.contains(&Keyword::Daybound))
                    {
                        self.set_day_night(crate::game::types::DayNight::Day, &mut events);
                    }

                    // CR 303.4f / 303.4h — an Aura permanent spell enters
                    // the battlefield attached to the permanent its single
                    // target chose. Wiring the `attached_to` link makes the
                    // Aura's `equipped_bonus` (P/T via layer 7c, keywords
                    // via layer 6) flow onto the enchanted creature, and the
                    // stale-link SBA in this file moves the Aura to the
                    // graveyard if its host ever leaves.
                    // Also attaches a bestowed enchantment-creature (CR
                    // 702.103) cast as an Aura, even though its printed type
                    // line isn't an Aura.
                    if self
                        .battlefield
                        .iter()
                        .any(|c| c.id == card_id && (c.definition.is_aura() || c.bestowed))
                        && let Some(crate::game::types::Target::Permanent(tid)) = target
                        && self.battlefield.iter().any(|c| c.id == tid)
                        && let Some(aura) =
                            self.battlefield.iter_mut().find(|c| c.id == card_id)
                    {
                        aura.attached_to = Some(tid);
                    }

                    // Evoke: schedule a self-sacrifice trigger that resolves
                    // AFTER the ETB triggers (so the ETB exile happens first,
                    // then the creature sacrifices itself).
                    if evoked {
                        self.stack.push(StackItem::Trigger {
                            source: card_id,
                            controller: caster,
                            effect: Box::new(Effect::Move {
                                what: crate::effect::Selector::This,
                                to: crate::effect::ZoneDest::Graveyard,
                            }),
                            target: None,
                            mode: None,
                            x_value: 0,
                            converged_value: 0,
                            trigger_source: None,
                            mana_spent: 0,
                            event_amount: 0,
                            intervening_if: None,
                        });
                    }

                    // Dash (CR 702.110): the dashed creature gains haste and
                    // returns to its owner's hand at the beginning of the next
                    // end step. Grant haste on the entering instance and arm
                    // the delayed bounce.
                    if dashed
                        && let Some(c) = self.battlefield.iter_mut().find(|c| c.id == card_id)
                    {
                        if !c.granted_keywords_eot.contains(&Keyword::Haste) {
                            c.granted_keywords_eot.push(Keyword::Haste);
                        }
                        self.delayed_triggers.push(crate::game::types::DelayedTrigger {
                            controller: caster,
                            source: card_id,
                            kind: crate::game::types::DelayedKind::NextEndStep,
                            effect: Effect::Move {
                                what: crate::effect::Selector::This,
                                to: crate::effect::ZoneDest::Hand(
                                    crate::effect::PlayerRef::OwnerOf(Box::new(
                                        crate::effect::Selector::This,
                                    )),
                                ),
                            },
                            target: None,
                            fires_once: true,
                        });
                    }

                    // Blitz (CR 702.152): the blitzed creature gains haste and
                    // "When this creature dies, draw a card," and is sacrificed
                    // at the beginning of the next end step. Grant haste on the
                    // entering instance and arm the two delayed triggers.
                    if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == card_id)
                        && c.blitzed
                    {
                        if !c.granted_keywords_eot.contains(&Keyword::Haste) {
                            c.granted_keywords_eot.push(Keyword::Haste);
                        }
                        self.delayed_triggers.push(crate::game::types::DelayedTrigger {
                            controller: caster,
                            source: card_id,
                            kind: crate::game::types::DelayedKind::WhenCardDies(card_id),
                            effect: Effect::Draw {
                                who: crate::effect::Selector::You,
                                amount: crate::effect::Value::Const(1),
                            },
                            target: None,
                            fires_once: true,
                        });
                        self.delayed_triggers.push(crate::game::types::DelayedTrigger {
                            controller: caster,
                            source: card_id,
                            kind: crate::game::types::DelayedKind::NextEndStep,
                            effect: Effect::SacrificeSource,
                            target: None,
                            fires_once: true,
                        });
                    }

                    // Suspend (CR 702.62f): a creature cast off its last time
                    // counter gains haste. The flag rode the instance from
                    // exile; clear it once consumed.
                    if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == card_id)
                        && c.cast_from_suspend
                    {
                        c.cast_from_suspend = false;
                        if !c.granted_keywords_eot.contains(&Keyword::Haste) {
                            c.granted_keywords_eot.push(Keyword::Haste);
                        }
                    }

                    // Push ETB triggers onto the stack — Elesh Norn
                    // replacement adjusts the trigger count (0 = suppressed
                    // by opponent's Norn, 1+N = each of your Norns adds an
                    // extra fire). The spell's `x_value` is threaded so
                    // ETB-trigger expressions like `Effect::AddCounter
                    // { amount: Value::XFromCost }` (Pterafractyl, Static
                    // Prison) read the actual paid X.
                    let etb_multiplier = crate::game::actions::etb_trigger_multiplier(
                        self,
                        caster,
                        Some(card_id),
                    );
                    for effect in etb_triggers {
                        // Strict Proctor's CR 614 tax — pay {amount} or
                        // sacrifice the source. Applied once per fire.
                        if !crate::game::actions::apply_etb_trigger_tax(
                            self, card_id, caster,
                        ) {
                            // Source sacrificed; remaining ETB triggers moot.
                            break;
                        }
                        let auto_target = self.auto_target_for_effect_avoiding(
                            &effect,
                            caster,
                            Some(card_id),
                        );
                        // CR 700.2b — modal ETB trigger mode pick at
                        // push-time (Biblioplex Tomekeeper's "choose up
                        // to one — prepare / unprepare").
                        let mode = self.pick_trigger_mode(&effect, card_id);
                        for _ in 0..etb_multiplier {
                            self.stack.push(StackItem::Trigger {
                                source: card_id,
                                controller: caster,
                                effect: Box::new(effect.clone()),
                                target: auto_target.clone(),
                                mode,
                                x_value,
                                converged_value,
                                trigger_source: Some(
                                    crate::game::effects::EntityRef::Permanent(card_id),
                                ),
                                mana_spent,
                                event_amount: 0,
                                intervening_if: None,
                            });
                        }
                    }

                    // CR 714.2b — a Saga enters with its first lore counter;
                    // chapter I fires off the same lore-counter placement.
                    let is_saga = self
                        .battlefield
                        .iter()
                        .any(|c| c.id == card_id && !c.definition.saga_chapters.is_empty());
                    if is_saga {
                        self.saga_advance(card_id);
                    }

                    // AnotherOfYours creature-ETB triggers are dispatched
                    // by the unified event pipeline (`dispatch_triggers_
                    // for_events` reading the `PermanentEntered` event).
                    // The synchronous push that used to live here was a
                    // duplicate — it both bypassed the `EventSpec.filter`
                    // (no CR 603.4 'if' check) and left `trigger_source`
                    // unset, so cards like Silverquill Chastiser ("when
                    // another Inkling ETBs, drain 1") double-fired with
                    // their filter ignored. Removed in push (modern_decks
                    // current revision) so the dispatcher handles it as
                    // the sole source of truth.
                } else {
                    let chosen_mode = mode.unwrap_or(0);
                    // CR 702.15 — Radiant Scrollwielder: while this instant/
                    // sorcery resolves, its controller gains life from any
                    // damage it deals. Stamp the seat so `deal_damage_to_from`
                    // can credit it, then clear after resolution.
                    let is_is = card.definition.card_types.iter().any(|t| {
                        matches!(t, crate::card::CardType::Instant | crate::card::CardType::Sorcery)
                    });
                    self.resolving_spell_lifelink_seat =
                        (is_is && self.controller_grants_spell_lifelink(caster)).then_some(caster);
                    let mut spell_events = self.continue_spell_resolution(
                        card,
                        caster,
                        target,
                        additional_targets,
                        chosen_mode,
                        x_value,
                        converged_value,
                        mana_spent,
                        None,
                    )?;
                    self.resolving_spell_lifelink_seat = None;
                    events.append(&mut spell_events);
                    if self.pending_decision.is_some() {
                        return Ok(events);
                    }
                }

                // SpellCast / YourControl triggers (Prowess, Magecraft,
                // Repartee, …) fire at *cast time* now (see
                // `finalize_cast`). The post-resolve fire here would
                // double-fire them. Kept the call site as a placeholder
                // for any future "after spell resolves" trigger types.
                let _ = (caster, card_id, is_noncreature);
            }
            StackItem::Trigger {
                source,
                controller,
                effect,
                target,
                mode,
                x_value,
                converged_value,
                trigger_source,
                mana_spent,
                event_amount,
                intervening_if,
            } => {
                // CR 603.4 — re-check the intervening 'if' clause as the
                // ability resolves. "If the condition isn't true at that
                // time, the ability is removed from the stack and does
                // nothing." We pop the trigger off the stack the same way
                // (the resolver caller already drained the StackItem) but
                // skip running its `effect`.
                if let Some(pred) = &intervening_if {
                    let mut ctx = crate::game::effects::EffectContext::for_trigger(
                        source,
                        controller,
                        target.clone(),
                        mode.unwrap_or(0),
                    );
                    ctx.trigger_source = trigger_source;
                    ctx.event_amount = event_amount;
                    ctx.x_value = x_value;
                    if !self.evaluate_predicate(pred, &ctx) {
                        // Trigger fizzles — no effect, no events.
                        let mut sba = self.check_state_based_actions();
                        events.append(&mut sba);
                        return Ok(events);
                    }
                }
                let chosen_mode = mode.unwrap_or(0);
                // CR 603.10 — scope the leaves-battlefield LKI read to this
                // resolution so `Value::PowerOf(This)` on a "when this dies"
                // body sees the dying object's last on-battlefield P/T.
                let had_lki = self.leaves_bf_lki.contains_key(&source);
                if had_lki {
                    self.resolving_lki_source = Some(source);
                }
                let mut trig_events = self.continue_trigger_resolution_with_source(
                    source,
                    controller,
                    *effect,
                    target,
                    chosen_mode,
                    x_value,
                    converged_value,
                    mana_spent,
                    trigger_source,
                    event_amount,
                )?;
                if had_lki {
                    self.resolving_lki_source = None;
                    self.leaves_bf_lki.remove(&source);
                }
                events.append(&mut trig_events);
                if self.pending_decision.is_some() {
                    return Ok(events);
                }
            }
        }

        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);

        Ok(events)
    }

    // ── Automatic step effects ────────────────────────────────────────────────

    /// CR 728.2 / 122.1i — rad-counter turn-based action. As the active
    /// player begins their precombat main phase, if they have any rad
    /// counters they mill that many cards; for each *nonland* card milled
    /// this way they lose 1 life and remove one rad counter.
    pub(crate) fn do_rad_counters(&mut self) -> Vec<GameEvent> {
        use crate::card::CardType;
        let p = self.active_player_idx;
        let mut events = Vec::new();
        let n = self.players[p].rad_counters;
        if n == 0 {
            return events;
        }
        for _ in 0..n {
            if self.players[p].library.is_empty() {
                break;
            }
            let card = self.players[p].library.remove(0);
            let cid = card.id;
            let is_land = card.definition.card_types.contains(&CardType::Land);
            if !self.route_to_graveyard(card, &mut events) {
                events.push(GameEvent::CardMilled { player: p, card_id: cid });
            }
            if !is_land {
                self.players[p].rad_counters = self.players[p].rad_counters.saturating_sub(1);
                self.adjust_life(p, -1);
                events.push(GameEvent::LifeLost { player: p, amount: 1 });
            }
        }
        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);
        events
    }

    /// CR 702.26 / 502.1 — phasing turn-based action at the start of the
    /// untap step (before permanents untap). Permanents the active player
    /// controls that are phased out phase in; permanents they control with
    /// phasing (plus anything attached to them) phase out. Phasing in/out is
    /// not a zone change: no ETB/LTB triggers fire and all state is retained,
    /// modelled by moving the `CardInstance` between `battlefield` and
    /// `phased_out` rather than re-creating it.
    pub(crate) fn do_phasing(&mut self) {
        let p = self.active_player_idx;
        // Direct phasers currently in play (computed *before* phase-in so a
        // permanent that phases in this step doesn't immediately phase back
        // out), plus any Aura/Equipment attached to one of them.
        let mut to_phase_out: std::collections::HashSet<crate::card::CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == p && c.has_keyword(&crate::card::Keyword::Phasing))
            .map(|c| c.id)
            .collect();
        if !to_phase_out.is_empty() {
            let attached: Vec<crate::card::CardId> = self
                .battlefield
                .iter()
                .filter(|c| c.attached_to.is_some_and(|h| to_phase_out.contains(&h)))
                .map(|c| c.id)
                .collect();
            to_phase_out.extend(attached);
        }
        // Phase IN: every phased-out permanent this player controls returns.
        let mut phased_in: Vec<crate::card::CardId> = Vec::new();
        let mut i = 0;
        while i < self.phased_out.len() {
            if self.phased_out[i].controller == p {
                let c = self.phased_out.remove(i);
                phased_in.push(c.id);
                self.battlefield.push(c);
            } else {
                i += 1;
            }
        }
        // Phase OUT the set captured above.
        if !to_phase_out.is_empty() {
            let mut idx = 0;
            while idx < self.battlefield.len() {
                if to_phase_out.contains(&self.battlefield[idx].id) {
                    let c = self.battlefield.remove(idx);
                    self.phased_out.push(c);
                } else {
                    idx += 1;
                }
            }
        }
        // CR 702.26 — "when this phases in" triggers. Phasing in isn't an ETB,
        // so we dispatch a dedicated `PermanentPhasedIn` event for each.
        if !phased_in.is_empty() {
            let evs: Vec<GameEvent> = phased_in
                .into_iter()
                .map(|card_id| GameEvent::PermanentPhasedIn { card_id })
                .collect();
            self.dispatch_triggers_for_events(&evs);
        }
    }

    pub(crate) fn do_untap(&mut self) {
        // CR 502.1 — phasing happens first, as a turn-based action.
        self.do_phasing();
        let p = self.active_player_idx;
        // Untap permanents YOU CONTROL on your untap step, not just
        // those you originally owned. A creature you've stolen
        // (Threaten / Mind Control) untaps on your turn; one of yours
        // that's been stolen does not. Filtering by `owner` here would
        // leave stolen permanents permanently tapped (or, conversely,
        // un-tap a stolen permanent on the wrong player's turn).
        //
        // CR 701.46a / 122.1d — Stun counters interpose on the untap
        // event: "If a permanent with one or more stun counters on it
        // would become untapped, instead remove one stun counter from
        // it." Implemented here by replacing the per-permanent untap
        // with "remove one stun counter if present, otherwise flip
        // tapped → false". Summoning sickness still clears
        // unconditionally since CR 302.1 / 506.4 attaches that to the
        // turn boundary, not the untap event.
        //
        // CR 502.3 untap-prevention — pre-compute the set of permanent
        // ids that are blocked from untapping this step by collecting
        // `StaticEffect::PreventUntap` selectors and intersecting them
        // with controlled permanents. Summoning sickness still clears
        // independently per CR 506.4 — the prevention only blocks the
        // tapped→untapped flip, not the sickness clearance.
        use crate::card::CounterType;
        use crate::card::SelectionRequirement;
        use crate::effect::StaticEffect;
        // CR 502.3 — Seedborn Muse: any player *other* than the active player
        // who controls an `UntapAllYoursEachUntapStep` permanent also untaps
        // their permanents during this untap step.
        let untappers: Vec<usize> = {
            let mut u = vec![p];
            for c in &self.battlefield {
                if c.controller != p
                    && !u.contains(&c.controller)
                    && c.definition.static_abilities.iter().any(|sa| {
                        matches!(sa.effect, StaticEffect::UntapAllYoursEachUntapStep)
                    })
                {
                    u.push(c.controller);
                }
            }
            u
        };
        let prevented: std::collections::HashSet<crate::card::CardId> = {
            let mut blocked = std::collections::HashSet::new();
            // Walk static abilities in play and OR each PreventUntap
            // selector's match set into the blocked set. `Selector::This`
            // blocks the static's own source (Basalt/Grim Monolith);
            // `EachPermanent(req)` blocks every matching permanent.
            let mut prevent_filters: Vec<SelectionRequirement> = Vec::new();
            for c in &self.battlefield {
                for sa in &c.definition.static_abilities {
                    match &sa.effect {
                        StaticEffect::PreventUntap {
                            applies_to: crate::effect::Selector::This,
                        } => {
                            blocked.insert(c.id);
                        }
                        StaticEffect::PreventUntap {
                            applies_to: crate::effect::Selector::EachPermanent(req),
                        } => prevent_filters.push(req.clone()),
                        // Aura-anchored prevention ("enchanted creature doesn't
                        // untap" — Claustrophobia): block the permanent this
                        // source is attached to.
                        StaticEffect::PreventUntap {
                            applies_to: crate::effect::Selector::AttachedTo(inner),
                        } if matches!(**inner, crate::effect::Selector::This) => {
                            if let Some(host) = c.attached_to {
                                blocked.insert(host);
                            }
                        }
                        _ => {}
                    }
                }
            }
            if !prevent_filters.is_empty() {
                for c in &self.battlefield {
                    if !untappers.contains(&c.controller) {
                        continue;
                    }
                    for req in &prevent_filters {
                        if self.evaluate_requirement(
                            req,
                            &crate::game::types::Target::Permanent(c.id),
                            c.controller,
                        ) {
                            blocked.insert(c.id);
                            break;
                        }
                    }
                }
            }
            blocked
        };
        // Track which permanents actually flip tapped→untapped so we can
        // fire CR 702.108 Inspired ("becomes untapped") triggers afterward.
        let mut untapped_now: Vec<crate::card::CardId> = Vec::new();
        for card in &mut self.battlefield {
            if untappers.contains(&card.controller) {
                // Summoning sickness clears only for the *active* player at the
                // turn boundary (CR 302.1 / 506.4). A Seedborn-untapped
                // permanent (controlled by another player) untaps but does not
                // shed sickness on someone else's turn.
                let active = card.controller == p;
                if prevented.contains(&card.id) {
                    // CR 502.3 — untap is prevented. Summoning sickness still
                    // clears per CR 506.4 (the turn-boundary tag, not the
                    // untap event).
                    if active {
                        card.summoning_sick = false;
                    }
                    continue;
                }
                // CR 702.83 — an exerted creature skips this untap. The flag
                // is one-shot: clear it so the creature untaps normally next
                // turn. No tapped→untapped flip, so no Inspired trigger.
                if card.skip_next_untap {
                    card.skip_next_untap = false;
                    if active {
                        card.summoning_sick = false;
                    }
                    continue;
                }
                if card.counter_count(CounterType::Stun) > 0 {
                    card.remove_counters(CounterType::Stun, 1);
                } else {
                    if card.tapped {
                        untapped_now.push(card.id);
                    }
                    card.tapped = false;
                }
                if active {
                    card.summoning_sick = false;
                }
            }
        }
        // CR 701.38 — goad lasts "until your next turn." When the goader's
        // (= active player p's) turn begins, drop their goad on every
        // creature so the must-attack requirement lifts.
        for card in &mut self.battlefield {
            card.goaded_by.retain(|&g| g != p);
            // CR 701.35 — detain lasts "until your next turn"; lift it when the
            // detaining player's (= active player p's) turn begins.
            if card.detained_by == Some(p) {
                card.detained_by = None;
            }
            // CR 702.142 — "attacked this turn" (Boast gate) resets each turn.
            card.attacked_this_turn = false;
        }
        self.players[p].lands_played_this_turn = 0;
        self.players[p].graveyard_cast_types_this_turn.clear();
        self.players[p].extra_land_plays = 0;
        // Raid (CR 702.108): the active player hasn't attacked yet this turn.
        self.players[p].attacked_this_turn = false;
        self.players[p].creatures_attacked_this_turn = 0;
        self.players[p].spells_cast_this_turn = 0;
        // Reset the Bloodthirst "damaged this turn" flag for *every* player
        // at the turn boundary (not just the active player) so a creature
        // cast on your turn reads damage dealt since this turn began.
        for pl in &mut self.players {
            pl.was_dealt_damage_this_turn = false;
            pl.lost_life_this_turn = false;
            pl.life_lost_this_turn = 0;
            pl.creatures_that_damaged_me_this_turn.clear();
            // Veil of Summer's "this turn" riders clear at the turn boundary
            // for every seat (CR 514.2 cleanup-scope grants).
            pl.spells_uncounterable_this_turn = false;
            pl.cast_blue_or_black_this_turn = false;
            pl.cant_cast_noncreature_this_turn = false;
            pl.silenced_this_turn = false;
        }
        // Reset Infusion / "if you gained life this turn" tracking for the
        // active player at the start of their turn. Other players' counters
        // tick down only at their own untaps so symmetric "this turn"
        // checks remain accurate per-player. (Same convention as
        // `lands_played_this_turn` and `spells_cast_this_turn`.)
        self.players[p].life_gained_this_turn = 0;
        // Reset cards-drawn tally for the active player. Powers Quandrix
        // scaling cards (Fractal Anomaly's "X = cards drawn this turn"
        // and similar). Other players' tallies advance independently
        // and are reset on their own untap.
        self.players[p].cards_drawn_this_turn = 0;
        // Reset the "cards left your graveyard this turn" tally; powers
        // Lorehold "if a card left your graveyard this turn" payoffs
        // (Living History, Primary Research, Wilt in the Heat) per turn.
        self.players[p].cards_left_graveyard_this_turn = 0;
        // Reset the "creatures died under your control this turn" tally;
        // powers Witherbloom "if a creature died under your control this
        // turn" end-step payoffs (Essenceknit Scholar).
        self.players[p].creatures_died_this_turn = 0;
        self.players[p].escalating_resolutions_this_turn = 0;
        // Reset the Revolt (CR 702.139) "permanent left the battlefield under
        // your control this turn" flag for the active player.
        self.players[p].permanent_left_battlefield_this_turn = false;
        // Reset the "cards exiled this turn" tally; powers Strixhaven
        // "if one or more cards were put into exile this turn" payoffs
        // (Ennis the Debate Moderator) per turn.
        self.players[p].cards_exiled_this_turn = 0;
        // Reset per-spell-type tallies (instant/sorcery vs creature
        // casts). These refine `spells_cast_this_turn` for cards that
        // need exact-type filtering (Potioner's Trove "instant or
        // sorcery only" gate, future Magecraft variants).
        self.players[p].instants_or_sorceries_cast_this_turn = 0;
        // One-shot IS-spell discounts are keyed off that tally, so they must
        // be cleared in lockstep with it (a stale `granted_at == 0` entry
        // would otherwise re-match after the reset).
        self.players[p].pending_is_discounts.clear();
        self.players[p].creatures_cast_this_turn = 0;
        // Clear Teferi, Time Raveler's "you may cast sorceries as though they
        // had flash" flag — it expires on the start of your next turn.
        self.players[p].sorceries_as_flash = false;
        // Clear "this turn" lifegain locks across **every player** — CR
        // "this turn" means the current turn, so a Skullcrack-style
        // lock set during the previous turn expires before priority
        // hits the new active player. Scoped wider than the per-player
        // counters above because the lock applies to whichever player
        // was targeted, not to the active player.
        for q in 0..self.players.len() {
            self.players[q].cannot_gain_life_this_turn = false;
        }
        // CR 702.108 — fire "becomes untapped" (Inspired) triggers for every
        // permanent that flipped tapped→untapped this step.
        if !untapped_now.is_empty() {
            let events: Vec<GameEvent> = untapped_now
                .into_iter()
                .map(|card_id| GameEvent::PermanentUntapped { card_id })
                .collect();
            self.dispatch_triggers_for_events(&events);
        }
    }

    /// CR 514 cleanup step. Returns `true` if it suspended on a pending
    /// decision (a `wants_ui` player choosing which cards to discard down to
    /// maximum hand size) — in which case the caller must return early and
    /// `submit_decision` will resume `finish_cleanup` + the step advance once
    /// the discard is answered. Returns `false` when cleanup completed
    /// synchronously.
    pub(crate) fn do_cleanup(&mut self) -> bool {
        // CR 514.1 — First, if the active player's hand contains more cards
        // than their maximum hand size (normally seven), they discard
        // enough cards to reduce their hand size to that number. This
        // turn-based action doesn't use the stack.
        //
        // For a `wants_ui` player we surface an interactive
        // `Decision::Discard` so the player picks which cards to pitch (and
        // suspend). For non-UI seats (tests, the bot's AutoDecider fallback)
        // we keep the deterministic first-card dump, routed through the
        // centralized `discard_card` path so the discard fires
        // `CardDiscarded` (CR 514.3 lets discard-matters triggers fire from
        // cleanup) and honors Madness (CR 702.35).
        let active = self.active_player_idx;
        // CR 402.2 — "Each player's maximum hand size is normally seven
        // cards. A player may have any number of cards in their hand, but as
        // part of their cleanup step, the player must discard excess cards
        // down to the maximum hand size." `Player.max_hand_size` is `None`
        // for "no maximum hand size" effects (skip entirely) and `Some(n)`
        // otherwise (discard down to `n`).
        if let Some(max) = self.effective_max_hand_size(active)
            && self.players[active].hand.len() > max
        {
            if self.players[active].wants_ui {
                let excess = (self.players[active].hand.len() - max) as u32;
                self.set_cleanup_discard_decision(active, excess);
                return true;
            }
            let mut cleanup_events = Vec::new();
            while self.players[active].hand.len() > max {
                let Some(cid) = self.players[active].hand.first().map(|c| c.id) else {
                    break;
                };
                self.discard_card(active, cid, &mut cleanup_events);
            }
            if !cleanup_events.is_empty() {
                self.dispatch_triggers_for_events(&cleanup_events);
            }
        }

        self.finish_cleanup();
        false
    }

    /// Pose the CR 514.1 cleanup discard-down as an interactive decision:
    /// the active player picks exactly `excess` cards from hand to discard.
    pub(crate) fn set_cleanup_discard_decision(&mut self, player: usize, excess: u32) {
        let hand: Vec<(CardId, String)> = self.players[player]
            .hand
            .iter()
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        self.pending_decision = Some(crate::game::types::PendingDecision {
            decision: Decision::Discard { player, count: excess, hand },
            resume: crate::game::types::ResumeContext::CleanupDiscard { player },
        });
    }

    /// CR 514.2 onward — the part of cleanup that runs after the discard-down
    /// step (which may have suspended for a UI player). Clears "until end of
    /// turn" state, removes marked damage, advances to the next player's turn,
    /// and hands them priority.
    pub(crate) fn finish_cleanup(&mut self) {
        // CR 514.2 — Second, the following actions happen simultaneously:
        // all damage marked on permanents is removed and all "until end of
        // turn" and "this turn" effects end.
        // Clear temporary pump effects (CardInstance-level bonuses still used as base)
        for card in &mut self.battlefield {
            card.clear_end_of_turn_effects();
        }
        // Until-end-of-turn flashback grants (SOS "Flashback") live on
        // graveyard cards, which `clear_end_of_turn_effects` above doesn't
        // reach — expire them here so the window closes at end of turn.
        for player in &mut self.players {
            for card in &mut player.graveyard {
                card.granted_flashback_eot = None;
            }
        }
        // Expire UntilEndOfTurn continuous effects from the layer system
        self.expire_end_of_turn_effects();
        // Snap control of EOT-stolen permanents (Act of Treason / Threaten)
        // back to their pre-steal controllers (CR 800.4).
        self.revert_temporary_control(&[
            crate::effect::Duration::EndOfTurn,
            crate::effect::Duration::UntilNextTurn,
        ]);
        // CR 707 — "becomes a copy ... until end of turn" swaps snap back.
        self.revert_temporary_copies(&[
            crate::effect::Duration::EndOfTurn,
            crate::effect::Duration::UntilNextTurn,
        ]);
        // CR 702.143b — foretold-this-turn cards become castable next turn.
        self.foretold_this_turn.clear();
        self.plotted_this_turn.clear();
        // CR 603.3d — "triggers only once each turn" abilities reset.
        self.triggered_once_per_turn_used.clear();
        // CR 505.1b — discard any unconsumed additional combat phases so they
        // don't bleed into the next turn (e.g. the turn ended before combat).
        self.additional_combat_phases = 0;
        // Clear all damage from creatures
        for card in &mut self.battlefield {
            card.damage = 0;
        }
        // Clear the per-turn "permanents gained a counter this turn"
        // tracker (used by Fractal Tender's end-step trigger). Resetting
        // at cleanup is the canonical "until end of turn" scope.
        self.permanents_gained_counter_this_turn.clear();
        // Clear transient granted triggers (Rabid Attack, Root
        // Manipulation EOT-duration grants).
        self.granted_triggers_eot.clear();
        // Close the "if it would die this turn, exile it instead" window
        // (Wilt in the Heat).
        self.dies_to_exile_eot.clear();
        // Expire event-keyed "when [card] dies this turn" delayed triggers
        // that never fired (CR 603.4 — the "this turn" window closes).
        self.delayed_triggers.retain(|dt| {
            !matches!(
                dt.kind,
                crate::game::types::DelayedKind::WhenCardDies(_)
                    | crate::game::types::DelayedKind::CreatureYouControlEntersThisTurn
                    | crate::game::types::DelayedKind::YourNextSpellCastThisTurn
            )
        });
        // CR 514.2 / CR 615.1 — "this turn" combat damage prevention
        // (Owlin Shieldmage's ETB, Holy Day-style fogs) expires at
        // cleanup along with the other until-end-of-turn flags.
        self.prevent_combat_damage_this_turn = false;
        self.combat_damage_prevented_creatures.clear();
        self.creature_etb_steal_this_turn.clear();
        self.cant_block_pairs.clear();
        // CR 615 — prevention shields and the "can't be prevented" rider
        // are "this turn" effects; they expire at cleanup too.
        self.prevention_shields.clear();
        self.damage_cant_be_prevented_this_turn = false;
        // Empty mana pools
        for player in &mut self.players {
            player.mana_pool.empty();
        }
        // CR 500.7 — extra turns. If the active player banked an extra
        // turn (Time Walk, Ral Zarek's -7 emblem), keep the turn instead
        // of passing: consume one charge and just bump the turn number.
        let active = self.active_player_idx;
        // Remember the just-ended turn's active player for the CR 502.2
        // day/night turn-based check at the next untap.
        self.previous_turn_active = Some(active);
        if self.players[active].is_alive() && self.players[active].extra_turns > 0 {
            self.players[active].extra_turns -= 1;
            self.turn_number += 1;
        } else {
            // Advance to the next non-eliminated player's turn (TurnStarted
            // fires on Untap entry). If the next player has pending skip
            // turns (Ral Zarek's -7), decrement and skip past them — keep
            // walking until we find a player with no skip-turn debt.
            // Safety cap at `players.len()` iterations to avoid an
            // infinite loop in pathological "everyone skips" scenarios.
            let n_players = self.players.len();
            for _ in 0..n_players.max(1) {
                self.active_player_idx = self.next_alive_seat(self.active_player_idx);
                self.turn_number += 1;
                let skipped = self.players[self.active_player_idx].skip_turns;
                if skipped == 0 {
                    break;
                }
                self.players[self.active_player_idx].skip_turns = skipped - 1;
                // Loop again — the current player's turn was just consumed
                // by the skip and we advance to the next.
            }
        }
        // Sweep expired `may_play_until` permissions across every zone.
        // Runs *after* the turn-number bump so `elapsed = turn_number -
        // granted_turn` reflects the cleanups that have actually
        // completed. EndOfThisTurn → expires after one bump (elapsed
        // ≥ 1). EndOfControllersNextTurn → expires after one full
        // controller-turn loop (elapsed ≥ player_count) — in a 2p game
        // that's 2 turn bumps later, i.e. the controller's *next*
        // cleanup.
        let player_count = self.players.len() as u32;
        let turn_number = self.turn_number;
        let sweep = |c: &mut crate::card::CardInstance| {
            if let Some(perm) = c.may_play_until {
                let elapsed = turn_number.saturating_sub(perm.granted_turn);
                let expired = match perm.duration {
                    crate::card::MayPlayDuration::EndOfThisTurn => elapsed >= 1,
                    crate::card::MayPlayDuration::EndOfControllersNextTurn => {
                        elapsed >= player_count.max(1)
                    }
                };
                if expired {
                    c.may_play_until = None;
                    // The miracle alt-cost shares the permission's lifetime.
                    c.granted_alt_cast_cost_eot = None;
                }
            }
        };
        for c in self.battlefield.iter_mut() { sweep(c); }
        for c in self.exile.iter_mut() { sweep(c); }
        for p in self.players.iter_mut() {
            for c in p.hand.iter_mut() { sweep(c); }
            for c in p.graveyard.iter_mut() { sweep(c); }
            for c in p.library.iter_mut() { sweep(c); }
        }
        self.give_priority_to_active();
    }

    // ── State-based actions ───────────────────────────────────────────────────

    /// CR 701.15 — apply a regeneration shield: remove one shield, tap the
    /// permanent, remove it from combat (as both attacker and blocker), and
    /// heal all marked damage. The permanent stays on the battlefield.
    pub(crate) fn apply_regeneration(&mut self, id: CardId) {
        if let Some(c) = self.battlefield_find_mut(id) {
            c.regeneration_shields = c.regeneration_shields.saturating_sub(1);
            c.tapped = true;
            c.damage = 0;
            c.dealt_deathtouch_damage = false;
        }
        // Remove from combat: drop it as a declared attacker and as a blocker.
        self.attacking.retain(|atk| atk.attacker != id);
        self.block_map.remove(&id);
        self.block_map.retain(|_, atk| *atk != id);
    }

    /// CR 800.4a — handle a player leaving the game: all cards/tokens they
    /// own leave with them (every zone), and permanents they controlled but
    /// don't own revert to their owners' control. Objects leaving this way
    /// are removed directly (not via the death/exile pipelines) since a
    /// departing player's objects "cease to exist" rather than being
    /// destroyed or sacrificed.
    pub(crate) fn objects_leave_with_player(&mut self, p: usize) {
        self.battlefield.retain(|c| c.owner != p);
        for c in &mut self.battlefield {
            if c.controller == p {
                c.controller = c.owner; // control-changing effects end
            }
        }
        self.exile.retain(|c| c.owner != p);
        self.players[p].hand.clear();
        self.players[p].library.clear();
        self.players[p].graveyard.clear();
        // CR 724.3 — if the monarch leaves the game, the active player
        // becomes the monarch (no monarch if the active player is the one
        // leaving).
        if self.monarch == Some(p) {
            self.monarch = if self.active_player_idx == p { None } else { Some(self.active_player_idx) };
        }
    }

    pub(crate) fn check_state_based_actions(&mut self) -> Vec<GameEvent> {
        let mut events = vec![];

        // +1/+1 and -1/-1 counters cancel each other out (CR 122.3 — the
        // SBA removes `N` of each kind, where `N` is the smaller count).
        for card in &mut self.battlefield {
            let plus = card
                .counters
                .get(&crate::card::CounterType::PlusOnePlusOne)
                .copied()
                .unwrap_or(0);
            let minus = card
                .counters
                .get(&crate::card::CounterType::MinusOneMinusOne)
                .copied()
                .unwrap_or(0);
            if plus > 0 && minus > 0 {
                let cancel = plus.min(minus);
                *card
                    .counters
                    .entry(crate::card::CounterType::PlusOnePlusOne)
                    .or_insert(0) -= cancel;
                *card
                    .counters
                    .entry(crate::card::CounterType::MinusOneMinusOne)
                    .or_insert(0) -= cancel;
            }
        }

        // CR 122.4 — "An effect can set the maximum number of counters of a
        // kind that a permanent can have." If a permanent has more than its
        // printed cap, the SBA prunes the excess down to the cap. Uses the
        // new `CardDefinition.max_counters_of_kind: Option<(CounterType,
        // u32)>` field — None ⇒ no cap, the default.
        for card in &mut self.battlefield {
            if let Some((kind, max)) = card.definition.max_counters_of_kind {
                let current = card.counters.get(&kind).copied().unwrap_or(0);
                if current > max {
                    *card.counters.entry(kind).or_insert(0) = max;
                }
            }
        }

        // Legend rule (CR 704.5j): if two+ legendaries with the same name
        // share a controller, that player chooses one to keep; the rest go to
        // their owners' graveyards. We group tied permanents, then consult the
        // controller's decider per group (AutoDecider keeps the newest).
        let legend_groups = {
            let mut order: Vec<(usize, &str)> = Vec::new();
            let mut groups: std::collections::HashMap<(usize, &str), Vec<(CardId, String)>> =
                std::collections::HashMap::new();
            // Walk descending by id so each group's vec is newest-first.
            let mut by_id: Vec<_> = self
                .battlefield
                .iter()
                .filter(|c| c.definition.supertypes.contains(&Supertype::Legendary))
                .collect();
            by_id.sort_by_key(|b| std::cmp::Reverse(b.id));
            for c in by_id {
                let key = (c.controller, c.definition.name);
                groups
                    .entry(key)
                    .or_insert_with(|| {
                        order.push(key);
                        Vec::new()
                    })
                    .push((c.id, c.definition.name.to_string()));
            }
            let mut out = Vec::new();
            for k in order {
                let dups = groups.remove(&k).unwrap_or_default();
                if dups.len() > 1 {
                    out.push((k.0, k.1.to_string(), dups));
                }
            }
            out
        };
        let legend_victims: Vec<CardId> = {
            let mut victims = Vec::new();
            for (player, name, duplicates) in legend_groups {
                // Ask the controller which to keep; default keeps newest.
                let kept = match self.decider.decide(&crate::decision::Decision::ChooseLegendToKeep {
                    player,
                    name,
                    duplicates: duplicates.clone(),
                }) {
                    crate::decision::DecisionAnswer::KeptLegend(id)
                        if duplicates.iter().any(|(d, _)| *d == id) =>
                    {
                        id
                    }
                    // Out-of-set / wrong answer → keep newest (highest id).
                    _ => duplicates.iter().map(|(id, _)| *id).max().unwrap_or(CardId(0)),
                };
                for (id, _) in &duplicates {
                    if *id != kept {
                        victims.push(*id);
                    }
                }
            }
            victims
        };
        for id in legend_victims {
            // Cache snapshot before zone change so AnotherOfYours-scope
            // triggers off legend-rule deaths see the right player AND
            // can introspect the dying card's printed types. Only a *creature*
            // dies (CR 700.4) — a legend-ruled planeswalker/artifact/enchant
            // leaves the battlefield without a CreatureDied event.
            if let Some(c) = self.battlefield.iter().find(|c| c.id == id) {
                if c.definition.is_creature() {
                    events.push(GameEvent::CreatureDied { card_id: id });
                }
                self.died_card_snapshots.insert(id, c.clone());
            }
            self.remove_from_battlefield_to_graveyard(id);
        }

        // World rule (CR 704.5k): if two or more permanents have the World
        // supertype, all except the one that has been a World permanent for
        // the shortest time (the newest, i.e. highest CardId) go to their
        // owners' graveyards. Unlike the legend rule this is global, not
        // per-controller.
        let world_victims: Vec<CardId> = {
            let worlds: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| c.definition.supertypes.contains(&Supertype::World))
                .map(|c| c.id)
                .collect();
            if worlds.len() > 1 {
                let keep = worlds.iter().copied().max().unwrap();
                worlds.into_iter().filter(|id| *id != keep).collect()
            } else {
                Vec::new()
            }
        };
        for id in world_victims {
            if let Some(c) = self.battlefield.iter().find(|c| c.id == id) {
                self.died_card_snapshots.insert(id, c.clone());
            }
            self.remove_from_battlefield_to_graveyard(id);
        }

        // Saga rule (CR 714.4 / 704.5x): a Saga whose lore counters have
        // reached its final chapter number is sacrificed — unless one of its
        // chapter abilities is still a trigger on the stack (so the last
        // chapter resolves before the Saga leaves).
        let saga_victims: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| {
                let Some(final_ch) = c.definition.saga_chapters.iter().map(|(n, _)| *n).max() else {
                    return false;
                };
                if c.counter_count(crate::card::CounterType::Lore) < final_ch {
                    return false;
                }
                // Still the source of a chapter ability on the stack?
                !self.stack.iter().any(|item| {
                    matches!(item, StackItem::Trigger { source, .. } if *source == c.id)
                })
            })
            .map(|c| c.id)
            .collect();
        for id in saga_victims {
            if let Some(c) = self.battlefield.iter().find(|c| c.id == id) {
                self.died_card_snapshots.insert(id, c.clone());
            }
            events.push(GameEvent::PermanentSacrificed {
                card_id: id,
                who: self.battlefield.iter().find(|c| c.id == id).map(|c| c.controller).unwrap_or(0),
            });
            self.remove_from_battlefield_to_graveyard(id);
        }

        // Collect dead creatures using layer-computed toughness.
        let computed = self.compute_battlefield();
        let dead: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| {
                if !c.definition.is_creature() {
                    return false;
                }
                // Indestructible stops destruction by damage but NOT by toughness ≤ 0.
                let computed_toughness = computed
                    .iter()
                    .find(|cp| cp.id == c.id)
                    .map(|cp| cp.toughness)
                    .unwrap_or(c.toughness());
                // Toughness ≤ 0 kills even indestructible creatures.
                if computed_toughness <= 0 {
                    return true;
                }
                // CR 704.5g: lethal damage = damage >= toughness.
                // CR 704.5h: any damage from a deathtouch source is lethal.
                // Indestructible creatures (keyword or counter) don't die to
                // either rule.
                if c.is_indestructible() {
                    return false;
                }
                if (c.damage as i32) >= computed_toughness {
                    return true;
                }
                c.dealt_deathtouch_damage && c.damage > 0
            })
            .map(|c| c.id)
            .collect();

        // Hushbringer (CR 614): suppress creature-death triggers while a
        // `SuppressCreatureEtbTriggers { also_dies }` static is in play.
        let dies_suppressed = crate::game::actions::creature_dies_triggers_suppressed(self);

        for id in dead {
            // CR 701.15 — regeneration shields replace destruction by
            // *damage* (lethal damage / deathtouch), but never destruction
            // from toughness ≤ 0 (that's a separate SBA, not a "destroy").
            // A surviving shield taps the creature, removes it from combat,
            // and heals marked damage instead of letting it die.
            let dies_by_lethal_toughness = self
                .battlefield
                .iter()
                .find(|c| c.id == id)
                .map(|c| {
                    let ct = computed
                        .iter()
                        .find(|cp| cp.id == id)
                        .map(|cp| cp.toughness)
                        .unwrap_or_else(|| c.toughness());
                    ct <= 0
                })
                .unwrap_or(false);
            let has_regen = self
                .battlefield
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.regeneration_shields > 0)
                .unwrap_or(false);
            if has_regen && !dies_by_lethal_toughness {
                self.apply_regeneration(id);
                continue;
            }

            events.push(GameEvent::CreatureDied { card_id: id });
            // Cache the dying card's snapshot so AnotherOfYours-scope
            // triggers AND printed-type filter predicates fire reliably
            // even for tokens. CR 111.7c's
            // "ceases to exist" SBA removes the token from every zone in
            // the same sweep — by dispatch time the zone-walking lookup
            // returns None. The cached `CardInstance` survives the sweep
            // and is consulted by `event_matches_spec` (controller lookup)
            // and `evaluate_requirement_static` (type/keyword/counter
            // filter). Cleared after `dispatch_triggers_for_events`.
            if let Some(c) = self.battlefield.iter().find(|c| c.id == id) {
                self.died_card_snapshots.insert(id, c.clone());
            }
            // Collect Dies triggers and Persist/Undying info before removing from battlefield.
            let (
                mut die_triggers,
                has_persist,
                has_undying,
                minus_count,
                plus_count,
                owner,
                controller_idx,
            ) = self
                .battlefield
                .iter()
                .find(|c| c.id == id)
                .map(|c| {
                    // CR 603.10a — "leaves-the-battlefield" triggers look
                    // back in time at the dying card. Only fire the dying
                    // card's own die-triggers whose scope says they can
                    // fire from self — i.e. SelfSource or YourControl /
                    // AnyPlayer. AnotherOfYours / OpponentControl /
                    // FromYourGraveyard are NOT self-fire scopes; skipping
                    // them here matches the printed Oracle semantics for
                    // "Whenever another creature you control dies" (must
                    // be another, not this dying card).
                    // Walk printed Dies triggers + any granted transient
                    // ones (Rabid Attack EOT "this creature gains 'die →
                    // draw a card'" grants ride on `granted_triggers_eot`).
                    let granted: &[crate::card::TriggeredAbility] = self
                        .granted_triggers_eot
                        .get(&c.id)
                        .map(Vec::as_slice)
                        .unwrap_or(&[]);
                    let triggers: Vec<(CardId, Effect, usize)> = c
                        .definition
                        .triggered_abilities
                        .iter()
                        .chain(granted)
                        // PermanentLeavesBattlefield ("when this leaves the
                        // battlefield") fires on any departure, including a
                        // lethal-damage death (Thought-Knot Seer); CreatureDied
                        // is suppressed by Hushbringer-style statics.
                        .filter(|t| match t.event.kind {
                            EventKind::CreatureDied => !dies_suppressed,
                            EventKind::PermanentLeavesBattlefield => true,
                            _ => false,
                        })
                        .filter(|t| matches!(
                            t.event.scope,
                            crate::effect::EventScope::SelfSource
                                | crate::effect::EventScope::YourControl
                                | crate::effect::EventScope::AnyPlayer
                                | crate::effect::EventScope::ActivePlayer,
                        ))
                        .map(|t| (c.id, t.effect.clone(), c.controller))
                        .collect();
                    let has_persist = c.definition.keywords.contains(&Keyword::Persist);
                    let has_undying = c.definition.keywords.contains(&Keyword::Undying);
                    let minus = c.counter_count(crate::card::CounterType::MinusOneMinusOne);
                    let plus = c.counter_count(crate::card::CounterType::PlusOnePlusOne);
                    (
                        triggers,
                        has_persist,
                        has_undying,
                        minus,
                        plus,
                        c.owner,
                        c.controller,
                    )
                })
                .unwrap_or_default();
            // CR 702.6e — Equipment-granted "dies" triggers fire as though
            // printed on the equipped creature (Skullclamp). Collect them
            // while the creature is still attached (pre-removal). Source is
            // the dying creature so `Selector::This` reads its last-known
            // info; controller is the creature's controller.
            for eq in &self.battlefield {
                if eq.attached_to != Some(id) {
                    continue;
                }
                let Some(bonus) = &eq.definition.equipped_bonus else { continue };
                for ta in &bonus.triggered_abilities {
                    if ta.event.kind == EventKind::CreatureDied && !dies_suppressed {
                        die_triggers.push((id, ta.effect.clone(), controller_idx));
                    }
                }
            }
            // Bump the controller's per-turn died-creature tally for
            // Witherbloom "if a creature died under your control this
            // turn" payoffs (Essenceknit Scholar).
            if controller_idx < self.players.len() {
                self.players[controller_idx].creatures_died_this_turn =
                    self.players[controller_idx].creatures_died_this_turn.saturating_add(1);
            }
            // CR 603.10 — stash an LKI snapshot before the creature leaves
            // so a "deals damage / makes tokens equal to its power" dies
            // body reads its counter-boosted P/T (Goldvein Hydra). Removed
            // when the trigger resolves (`resolve_top_of_stack`).
            if !die_triggers.is_empty()
                && let Some(c) = self.battlefield.iter().find(|c| c.id == id)
            {
                self.leaves_bf_lki.insert(id, c.clone());
            }
            self.remove_from_battlefield_to_graveyard(id);
            // Push Dies triggers to the stack for resolution.
            for (source, effect, controller) in die_triggers {
                let auto_target =
                    self.auto_target_for_effect_avoiding(&effect, controller, Some(source));
                self.stack.push(StackItem::Trigger {
                    source,
                    controller,
                    effect: Box::new(effect),
                    target: auto_target,
                    mode: None,
                    x_value: 0,
                    converged_value: 0,
                trigger_source: None,
                    mana_spent: 0,
                    event_amount: 0,
                    intervening_if: None,
                });
            }
            // Persist / Undying return (CR 702.79 / 702.92), shared with the
            // destroy / sacrifice funnels via `return_persist_undying`.
            self.return_persist_undying(
                id, owner, (has_persist, has_undying, minus_count, plus_count), &mut events,
            );
            let _ = controller_idx; // used via closure above
        }

        // Planeswalkers with 0 loyalty die (CR 704.5i).
        let pw_dead: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| {
                c.definition.is_planeswalker()
                    && c.counter_count(crate::card::CounterType::Loyalty) == 0
            })
            .map(|c| c.id)
            .collect();
        for id in pw_dead {
            events.push(GameEvent::PlaneswalkerDied { card_id: id });
            self.remove_from_battlefield_to_graveyard(id);
        }

        // CR 702.103e — a bestowed permanent whose enchanted creature has
        // left the battlefield is no longer an Aura; it stays in play and
        // reverts to a creature (clear `bestowed` + the attachment link).
        // Run before the orphan-Aura sweep so it isn't sent to the gy.
        let unbestowed: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.bestowed)
            .filter(|c| match c.attached_to {
                None => true,
                Some(host) => !self.battlefield.iter().any(|b| b.id == host),
            })
            .map(|c| c.id)
            .collect();
        for id in unbestowed {
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == id) {
                c.bestowed = false;
                c.attached_to = None;
            }
        }

        // Auras with no valid attachment target go to their owner's graveyard (CR 704.5n/5q).
        let orphaned_auras: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.definition.is_aura())
            .filter(|c| {
                match c.attached_to {
                    None => true, // not attached to anything
                    Some(attached_id) => !self.battlefield.iter().any(|b| b.id == attached_id),
                }
            })
            .map(|c| c.id)
            .collect();
        for id in orphaned_auras {
            // Fire any leaves-the-battlefield triggers on the Aura itself
            // (CR 603.6d) — e.g. Rancor's "return it to its owner's hand".
            events.append(&mut self.remove_to_graveyard_with_triggers(id));
        }

        // CR 704.5n — "If an Equipment or Fortification is attached to an
        // illegal permanent or to a player, it becomes unattached from
        // that permanent or player. It remains on the battlefield."
        // Illegal here means the attached card isn't on the battlefield
        // anymore (e.g. equipped creature died) OR the target permanent
        // is no longer a legal target (no creature subtype for Equipment).
        // The Equipment itself stays in play — only the link is cleared.
        let stale_equipment_links: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.definition.is_equipment())
            .filter_map(|c| {
                let attached = c.attached_to?;
                let is_still_legal = self
                    .battlefield
                    .iter()
                    .any(|b| b.id == attached && b.definition.is_creature());
                if !is_still_legal { Some(c.id) } else { None }
            })
            .collect();
        for id in stale_equipment_links {
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == id) {
                c.attached_to = None;
            }
        }

        // CR 702.95h — Soulbond pairs break when either creature leaves the
        // battlefield. Clear any link that points at a card no longer in play.
        let on_bf: std::collections::HashSet<CardId> =
            self.battlefield.iter().map(|c| c.id).collect();
        for c in &mut self.battlefield {
            if let Some(p) = c.soulbond_partner
                && !on_bf.contains(&p)
            {
                c.soulbond_partner = None;
            }
        }

        // CR 704.5d — a token that's not on the battlefield ceases to exist.
        // Dies / leaves-battlefield triggers have already fired by this point
        // (they queue into the events vec before this scan), so dropping the
        // token from its post-bf zone now matches the timing real MTG would
        // produce. Without this, dead tokens linger in graveyards (and would
        // count toward graveyard-size effects, mill prompts, etc.).
        for player in &mut self.players {
            player.graveyard.retain(|c| !c.is_token);
            player.hand.retain(|c| !c.is_token);
            player.library.retain(|c| !c.is_token);
        }
        self.exile.retain(|c| !c.is_token);

        // Player loss conditions (CR 704.5a/b/c). Eliminated players are
        // removed from turn/priority rotation; the game ends when ≤ 1
        // team alive (see surviving-teams check below).
        //
        // Phase F: `effective_life(i)` collapses the solo-life and
        // shared-pool (2HG) cases. When `Team.shared_life` is `Some(n)`,
        // both teammates' effective life is `n`, so dropping the pool
        // to ≤ 0 eliminates both members simultaneously (CR 810.8 +
        // 704.5a). Poison stays per-player (CR 810.7b — 2HG shares
        // life but not poison; an individual teammate hitting 10
        // poison still loses).
        let mut newly_eliminated: Vec<usize> = Vec::new();
        for i in 0..self.players.len() {
            if self.players[i].eliminated {
                continue;
            }
            // Phase M: 21-commander-damage SBA (CR 704.5v). Any
            // single (this-player, commander) entry of ≥ 21 in
            // `commander_damage` loses the game for this player. We
            // collect the check separately from life / poison so
            // the cause is debuggable.
            let lost_to_commander = self
                .commander_damage
                .iter()
                .any(|((victim, _), amt)| *victim == i && *amt >= 21);
            let lost = self.effective_life(i) <= 0
                || self.players[i].poison_counters >= 10
                || lost_to_commander;
            if lost {
                self.players[i].eliminated = true;
                newly_eliminated.push(i);
            }
        }
        // CR 800.4a — when a player leaves the game, every card and token
        // they own leaves with them, and permanents they controlled but
        // didn't own revert to their owners' control. (Stack items the
        // departed player controlled ceasing to exist is a remaining gap;
        // tracked in TODO.md.)
        for &p in &newly_eliminated {
            self.objects_leave_with_player(p);
        }

        // CR 104.2 / 810.7: the game ends when only one *team* has
        // players remaining (in solo-team formats — 1v1, FFA — a team
        // is one seat, so this reduces to "only one alive player").
        // Pre-Phase-G this checked alive seats directly, which in 2HG
        // would have ended the match as soon as one of the four
        // players died even though their teammate was still in.
        if self.game_over.is_none() {
            let alive: Vec<usize> = (0..self.players.len())
                .filter(|i| !self.players[*i].eliminated)
                .collect();
            let mut surviving_teams: Vec<crate::team::TeamId> = alive
                .iter()
                .map(|&s| self.team_of(s))
                .collect();
            surviving_teams.sort_by_key(|t| t.0);
            surviving_teams.dedup();
            match surviving_teams.len() {
                0 => {
                    self.game_over = Some(None);
                    events.push(GameEvent::GameOver { winner: None });
                }
                1 => {
                    // Report the winning team's first alive seat (by
                    // seat number) as the `winner`. For solo-team
                    // formats this is the literal winner; for 2HG it
                    // identifies the surviving team via a
                    // representative member, which is enough to let
                    // the server / UI resolve to a team result.
                    let winner_team = surviving_teams[0];
                    let mut reps: Vec<usize> = alive
                        .iter()
                        .copied()
                        .filter(|&s| self.team_of(s) == winner_team)
                        .collect();
                    reps.sort();
                    let winner = reps[0];
                    self.game_over = Some(Some(winner));
                    events.push(GameEvent::GameOver { winner: Some(winner) });
                }
                _ => {}
            }
        }

        events
    }

    /// CR 506.4 — A permanent is removed from combat if it leaves the
    /// battlefield. Called by every battlefield-removal path
    /// (`move_card_to`, `remove_from_battlefield_to_graveyard`,
    /// `remove_from_battlefield_to_exile`, etc.) so the post-removal
    /// combat state stays consistent. Prunes `self.attacking` (the
    /// attacker slot) and `self.block_map` (both blocker keys and
    /// attacker values).
    pub(crate) fn remove_from_combat(&mut self, id: CardId) {
        self.attacking.retain(|a| a.attacker != id);
        self.block_map
            .retain(|blocker, attacker| *blocker != id && *attacker != id);
    }

    pub(crate) fn remove_from_battlefield_to_graveyard(&mut self, id: CardId) {
        if let Some(pos) = self.battlefield.iter().position(|c| c.id == id) {
            let card = self.battlefield.remove(pos);
            self.remove_effects_from_source(id);
            self.remove_from_combat(id);
            // CR 122.1h — Finality counters redirect Battlefield →
            // Graveyard to Battlefield → Exile. Wilt in the Heat's "if
            // that creature would die this turn, exile it instead" rides
            // the same redirect via `dies_to_exile_eot`. We must check
            // both here because the card has been removed from the
            // battlefield before `resolve_zone_change` walks for it.
            // CR 614 — "If a nontoken creature an opponent controls would die,
            // exile it instead" (Valentin, Dean of the Vein). Detect an active
            // `ExileDyingOpponentCreatures` static controlled by an opponent of
            // the dying creature, redirect to exile, and capture its reflexive
            // "when you do" effect to fire after placement.
            let valentin_redirect: Option<(usize, Option<Effect>)> = if card.definition.is_creature()
                && !card.is_token
            {
                self.battlefield.iter().find_map(|src| {
                    src.definition.static_abilities.iter().find_map(|sa| {
                        if let crate::effect::StaticEffect::ExileDyingOpponentCreatures { when_you_do } =
                            &sa.effect
                            && src.controller != card.controller
                        {
                            Some((src.controller, when_you_do.as_deref().cloned()))
                        } else {
                            None
                        }
                    })
                })
            } else {
                None
            };
            let initial_to = if card.counter_count(crate::card::CounterType::Finality) > 0
                || self.dies_to_exile_eot.contains(&id)
                || valentin_redirect.is_some()
            {
                crate::card::Zone::Exile
            } else {
                crate::card::Zone::Graveyard
            };
            let resolved = self.resolve_zone_change(
                id,
                crate::card::Zone::Battlefield,
                initial_to,
            );
            // CR 702.69 — bump the turn's "permanents put into a graveyard
            // from the battlefield" tally for Gravestorm. Only when the
            // card actually landed in a graveyard (Finality / dies-to-exile
            // redirects don't count).
            if resolved == crate::card::Zone::Graveyard {
                self.permanents_to_graveyard_this_turn =
                    self.permanents_to_graveyard_this_turn.saturating_add(1);
            }
            // CR 702.139 — Revolt: a permanent left the battlefield under its
            // controller this turn.
            if card.controller < self.players.len() {
                self.players[card.controller].permanent_left_battlefield_this_turn = true;
            }
            self.place_card_at_resolved_zone(card, resolved);
            let mut events = Vec::new();
            self.return_linked_exiles(id, &mut events);
            // Fire Valentin's reflexive "when you do, …" for the static's
            // controller (CR 603.x reflexive trigger off the replacement).
            if let Some((controller, Some(effect))) = valentin_redirect {
                let auto_target =
                    self.auto_target_for_effect_avoiding(&effect, controller, None);
                self.stack.push(StackItem::Trigger {
                    source: id,
                    controller,
                    effect: Box::new(effect),
                    target: auto_target,
                    mode: None,
                    x_value: 0,
                    converged_value: 0,
                    trigger_source: None,
                    mana_spent: 0,
                    event_amount: 0,
                    intervening_if: None,
                });
            }
        }
    }

    pub(crate) fn remove_from_battlefield_to_exile(&mut self, id: CardId) {
        if let Some(pos) = self.battlefield.iter().position(|c| c.id == id) {
            let card = self.battlefield.remove(pos);
            self.remove_effects_from_source(id);
            self.remove_from_combat(id);
            let resolved = self.resolve_zone_change(
                id,
                crate::card::Zone::Battlefield,
                crate::card::Zone::Exile,
            );
            // CR 702.139 — Revolt: a permanent left the battlefield this turn.
            if card.controller < self.players.len() {
                self.players[card.controller].permanent_left_battlefield_this_turn = true;
            }
            self.place_card_at_resolved_zone(card, resolved);
            let mut events = Vec::new();
            self.return_linked_exiles(id, &mut events);
        }
    }

    /// Internal: drop `card` into `zone` (the result of a replacement
    /// resolver walk). Handles the terminal-zone shapes; for
    /// `Zone::Command` falls back to graveyard with a debug-assert
    /// until Phase I adds the per-player command zone storage.
    /// `Zone::Battlefield` / `Zone::Stack` likewise fall back —
    /// those shouldn't appear as legitimate redirect targets.
    pub(crate) fn place_card_at_resolved_zone(
        &mut self,
        card: CardInstance,
        zone: crate::card::Zone,
    ) {
        use crate::card::Zone;
        let owner = card.owner;
        // CR 702.95h — a card leaving the battlefield is no longer Soulbond-
        // paired. Clear its own link so a later re-entry can re-pair cleanly
        // (the SBA in `check_state_based_actions` clears the partner's side).
        let mut card = card;
        card.soulbond_partner = None;
        // CR 708.10 — a face-down permanent is turned face up as it leaves
        // the battlefield (no-op unless it carries a stashed real definition).
        card.turn_face_up();
        // CR 707 — a temporary copy reverts as it leaves.
        self.revert_copy_on_leave(&mut card);
        match zone {
            // CR 614.6 — Rest in Peace / Leyline of the Void redirect the
            // graveyard arrival to exile.
            Zone::Graveyard if self.graveyard_exiled_for(owner) => self.exile.push(card),
            Zone::Graveyard => self.players[owner].send_to_graveyard(card),
            Zone::Exile => self.exile.push(card),
            Zone::Hand => self.players[owner].hand.push(card),
            // Top of owner's library. Replacement effects don't carry
            // a position field today; if a future replacement needs
            // bottom / shuffled, extend the type.
            Zone::Library => self.players[owner].library.insert(0, card),
            Zone::Command => self.players[owner].command.push(card),
            Zone::Battlefield | Zone::Stack => {
                // Unsupported as a replacement redirect target — the
                // card has already lost its battlefield identity
                // (cleared damage / counters / continuous effects)
                // by the time we reach here. Fall back to graveyard.
                debug_assert!(
                    false,
                    "replacement redirect to Battlefield/Stack is unsupported"
                );
                self.players[owner].send_to_graveyard(card);
            }
        }
    }

    /// Remove a permanent from the battlefield to its graveyard and collect any
    /// `Dies` triggered abilities, returning them as events after the fact.
    /// (This is the version used by destroy/damage effects that want to fire triggers.)
    pub(crate) fn remove_to_graveyard_with_triggers(&mut self, id: CardId) -> Vec<GameEvent> {
        // Collect both `CreatureDied` and `PermanentLeavesBattlefield`
        // self-source triggers off the leaving permanent. CreatureDied
        // only matters for creatures (Solitude evoke-sac etc.);
        // PermanentLeavesBattlefield is the broader "when this leaves the
        // battlefield" hook used by Chromatic Star, Roomba-style cards,
        // and any future non-creature die-trigger.
        // Hushbringer (CR 614): creature-death triggers are suppressed while
        // a `SuppressCreatureEtbTriggers { also_dies }` static is in play.
        let dies_suppressed = crate::game::actions::creature_dies_triggers_suppressed(self);
        let (leave_triggers, dying_creature_controller): (Vec<(CardId, Effect, usize)>, Option<usize>) = self
            .battlefield
            .iter()
            .find(|c| c.id == id)
            .map(|c| {
                let is_creature = c.definition.is_creature();
                // Walk printed SelfSource LTB triggers + any transient
                // granted ones (Rabid Attack-style "this creature gains
                // 'when this creature dies, draw a card'" grants ride
                // on `granted_triggers_eot[c.id]`).
                let granted: &[crate::card::TriggeredAbility] = self
                    .granted_triggers_eot
                    .get(&c.id)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                let triggers = c.definition
                    .triggered_abilities
                    .iter()
                    .chain(granted)
                    .filter(|t| matches!(t.event.scope, EventScope::SelfSource))
                    .filter(|t| match t.event.kind {
                        EventKind::PermanentLeavesBattlefield => true,
                        EventKind::CreatureDied => is_creature && !dies_suppressed,
                        _ => false,
                    })
                    .map(|t| (c.id, t.effect.clone(), c.controller))
                    .collect();
                let creature_controller = if is_creature { Some(c.controller) } else { None };
                (triggers, creature_controller)
            })
            .unwrap_or_default();
        // Capture Persist/Undying info before the card leaves the battlefield.
        let (persist_has, undying_has, persist_minus, persist_plus, persist_owner) = self
            .battlefield
            .iter()
            .find(|c| c.id == id)
            .map(|c| {
                (
                    c.definition.keywords.contains(&Keyword::Persist),
                    c.definition.keywords.contains(&Keyword::Undying),
                    c.counter_count(crate::card::CounterType::MinusOneMinusOne),
                    c.counter_count(crate::card::CounterType::PlusOnePlusOne),
                    c.owner,
                )
            })
            .unwrap_or((false, false, 0, 0, 0));
        // Bump the controller's per-turn died-creature tally for
        // Witherbloom payoffs (Essenceknit Scholar). This path is the
        // standard destroy / damage-lethal route that bypasses the SBA
        // dies handler in `apply_state_based_actions`; we duplicate the
        // bump so all destroy paths agree.
        if let Some(controller_idx) = dying_creature_controller
            && controller_idx < self.players.len()
        {
            self.players[controller_idx].creatures_died_this_turn =
                self.players[controller_idx].creatures_died_this_turn.saturating_add(1);
        }
        // Snapshot the card if it carries a SelfSource `PermanentSacrificed`
        // trigger so the dispatcher can fire it from last-known info after the
        // permanent has left (the sacrifice funnels push a `PermanentSacrificed`
        // event but the source is already gone by dispatch time — Carrot Cake).
        let has_sac_self_trigger = self
            .battlefield
            .iter()
            .find(|c| c.id == id)
            .map(|c| {
                let granted: &[crate::card::TriggeredAbility] = self
                    .granted_triggers_eot
                    .get(&c.id)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                c.definition
                    .triggered_abilities
                    .iter()
                    .chain(granted)
                    .any(|t| {
                        matches!(t.event.scope, EventScope::SelfSource)
                            && matches!(t.event.kind, EventKind::PermanentSacrificed)
                    })
            })
            .unwrap_or(false);
        // Last-known-info snapshot (CR 603.10): a "when this leaves the
        // battlefield" trigger resolves after the permanent is gone, so cache
        // its pre-removal state for selectors that read it (e.g. an Aura's
        // `AttachedTo(This)` → enchanted creature — Parallax Dementia).
        if (!leave_triggers.is_empty() || has_sac_self_trigger)
            && let Some(c) = self.battlefield.iter().find(|c| c.id == id)
        {
            self.died_card_snapshots.insert(id, c.clone());
            // CR 603.10 — keep a longer-lived LKI snapshot so a
            // "deals damage / makes tokens equal to its power" body reads
            // the counter-boosted P/T at resolution (Goldvein Hydra,
            // Cacophony Scamp). Removed when the trigger resolves.
            self.leaves_bf_lki.insert(id, c.clone());
        }
        // Capture owner + land-ness before removal so we can emit a
        // `CardPutIntoGraveyard` event (CR 700 — "put into a graveyard from
        // the battlefield") once we confirm the card actually landed in the
        // graveyard (Finality / dies-to-exile redirects send it elsewhere).
        let gy_info = self
            .battlefield
            .iter()
            .find(|c| c.id == id)
            .map(|c| (c.owner, c.definition.card_types.contains(&crate::card::CardType::Land)));
        self.remove_from_battlefield_to_graveyard(id);
        let mut out = Vec::new();
        if let Some((owner, is_land)) = gy_info
            && self.players[owner].graveyard.iter().any(|c| c.id == id)
        {
            out.push(GameEvent::CardPutIntoGraveyard { player: owner, card_id: id, is_land });
        }
        for (source, effect, controller) in leave_triggers {
            let auto_target =
                self.auto_target_for_effect_avoiding(&effect, controller, Some(source));
            self.stack.push(StackItem::Trigger {
                source,
                controller,
                effect: Box::new(effect),
                target: auto_target,
                mode: None,
                x_value: 0,
                converged_value: 0,
            trigger_source: None,
                mana_spent: 0,
                event_amount: 0,
                intervening_if: None,
            });
        }
        // CR 702.79 / 702.92 — Persist / Undying apply on *any* death, not
        // just lethal-damage SBA. The destroy / sacrifice funnels route
        // through here, so return the creature now if it qualifies.
        self.return_persist_undying(
            id, persist_owner, (persist_has, undying_has, persist_minus, persist_plus), &mut out,
        );
        out // dies-trigger events are on the stack; Persist returns are in `out`.
    }

    /// CR 702.79 / 702.92 — Persist / Undying return. Shared by every death
    /// funnel (SBA lethal damage, `Effect::Destroy`, sacrifice) so a creature
    /// with Persist/Undying returns regardless of how it died. `owner` and the
    /// counter counts must be captured from the dying card *before* it left the
    /// battlefield.
    /// `info` bundles `(has_persist, has_undying, minus_counter_count,
    /// plus_counter_count)` captured from the dying card before removal.
    pub(crate) fn return_persist_undying(
        &mut self,
        id: CardId,
        owner: usize,
        info: (bool, bool, u32, u32),
        events: &mut Vec<GameEvent>,
    ) {
        use crate::card::CounterType;
        let (has_persist, has_undying, minus_count, plus_count) = info;
        let kind = if has_persist && minus_count == 0 {
            CounterType::MinusOneMinusOne
        } else if has_undying && plus_count == 0 {
            CounterType::PlusOnePlusOne
        } else {
            return;
        };
        if let Some(pos) = self.players[owner].graveyard.iter().position(|c| c.id == id) {
            let mut returned = self.players[owner].graveyard.remove(pos);
            self.players[owner].cards_left_graveyard_this_turn =
                self.players[owner].cards_left_graveyard_this_turn.saturating_add(1);
            returned.damage = 0;
            returned.summoning_sick = true;
            returned.add_counters(kind, 1);
            let rid = returned.id;
            events.push(GameEvent::CardLeftGraveyard { player: owner, card_id: rid });
            self.battlefield.push(returned);
            events.push(GameEvent::PermanentEntered { card_id: rid });
        }
    }
}
