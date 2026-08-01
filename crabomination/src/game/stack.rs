use super::*;
use crate::card::{Keyword, Supertype};
use crate::decision::{Decision, DecisionAnswer};
use crate::effect::{Effect, EventKind, EventScope};
use crate::game::types::{DelayedKind, DelayedTrigger};

/// A collected death/leaves trigger to fire from a dying permanent:
/// `(source, effect, controller, intervening/subject filter)`.
type DeathTrigger = (CardId, Effect, usize, Option<crate::card::Predicate>);

/// How a CR 514 cleanup round ended, telling the caller how to continue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupOutcome {
    /// Suspended on a `wants_ui` discard-down decision (CR 514.1);
    /// `submit_decision` resumes via `finish_cleanup`.
    Suspended,
    /// The cleanup actions fired triggers or SBAs acted — players receive
    /// priority in the cleanup step (CR 514.3a); when they all pass with an
    /// empty stack another cleanup round runs.
    PriorityGranted,
    /// Nothing happened: no priority window (CR 514.3), the turn is over and
    /// `end_turn` already ran — advance to the next turn's untap.
    TurnOver,
}

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
    pub(crate) fn pick_trigger_mode(
        &mut self,
        effect: &Effect,
        source: CardId,
        controller: usize,
    ) -> Option<usize> {
        if let Effect::ChooseMode(modes) = effect {
            if modes.is_empty() {
                return None;
            }
            // A `wants_ui` controller picks through the client modal at
            // resolution time instead of the synchronous decider (which
            // would silently take mode 0 — Riot creatures never hasty).
            // Deferral is gated on no mode requiring a target, since target
            // slots are assigned at push time, before the pick exists.
            if self.players.get(controller).is_some_and(|p| p.wants_ui)
                && modes.iter().all(|m| !m.requires_target())
            {
                return Some(crate::game::types::MODE_PICK_DEFERRED);
            }
            let answer = self.decider.decide(&Decision::ChooseMode {
                source,
                num_modes: modes.len(),
                mode_texts: modes.iter().map(|m| m.effect_short_text()).collect(),
            });
            if let DecisionAnswer::Mode(idx) = answer {
                return Some(idx.min(modes.len() - 1));
            }
            return None;
        }
        // CR 603.7 — a modal buried behind a reflexive payment (Voltstorm
        // Angel's "pay {E}{E}. When you do, choose one …") owns its pick at
        // resolution, *after* the payment succeeds. Defer it: the mode isn't
        // read until the wrappers run, and `MODE_PICK_DEFERRED` routes a UI
        // seat to the client modal and a bot to its decider (both post-payment,
        // since the payment only ever runs when accepted).
        if let Some(modes) = Self::governing_modal(effect)
            && !modes.is_empty()
            && modes.iter().all(|m| !m.requires_target())
        {
            return Some(crate::game::types::MODE_PICK_DEFERRED);
        }
        None
    }

    /// Unwrap reflexive-payment wrappers (`MayDo`/`MayPay*`/`PayEnergy*`) and
    /// the cost-capture wrappers the activation path adds
    /// (`WithSacrificedPt`/`WithTappedPower`) to find a nested `ChooseMode`.
    /// Only descends single-child bodies — a `Seq` or branching effect is left
    /// to its own resolution-time handling.
    pub(crate) fn governing_modal(effect: &Effect) -> Option<&Vec<Effect>> {
        match effect {
            Effect::ChooseMode(modes) => Some(modes),
            Effect::MayDo { body, .. }
            | Effect::PayEnergy { then: body, .. }
            | Effect::PayEnergyValue { then: body, .. }
            | Effect::WithSacrificedPt { body, .. }
            | Effect::WithTappedPower { body, .. } => Self::governing_modal(body),
            Effect::MayPay { body, .. } | Effect::MayPayLife { body, .. } => {
                Self::governing_modal(body)
            }
            _ => None,
        }
    }
}

impl GameState {
    // ── Pass priority ─────────────────────────────────────────────────────────

    pub fn pass_priority(&mut self) -> Result<Vec<GameEvent>, GameError> {
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

        // MTG rule 500.4: mana pools empty at the end of each step and phase
        // (Kruphix converts to colorless instead, CR 106.4 override).
        self.empty_mana_pools();

        // Auto-declare empty blockers if no one blocked.
        let mut events = vec![];
        if self.step == TurnStep::DeclareBlockers
            && !self.attacking.is_empty()
            && !self.blockers_declared
        {
            self.blockers_declared = true;
            // CR 509.3g — every attacker went unblocked; "attacks and isn't
            // blocked" triggers fire on this path too, not just on an
            // explicit DeclareBlockers action. Dispatch inline: if anything
            // triggered, stop here (priority round for the trigger) instead
            // of advancing into combat damage in the same pass.
            let unblocked: Vec<GameEvent> = self
                .attacking
                .iter()
                .map(|atk| GameEvent::AttackerWentUnblocked { attacker: atk.attacker })
                .collect();
            // Dispatched here, not via the returned events — the caller
            // (`perform_action`) re-dispatches returned events, which would
            // double-fire the triggers.
            let stack_before = self.stack.len();
            self.dispatch_triggers_for_events(&unblocked);
            if self.stack.len() > stack_before || self.pending_decision.is_some() {
                self.give_priority_to_active();
                return Ok(events);
            }
        }

        if self.step == TurnStep::Cleanup {
            // All players passed during a CR 514.3a cleanup priority round
            // with an empty stack — another cleanup round happens. It may
            // suspend (UI discard), grant priority again (more triggers), or
            // finish the turn.
            return match self.do_cleanup(&mut events) {
                CleanupOutcome::Suspended | CleanupOutcome::PriorityGranted => Ok(events),
                CleanupOutcome::TurnOver => self.advance_step(events),
            };
        }

        self.advance_step(events)
    }

    /// Compute and enter the step following the current one, running its
    /// turn-based entry actions (untap, draw, combat resolution, step
    /// triggers, …). Split out of `pass_priority` so the cleanup discard
    /// resume path can re-run it after a suspended discard is answered.
    pub fn advance_step(
        &mut self,
        mut events: Vec<GameEvent>,
    ) -> Result<Vec<GameEvent>, GameError> {
        // CR 702.94 — a step-bounded Miracle window ("cast it now or lose
        // the chance") dies at the step transition; the alt-cost shares the
        // permission's lifetime.
        self.clear_step_bounded_may_play();
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

        // CR 506 — skip the active player's combat phase (Stonehorn
        // Dignitary). When we'd enter Begin Combat with a banked skip charge,
        // consume it and jump straight to the postcombat main — no
        // begin-combat triggers, declares, or damage. (The post-main extra
        // combat below is unaffected; only the scheduled combat is skipped.)
        if next == TurnStep::BeginCombat
            && self.step == TurnStep::PreCombatMain
            && self.players[self.active_player_idx].skip_next_combat > 0
        {
            self.players[self.active_player_idx].skip_next_combat -= 1;
            next = TurnStep::PostCombatMain;
        }

        // CR 505.1b — additional combat phase. When the active player leaves
        // End of Combat with a banked extra phase, loop back to Begin Combat
        // (a fresh combat) instead of advancing to the postcombat main.
        if self.step == TurnStep::EndCombat && self.additional_combat_phases > 0 {
            self.additional_combat_phases -= 1;
            next = TurnStep::BeginCombat;
        }

        // CR 505.1b — "after this main phase, an additional combat phase
        // followed by an additional main phase" (Relentless Assault). When
        // the active player leaves the postcombat main with one banked,
        // enter Begin Combat; EndCombat → PostCombatMain then supplies the
        // extra main. (A precombat-main cast banks the phase until after
        // the turn's scheduled combat rather than inserting it before.)
        if self.step == TurnStep::PostCombatMain && self.additional_post_main_combats > 0 {
            self.additional_post_main_combats -= 1;
            next = TurnStep::BeginCombat;
        }

        // CR 500.7 — additional end step. When the active player leaves the
        // End step with one banked, loop back to another End step instead of
        // advancing to cleanup (Y'shtola Rhul).
        if self.step == TurnStep::End && self.additional_end_steps > 0 {
            self.additional_end_steps -= 1;
            next = TurnStep::End;
        }

        // CR 500.9 — additional upkeep step. When the active player leaves
        // the Upkeep with one banked, loop back to another Upkeep instead of
        // advancing to Draw (Paradox Haze).
        if self.step == TurnStep::Upkeep && self.additional_upkeep_steps > 0 {
            self.additional_upkeep_steps -= 1;
            next = TurnStep::Upkeep;
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

        // CR 614.10 — skip-step replacements (Eon Hub / Stasis family). A
        // skipped upkeep or draw step never occurs: no turn-based actions,
        // step triggers, or priority — advance straight through. (A skipped
        // untap is handled in the Untap arm so the turn still starts.)
        if matches!(next, TurnStep::Upkeep | TurnStep::Draw)
            && self.step_skipped_for(self.active_player_idx, next)
        {
            return self.advance_step(events);
        }

        match next {
            // Untap has no priority window — auto-execute and move on.
            TurnStep::Untap => {
                // CR 615 — "until your next turn" damage locks (Kiora's +1)
                // end as the granting player's turn begins.
                let ap = self.active_player_idx;
                self.damage_locked_until_turn_of.retain(|(_, seat)| *seat != ap);
                // CR 614.10 — a skipped untap step skips its turn-based
                // actions (untapping, phasing, day/night), but the turn
                // itself still begins.
                if !self.step_skipped_for(self.active_player_idx, TurnStep::Untap) {
                    // CR 502.2 — day/night turn-based check. Runs BEFORE
                    // do_untap so an extra turn (previous active == active)
                    // reads the real previous-turn spell count rather than
                    // the counter do_untap is about to reset. A transition
                    // here fires "day becomes night …" triggers (Brimstone
                    // Vandal) — dispatch them before priority is given out.
                    let mut dn_evs = Vec::new();
                    self.check_day_night_transition(&mut dn_evs);
                    if !dn_evs.is_empty() {
                        self.dispatch_triggers_for_events(&dn_evs);
                        events.append(&mut dn_evs);
                    }
                    self.do_untap();
                }
                // CR 611.2b — "until the next turn" / "until your next turn"
                // continuous effects end as the relevant turn begins.
                let (active, turn) = (self.active_player_idx, self.turn_number);
                self.continuous_effects.retain(|e| match e.duration {
                    crate::game::layers::EffectDuration::UntilNextTurn => false,
                    crate::game::layers::EffectDuration::UntilYourNextTurn {
                        player,
                        installed_turn,
                    } => !(player == active && turn > installed_turn),
                    _ => true,
                });
                // CR 723.1 — a pending player-control effect applies to the
                // next turn its target actually takes, and expires here.
                self.apply_pending_player_control(self.active_player_idx);
                events.push(GameEvent::TurnStarted {
                    player: self.active_player_idx,
                    turn: self.turn_number,
                });
                // No priority in Untap (CR 502.4) — immediately advance to
                // Upkeep. Seed the pass count so the single pass below counts
                // as everyone passing rather than needing a phantom extra
                // client pass.
                self.priority.player_with_priority = self.active_player_idx;
                self.priority.consecutive_passes = self.alive_count().saturating_sub(1);
                let mut upkeep_events = self.pass_priority()?;
                events.append(&mut upkeep_events);
                return Ok(events);
            }
            TurnStep::Draw => {
                // CR 103.7a — only the *starting player's first* draw step is
                // skipped. Consuming the flag unconditionally handed the skip
                // to whoever reached a draw step first, so a game advanced
                // past turn 1 (every test fixture) robbed the wrong seat.
                let skips_draw_step = self.battlefield.iter().any(|c| {
                    c.controller == self.active_player_idx
                        && c.definition.static_abilities.iter().any(|sa| {
                            sa.effect == crate::effect::StaticEffect::ControllerSkipsDrawStep
                        })
                });
                let first_draw_skip =
                    std::mem::take(&mut self.skip_first_draw) && self.turn_number == 1;
                // CR 614.13 — a "skip your next draw step" charge is only spent
                // when it's the effect doing the skipping; another skip already
                // in force leaves the charge waiting for the next draw step.
                let charged_skip = !first_draw_skip
                    && !skips_draw_step
                    && self.players[self.active_player_idx].skip_next_draw_step > 0;
                if charged_skip {
                    self.players[self.active_player_idx].skip_next_draw_step -= 1;
                }
                if first_draw_skip || skips_draw_step || charged_skip {
                } else {
                    let p = self.active_player_idx;
                    // CR 504.1 — the turn-based draw-step draw is exempt from
                    // Notion Thief's redirect ("except the first one they draw").
                    self.in_turn_based_draw = true;
                    let drew = self.draw_one(p, &mut events);
                    self.in_turn_based_draw = false;
                    if !drew {
                        // CR 104.3c (or the Lab-Man win override). Game-over
                        // check happens inside SBA.
                        self.lose_to_empty_draw(p);
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
                self.upkeep_steps_this_turn = self.upkeep_steps_this_turn.saturating_add(1);
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
                // CR 702.29 — echo: pay-or-sacrifice on the first upkeep.
                let mut echo = self.process_echo();
                events.append(&mut echo);
                // CR 702.50a — Epic spells copy at the controller's upkeep.
                let mut epic = self.process_epic();
                events.append(&mut epic);
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
                self.combat_phases_this_turn = self.combat_phases_this_turn.saturating_add(1);
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
                self.end_steps_this_turn = self.end_steps_this_turn.saturating_add(1);
                // CR 725.2 — "At the beginning of the monarch's end step,
                // that player draws a card." An inherent triggered ability
                // with no source, so it uses the stack and can be responded
                // to (and doubled) like any other trigger.
                if self.monarch == Some(self.active_player_idx) {
                    let seat = self.active_player_idx;
                    self.stack.push(
                        crate::game::types::TriggerPush::new(
                            CardId(0),
                            seat,
                            Effect::Draw {
                                who: crate::effect::Selector::You,
                                amount: crate::card::Value::ONE,
                            },
                        )
                        .build(),
                    );
                }
                // CR 702.183 — Impending time counters tick at the beginning of
                // the controller's end step; the permanent becomes a creature
                // when the last is gone.
                let mut imp = self.process_impending();
                events.append(&mut imp);
                // MKM — solve any Cases whose condition is now met (before end-step
                // triggers so a Case File Auditor sees "whenever you solve a Case").
                self.process_case_solves(&mut events);
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
                self.noncreature_spells_cast_this_turn = 0;
                for pl in &mut self.players {
                    pl.spells_cast_this_game_turn = 0;
                    pl.noncreature_spells_cast_this_game_turn = 0;
                    pl.nonartifact_spells_cast_this_game_turn = 0;
                    pl.multicolored_spells_cast_this_turn = 0;
                    pl.spell_names_cast_this_turn.clear();
                    pl.spells_cast_from_hand_this_turn = 0;
                    pl.oil_activity_this_turn = false;
                    pl.channel_life_for_mana = false;
                    // CR 603.7e — unused "your next creature spell this turn"
                    // riders expire with the turn.
                    pl.pending_creature_etb_counters.clear();
                    pl.pending_creature_etb_keywords.clear();
                    // The Chain Veil's banked activations and the
                    // "did you activate a loyalty ability" flag are per-turn.
                    pl.extra_loyalty_activations = 0;
                    pl.activated_loyalty_this_turn = false;
                }
                self.mana_spent_on_spells_this_turn = 0;
                self.permanents_to_graveyard_this_turn = 0;
                // CR 514.3 — "at the beginning of the next cleanup step"
                // delayed triggers (Waylay) go on the stack here; the
                // `finish_cleanup` check below then grants priority.
                self.fire_step_triggers(TurnStep::Cleanup);
                // CR 514.3 — no player receives priority during cleanup
                // unless its turn-based actions fire triggers or SBAs act;
                // run them immediately on entering the step.
                match self.do_cleanup(&mut events) {
                    CleanupOutcome::Suspended | CleanupOutcome::PriorityGranted => {}
                    CleanupOutcome::TurnOver => return self.advance_step(events),
                }
            }
            TurnStep::EndCombat => {
                // CR 511.1 — "at end of combat" triggers, including the
                // `DelayedKind::EndOfCombat` queue (Triton Tactics, Fortune).
                self.fire_step_triggers(TurnStep::EndCombat);
                self.give_priority_to_active();
            }
            // Master Warcraft — the declaration steps hand priority to the
            // outside chooser so it, not the active/defending player, submits
            // the declaration. Without a chooser these behave like `_`.
            TurnStep::DeclareAttackers | TurnStep::DeclareBlockers
                if self.combat_chooser.is_some() =>
            {
                self.priority.player_with_priority = self.combat_chooser.unwrap_or(self.active_player_idx);
                self.priority.consecutive_passes = 0;
            }
            _ => {
                self.give_priority_to_active();
            }
        }

        Ok(events)
    }

    /// MKM — at the beginning of the active player's end step, each of their
    /// unsolved Cases whose "To solve" condition holds becomes solved. Emits a
    /// `GameEvent::CaseSolved` per newly-solved Case and dispatches "whenever
    /// you solve a Case" triggers (Case File Auditor).
    pub fn process_case_solves(&mut self, events: &mut Vec<GameEvent>) {
        let active = self.active_player_idx;
        let candidates: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == active && !c.case_solved && c.definition.case.is_some())
            .map(|c| c.id)
            .collect();
        let mut solved_events = Vec::new();
        for id in candidates {
            let Some(pred) = self
                .battlefield
                .iter()
                .find(|c| c.id == id)
                .and_then(|c| c.definition.case.as_ref().map(|d| d.to_solve.clone()))
            else {
                continue;
            };
            let ctx = crate::game::effects::EffectContext::for_trigger(id, active, None, 0);
            if !self.evaluate_predicate(&pred, &ctx) {
                continue;
            }
            if let Some(card) = self.battlefield.iter_mut().find(|c| c.id == id)
                && card.solve_case()
            {
                solved_events.push(GameEvent::CaseSolved { case: id, controller: active });
            }
        }
        if !solved_events.is_empty() {
            self.dispatch_triggers_for_events(&solved_events);
            events.extend(solved_events);
        }
    }

    /// Push step-based triggers onto the stack for the given step.
    /// Fires `EventKind::StepBegins(step)` triggers. Scope controls which
    /// players' permanents' triggers fire: `ActivePlayer` is default for
    /// "at the beginning of your upkeep"; `AnyPlayer` fires for everyone.
    /// Also processes any `delayed_triggers` whose kind matches this step
    /// (e.g. Pact upkeep cost, Goryo's exile-at-end-step).
    pub fn fire_step_triggers(&mut self, step: TurnStep) {
        let active = self.active_player_idx;
        let kind = EventKind::StepBegins(step);
        // Collect candidate (source, effect, controller, filter) tuples for
        // each battlefield permanent's matching trigger. We snapshot the
        // optional `event.filter` predicate alongside the effect so we can
        // re-check it after gathering — predicate evaluation needs
        // `&self.evaluate_predicate(...)` which can't run inside the inner
        // closure due to the `iter` borrow.
        let scope_matches = |scope: &EventScope, controller: usize| match scope {
            EventScope::AnyPlayer => true,
            EventScope::ActivePlayer | EventScope::YourControl | EventScope::SelfSource => {
                controller == active
            }
            EventScope::OpponentControl => controller != active,
            EventScope::AnotherOfYours => false,
            EventScope::FromYourGraveyard => false, // walked separately below
            EventScope::YourPermanentTargetedByOpponent
            | EventScope::YourCreatureTargeted
            | EventScope::EnchantedBySource
            | EventScope::YourSourceDamagedOpponent
            | EventScope::OpponentSourceDamagedYou
            | EventScope::YourOtherSourceDamagedOpponent
            | EventScope::YouTapped => false, // event-based
            EventScope::ControllerAttackedByOpponent
            | EventScope::ControllerPlaneswalkerAttackedByOpponent => false, // combat-based
        };
        let mut candidates: Vec<(CardId, Effect, usize, Option<crate::card::Predicate>)> = self
            .battlefield
            .iter()
            .flat_map(|c| {
                // Printed triggers plus statics-granted ones (Kataki's "All
                // artifacts have '…upkeep…'"), both firing off `c`.
                let granted = self.statics_granted_triggers_for(c);
                c.definition
                    .triggered_abilities
                    .iter()
                    .cloned()
                    .chain(granted)
                    .filter(|t| t.event.kind == kind)
                    .filter(|t| scope_matches(&t.event.scope, c.controller))
                    .map(|t| (c.id, t.effect, c.controller, t.event.filter))
                    .collect::<Vec<_>>()
            })
            .collect();
        // CR 702.6e / 303.4 — step triggers granted to a permanent by an
        // attached Aura/Equipment's `equipped_bonus` fire as though printed on
        // the host ("Enchanted creature has 'At the beginning of your upkeep,
        // you lose 1 life'" — Pillory of the Sleepless). Source is the host
        // (unless `triggers_on_equipment`); "your" scope keys on the host's
        // controller. Combat-damage/dies equip triggers use other kinds and so
        // are untouched here.
        for eq in &self.battlefield {
            let Some(host_id) = eq.attached_to else { continue };
            let Some(bonus) = &eq.definition.equipped_bonus else { continue };
            let Some(host) = self.battlefield.iter().find(|c| c.id == host_id) else { continue };
            for t in &bonus.triggered_abilities {
                if t.event.kind == kind && scope_matches(&t.event.scope, host.controller) {
                    let source = if bonus.triggers_on_equipment { eq.id } else { host_id };
                    candidates.push((source, t.effect.clone(), host.controller, t.event.filter.clone()));
                }
            }
        }
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
        type DelayedFire = (
            CardId,
            Effect,
            usize,
            Option<Target>,
            Option<crate::game::effects::EntityRef>,
        );
        let mut delayed_to_fire: Vec<DelayedFire> = Vec::new();
        let mut keep: Vec<DelayedTrigger> = Vec::new();
        for dt in std::mem::take(&mut self.delayed_triggers) {
            let matches = match (&dt.kind, step) {
                (DelayedKind::YourNextUpkeep, TurnStep::Upkeep) => dt.controller == active,
                (DelayedKind::YourNextMainPhase, TurnStep::PreCombatMain) => {
                    dt.controller == active
                }
                (DelayedKind::NextEndStep, TurnStep::End) => true,
                (DelayedKind::NextCleanupStep, TurnStep::Cleanup) => true,
                (DelayedKind::EachCombatThisTurn, TurnStep::BeginCombat) => {
                    dt.controller == active
                }
                (DelayedKind::EndOfCombat, TurnStep::EndCombat) => true,
                (DelayedKind::NextCombat, TurnStep::BeginCombat) => true,
                _ => false,
            };
            if matches {
                delayed_to_fire.push((
                    dt.source,
                    dt.effect.clone(),
                    dt.controller,
                    dt.target.clone(),
                    dt.bound_subject.or_else(|| {
                        dt.bound_token.map(crate::game::effects::EntityRef::Permanent)
                    }),
                ));
                if !dt.fires_once {
                    keep.push(dt);
                }
            } else {
                keep.push(dt);
            }
        }
        self.delayed_triggers = keep;

        // Build a single queue (delayed triggers first, then step
        // triggers; APNAP-sorted below) so `drain_trigger_queue` can surface
        // `Decision::ChooseTarget` for wants_ui controllers instead of
        // silently auto-targeting them.
        let mut queue: Vec<PendingTriggerPush> = Vec::new();
        for (source, effect, controller, captured_target, subject) in delayed_to_fire {
            let mode = self.pick_trigger_mode(&effect, source, controller);
            // Delayed triggers may have captured a target at registration
            // time (e.g. Pact's "lose the game"). If so, push immediately
            // with that target — we already passed the targeting moment.
            if captured_target.is_some() {
                self.push_pending_trigger(
                    PendingTriggerPush {
                        source,
                        controller,
                        effect,
                        subject,
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
                subject,
                event_amount: 0,
                mode,
                intervening_if: None,
            });
        }
        for (source, effect, controller, intervening_if) in triggers_with_filter {
            let mode = self.pick_trigger_mode(&effect, source, controller);
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
        // CR 603.3b — APNAP order: the active player's triggers push first
        // (resolving last). Battlefield-Vec order is otherwise preserved as
        // each controller's chosen same-controller order (stable sort).
        let n_players = self.players.len();
        let apnap_rank = |seat: usize| -> usize {
            (seat + n_players - active) % n_players.max(1)
        };
        queue.sort_by_key(|t| apnap_rank(t.controller));
        self.drain_trigger_queue(queue);
    }

    /// CR 714 — put a lore counter on a Saga and fire the chapter ability/ies
    /// for the new lore-counter total. Called on ETB (chapter I) and as a
    /// turn-based action at the start of the controller's precombat main.
    /// CR 714.2b — a Saga entering the battlefield gets its first lore
    /// counter. CR 702.155 (Read Ahead): a read-ahead Saga instead enters with
    /// a chosen number of lore counters (1..final chapter), firing only the
    /// chosen chapter. Non-read-ahead Sagas defer to [`saga_advance`].
    pub(crate) fn saga_enter_advance(&mut self, card_id: CardId) {
        let Some(card) = self.battlefield.iter().find(|c| c.id == card_id) else {
            return;
        };
        if card.definition.saga_chapters.is_empty() {
            return;
        }
        if !card.definition.read_ahead {
            self.saga_advance(card_id);
            return;
        }
        let final_ch = card
            .definition
            .saga_chapters
            .iter()
            .map(|(n, _)| *n)
            .max()
            .unwrap_or(1);
        let controller = card.controller;
        // Choose the starting chapter (1..=final). AutoDecider/tests answer
        // through the installed decider; the amount is clamped into range.
        let decision = crate::decision::Decision::ChooseAmount {
            source: card_id,
            max: final_ch,
            prompt: "Read ahead — choose a starting chapter".into(),
        };
        let chosen = match self.decider.decide(&decision) {
            crate::decision::DecisionAnswer::Amount(n) => n.clamp(1, final_ch),
            _ => 1,
        };
        if let Some(card) = self.battlefield.iter_mut().find(|c| c.id == card_id) {
            card.add_counters(crate::card::CounterType::Lore, chosen);
        }
        let effects: Vec<Effect> = self
            .battlefield
            .iter()
            .find(|c| c.id == card_id)
            .map(|card| {
                card.definition
                    .saga_chapters
                    .iter()
                    .filter(|(n, _)| *n == chosen)
                    .map(|(_, e)| e.clone())
                    .collect()
            })
            .unwrap_or_default();
        let mut queue: Vec<PendingTriggerPush> = Vec::new();
        for effect in effects {
            let mode = self.pick_trigger_mode(&effect, card_id, controller);
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

    pub fn saga_advance(&mut self, card_id: CardId) {
        let Some(card) = self.battlefield.iter_mut().find(|c| c.id == card_id) else {
            return;
        };
        if card.definition.saga_chapters.is_empty() {
            return;
        }
        let before = card.counter_count(crate::card::CounterType::Lore);
        card.add_counters(crate::card::CounterType::Lore, 1);
        self.saga_chapters_crossed(card_id, before, before + 1);
    }

    /// CR 714.2b — "when one or more lore counters are put onto this Saga, if
    /// the number of lore counters on it was less than N and became at least
    /// N, [effect]". Fires every chapter whose threshold the count just
    /// crossed, in chapter order, so a proliferate or a doubled placement
    /// doesn't skip a chapter. Called from `saga_advance` and from every
    /// generic lore-counter placement.
    pub(crate) fn saga_chapters_crossed(&mut self, card_id: CardId, before: u32, after: u32) {
        if after <= before {
            return;
        }
        let Some(card) = self.battlefield.iter().find(|c| c.id == card_id) else {
            return;
        };
        let controller = card.controller;
        let mut chapters: Vec<(u32, Effect)> = card
            .definition
            .saga_chapters
            .iter()
            .filter(|(n, _)| *n > before && *n <= after)
            .map(|(n, e)| (*n, e.clone()))
            .collect();
        chapters.sort_by_key(|(n, _)| *n);
        let mut queue: Vec<PendingTriggerPush> = Vec::new();
        for (_, effect) in chapters {
            let mode = self.pick_trigger_mode(&effect, card_id, controller);
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
        if !queue.is_empty() {
            self.drain_trigger_queue(queue);
        }
    }

    // ── Stack resolution ──────────────────────────────────────────────────────

    pub fn resolve_top_of_stack(&mut self) -> Result<Vec<GameEvent>, GameError> {
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

                // CR 702.140 — a creature with mutate merges onto its host
                // permanent instead of entering on its own. If the host is no
                // longer a legal mutate target it enters as a normal creature
                // (CR 702.140i), so we only divert when the host is legal.
                if let Some((host_id, on_top)) = card.mutate_onto {
                    let legal_host = self.battlefield.iter().any(|c| {
                        c.id == host_id
                            && c.controller == caster
                            && c.definition.is_creature()
                            && !c
                                .definition
                                .has_creature_type(crate::card::CreatureType::Human)
                    });
                    if legal_host {
                        let mut incoming = card;
                        incoming.mutate_onto = None;
                        let host = self
                            .battlefield
                            .iter_mut()
                            .find(|c| c.id == host_id)
                            .expect("legal_host verified");
                        host.apply_mutate(incoming, on_top);
                        events.push(GameEvent::Mutated { card_id: host_id });
                        // CR 702.140f — "Whenever this creature mutates" from
                        // every card in the merged pile (now unioned into the
                        // host's live definition).
                        let mutate_effects: Vec<Effect> = self
                            .battlefield
                            .iter()
                            .find(|c| c.id == host_id)
                            .map(|c| {
                                c.definition
                                    .triggered_abilities
                                    .iter()
                                    .filter(|t| {
                                        t.event.kind == EventKind::Mutated
                                            && matches!(t.event.scope, EventScope::SelfSource)
                                    })
                                    .map(|t| t.effect.clone())
                                    .collect()
                            })
                            .unwrap_or_default();
                        for effect in mutate_effects {
                            let auto_target = self.auto_target_for_effect(&effect, caster);
                            self.stack.push(
                                TriggerPush::new(host_id, caster, effect)
                                    .target(auto_target)
                                    .trigger_source(Some(
                                        crate::game::effects::EntityRef::Permanent(host_id),
                                    ))
                                    .build(),
                            );
                        }
                        let mut sba = self.check_state_based_actions();
                        events.append(&mut sba);
                        return Ok(events);
                    }
                }

                // CR 715 / 702.183 — while cast as its Adventure/Omen half the
                // card is its instant/sorcery half, so it resolves down the
                // spell path (not onto the battlefield) regardless of its
                // creature card type.
                let is_noncreature = card.casting_alt_half() || !card.definition.is_creature();

                // CR 608.2b — an Aura spell re-checks its enchant target as
                // it tries to resolve; if the target is illegal (gone,
                // filter mismatch, granted Hexproof/Shroud) the spell
                // doesn't resolve — countered into its owner's graveyard,
                // never entering the battlefield (no ETB). Bestowed casts
                // are exempt: CR 702.103e resolves them as the creature.
                if card.definition.is_aura()
                    && !card.bestowed
                    && !card.casting_alt_half()
                    && let Some(t) = &target
                {
                    let gone = matches!(t, Target::Permanent(tid)
                        if self.battlefield_find(*tid).is_none());
                    let filter_fail = card
                        .definition
                        .effect
                        .target_filter_for_slot_in_mode_kicked(0, mode, card.kicked)
                        .is_some_and(|f| {
                            !self.evaluate_requirement_static(f, t, caster, Some(card.id))
                        });
                    let untargetable = self
                        .check_target_legality_with_source(t, caster, Some(card.id))
                        .is_err();
                    if gone || filter_fail || untargetable {
                        if !card.is_token {
                            self.route_to_graveyard(card, &mut events);
                        }
                        return Ok(events);
                    }
                }

                if card.definition.is_permanent() && !card.casting_alt_half() {
                    // Collect ETB triggers before moving card into battlefield.
                    // `mut` so the enters-as-copy path can swap in the
                    // copied object's ETB triggers (CR 707.5).
                    // Carry each self-ETB trigger's intervening-`if`
                    // (`event.filter`) so it can be re-evaluated once the card
                    // is on the battlefield (CR 603.4) — the inline resolution
                    // path used to drop it, firing filtered self-ETB triggers
                    // unconditionally (Corrupted / kicker-gated ETBs).
                    let mut etb_triggers: Vec<(Effect, Option<crate::card::Predicate>)> = card
                        .definition
                        .triggered_abilities
                        .iter()
                        .filter(|t| t.event.kind == EventKind::EntersBattlefield
                            && matches!(t.event.scope, EventScope::SelfSource))
                        .map(|t| (t.effect.clone(), t.event.filter.clone()))
                        .collect();
                    let evoked = card.evoked;
                    let dashed = card.dashed;
                    // CR 614.12 — capture the "enters with N counters"
                    // replacement before the card moves to the battlefield;
                    // we apply the counters immediately after pushing,
                    // BEFORE the next state-based-action sweep, so a printed
                    // 0/0 body (Pterafractyl, Symmathematics) survives ETB.
                    let enters_spec = card.definition.enters_with_counters.clone();
                    // CR 614.12 — Patched Plaything's "enters with two -1/-1
                    // counters if you cast it from your hand" reads the cast
                    // zone via `Predicate::CastFromHand`.
                    let cast_from_hand = card.cast_from_hand;
                    let mut card = card;
                    // CR 608.3a — a permanent spell enters under the control
                    // of its caster (matters for casts of opponent-owned
                    // cards: Gonti, Hostage Taker).
                    card.controller = caster;
                    card.controller = self.apply_etb_control_replacement(&card, card.controller);
                    // Stamp the cast cost so ETB riders can read it after the
                    // spell leaves the stack (Astelli Reclaimer's MV-≤-X return).
                    card.cast_mana_spent = mana_spent;
                    // Stamp the cast's X so ETB *triggered* abilities can read
                    // it (Dune Drifter's MV-≤-X graveyard return).
                    card.cast_x_value = x_value;
                    // CR 702.150c — a Compleated planeswalker cast with life
                    // enters with two fewer loyalty per pip paid with life.
                    if card.compleated_life_paid > 0 && card.definition.is_planeswalker() {
                        let loyalty = card
                            .definition
                            .base_loyalty
                            .saturating_sub(card.compleated_life_paid);
                        card.compleated_life_paid = 0;
                        if loyalty > 0 {
                            card.counters
                                .insert(crate::card::CounterType::Loyalty, loyalty);
                        } else {
                            card.counters.remove(&crate::card::CounterType::Loyalty);
                        }
                    }
                    let room_door = card.definition.room.as_ref().map(|_| {
                        usize::from(card.split_cast == Some(1))
                    });
                    // `Effect::CopySpellWithRiders` — the copy's stamped
                    // riders apply as it resolves into a permanent.
                    let resolve_riders = card.resolve_riders.take();
                    // This permanent entered because its spell was cast (CR
                    // 400.7 new object) — powers "if you cast it" ETB gates.
                    card.entered_by_cast = true;
                    self.battlefield.push(card);
                    if let Some((grant_haste, sacrifice_eot)) = resolve_riders {
                        if grant_haste {
                            self.grant_keyword_eot(card_id, crate::card::Keyword::Haste);
                        }
                        if sacrifice_eot {
                            self.delayed_triggers.push(crate::game::types::DelayedTrigger {
                                controller: caster,
                                source: card_id,
                                kind: crate::game::types::DelayedKind::NextEndStep,
                                effect: Effect::SacrificeSource,
                                target: None,
                                bound_token: None,
                                bound_subject: None,
                                fires_once: true,
                            });
                        }
                    }
                    // CR 310.6 — a cast Siege's controller chooses an opponent
                    // to protect it (the lone opponent in 2-player; multiplayer
                    // choice is a follow-up).
                    if let Some(c) = self.battlefield.iter().find(|c| c.id == card_id)
                        && c.definition.is_battle()
                        && c.protected_by.is_none()
                    {
                        let ctrl = c.controller;
                        let protector = (0..self.players.len())
                            .find(|&pl| pl != ctrl && self.players[pl].is_alive());
                        if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == card_id) {
                            c.protected_by = protector;
                        }
                    }
                    // CR 614 — control-setting and enters-tapped replacements
                    // (Captive Audience; Imposing Sovereign, Urabrask) apply to
                    // cast permanents too.
                    self.apply_enters_under_opponent_control(card_id);
                    self.apply_enters_tapped_replacement(card_id);
                    // CR 709.5d — a Room enters with the cast door unlocked
                    // (709.5h: its unlock trigger fires).
                    if let Some(door) = room_door {
                        self.set_room_door_unlocked(card_id, door == 1, &mut events);
                    }
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
                    // creature entry the caster controls) — plus Oath of
                    // Gideon's loyalty rider, which is the planeswalker case
                    // the same collector serves.
                    if is_creature_resolve || self.battlefield.iter().any(|c| {
                        c.id == card_id && c.definition.is_planeswalker()
                    }) {
                        for (kind, n) in self.chosen_type_etb_counter_specs(card_id, caster) {
                            counter_specs.push((kind, crate::effect::Value::Const(n as i32)));
                        }
                    }
                    // CR 603.7e — one-shot "your next creature spell enters
                    // with N counters / these keywords" riders (FIN "Summon"
                    // saga chapters — Fenrir II counters, Brynhildr Gestalt haste).
                    if is_creature_resolve {
                        for (kind, n) in
                            std::mem::take(&mut self.players[caster].pending_creature_etb_counters)
                        {
                            counter_specs.push((kind, crate::effect::Value::Const(n as i32)));
                        }
                        let kws =
                            std::mem::take(&mut self.players[caster].pending_creature_etb_keywords);
                        for kw in kws {
                            self.grant_keyword_eot(card_id, kw);
                        }
                    }
                    // Cast-time ETB-counter riders stamped on the instance
                    // (Noctis's graveyard-cast finality counter).
                    if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == card_id) {
                        for (kind, n) in std::mem::take(&mut c.pending_etb_counters) {
                            counter_specs.push((kind, crate::effect::Value::Const(n as i32)));
                        }
                    }
                    // CR 702.44 — Sunburst: one counter per color of mana
                    // spent (a +1/+1 counter when it enters as a creature, a
                    // charge counter otherwise). The cast's converge count is
                    // exactly "colors of mana spent".
                    if converged_value > 0
                        && self.battlefield.iter().any(|c| {
                            c.id == card_id
                                && c.definition.keywords.contains(&crate::card::Keyword::Sunburst)
                        })
                    {
                        let kind = if is_creature_resolve {
                            crate::card::CounterType::PlusOnePlusOne
                        } else {
                            crate::card::CounterType::Charge
                        };
                        counter_specs
                            .push((kind, crate::effect::Value::Const(converged_value as i32)));
                    }
                    // CR 122.1 — Solemnity drops enters-with-counters.
                    if self.counters_locked() { counter_specs.clear(); }
                    for (kind, value) in counter_specs {
                        let mut etb_ctx = crate::game::effects::EffectContext::for_spell_with_source(
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
                        etb_ctx.cast_from_hand = cast_from_hand;
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
                    // CR 614 — "As this permanent enters, [effect]" (Ixidron).
                    // Before the first SBA sweep so a `*/*` body sized off the
                    // effect never dies as a 0/0.
                    self.apply_as_enters_effect(card_id);
                    // CR 614 — "As this enters, it becomes your choice of …"
                    // (Corrupted Shapeshifter). Applied before SBA so a
                    // printed */* body never dies as a 0/0.
                    self.apply_enters_as_choice(card_id);
                    // CR 614 — "As this enters, choose [mode A] or [mode B]"
                    // (the Tarkir Siege cycle). Bakes the chosen mode's
                    // abilities onto the permanent as it enters.
                    self.apply_enters_mode_choice(card_id);
                    // CR 707 — "enters as a copy of [filter]" replacement.
                    // Applied here, before the first SBA sweep, so a 0/0
                    // copier (Clone, Phantasmal Image) never dies as a 0/0.
                    if self.apply_enters_as_copy(card_id, caster, &mut events) {
                        // CR 616.1c/616.1g — the copy replacement outranks the
                        // enters-tapped one, so re-decide tappedness against the
                        // copied characteristics.
                        self.reapply_enters_tapped_after_copy(card_id);
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
                                    .map(|t| (t.effect.clone(), t.event.filter.clone()))
                                    .collect()
                            })
                            .unwrap_or_default();
                    }

                    // Statics-granted ETBs ("Slivers you control have 'When
                    // this enters…'" — Lavabelly) fire as though printed;
                    // gathered now that the entrant is on the battlefield.
                    if let Some(c) = self.battlefield_find(card_id) {
                        etb_triggers.extend(
                            self.statics_granted_triggers_for(c)
                                .into_iter()
                                .filter(|t| {
                                    t.event.kind == EventKind::EntersBattlefield
                                        && matches!(t.event.scope, EventScope::SelfSource)
                                })
                                .map(|t| (t.effect, t.event.filter)),
                        );
                    }

                    events.push(GameEvent::PermanentEntered { card_id });

                    // CR 702.165 — a permanent spell cast with its Gift promised
                    // gives the gift as it enters. Emit once for "whenever you
                    // give a gift" payoffs (Jolly Gerbils).
                    if self
                        .battlefield_find(card_id)
                        .is_some_and(|c| c.gift_promised && c.definition.gift.is_some())
                    {
                        events.push(GameEvent::GiftGiven { player: caster });
                    }

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
                    // CR 303.4a — "enchant player" Auras anchor to a seat.
                    if let Some(crate::game::types::Target::Player(seat)) = target
                        && let Some(aura) =
                            self.battlefield.iter_mut().find(|c| c.id == card_id)
                        && aura.definition.is_aura()
                    {
                        aura.attached_to_player = Some(seat);
                    }
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
                        // CR 303.4 — fire "an Aura you control became attached"
                        // triggers (Siona). Only for true Auras, not bestowed
                        // creature spells (which entered as creatures).
                        if self.battlefield.iter().any(|c| c.id == card_id && c.definition.is_aura())
                        {
                            events.push(GameEvent::AuraAttached { aura: card_id, attached_to: tid });
                        }
                    }

                    // Evoke: schedule a self-sacrifice trigger that resolves
                    // AFTER the ETB triggers (so the ETB exile happens first,
                    // then the creature sacrifices itself).
                    if evoked {
                        self.stack.push(
                        TriggerPush::new(card_id, caster, Effect::Move {
                            what: crate::effect::Selector::This,
                            to: crate::effect::ZoneDest::Graveyard,
                        })
                        .build(),
                        );
                    }

                    // Dash (CR 702.110): the dashed creature gains haste and
                    // returns to its owner's hand at the beginning of the next
                    // end step. Grant haste on the entering instance and arm
                    // the delayed bounce.
                    if dashed
                        && self.battlefield.iter().any(|c| c.id == card_id)
                    {
                        self.grant_keyword_eot(card_id, Keyword::Haste);
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
                            bound_token: None,
                            bound_subject: None,
                            fires_once: true,
                        });
                    }

                    // Blitz (CR 702.152): the blitzed creature gains haste and
                    // "When this creature dies, draw a card," and is sacrificed
                    // at the beginning of the next end step. Grant haste on the
                    // entering instance and arm the two delayed triggers.
                    if self.battlefield.iter().any(|c| c.id == card_id && c.blitzed) {
                        self.grant_keyword_eot(card_id, Keyword::Haste);
                        self.delayed_triggers.push(crate::game::types::DelayedTrigger {
                            controller: caster,
                            source: card_id,
                            kind: crate::game::types::DelayedKind::WhenCardDies(card_id),
                            effect: Effect::Draw {
                                who: crate::effect::Selector::You,
                                amount: crate::effect::Value::Const(1),
                            },
                            target: None,
                            bound_token: None,
                            bound_subject: None,
                            fires_once: true,
                        });
                        self.delayed_triggers.push(crate::game::types::DelayedTrigger {
                            controller: caster,
                            source: card_id,
                            kind: crate::game::types::DelayedKind::NextEndStep,
                            effect: Effect::SacrificeSource,
                            target: None,
                            bound_token: None,
                            bound_subject: None,
                            fires_once: true,
                        });
                    }

                    // Warp (EOE): a permanent cast for its warp cost is exiled
                    // at the beginning of the next end step and may be recast
                    // from exile later for its full cost. Arm a delayed
                    // exile-then-grant-may-play; clear the flag once consumed.
                    if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == card_id)
                        && c.warped
                    {
                        c.warped = false;
                        self.delayed_triggers.push(crate::game::types::DelayedTrigger {
                            controller: caster,
                            source: card_id,
                            kind: crate::game::types::DelayedKind::NextEndStep,
                            effect: Effect::Seq(vec![
                                Effect::Move {
                                    what: crate::effect::Selector::This,
                                    to: crate::effect::ZoneDest::Exile,
                                },
                                Effect::GrantMayPlay {
                                    what: crate::effect::Selector::LastMoved,
                                    duration: crate::card::MayPlayDuration::WhileExiled,
                                    to_owner: true,
                                    exile_after: false,
                                    pay_own_cost: true,
                                    any_color: false,
                                },
                            ]),
                            target: None,
                            bound_token: None,
                            bound_subject: None,
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
                        self.grant_keyword_eot(card_id, Keyword::Haste);
                    }

                    // Push ETB triggers onto the stack — Elesh Norn
                    // replacement adjusts the trigger count (0 = suppressed
                    // by opponent's Norn, 1+N = each of your Norns adds an
                    // extra fire). The spell's `x_value` is threaded so
                    // ETB-trigger expressions like `Effect::AddCounter
                    // { amount: Value::XFromCost }` (Pterafractyl, Static
                    // Prison) read the actual paid X.
                    let base_mult = crate::game::actions::etb_trigger_multiplier(
                        self,
                        caster,
                        Some(card_id),
                    );
                    // Subtype/supertype-keyed trigger doublers (Harmonic
                    // Prodigy, Annie Joins Up) apply to the entrant's own ETB
                    // on the cast path too, not only to `fire_self_etb_triggers`.
                    let etb_multiplier = if base_mult == 0 {
                        0
                    } else {
                        base_mult
                            + crate::game::actions::ally_trigger_extra_fires(self, caster, card_id)
                    };
                    for (effect, filter) in etb_triggers {
                        // CR 603.4 — honor the trigger's intervening-`if`
                        // (`event.filter`) now that the source is on the
                        // battlefield; a false condition skips the fire.
                        if let Some(pred) = &filter {
                            let mut ctx = crate::game::effects::EffectContext::for_trigger(
                                card_id, caster, None, 0,
                            );
                            // Carry the source's cast-mode flags so cast-property
                            // intervening-`if`s (SpellWasKicked / SpellWasBargained
                            // / …) read true when the mode was paid.
                            if let Some(c) = self.battlefield_find(card_id) {
                                ctx.kicked = c.kicked;
                                ctx.bargained = c.bargained;
                                ctx.cast_from_hand = c.cast_from_hand;
                                ctx.cast_via_mayhem = c.cast_via_mayhem;
                            }
                            if !self.evaluate_predicate(pred, &ctx) {
                                continue;
                            }
                        }
                        // Strict Proctor's CR 614 tax — pay {amount} or
                        // sacrifice the source. Applied once per fire.
                        if !crate::game::actions::apply_etb_trigger_tax(
                            self, card_id, caster,
                        ) {
                            // Source sacrificed; remaining ETB triggers moot.
                            break;
                        }
                        // CR 700.2b — modal ETB trigger mode pick at
                        // push-time (Biblioplex Tomekeeper's "choose up
                        // to one — prepare / unprepare").
                        let mode = self.pick_trigger_mode(&effect, card_id, caster);
                        // Per-copy target choice for doubled fires
                        // (CR 603.3d): each copy prefers a target the prior
                        // copies didn't pick, so the second Solitude exile
                        // under Elesh Norn aims at a fresh creature.
                        let mut avoid = vec![card_id];
                        for _ in 0..etb_multiplier {
                            // Converge-aware: an ETB trigger reading the
                            // cast's converge count (Sundering Archaic)
                            // must pick its target under the concrete cap.
                            let auto_target = self.auto_target_for_effect_avoiding_set_xc(
                                &effect,
                                caster,
                                &avoid,
                                x_value,
                                converged_value,
                            );
                            if let Some(Target::Permanent(tid)) = &auto_target {
                                avoid.push(*tid);
                            }
                            // CR 115.1c — an engine-resolved "up to N target"
                            // ETB ability (Azorius Justiciar's detain-two)
                            // maximizes its targets; fill slots 1.. the way the
                            // cast path threads `additional_targets`.
                            let additional = self.auto_extra_targets_for(
                                &effect, card_id, caster, auto_target.clone(),
                            );
                            for t in &additional {
                                if let Target::Permanent(tid) = t {
                                    avoid.push(*tid);
                                }
                            }
                            self.stack.push(
                                TriggerPush::new(card_id, caster, effect.clone())
                                    .target(auto_target.clone())
                                    .additional_targets(additional)
                                    .mode(mode)
                                    .x_value(x_value)
                                    .converged_value(converged_value)
                                    .trigger_source(Some(
                                crate::game::effects::EntityRef::Permanent(card_id),
                            ))
                                    .mana_spent(mana_spent)
                                    .build(),
                            );
                            // CR 603 — a triggered ability that DECLARES a
                            // target slot (printed "target …" wording) fires
                            // "becomes the target" listeners (Tenured
                            // Concocter), same as the cast/activated paths.
                            if effect.requires_target()
                                && let Some(Target::Permanent(tid)) = &auto_target
                                && self.battlefield_find(*tid).is_some()
                            {
                                let evs = vec![GameEvent::BecameTarget {
                                    target: *tid,
                                    caster,
                                }];
                                self.dispatch_triggers_for_events(&evs);
                            }
                        }
                    }

                    // CR 714.2b — a Saga enters with its first lore counter;
                    // chapter I fires off the same lore-counter placement.
                    let is_saga = self
                        .battlefield
                        .iter()
                        .any(|c| c.id == card_id && !c.definition.saga_chapters.is_empty());
                    if is_saga {
                        self.saga_enter_advance(card_id);
                    }

                    // CR 716.2 — a Class enters the battlefield at level 1.
                    if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == card_id)
                        && c.definition.is_class()
                    {
                        c.class_level = 1;
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
                    // Pestilent Spirit — the resolving I/S has deathtouch while
                    // its controller has the granting static.
                    self.resolving_spell_deathtouch_seat =
                        (is_is && self.controller_grants_spell_deathtouch(caster)).then_some(caster);
                    // Track the I/S caster so damage the spell deals can fire
                    // "whenever an instant or sorcery you control deals damage"
                    // (Blaze Commando), once per resolution.
                    self.resolving_spell_caster = is_is.then_some(caster);
                    self.spell_damage_trigger_fired = false;
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
                    self.resolving_spell_deathtouch_seat = None;
                    self.resolving_spell_caster = None;
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
                additional_targets,
                activated: _,
                trigger_player,
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
                // CR 603.10 — scope the dead *subject*'s LKI read too, so
                // Jenova's "draw cards equal to its power" reads the dying
                // Mutant's counter-boosted power (not its printed value).
                let lki_subject = match trigger_source {
                    Some(crate::game::effects::EntityRef::Card(c))
                    | Some(crate::game::effects::EntityRef::Permanent(c))
                        if c != source && self.leaves_bf_lki.contains_key(&c) =>
                    {
                        self.resolving_lki_subject = Some(c);
                        Some(c)
                    }
                    _ => None,
                };
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
                    trigger_player,
                    additional_targets,
                )?;
                if had_lki {
                    self.resolving_lki_source = None;
                    self.leaves_bf_lki.remove(&source);
                }
                if let Some(sid) = lki_subject {
                    self.resolving_lki_subject = None;
                    self.leaves_bf_lki.remove(&sid);
                }
                events.append(&mut trig_events);
                if self.pending_decision.is_some() {
                    return Ok(events);
                }
            }
        }

        // CR 724 — Effect::EndTheTurn fired during this resolution: exile
        // the rest of the stack, clear combat, and jump to cleanup.
        if self.end_turn_requested {
            self.end_turn_requested = false;
            return self.do_end_the_turn(events);
        }

        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);

        Ok(events)
    }

    /// CR 724.1 — end the turn: exile every spell and ability still on the
    /// stack (real cards go to exile; trigger items and token copies cease),
    /// remove everything from combat, then advance from the end step
    /// straight into cleanup (damage wear-off, "this turn" expiry, and the
    /// discard-to-hand-size all happen there as normal).
    pub(crate) fn do_end_the_turn(
        &mut self,
        mut events: Vec<GameEvent>,
    ) -> Result<Vec<GameEvent>, GameError> {
        while let Some(item) = self.stack.pop() {
            if let StackItem::Spell { card, .. } = item
                && !card.is_token
            {
                let cid = card.id;
                self.exile.push(*card);
                events.push(GameEvent::PermanentExiled { card_id: cid });
            }
        }
        // CR 724.1b — remove all attackers and blockers from combat.
        self.attacking.clear();
        self.block_map.clear();
        self.blocked_attackers.clear();
        self.blockers_declared = false;
        // CR 724.1d — the turn skips straight to the cleanup step.
        self.step = TurnStep::End;
        self.advance_step(events)
    }

    // ── Automatic step effects ────────────────────────────────────────────────

    /// CR 702.50a — copy each of the active player's resolved Epic spells
    /// onto the stack at the beginning of their upkeep. The copy keeps the
    /// original targets while they're legal, else auto-picks fresh ones
    /// ("you may choose new targets"; AutoDecider keeps/repairs).
    pub fn process_epic(&mut self) -> Vec<GameEvent> {
        let p = self.active_player_idx;
        let mut events = Vec::new();
        if self.players[p].epic_spells.is_empty() {
            return events;
        }
        let epics = self.players[p].epic_spells.clone();
        for e in epics {
            let Some(def) = crate::catalog::lookup_by_name(&e.name) else { continue };
            let new_id = self.next_id();
            let mut copy = crate::card::CardInstance::new(new_id, def.clone(), p);
            // CR 707.10a — a copy of a spell ceases to exist off the stack.
            copy.is_token = true;
            let target = match &e.target {
                Some(t @ Target::Permanent(tid)) => {
                    let still_legal = self.battlefield_find(*tid).is_some()
                        && self
                            .check_target_legality_with_source(t, p, Some(new_id))
                            .is_ok();
                    if still_legal {
                        e.target.clone()
                    } else {
                        self.auto_target_for_effect(&def.effect, p)
                    }
                }
                other => other.clone(),
            };
            self.stack.push(StackItem::Spell {
                card: Box::new(copy),
                caster: p,
                target,
                additional_targets: e.additional_targets.clone(),
                mode: e.mode,
                x_value: e.x_value,
                converged_value: 0,
                mana_spent: 0,
                uncounterable: true, // copies can't be countered
            });
            events.push(GameEvent::SpellsCopied { original: new_id, count: 1, controller: p });
        }
        events
    }

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
                let applied = self.adjust_life_applied(p, -1);
                if applied < 0 {
                    events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                }
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
    pub fn do_phasing(&mut self) {
        let p = self.active_player_idx;
        // Direct phasers currently in play (computed *before* phase-in so a
        // permanent that phases in this step doesn't immediately phase back
        // out), plus any Aura/Equipment attached to one of them.
        // Computed keywords so layer-granted Phasing (Teferi's Veil-style
        // statics) phases out too, not just the printed/EOT-granted keyword.
        let mut to_phase_out: std::collections::HashSet<crate::card::CardId> = self
            .compute_battlefield()
            .iter()
            .filter(|c| {
                c.controller == p && c.keywords.contains(&crate::card::Keyword::Phasing)
            })
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
        // Phase IN: every phased-out permanent this player controls returns —
        // except "until [source] leaves" phase-outs (CR 702.26 — Out of
        // Time), which `on_left_battlefield` returns instead.
        let mut phased_in: Vec<crate::card::CardId> = Vec::new();
        let mut i = 0;
        while i < self.phased_out.len() {
            if self.phased_out[i].controller == p && self.phased_out[i].phased_out_by.is_none() {
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

    /// CR 502.3 / 122.1d — whether `card_id` is currently stopped from untapping
    /// during its controller's next untap step by a continuous prevention (a
    /// `PreventUntap` static in any of its selector forms, or a stun counter
    /// waiting to be removed). A read-only sibling of the set `do_untap`
    /// computes, surfaced through the server view so the UI can flag locked
    /// permanents. Player-scoped skips (Yosei, Bontu's) aren't included — those
    /// are one-shot flags, not a property of the permanent.
    pub fn untap_prevented_by_static(&self, card_id: crate::card::CardId) -> bool {
        use crate::card::CounterType;
        use crate::effect::{Selector, StaticEffect};
        let Some(card) = self.battlefield_find(card_id) else { return false };
        if card.counter_count(CounterType::Stun) > 0 {
            return true;
        }
        // Every other reason `do_untap` skips a permanent, so the client's
        // "won't untap" badge tells the same story the untap step will.
        if card.skip_next_untap
            || card.definition.keywords.iter().any(|k| {
                matches!(k, crate::card::Keyword::DoesntUntapWhileCounter(kind)
                    if card.counter_count(*kind) > 0)
            })
            || card
                .untap_locked_by
                .is_some_and(|src| self.battlefield_find(src).is_some_and(|s| s.tapped))
            || card
                .untap_locked_while_present
                .is_some_and(|src| self.battlefield_find(src).is_some())
            // Vedalken Shackles / Entrancing Lyre keep *themselves* down while
            // the lock they installed still holds.
            || self
                .temporary_control
                .iter()
                .any(|tc| tc.while_source_tapped && tc.source == Some(card_id))
        {
            return true;
        }
        // Turn-scoped player locks (Bontu's Last Reckoning, Blinding Beam).
        let pl = &self.players[card.controller];
        if (pl.lands_dont_untap_next_untap > 0 && card.definition.is_land())
            || (pl.creatures_dont_untap_next_untap > 0 && card.definition.is_creature())
            || pl.skip_next_untap_step > 0
        {
            return true;
        }
        self.battlefield.iter().any(|c| {
            c.definition.static_abilities.iter().any(|sa| match &sa.effect {
                StaticEffect::PreventUntap { applies_to: Selector::This } => c.id == card_id,
                StaticEffect::PreventUntap { applies_to: Selector::AttachedTo(inner) }
                    if matches!(**inner, Selector::This) =>
                {
                    c.attached_to == Some(card_id)
                }
                StaticEffect::PreventUntap { applies_to: Selector::EachPermanent(req) } => {
                    card.controller == c.controller
                        && self.evaluate_requirement(
                            req,
                            &crate::game::types::Target::Permanent(card_id),
                            card.controller,
                        )
                }
                _ => false,
            })
        })
    }

    pub fn do_untap(&mut self) {
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
        // CR 502.3 — a player who's been made to skip their next untap step
        // (Yosei) doesn't untap; consume one charge. Seedborn-style other
        // untappers still untap their own permanents.
        let active_skips_untap = if self.players[p].skip_next_untap_step > 0 {
            self.players[p].skip_next_untap_step -= 1;
            true
        } else {
            false
        };
        // Bontu's Last Reckoning — the active player's lands skip this untap.
        let active_lands_skip_untap = if self.players[p].lands_dont_untap_next_untap > 0 {
            self.players[p].lands_dont_untap_next_untap -= 1;
            true
        } else {
            false
        };
        // Blinding Beam — the active player's creatures skip this untap.
        let active_creatures_skip_untap =
            if self.players[p].creatures_dont_untap_next_untap > 0 {
                self.players[p].creatures_dont_untap_next_untap -= 1;
                true
            } else {
                false
            };
        let untappers: Vec<usize> = {
            let mut u = if active_skips_untap { vec![] } else { vec![p] };
            for c in &self.battlefield {
                if c.controller != p
                    && !u.contains(&c.controller)
                    && c.definition.static_abilities.iter().any(|sa| {
                        matches!(
                            self.active_static(&sa.effect, c),
                            Some(StaticEffect::UntapAllYoursEachUntapStep)
                        )
                    })
                {
                    u.push(c.controller);
                }
            }
            u
        };
        // CR 502.3 — the filtered sibling (Prophet of Kruphix): an off-turn
        // controller untaps only the permanents matching the static's filter.
        let filtered_untap: std::collections::HashSet<crate::card::CardId> = {
            let filters: Vec<(usize, SelectionRequirement)> = self
                .battlefield
                .iter()
                .filter(|c| c.controller != p && !untappers.contains(&c.controller))
                .flat_map(|c| {
                    c.definition.static_abilities.iter().filter_map(|sa| {
                        match self.active_static(&sa.effect, c) {
                            Some(StaticEffect::UntapYoursEachUntapStepFiltered(f)) => {
                                Some((c.controller, f.clone()))
                            }
                            _ => None,
                        }
                    })
                })
                .collect();
            let mut set = std::collections::HashSet::new();
            // Endbringer — "untap this during each other player's untap step".
            for c in &self.battlefield {
                if c.controller != p
                    && c.definition
                        .static_abilities
                        .iter()
                        .any(|sa| matches!(sa.effect, StaticEffect::UntapSelfEachOtherUntapStep))
                {
                    set.insert(c.id);
                }
            }
            for (seat, req) in &filters {
                for c in &self.battlefield {
                    if c.controller == *seat
                        && self.evaluate_requirement(
                            req,
                            &crate::game::types::Target::Permanent(c.id),
                            *seat,
                        )
                    {
                        set.insert(c.id);
                    }
                }
            }
            set
        };
        // Storage Matrix — while an untapped one is out, each untapping player
        // picks artifact / creature / land and untaps only that type this step.
        // The auto pick is whichever type would free the most permanents.
        let matrix_choice: std::collections::HashMap<usize, crate::card::CardType> = {
            let live = self.battlefield.iter().any(|c| {
                !c.tapped
                    && c.definition.static_abilities.iter().any(|sa| {
                        matches!(sa.effect, StaticEffect::UntapOnlyChosenTypeWhileUntapped)
                    })
            });
            let mut out = std::collections::HashMap::new();
            if live {
                use crate::card::CardType;
                for seat in &untappers {
                    let count = |t: &CardType| {
                        self.battlefield
                            .iter()
                            .filter(|c| {
                                c.controller == *seat
                                    && c.tapped
                                    && c.definition.card_types.contains(t)
                            })
                            .count()
                    };
                    let best = [CardType::Land, CardType::Creature, CardType::Artifact]
                        .into_iter()
                        .max_by_key(|t| count(t))
                        .unwrap();
                    out.insert(*seat, best);
                }
            }
            out
        };
        let prevented: std::collections::HashSet<crate::card::CardId> = {
            let mut blocked = std::collections::HashSet::new();
            // Storage Matrix's off-type permanents are simply blocked.
            if !matrix_choice.is_empty() {
                for c in &self.battlefield {
                    if let Some(t) = matrix_choice.get(&c.controller)
                        && !c.definition.card_types.contains(t)
                    {
                        blocked.insert(c.id);
                    }
                }
            }
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
            // Bontu's Last Reckoning — block the active player's lands.
            if active_lands_skip_untap {
                for c in &self.battlefield {
                    if c.controller == p && c.definition.is_land() {
                        blocked.insert(c.id);
                    }
                }
            }
            // Blinding Beam — block the active player's creatures.
            if active_creatures_skip_untap {
                for c in &self.battlefield {
                    if c.controller == p && c.definition.is_creature() {
                        blocked.insert(c.id);
                    }
                }
            }
            blocked
        };
        // Entrancing Lyre tap-lock: gather the ids referenced as a lock source
        // and the set of currently-tapped permanents. A lock source keeps
        // itself tapped (the "you may choose not to untap this artifact"
        // clause, modeled as: stay tapped while it still locks a creature); a
        // locked permanent skips its untap while its source remains tapped.
        // Vedalken Shackles joins the same set: while it holds a stolen
        // creature it stays tapped ("you may choose not to untap this").
        let lock_sources: std::collections::HashSet<crate::card::CardId> = self
            .battlefield
            .iter()
            .filter_map(|c| c.untap_locked_by)
            .chain(
                self.temporary_control
                    .iter()
                    .filter(|tc| tc.while_source_tapped)
                    .filter_map(|tc| tc.source),
            )
            .collect();
        let tapped_now_set: std::collections::HashSet<crate::card::CardId> =
            self.battlefield.iter().filter(|c| c.tapped).map(|c| c.id).collect();
        // Shipbreaker Kraken — a presence-lock holds while its source is still
        // on the battlefield.
        let on_battlefield: std::collections::HashSet<crate::card::CardId> =
            self.battlefield.iter().map(|c| c.id).collect();
        // CR 502.3 — "Players can't untap more than one [filter] during their
        // untap steps" (Winter Moon, Imi Statue). Pre-resolve each cap's
        // matching permanents (the untap loop below borrows the battlefield
        // mutably), then let each player untap at most one per cap.
        let untap_caps: Vec<std::collections::HashSet<crate::card::CardId>> = self
            .battlefield
            .iter()
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter_map(|sa| match &sa.effect {
                StaticEffect::MaxOneUntapPerStep { filter } => Some(filter),
                _ => None,
            })
            .map(|filter| {
                self.battlefield
                    .iter()
                    .filter(|c| {
                        self.evaluate_requirement_static(
                            filter,
                            &Target::Permanent(c.id),
                            c.controller,
                            None,
                        )
                    })
                    .map(|c| c.id)
                    .collect()
            })
            .collect();
        let mut capped_untaps: std::collections::HashMap<(usize, usize), u32> =
            std::collections::HashMap::new();
        // CR 502.1 — ask each `MayChooseNotToUntap` permanent's controller
        // before the loop (which borrows the battlefield mutably).
        let may_decline: std::collections::HashSet<crate::card::CardId> = {
            let asking: Vec<(crate::card::CardId, usize, &'static str)> = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.tapped
                        && untappers.contains(&c.controller)
                        && c.definition.keywords.contains(&crate::card::Keyword::MayChooseNotToUntap)
                })
                .map(|c| (c.id, c.controller, c.definition.name))
                .collect();
            let mut set = std::collections::HashSet::new();
            for (id, _seat, name) in asking {
                if let crate::decision::DecisionAnswer::Bool(true) =
                    self.decider.decide(&crate::decision::Decision::OptionalTrigger {
                        source: id,
                        description: format!("Leave {name} tapped?"),
                    })
                {
                    set.insert(id);
                }
            }
            set
        };
        // Track which permanents actually flip tapped→untapped so we can
        // fire CR 702.108 Inspired ("becomes untapped") triggers afterward.
        let mut untapped_now: Vec<crate::card::CardId> = Vec::new();
        for card in &mut self.battlefield {
            if untappers.contains(&card.controller) || filtered_untap.contains(&card.id) {
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
                // Entrancing Lyre — a lock source keeps itself tapped while it
                // still locks a creature.
                if card.tapped && lock_sources.contains(&card.id) {
                    if active {
                        card.summoning_sick = false;
                    }
                    continue;
                }
                // Steel Dromedary — "doesn't untap during your untap step if
                // it has a [kind] counter on it".
                if card.definition.keywords.iter().any(|k| {
                    matches!(k, crate::card::Keyword::DoesntUntapWhileCounter(kind)
                        if card.counter_count(*kind) > 0)
                }) {
                    if active {
                        card.summoning_sick = false;
                    }
                    continue;
                }
                // CR 502.1 — "you may choose not to untap this" (Hisoka's
                // Guard). Asked as "leave tapped?", so a headless seat's
                // default `false` untaps normally.
                if card.tapped && may_decline.contains(&card.id) {
                    if active {
                        card.summoning_sick = false;
                    }
                    continue;
                }
                // A locked permanent skips its untap while its source is still
                // tapped on the battlefield; otherwise the lock releases.
                if let Some(src) = card.untap_locked_by {
                    if tapped_now_set.contains(&src) {
                        if active {
                            card.summoning_sick = false;
                        }
                        continue;
                    }
                    card.untap_locked_by = None;
                }
                if let Some(src) = card.untap_locked_while_present {
                    if on_battlefield.contains(&src) {
                        if active {
                            card.summoning_sick = false;
                        }
                        continue;
                    }
                    card.untap_locked_while_present = None;
                }
                // CR 502.3 — a tapped permanent beyond the first this player
                // untaps under any active cap stays tapped.
                if card.tapped
                    && let Some(i) = untap_caps.iter().position(|s| s.contains(&card.id))
                {
                    let n = capped_untaps.entry((i, card.controller)).or_insert(0);
                    if *n >= 1 {
                        if active {
                            card.summoning_sick = false;
                        }
                        continue;
                    }
                    *n += 1;
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
        // CR 502.3 — "untap this during each other player's untap step"
        // (Thousand Moons Infantry). On someone else's untap step, untap each
        // such permanent its controller didn't already untap above (Stun
        // counters still interpose). Summoning sickness is untouched — it only
        // clears on the controller's own turn boundary.
        for card in &mut self.battlefield {
            if untappers.contains(&card.controller) {
                continue;
            }
            if !card
                .definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::UntapSelfEachUntapStep))
            {
                continue;
            }
            if card.counter_count(CounterType::Stun) > 0 {
                card.remove_counters(CounterType::Stun, 1);
            } else if card.tapped {
                untapped_now.push(card.id);
                card.tapped = false;
            }
        }
        // CR 502.3 — an Aura's "Enchanted land untaps during each other
        // player's untap step" (Urban Burgeoning). Collect the hosts whose
        // aura carries the static and whose controller isn't the untapper.
        let untap_hosts: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|src| {
                !untappers.contains(&src.controller)
                    && src.definition.static_abilities.iter().any(|sa| {
                        matches!(sa.effect, StaticEffect::UntapAttachedEachUntapStep)
                    })
            })
            .filter_map(|src| src.attached_to)
            .collect();
        for host in untap_hosts {
            if let Some(card) = self.battlefield_find_mut(host) {
                if card.counter_count(CounterType::Stun) > 0 {
                    card.remove_counters(CounterType::Stun, 1);
                } else if card.tapped {
                    card.tapped = false;
                    untapped_now.push(host);
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
            card.blocked_this_turn = false;
            card.blocked_attackers_this_turn.clear();
        }
        self.players[p].lands_played_this_turn = 0;
        self.players[p].graveyard_cast_types_this_turn.clear();
        // "Protection from everything until your next turn" expires as that
        // player's turn begins (The One Ring).
        self.players[p].protected_from_everything = false;
        // "Opponents' spells cost more until your next turn" expires too
        // (Elspeth Conquers Death II).
        self.turn_scoped_spell_taxes.retain(|t| t.controller != p);
        // "Opponents can't cast spells named X until your next turn"
        // (Academic Probation mode 0) expires as the lock owner's turn begins.
        self.players[p].opponents_cant_cast_named.clear();
        // Stagger damage-doubling windows expire as the registrant's turn
        // begins (Lightning, Army of One).
        self.staggered_damage_players.retain(|(_, reg)| *reg != p);
        // "Until your next turn, whenever a creature attacks you…" floating
        // triggers (Tamiyo +2) expire as their controller's turn begins.
        self.delayed_triggers.retain(|dt| {
            !(matches!(
                dt.kind,
                crate::game::types::DelayedKind::CreatureAttacksYouUntilYourNextTurn
            ) && dt.controller == p)
        });
        self.players[p].extra_land_plays = 0;
        // CR 702.179 — "speed increases once on each of your turns": clear the
        // active player's per-turn speed-bump flag as their turn begins.
        self.players[p].speed_increased_this_turn = false;
        // Raid (CR 702.108): the active player hasn't attacked yet this turn.
        self.players[p].attacked_this_turn = false;
        // "Until your next turn" player grants expire at their owner's untap
        // (Blossoming Calm's hexproof).
        self.players[p].hexproof_until_next_turn = false;
        self.players[p].creatures_attacked_this_turn = 0;
        self.players[p].spells_cast_this_turn = 0;
        self.players[p].spells_cast_from_hand_this_turn = 0;
        // Reset the Bloodthirst "damaged this turn" flag for *every* player
        // at the turn boundary (not just the active player) so a creature
        // cast on your turn reads damage dealt since this turn began.
        for pl in &mut self.players {
            pl.was_dealt_damage_this_turn = false;
            pl.damage_taken_this_turn = 0;
            pl.poison_capped_this_turn = false;
            pl.lost_life_this_turn = false;
            pl.life_lost_this_turn = 0;
            pl.creatures_that_damaged_me_this_turn.clear();
            pl.lands_entered_this_turn = 0;
            pl.creature_spell_countered_by_opponent_this_turn = false;
            pl.noncreature_destroyed_by_opponent_this_turn = false;
            pl.prowl_types_this_turn.clear();
            pl.prowl_any_type_this_turn = false;
            // Veil of Summer's "this turn" riders clear at the turn boundary
            // for every seat (CR 514.2 cleanup-scope grants).
            pl.spells_uncounterable_this_turn = false;
            pl.creature_spells_uncounterable_this_turn = false;
            pl.hexproof_from_colors_this_turn.clear();
            pl.cast_blue_or_black_this_turn = false;
            pl.spell_casts_this_turn.clear();
            pl.statics_ignored_this_turn.clear();
            pl.cant_cast_noncreature_this_turn = false;
            pl.free_spells_from_hand_this_turn = false;
            pl.play_from_graveyard_this_turn = false;
            pl.graveyard_bound_exiled_this_turn = false;
            pl.silenced_this_turn = false;
            pl.warped_spell_this_turn = false;
            pl.searched_library_this_turn = false;
            pl.cards_to_graveyard_this_turn = 0;
            pl.descended_this_turn = false;
            pl.descend_count_this_turn = 0;
            pl.discarded_this_turn.clear();
            pl.permanents_sacrificed_this_turn = 0;
            pl.artifacts_sacrificed_this_turn = 0;
            // CR 702.179 — Freerunning's combat-damage gate is per-turn.
            pl.dealt_combat_damage_to_player_this_turn = false;
            // Quest for Pure Flame's turn-scoped source-damage doubling.
            pl.double_your_source_damage_this_turn = false;
            // CR 700.13 — "committed a crime this turn" resets each turn.
            pl.committed_crime_this_turn = false;
            // CR 708 — "entered face down / turned face up this turn" resets.
            pl.face_down_activity_this_turn = false;
            // CR 401.6 — turn-scoped play-from-top permission ends at cleanup.
            pl.play_from_top_this_turn = false;
            // Johann's once-per-turn top-of-library cast resets each turn.
            pl.cast_from_library_top_this_turn = false;
        }
        // Reset Infusion / "if you gained life this turn" tracking for the
        // active player at the start of their turn. Other players' counters
        // tick down only at their own untaps so symmetric "this turn"
        // checks remain accurate per-player. (Same convention as
        // `lands_played_this_turn` and `spells_cast_this_turn`.)
        self.players[p].life_gained_this_turn = 0;
        // "For the first time each turn" life-gain gates key on the turn
        // boundary itself, so this flag resets for EVERY player at each
        // untap (unlike the per-player-turn tallies above).
        for pl in &mut self.players {
            pl.gained_life_earlier_this_turn = false;
        }
        // Reset cards-drawn tally for the active player. Powers Quandrix
        // scaling cards (Fractal Anomaly's "X = cards drawn this turn"
        // and similar). Other players' tallies advance independently
        // and are reset on their own untap.
        self.players[p].cards_drawn_this_turn = 0;
        // Reset the per-turn {E}-spent tally (Izzet Generatorium's draw gate).
        self.players[p].energy_spent_this_turn = 0;
        // Reset the "cards left your graveyard this turn" tally; powers
        // Lorehold "if a card left your graveyard this turn" payoffs
        // (Living History, Primary Research, Wilt in the Heat) per turn.
        self.players[p].cards_left_graveyard_this_turn = 0;
        // Reset the "creatures died under your control this turn" tally;
        // powers Witherbloom "if a creature died under your control this
        // turn" end-step payoffs (Essenceknit Scholar).
        self.players[p].creatures_died_this_turn = 0;
        self.players[p].zuberas_died_this_turn = 0;
        self.players[p].escalating_resolutions_this_turn = 0;
        // Reset the Revolt (CR 702.139) "permanent left the battlefield under
        // your control this turn" flag for the active player.
        self.players[p].permanent_left_battlefield_this_turn = false;
        // EOE Void — reset the game-wide "a nonland permanent left this turn"
        // flag at the turn boundary.
        self.nonland_permanent_left_bf_this_turn = false;
        // Reset the "cards exiled this turn" tally; powers Strixhaven
        // "if one or more cards were put into exile this turn" payoffs
        // (Ennis the Debate Moderator) per turn.
        self.players[p].cards_exiled_this_turn = 0;
        self.players[p].cards_discarded_this_turn = 0;
        // Reset per-spell-type tallies (instant/sorcery vs creature
        // casts). These refine `spells_cast_this_turn` for cards that
        // need exact-type filtering (Potioner's Trove "instant or
        // sorcery only" gate, future Magecraft variants).
        self.players[p].instants_or_sorceries_cast_this_turn = 0;
        // One-shot IS-spell discounts are keyed off that tally, so they must
        // be cleared in lockstep with it (a stale `granted_at == 0` entry
        // would otherwise re-match after the reset).
        self.players[p].pending_is_discounts.clear();
        self.players[p].pending_spell_discounts.clear();
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
            self.players[q].life_locked_this_turn = false;
            // CR 104.3d — Angel's Grace's protections end with the turn.
            self.players[q].cant_lose_this_turn = false;
            self.players[q].damage_floor_this_turn = false;
        }
        // Forbidding Spirit's "until your next turn" attack tax expires when
        // the taxed player's own turn begins.
        self.players[self.active_player_idx].attack_tax_until_your_turn = 0;
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

    /// CR 514 cleanup step turn-based actions (514.1 discard-down, 514.2
    /// wear-off, 514.3 priority check). See `CleanupOutcome` for how callers
    /// continue.
    pub fn do_cleanup(&mut self, events: &mut Vec<GameEvent>) -> CleanupOutcome {
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
                return CleanupOutcome::Suspended;
            }
            let mut cleanup_events = Vec::new();
            let mut discarded = 0u32;
            while self.players[active].hand.len() > max {
                let Some(cid) = self.players[active].hand.first().map(|c| c.id) else {
                    break;
                };
                self.discard_card(active, cid, &mut cleanup_events);
                discarded += 1;
            }
            // CR 701.9 / 514.3 — the cleanup discard is still "discard one or
            // more cards"; emit the batch so those triggers fire (Containment
            // Construct-style payoffs, "whenever you discard …").
            if discarded > 0 {
                cleanup_events.push(GameEvent::DiscardedBatch { player: active, count: discarded });
            }
            if !cleanup_events.is_empty() {
                // Dispatched here only — the caller re-dispatches whatever it
                // returns, so appending these would double-fire the triggers.
                self.dispatch_triggers_for_events(&cleanup_events);
            }
        }

        self.finish_cleanup(events)
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
    /// step (which may have suspended for a UI player): wear-off, then the
    /// CR 514.3 priority check, then (if nothing is pending) the turn end.
    pub(crate) fn finish_cleanup(&mut self, events: &mut Vec<GameEvent>) -> CleanupOutcome {
        self.cleanup_wear_off();
        // CR 514.3a — check state-based actions and triggered abilities. If
        // anything is waiting, players receive priority in the cleanup step;
        // once they all pass with an empty stack, another cleanup happens.
        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);
        if !self.stack.is_empty() || self.pending_decision.is_some() {
            if self.pending_decision.is_none() {
                self.give_priority_to_active();
            }
            return CleanupOutcome::PriorityGranted;
        }
        // CR 514.3 — normally no player receives priority during cleanup;
        // the turn simply ends.
        self.end_turn();
        CleanupOutcome::TurnOver
    }

    /// CR 514.2 — the cleanup wear-off: clears "until end of turn" / "this
    /// turn" state and removes marked damage.
    fn cleanup_wear_off(&mut self) {
        // Bubbling Muck's floating "Swamps add an extra {B}" grant is a
        // "until end of turn" effect (CR 514.2).
        self.extra_mana_on_land_tap_this_turn.clear();
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
                card.granted_harmonize_eot = None;
            }
            // "[Filter] spells cost {N} less this turn" grants end (CR 514.2).
            player.turn_spell_discounts.clear();
            // "Until end of turn" +1/+1 counter bonus (Prairie Dog) ends.
            player.extra_plus_one_counters_this_turn = 0;
            player.extra_etb_p1p1_counters_this_turn = 0;
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
        self.entered_from_graveyard_this_turn.clear();
        self.entered_from_exile_this_turn.clear();
        // CR 603.3d — "triggers only once each turn" abilities reset.
        self.triggered_once_per_turn_used.clear();
        self.per_subject_trigger_uses.clear();
        // Single Combat — the creature/planeswalker cast lock ends at the end
        // of the registerer's first turn strictly after it was registered
        // ("until the end of your next turn").
        let (active, turn) = (self.active_player_idx, self.turn_number);
        self.creature_pw_cast_locks.retain(|(reg, t)| !(*reg == active && turn > *t));
        // CR 505.1b — discard any unconsumed additional combat phases so they
        // don't bleed into the next turn (e.g. the turn ended before combat).
        self.additional_combat_phases = 0;
        self.combat_chooser = None;
        self.additional_post_main_combats = 0;
        self.combat_phases_this_turn = 0;
        self.additional_end_steps = 0;
        self.end_steps_this_turn = 0;
        self.additional_upkeep_steps = 0;
        self.upkeep_steps_this_turn = 0;
        self.graveyard_from_battlefield_this_turn.clear();
        // Clear all damage from creatures
        for card in &mut self.battlefield {
            card.damage = 0;
        }
        // Clear the per-turn "permanents gained a counter this turn"
        // tracker (used by Fractal Tender's end-step trigger). Resetting
        // at cleanup is the canonical "until end of turn" scope.
        self.permanents_gained_counter_this_turn.clear();
        self.permanents_amplified_counter_this_turn.clear();
        // CR 603-style "Nth time this turn" escalation counters reset.
        self.ability_resolutions_this_turn.clear();
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
                    | crate::game::types::DelayedKind::CreatureYouControlDiesThisTurn
                    | crate::game::types::DelayedKind::MatchingCreatureDiesThisTurn(_)
                    | crate::game::types::DelayedKind::CreatureYouControlDealsCombatDamageThisTurn
                    | crate::game::types::DelayedKind::YourNextSpellCastThisTurn
                    | crate::game::types::DelayedKind::YourNextInstantSorceryCastThisTurn
                    | crate::game::types::DelayedKind::EachCombatThisTurn
                    | crate::game::types::DelayedKind::MatchingCreatureAttacksThisTurn(_)
                    | crate::game::types::DelayedKind::SourceDealsDamageThisTurn(_)
                    | crate::game::types::DelayedKind::YouGainLifeThisTurn
                    | crate::game::types::DelayedKind::CardEntersOpponentGraveyardThisTurn
                    | crate::game::types::DelayedKind::OpponentCausesYouToDiscardThisTurn
            )
        });
        // CR 514.2 / CR 615.1 — "this turn" combat damage prevention
        // (Owlin Shieldmage's ETB, Holy Day-style fogs) expires at
        // cleanup along with the other until-end-of-turn flags.
        self.prevent_combat_damage_this_turn = false;
        self.prevent_combat_damage_except = None;
        self.combat_damage_prevented_creatures.clear();
        self.combat_damage_prevented_to_this_turn.clear();
        self.combat_damage_prevented_by_this_turn.clear();
        self.combat_damage_prevented_to_players_this_turn.clear();
        self.auras_at_death.clear();
        self.creature_etb_steal_this_turn.clear();
        self.search_tax_paid_this_turn.clear();
        self.damage_prevented_sources.clear();
        self.cant_block_pairs.clear();
        self.attack_despite_defender_this_turn.clear();
        // CR 615 — prevention shields and the "can't be prevented" rider
        // are "this turn" effects; they expire at cleanup too.
        self.prevention_shields.clear();
        self.damage_cant_be_prevented_this_turn = false;
        self.damage_redirect_this_turn.clear();
        self.combat_damage_redirect_this_turn.clear();
        self.doubled_damage_sources_this_turn.clear();
        self.damaged_creatures_die_this_turn = false;
        self.creature_deaths_drain_toughness_this_turn = false;
        self.no_search_this_turn = false;
        self.skipped_steps_this_turn.clear();
        self.block_poison_this_turn = 0;
        // CR 500.4 — "kept this turn" mana (Savage Ventmaw) expires now, so the
        // final empty of the turn actually removes it.
        for p in self.players.iter_mut() {
            p.kept_mana_this_turn.empty();
        }
        // Empty mana pools (Kruphix converts to colorless instead).
        self.empty_mana_pools();
    }

    /// The end of the turn: consume extra turns / skip-turn debt, advance the
    /// active player and turn number, and sweep expired play permissions.
    fn end_turn(&mut self) {
        // Rotate the per-turn creature-entry log (Ephara reads "last turn"
        // at each upkeep, so the rotation is per game turn, not per player).
        for pl in &mut self.players {
            pl.creatures_entered_last_turn = std::mem::take(&mut pl.creatures_entered_this_turn);
            pl.artifacts_entered_this_turn = 0;
            pl.planeswalkers_entered_this_turn = 0;
            pl.nonland_permanents_entered_this_turn = 0;
            pl.mounts_vehicles_entered_this_turn = 0;
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
            self.current_turn_is_extra = true;
            self.turn_number += 1;
        } else {
            self.current_turn_is_extra = false;
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
                    crate::card::MayPlayDuration::WhileExiled => false,
                    // Step-bounded miracle windows are also dead by turn end.
                    crate::card::MayPlayDuration::EndOfThisStep => true,
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
        self.remove_permanent_from_combat(id);
    }

    /// CR 506.4 — remove a permanent from combat: it stops being a declared
    /// attacker and stops blocking anything. An attacker it was blocking
    /// remains blocked (CR 509.1b); the permanent stays on the battlefield.
    pub(crate) fn remove_permanent_from_combat(&mut self, id: CardId) {
        self.attacking.retain(|atk| atk.attacker != id);
        self.block_map.remove(&id);
        self.block_map.retain(|_, atks| {
            atks.retain(|a| *a != id);
            !atks.is_empty()
        });
    }

    /// CR 800.4a — handle a player leaving the game: all cards/tokens they
    /// own leave with them (every zone), and permanents they controlled but
    /// don't own revert to their owners' control. Objects leaving this way
    /// are removed directly (not via the death/exile pipelines) since a
    /// departing player's objects "cease to exist" rather than being
    /// destroyed or sacrificed.
    pub(crate) fn objects_leave_with_player(&mut self, p: usize) {
        self.battlefield.retain(|c| c.owner != p);
        let reverts: Vec<(CardId, usize)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == p)
            .map(|c| (c.id, c.owner))
            .collect();
        for (id, owner) in reverts {
            self.change_control(id, owner); // control-changing effects end
        }
        self.exile.retain(|c| c.owner != p);
        self.players[p].hand.clear();
        self.players[p].library.clear();
        self.players[p].graveyard.clear();
        // CR 725.4 — if the monarch leaves the game, the active player
        // becomes the monarch; if the active player is the one leaving (or
        // there is no active player), the next player in turn order who can
        // still become the monarch does.
        if self.monarch == Some(p) {
            self.monarch = if self.active_player_idx != p && !self.players[self.active_player_idx].eliminated {
                Some(self.active_player_idx)
            } else {
                let n = self.players.len();
                (1..n)
                    .map(|off| (self.active_player_idx + off) % n)
                    .find(|&q| q != p && !self.players[q].eliminated)
            };
            let mut events = vec![];
            self.return_monarch_guarded_exiles(self.monarch, &mut events);
        }
    }


    /// CR 704.5u / 702.158c — while a space-sculptor permanent is on the
    /// battlefield, every creature without a sector designation gets one; its
    /// controller chooses, opponents of the sculptor's controller first. When
    /// the last sculptor leaves, the designations clear (CR 702.158b).
    /// Auto-assignment spreads a player's creatures round-robin.
    pub(crate) fn assign_sectors(&mut self) {
        use crate::card::{Keyword, Sector};
        // Hot path: this runs on every SBA check, so bail before allocating
        // when no sculptor is out and nothing carries a designation.
        let mut sculptors: Vec<usize> = Vec::new();
        let mut any_sector = false;
        let mut any_unassigned = false;
        for c in &self.battlefield {
            if c.definition.keywords.contains(&Keyword::SpaceSculptor)
                && !sculptors.contains(&c.controller)
            {
                sculptors.push(c.controller);
            }
            any_sector |= c.sector.is_some();
            any_unassigned |= c.sector.is_none() && c.definition.is_creature();
        }
        if sculptors.is_empty() {
            if any_sector {
                for c in &mut self.battlefield {
                    c.sector = None;
                }
            }
            return;
        }
        if !any_unassigned {
            return;
        }
        // Opponents of a sculptor's controller assign first (CR 702.158c).
        let mut seats: Vec<usize> = (0..self.players.len()).collect();
        seats.sort_by_key(|p| sculptors.contains(p));
        for seat in seats {
            let mut next = self
                .battlefield
                .iter()
                .filter(|c| c.controller == seat && c.sector.is_some())
                .count();
            for c in &mut self.battlefield {
                if c.controller == seat && c.definition.is_creature() && c.sector.is_none() {
                    c.sector = Some(Sector::ALL[next % Sector::ALL.len()]);
                    next += 1;
                }
            }
        }
    }

    pub fn check_state_based_actions(&mut self) -> Vec<GameEvent> {
        let mut events = vec![];

        self.assign_sectors();

        // CR 603.8 — state-triggered flip (Student of Elements: "When this
        // creature has flying, flip it"). Cheap guard so the common board pays
        // nothing; only compute the layer view when an unflipped state-flip
        // card is present, then flip any whose *computed* keywords now satisfy
        // the condition. Flipping clears the condition, so it fires once.
        if self
            .battlefield
            .iter()
            .any(|c| c.definition.flip_when_has_keyword.is_some() && !c.flipped)
        {
            let computed = self.compute_battlefield();
            let to_flip: Vec<CardId> = self
                .battlefield
                .iter()
                .filter_map(|c| {
                    let kw = c.definition.flip_when_has_keyword.as_ref()?;
                    let has = computed.iter().find(|p| p.id == c.id)?.keywords.contains(kw);
                    (has && !c.flipped).then_some(c.id)
                })
                .collect();
            for id in to_flip {
                self.flip_permanent(id, &mut events);
            }
        }

        // CR 603.8 — state-triggered flip on a board predicate (Rune-Tail's
        // "when you have 30 or more life"). Same once-only shape as the
        // keyword variant above.
        let flip_when: Vec<(CardId, usize)> = self
            .battlefield
            .iter()
            .filter(|c| c.definition.flip_when_predicate.is_some() && !c.flipped)
            .map(|c| (c.id, c.controller))
            .collect();
        for (id, ctrl) in flip_when {
            let Some(pred) =
                self.battlefield_find(id).and_then(|c| c.definition.flip_when_predicate.clone())
            else {
                continue;
            };
            let ctx = crate::game::effects::EffectContext::for_ability(id, ctrl, None);
            if self.evaluate_predicate(&pred, &ctx) {
                self.flip_permanent(id, &mut events);
            }
        }

        // CR 603.8 — "when [condition], sacrifice this" state trigger
        // (Phylactery Lich). Evaluated from each carrier's own controller.
        let sacrifice_when: Vec<(CardId, usize)> = self
            .battlefield
            .iter()
            .filter(|c| c.definition.sacrifice_when.is_some())
            .map(|c| (c.id, c.controller))
            .collect();
        for (id, ctrl) in sacrifice_when {
            let Some(pred) =
                self.battlefield_find(id).and_then(|c| c.definition.sacrifice_when.clone())
            else {
                continue;
            };
            let ctx = crate::game::effects::EffectContext::for_ability(id, ctrl, None);
            if self.evaluate_predicate(&pred, &ctx) {
                self.sacrifice_one(id, ctrl, &mut events);
            }
        }

        // CR 603.8 — the general state trigger: "When [condition], [effect]."
        // Latched per permanent so it goes on the stack once while the
        // condition holds and re-arms once it's false again.
        let state_triggers: Vec<(CardId, usize)> = self
            .battlefield
            .iter()
            .filter(|c| c.definition.state_trigger.is_some())
            .map(|c| (c.id, c.controller))
            .collect();
        for (id, ctrl) in state_triggers {
            let Some(st) =
                self.battlefield_find(id).and_then(|c| c.definition.state_trigger.clone())
            else {
                continue;
            };
            let ctx = crate::game::effects::EffectContext::for_ability(id, ctrl, None);
            if !self.evaluate_predicate(&st.condition, &ctx) {
                self.state_trigger_armed.remove(&id);
                continue;
            }
            if self.state_trigger_armed.insert(id) {
                let target = self.auto_target_for_effect(&st.effect, ctrl);
                self.stack.push(
                    crate::game::types::TriggerPush::new(id, ctrl, st.effect)
                        .target(target)
                        .build(),
                );
            }
        }

        // CR 603.8 — "when a player other than its owner controls it" state
        // trigger (Bronze Bombshell): that player sacrifices it, then it deals
        // N damage to them. Latched in `steal_penalty_armed` so it fires once
        // per control change; the latch clears when control returns to owner.
        let steal_penalties: Vec<(CardId, usize, u32)> = self
            .battlefield
            .iter()
            .filter_map(|c| {
                let dmg = c.definition.sacrifice_and_burn_when_stolen?;
                (c.controller != c.owner).then_some((c.id, c.controller, dmg))
            })
            .collect();
        for id in self.battlefield.iter().map(|c| c.id).collect::<Vec<_>>() {
            if let Some(c) = self.battlefield_find(id)
                && c.controller == c.owner
            {
                self.steal_penalty_armed.remove(&id);
            }
        }
        for (id, thief, dmg) in steal_penalties {
            if self.steal_penalty_armed.insert(id) {
                self.push_pending_trigger(
                    crate::game::types::PendingTriggerPush {
                        source: id,
                        controller: thief,
                        effect: Effect::Seq(vec![
                            Effect::SacrificeSource,
                            Effect::DealDamage {
                                to: crate::effect::Selector::Player(crate::effect::PlayerRef::You),
                                amount: crate::card::Value::Const(dmg as i32),
                            },
                        ]),
                        subject: None,
                        event_amount: 0,
                        mode: None,
                        intervening_if: None,
                    },
                    None,
                );
            }
        }

        // CR 611.2c — "for as long as this artifact remains tapped" steals
        // (Vedalken Shackles) end the moment the source untaps or leaves.
        if self.temporary_control.iter().any(|tc| tc.while_source_tapped) {
            let mut kept = Vec::new();
            for tc in std::mem::take(&mut self.temporary_control) {
                let holds = !tc.while_source_tapped
                    || tc.source.and_then(|s| self.battlefield_find(s)).is_some_and(|c| c.tapped);
                if holds {
                    kept.push(tc);
                } else {
                    self.change_control(tc.card, tc.original_controller);
                }
            }
            self.temporary_control = kept;
        }

        // Same clause on the continuous-effect side (Hisoka's Guard's shroud
        // grant): drop it once the granting source is untapped or gone.
        if self
            .continuous_effects
            .iter()
            .any(|e| e.duration == crate::game::layers::EffectDuration::WhileSourceTapped)
        {
            let still_tapped: std::collections::HashSet<CardId> =
                self.battlefield.iter().filter(|c| c.tapped).map(|c| c.id).collect();
            self.continuous_effects.retain(|e| {
                e.duration != crate::game::layers::EffectDuration::WhileSourceTapped
                    || still_tapped.contains(&e.source)
            });
        }

        // CR 603.8 state trigger — "When you control no other [filter],
        // sacrifice this permanent" (Synod Centurion). Latched in
        // `no_other_sacrifice_armed` so it queues once while the condition
        // holds; the latch clears as soon as a match is back.
        let no_other: Vec<(CardId, usize, crate::card::SelectionRequirement)> = self
            .battlefield
            .iter()
            .filter_map(|c| {
                let f = c.definition.sacrifice_when_you_control_no_other.clone()?;
                Some((c.id, c.controller, f))
            })
            .collect();
        for (id, seat, filter) in no_other {
            let alone = !self.battlefield.iter().any(|o| {
                o.id != id
                    && o.controller == seat
                    && self.evaluate_requirement_static(
                        &filter,
                        &crate::game::types::Target::Permanent(o.id),
                        seat,
                        Some(id),
                    )
            });
            if !alone {
                self.no_other_sacrifice_armed.remove(&id);
            } else if self.no_other_sacrifice_armed.insert(id) {
                self.push_pending_trigger(
                    crate::game::types::PendingTriggerPush {
                        source: id,
                        controller: seat,
                        effect: Effect::SacrificeSource,
                        subject: None,
                        event_amount: 0,
                        mode: None,
                        intervening_if: None,
                    },
                    None,
                );
            }
        }

        // CR 704.8 — a permanent that leaves the battlefield during this
        // sweep reports last-known information from the state *before* any of
        // the sweep's actions ran. Snapshot the ±1/±1 counts here, ahead of the
        // 122.3 annihilation, so Persist/Undying read the pre-sweep pile
        // (Young Wolf with a +1/+1 counter that takes three -1/-1 counters
        // dies for good).
        let pre_sba_pm_counters: std::collections::HashMap<CardId, (u32, u32)> = self
            .battlefield
            .iter()
            .map(|c| {
                (
                    c.id,
                    (
                        c.counter_count(crate::card::CounterType::MinusOneMinusOne),
                        c.counter_count(crate::card::CounterType::PlusOnePlusOne),
                    ),
                )
            })
            .collect();

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
            // CR 704.5j reads *current* supertypes, so a continuous grant of
            // the Legendary supertype (Leyline of Singularity, the Ring's
            // emblem) counts. Only pay for the layer computation when such a
            // grant is live; otherwise the printed supertype is authoritative.
            let supertype_grant_active = self.battlefield.iter().any(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(
                        sa.effect,
                        crate::effect::StaticEffect::AllNonlandPermanentsAreLegendary
                    )
                })
            }) || (0..self.players.len()).any(|s| self.players[s].ring_temptations >= 1);
            let is_legendary = |c: &CardInstance| -> bool {
                c.definition.supertypes.contains(&Supertype::Legendary)
                    || (supertype_grant_active
                        && self
                            .computed_permanent(c.id)
                            .is_some_and(|cp| cp.supertypes.contains(&Supertype::Legendary)))
            };
            // CR 704.5j — Mirror Gallery turns the legend rule off entirely.
            if self.battlefield.iter().any(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::LegendRuleDoesntApply)
                })
            }) {
                return Vec::new();
            }
            // Walk descending by id so each group's vec is newest-first.
            let mut by_id: Vec<_> = self
                .battlefield
                .iter()
                .filter(|c| is_legendary(c))
                // Aeve — "isn't legendary if it's a token".
                .filter(|c| !(c.is_token && c.definition.nonlegendary_as_token))
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
            // CR 704.5j exception — a same-name group of exactly two whose
            // members all carry `legend_pair_exempt` (Brothers Yamazaki) is
            // skipped: the legend rule doesn't apply to them.
            let pair_exempt = |dups: &[(CardId, String)]| -> bool {
                dups.len() == 2
                    && dups.iter().all(|(id, _)| {
                        self.battlefield_find(*id)
                            .is_some_and(|c| c.definition.legend_pair_exempt)
                    })
            };
            let mut out = Vec::new();
            for k in order {
                let dups = groups.remove(&k).unwrap_or_default();
                if dups.len() > 1 && !pair_exempt(&dups) {
                    out.push((k.0, k.1.to_string(), dups));
                }
            }
            out
        };
        let legend_victims: Vec<CardId> = {
            let mut victims = Vec::new();
            for (player, name, duplicates) in legend_groups {
                // Ask the controller which to keep; default keeps newest.
                // Auto default: keep the copy carrying the most accumulated
                // board state (counters + attachments), tiebreak newest —
                // "keep newest" alone sacrificed the aura'd/countered copy
                // to a freshly cast vanilla one. Scripted deciders steer.
                let kept = match self.decider.kind() {
                    crate::decision::DeciderKind::Auto => duplicates
                        .iter()
                        .map(|(id, _)| {
                            let inst = self.battlefield.iter().find(|c| c.id == *id);
                            let counters: u32 =
                                inst.map(|c| c.counters.values().sum()).unwrap_or(0);
                            let attached = self
                                .battlefield
                                .iter()
                                .filter(|c| c.attached_to == Some(*id))
                                .count() as u32;
                            (counters + attached, id.0, *id)
                        })
                        .max()
                        .map(|(_, _, id)| id)
                        .unwrap_or(CardId(0)),
                    _ => match self.decider.decide(&crate::decision::Decision::ChooseLegendToKeep {
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
                    },
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
            self.remove_from_battlefield_to_graveyard_raw(id);
        }

        // World rule (CR 704.5k): if two or more permanents have the World
        // supertype, all except the one with the newest timestamp go to their
        // owners' graveyards; on a timestamp tie ALL of them go. Unlike the
        // legend rule this is global, not per-controller.
        let world_victims: Vec<CardId> = {
            let worlds: Vec<(CardId, u64)> = self
                .battlefield
                .iter()
                .filter(|c| c.definition.supertypes.contains(&Supertype::World))
                .map(|c| (c.id, c.battlefield_timestamp))
                .collect();
            if worlds.len() > 1 {
                let newest = worlds.iter().map(|&(_, ts)| ts).max().unwrap();
                let tied = worlds.iter().filter(|&&(_, ts)| ts == newest).count() > 1;
                worlds
                    .into_iter()
                    .filter(|&(_, ts)| tied || ts != newest)
                    .map(|(id, _)| id)
                    .collect()
            } else {
                Vec::new()
            }
        };
        for id in world_victims {
            if let Some(c) = self.battlefield.iter().find(|c| c.id == id) {
                self.died_card_snapshots.insert(id, c.clone());
            }
            self.remove_from_battlefield_to_graveyard_raw(id);
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
            self.remove_from_battlefield_to_graveyard_raw(id);
        }

        // Collect dead creatures using layer-computed toughness.
        let computed = self.compute_battlefield();
        let dead: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| {
                // CR 704.5g/CR 305.7 — use the *computed* type so an animated
                // land (earthbend, creature-lands granted Creature via layers)
                // dies to lethal damage / toughness ≤ 0 like any creature, not
                // just cards printed as creatures.
                let cp = computed.iter().find(|cp| cp.id == c.id);
                let is_creature = cp
                    .map(|cp| cp.card_types.contains(&crate::card::CardType::Creature))
                    .unwrap_or_else(|| c.definition.is_creature());
                if !is_creature {
                    return false;
                }
                // Indestructible stops destruction by damage but NOT by toughness ≤ 0.
                let computed_toughness = cp.map(|cp| cp.toughness).unwrap_or(c.toughness());
                // Toughness ≤ 0 kills even indestructible creatures.
                if computed_toughness <= 0 {
                    return true;
                }
                // CR 704.5g: lethal damage = damage >= toughness.
                // CR 704.5h: any damage from a deathtouch source is lethal.
                // Indestructible creatures don't die to either rule. Read the
                // *computed* keyword so a layer-6 grant (Aura / Equipment /
                // anthem — Shielded by Faith) counts, not just the printed
                // keyword + indestructible counter on the instance.
                let indestructible = cp
                    .map(|cp| cp.keywords.contains(&crate::card::Keyword::Indestructible))
                    .unwrap_or(false)
                    || c.is_indestructible();
                if indestructible {
                    return false;
                }
                // Zilortha — lethal is measured against power, not toughness,
                // for any creature a LethalDamageByPower static matches. The
                // power threshold can be 0, so gate on actual damage being
                // marked (a 0-power creature dies only once it's been dealt
                // damage; an undamaged one survives — CR 704.5g ruling).
                let lethal_threshold = if self.lethal_damage_by_power(c.id) {
                    computed
                        .iter()
                        .find(|cp| cp.id == c.id)
                        .map(|cp| cp.power)
                        .unwrap_or(c.power())
                } else {
                    computed_toughness
                };
                if c.damage > 0 && (c.damage as i32) >= lethal_threshold {
                    return true;
                }
                // Shriveling Rot — "whenever a creature is dealt damage,
                // destroy it" for the rest of the turn. Indestructible has
                // already returned above, matching "destroy".
                if self.damaged_creatures_die_this_turn && c.damage > 0 {
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
                .map(|c| c.regeneration_shields > 0 && !c.cant_regenerate_this_turn)
                .unwrap_or(false);
            if has_regen && !dies_by_lethal_toughness {
                self.apply_regeneration(id);
                continue;
            }
            // CR 702.89 — umbra armor replaces destruction (not the
            // toughness-≤-0 SBA, which isn't destruction).
            if !dies_by_lethal_toughness && self.apply_umbra_armor(id, &mut events) {
                continue;
            }

            self.apply_death_toughness_drain(id);
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
            if let Some(c) = self.dying_snapshot(id) {
                self.died_card_snapshots.insert(id, c);
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
                    let triggers: Vec<(CardId, Effect, usize, Option<crate::card::Predicate>)> = c
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
                        // Keep the intervening/subject filter so the self-death
                        // funnel below can re-check it against the dying creature
                        // (CR 603.4/603.10a). SelfSource triggers usually carry
                        // no filter; a filtered "this or another [type] you
                        // control dies" (YourControl) must not fire when the
                        // dying source itself fails the filter.
                        .map(|t| (c.id, t.effect.clone(), c.controller, t.event.filter.clone()))
                        .collect();
                    let has_persist = c.definition.keywords.contains(&Keyword::Persist);
                    let has_undying = c.definition.keywords.contains(&Keyword::Undying);
                    // CR 704.8 — read the pre-sweep pile, not the post-122.3 one.
                    let (minus, plus) = pre_sba_pm_counters.get(&c.id).copied().unwrap_or((
                        c.counter_count(crate::card::CounterType::MinusOneMinusOne),
                        c.counter_count(crate::card::CounterType::PlusOnePlusOne),
                    ));
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
                        die_triggers.push((id, ta.effect.clone(), controller_idx, ta.event.filter.clone()));
                    }
                }
            }
            // Bump the controller's per-turn died-creature tally for
            // Witherbloom "if a creature died under your control this
            // turn" payoffs (Essenceknit Scholar).
            if controller_idx < self.players.len() {
                self.players[controller_idx].creatures_died_this_turn =
                    self.players[controller_idx].creatures_died_this_turn.saturating_add(1);
                // Zubera cycle: count Zubera deaths separately (read off the
                // still-present dying creature's subtypes).
                if self.battlefield.iter().any(|c| c.id == id
                    && c.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Zubera))
                {
                    self.players[controller_idx].zuberas_died_this_turn =
                        self.players[controller_idx].zuberas_died_this_turn.saturating_add(1);
                }
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
            let was_land = self
                .battlefield
                .iter()
                .find(|c| c.id == id)
                .is_some_and(|c| c.definition.is_land());
            self.remove_from_battlefield_to_graveyard_raw(id);
            // CR 700 — emit the graveyard-arrival event (only when the card
            // actually landed there; Finality / RIP redirects don't count).
            // Emrakul-style "put into a graveyard from anywhere" triggers
            // listen for this.
            if self.players[owner].graveyard.iter().any(|c| c.id == id) {
                events.push(GameEvent::CardPutIntoGraveyard {
                    player: owner,
                    card_id: id,
                    is_land: was_land,
                });
            }
            // Push Dies triggers to the stack for resolution.
            // CR 603.10a — a self-death "this or another creature you control
            // dies" trigger (YourControl scope) reads the dying creature's own
            // mana value through the event amount, so an event-amount-relative
            // target filter (`ManaValueLessThanEventAmount` — Jackdaw Savior)
            // can find a legal target during enumeration and at resolution.
            let died_ev_amount =
                self.event_amount_for(&GameEvent::CreatureDied { card_id: id });
            self.trigger_event_amount_scratch = died_ev_amount;
            for (source, effect, controller, filter) in die_triggers {
                // CR 603.4/603.10a — a filtered self-death trigger ("this or
                // another [type] you control dies") checks its intervening
                // filter against the dying creature, bound as TriggerSource via
                // the cached death snapshot, before firing.
                if let Some(pred) = &filter {
                    let mut ctx = crate::game::effects::EffectContext::for_trigger(
                        source, controller, None, 0,
                    );
                    ctx.trigger_source = crate::game::effects::event_subject(
                        &GameEvent::CreatureDied { card_id: id },
                        &EventKind::CreatureDied,
                    );
                    ctx.event_amount = died_ev_amount;
                    if !self.evaluate_predicate(pred, &ctx) {
                        continue;
                    }
                }
                let auto_target =
                    self.auto_target_for_effect_avoiding(&effect, controller, Some(source));
                self.stack.push(
                    TriggerPush::new(source, controller, effect)
                        .target(auto_target)
                        .event_amount(died_ev_amount)
                        .build(),
                );
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
            self.remove_from_battlefield_to_graveyard_raw(id);
        }

        // CR 310.10 / 704.5x — a battle with no defense counters is defeated.
        let defeated_battles: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| {
                c.definition.is_battle()
                    && c.definition.defense > 0
                    && c.counter_count(crate::card::CounterType::Defense) == 0
            })
            .map(|c| c.id)
            .collect();
        for id in defeated_battles {
            self.defeat_battle(id, &mut events);
        }

        // CR 702.103f — a bestowed permanent that is unattached, or attached
        // to an illegal object, ceases to be bestowed: it stays in play and
        // reverts to a creature (the rule's stated exception to 704.5m, so it
        // never hits the graveyard). Run before the orphan-Aura sweep so it
        // isn't swept away.
        let unbestowed: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.bestowed)
            .filter(|c| match c.attached_to {
                None => true,
                Some(host) => {
                    let Some(h) = self.battlefield.iter().find(|b| b.id == host) else {
                        return true;
                    };
                    // The host must still be legal to enchant: a creature, not
                    // protected from the bestowed Aura, and matching the
                    // printed "enchant ___" filter.
                    self.is_protected_from(c.id, host)
                        || !self
                            .computed_permanent(host)
                            .map(|cp| cp.card_types.contains(&crate::card::CardType::Creature))
                            .unwrap_or_else(|| h.definition.is_creature())
                        || c.definition.aura_enchant_filter().is_some_and(|f| {
                            !self.evaluate_requirement(f, &Target::Permanent(host), c.controller)
                        })
                }
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
            .filter(|c| c.definition.is_aura() && c.attached_to_player.is_none())
            .filter(|c| {
                match c.attached_to {
                    None => true, // not attached to anything
                    Some(attached_id) => !self.battlefield.iter().any(|b| b.id == attached_id),
                }
            })
            .map(|c| c.id)
            .collect();
        for id in orphaned_auras {
            // Record (aura → host) before the Aura leaves, so "whenever an
            // enchanted creature dies" payoffs can count the Auras that were
            // on the dying host (Hateful Eidolon, Dawn Evangel). Only meaningful
            // when the lost host is gone (the common death case).
            if let Some(aura) = self.battlefield.iter().find(|c| c.id == id)
                && let Some(host) = aura.attached_to
                && !self.battlefield.iter().any(|b| b.id == host)
            {
                self.auras_at_death.entry(host).or_default().push((id, aura.controller));
                // Snapshot the leaving Aura so its "when enchanted creature
                // dies" trigger (EnchantedBySource) can fire via LKI even
                // though the Aura itself is gone (Minion's Return).
                self.died_card_snapshots.insert(id, aura.clone());
            }
            // Fire any leaves-the-battlefield triggers on the Aura itself
            // (CR 603.6d) — e.g. Rancor's "return it to its owner's hand".
            events.append(&mut self.remove_to_graveyard_with_triggers(id));
        }

        // CR 704.5n / 704.5m / 303.4f — an Aura attached to an object it can
        // no longer legally enchant (host lost the required type, a "you
        // control" Aura's host changed controllers, or the host gained
        // protection from the Aura) is put into its owner's graveyard.
        // The filter half is only checked when the Aura's "enchant ___"
        // filter is recoverable — distinct from the missing-host sweep
        // above. Bestowed Auras are exempt (their host loss reverts them
        // to creatures, handled earlier).
        let illegally_attached: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.definition.is_aura() && !c.bestowed)
            .filter_map(|c| {
                let host = c.attached_to?;
                if !self.battlefield.iter().any(|b| b.id == host) {
                    return None; // missing host: handled by the sweep above
                }
                // CR 704.5m — protection covers "can't be Enchanted by": a
                // host with protection from the Aura's qualities (its color,
                // card type, "everything", …) sheds the Aura.
                // CR 702.16k — Spectra Ward's "this effect doesn't remove
                // Auras": a `protection_keeps_auras` Aura on the host makes
                // the whole host immune to the protection shed.
                let keeps_auras = self.battlefield.iter().any(|a| {
                    a.attached_to == Some(host)
                        && a.definition
                            .equipped_bonus
                            .as_ref()
                            .is_some_and(|b| b.protection_keeps_auras)
                });
                if !keeps_auras && self.is_protected_from(c.id, host) {
                    return Some(c.id);
                }
                let filter = c.definition.aura_enchant_filter()?;
                if !self.evaluate_requirement(filter, &Target::Permanent(host), c.controller) {
                    Some(c.id)
                } else {
                    None
                }
            })
            .collect();
        for id in illegally_attached {
            events.append(&mut self.remove_to_graveyard_with_triggers(id));
        }

        // CR 704.5y — if a permanent has more than one Role controlled by
        // the same player attached, each but the newest (by battlefield
        // timestamp, CardId tiebreak) goes to its owner's graveyard.
        let stale_roles: Vec<CardId> = {
            let mut by_host: std::collections::HashMap<(CardId, usize), Vec<(u64, CardId)>> =
                std::collections::HashMap::new();
            for c in self.battlefield.iter().filter(|c| {
                c.definition
                    .subtypes
                    .enchantment_subtypes
                    .contains(&crate::card::EnchantmentSubtype::Role)
            }) {
                if let Some(host) = c.attached_to {
                    by_host
                        .entry((host, c.controller))
                        .or_default()
                        .push((c.battlefield_timestamp, c.id));
                }
            }
            by_host
                .into_values()
                .filter(|roles| roles.len() > 1)
                .flat_map(|mut roles| {
                    roles.sort();
                    roles.pop(); // keep the newest
                    roles.into_iter().map(|(_, id)| id)
                })
                .collect()
        };
        for id in stale_roles {
            events.append(&mut self.remove_to_graveyard_with_triggers(id));
        }

        // CR 704.5z — a player who controls a "Start your engines!" permanent
        // and has no speed gets speed 1. (The self-ETB path also seeds it;
        // this SBA covers blink/control-change/token-copy arrivals.)
        for seat in 0..self.players.len() {
            if self.players[seat].speed == 0
                && self.battlefield.iter().any(|c| {
                    c.controller == seat
                        && c.definition
                            .keywords
                            .contains(&crate::card::Keyword::StartYourEngines)
                })
            {
                self.players[seat].speed = 1;
            }
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
                    .any(|b| b.id == attached && b.definition.is_creature())
                    // CR 301.5c — the printed "only to a [filter]" restriction
                    // is checked continuously (Konda's Banner).
                    && c.definition.attach_only_filter.as_ref().is_none_or(|f| {
                        self.evaluate_requirement_static(
                            f,
                            &Target::Permanent(attached),
                            c.controller,
                            Some(c.id),
                        )
                    });
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
            // Phyrexian Unlife — the life half of the loss SBA is skipped
            // (poison / commander losses still apply).
            let unlife = self.player_unlife_active(i);
            let lost = (self.effective_life(i) <= 0 && !unlife)
                || self.players[i].poison_counters >= 10
                || lost_to_commander;
            // CR 104.3d — a player who can't lose (Angel's Grace, Platinum
            // Angel, an opponent's Abyssal Persecutor) skips the loss SBAs;
            // the qualifying state (life ≤ 0, poison ≥ 10) persists.
            // CR 704.7 — a single loss replacement (Lich's Mirror) covers
            // every SBA that would end the game for this player at once.
            if lost && !self.player_cant_lose_game(i) && !self.apply_loss_reset(i) {
                // Stamp the authoritative cause most-specific-first, matching
                // the SBA order that would fire (life, then poison, then
                // commander damage — CR 704.5a/c/v).
                use crate::player::LossCause;
                let cause = if self.effective_life(i) <= 0 && !unlife {
                    LossCause::LifeDepleted
                } else if self.players[i].poison_counters >= 10 {
                    LossCause::Poison
                } else {
                    LossCause::CommanderDamage
                };
                self.players[i].eliminated = true;
                self.players[i].loss_cause.get_or_insert(cause);
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
    /// (`move_card_to`, `remove_from_battlefield_to_graveyard_raw`,
    /// `remove_from_battlefield_to_exile`, etc.) so the post-removal
    /// combat state stays consistent. Prunes `self.attacking` (the
    /// attacker slot) and `self.block_map` (both blocker keys and
    /// attacker values).
    pub(crate) fn remove_from_combat(&mut self, id: CardId) {
        self.attacking.retain(|a| a.attacker != id);
        self.block_map.remove(&id);
        self.block_map.retain(|_, atks| {
            atks.retain(|a| *a != id);
            !atks.is_empty()
        });
    }

    /// **Raw** battlefield→graveyard move: zone change + replacements only.
    /// Fires NO dies/LTB triggers, no Persist/Undying, no sacrifice events —
    /// callers must handle those themselves (the SBA paths do). New effect
    /// arms should use `remove_to_graveyard_with_triggers` or
    /// `sacrifice_one` instead (audit P3: death-funnel bypass family).
    pub fn remove_from_battlefield_to_graveyard_raw(&mut self, id: CardId) {
        if let Some(mut card) = Self::take_card(&mut self.battlefield, id) {
            self.remove_effects_from_source(id);
            self.remove_from_combat(id);
            self.collect_leaver_counters(&card);
            // Churning Reservoir: an oil-countered permanent bound for a
            // graveyard flips its controller's oil-activity flag.
            if card.counter_count(crate::card::CounterType::Oil) > 0 {
                self.players[card.controller].oil_activity_this_turn = true;
            }
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
            let valentin_redirect: Option<(CardId, usize, Option<Effect>)> = if card.definition.is_creature()
                && !card.is_token
            {
                self.battlefield.iter().find_map(|src| {
                    src.definition.static_abilities.iter().find_map(|sa| {
                        if let crate::effect::StaticEffect::ExileDyingOpponentCreatures { when_you_do } =
                            &sa.effect
                            && src.controller != card.controller
                        {
                            Some((src.id, src.controller, when_you_do.as_deref().cloned()))
                        } else {
                            None
                        }
                    })
                })
            } else {
                None
            };
            // CR 614 — "If this permanent would be put into a graveyard, put
            // it on top of its owner's library instead" (Pulmonic Sliver's
            // Sliver-wide grant; the "may" is auto-taken).
            let library_top_redirect = self.battlefield.iter().any(|src| {
                src.definition.static_abilities.iter().any(|sa| {
                    matches!(&sa.effect,
                        crate::effect::StaticEffect::DiesToLibraryTopInstead { filter }
                        if crate::game::layers::requirement_matches_card(
                            filter, &card, src.controller))
                })
            });
            // CR 614 — "…would die, return it to its owner's hand instead"
            // (Necromancer's Magemark). Same site as the library-top redirect.
            let hand_redirect = self
                .battlefield
                .iter()
                .filter_map(|src| {
                    src.definition.static_abilities.iter().find_map(|sa| match &sa.effect {
                        crate::effect::StaticEffect::DiesToOwnersHandInstead { filter } => {
                            Some((filter.clone(), src.controller))
                        }
                        _ => None,
                    })
                })
                .collect::<Vec<_>>()
                .iter()
                // The stateful evaluator, not `requirement_matches_card` — the
                // filter is typically `IsEnchanted`, which has to scan the
                // battlefield for the Aura still attached to the dying card.
                .any(|(filter, ctrl)| self.evaluate_requirement_on_card(filter, &card, *ctrl));
            let initial_to = if card.counter_count(crate::card::CounterType::Finality) > 0
                || self.dies_to_exile_eot.contains(&id)
                || card.definition.dies_to_exile
                || valentin_redirect.is_some()
            {
                crate::card::Zone::Exile
            } else if library_top_redirect || card.definition.dies_to_library_bottom {
                crate::card::Zone::Library
            } else if hand_redirect {
                crate::card::Zone::Hand
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
                self.graveyard_from_battlefield_this_turn.insert(id);
                // CR 700.4 — record the death for the batched `PermanentDied`
                // synthesis (dispatch drains this into "creature or artifact
                // you control dies" triggers). CreatureDied already covers
                // creatures at every site; this backfills non-creature deaths.
                self.pending_permanent_deaths.push((
                    id,
                    card.controller,
                    card.definition.is_creature(),
                    card.definition.card_types.contains(&crate::card::CardType::Artifact),
                ));
            }
            // CR 702.139 — Revolt: a permanent left the battlefield under its
            // controller this turn.
            if card.controller < self.players.len() {
                self.players[card.controller].permanent_left_battlefield_this_turn = true;
            }
            // EOE Void — a nonland permanent left the battlefield this turn.
            if !card.definition.is_land() {
                self.nonland_permanent_left_bf_this_turn = true;
            }
            // Stamp `exiled_with` so the static's controller can recur the
            // card later (Gisa, Glorious Resurrector's upkeep mass-reanimate).
            if let (Some((src_id, _, _)), crate::card::Zone::Exile) =
                (&valentin_redirect, resolved)
            {
                card.exiled_with = Some(*src_id);
            }
            self.place_card_at_resolved_zone(card, resolved);
            let mut events = Vec::new();
            self.on_left_battlefield(id, &mut events);
            // Fire Valentin's reflexive "when you do, …" for the static's
            // controller (CR 603.x reflexive trigger off the replacement).
            if let Some((_src, controller, Some(effect))) = valentin_redirect {
                let auto_target =
                    self.auto_target_for_effect_avoiding(&effect, controller, None);
                self.stack.push(
                    TriggerPush::new(id, controller, effect)
                        .target(auto_target)
                        .build(),
                );
            }
        }
    }

    /// Return a permanent to its owner's hand, unwinding combat/attachments and
    /// emitting the leave-the-battlefield events. Used by cost lines that bounce
    /// the source (`ActivatedAbility::bounce_self_cost` — Magosi).
    pub fn remove_from_battlefield_to_hand(&mut self, id: CardId) -> Vec<GameEvent> {
        let mut events = Vec::new();
        let Some(mut card) = Self::take_card(&mut self.battlefield, id) else {
            return events;
        };
        self.remove_effects_from_source(id);
        self.remove_from_combat(id);
        self.collect_leaver_counters(&card);
        let owner = card.owner;
        // CR 122.2 / 702.139 — counters vanish on the zone change, and the
        // "a permanent left the battlefield this turn" watchers latch.
        card.counters.clear();
        card.keyword_counters.clear();
        if card.controller < self.players.len() {
            self.players[card.controller].permanent_left_battlefield_this_turn = true;
        }
        if !card.definition.is_land() {
            self.nonland_permanent_left_bf_this_turn = true;
        }
        let leaver = card.definition.is_creature().then_some((card.id, card.controller));
        if !card.is_token {
            self.players[owner].hand.push(card);
        }
        self.on_left_battlefield(id, &mut events);
        if let Some((card_id, controller)) = leaver {
            events.push(GameEvent::CreatureLeftWithoutDying { card_id, controller });
        }
        events.push(GameEvent::PermanentReturnedToHand { card_id: id, player: owner });
        events
    }

    pub fn remove_from_battlefield_to_exile(&mut self, id: CardId) {
        if let Some(card) = Self::take_card(&mut self.battlefield, id) {
            self.remove_effects_from_source(id);
            self.remove_from_combat(id);
            self.collect_leaver_counters(&card);
            let resolved = self.resolve_zone_change(
                id,
                crate::card::Zone::Battlefield,
                crate::card::Zone::Exile,
            );
            // CR 702.139 — Revolt: a permanent left the battlefield this turn.
            if card.controller < self.players.len() {
                self.players[card.controller].permanent_left_battlefield_this_turn = true;
            }
            // EOE Void — a nonland permanent left the battlefield this turn.
            if !card.definition.is_land() {
                self.nonland_permanent_left_bf_this_turn = true;
            }
            // CR 603.6 — exile is a non-graveyard exit: a creature leaves
            // without dying (Dour Port-Mage / Three Tree Scribe watchers).
            let leaver =
                (card.definition.is_creature()).then_some((card.id, card.controller));
            self.place_card_at_resolved_zone(card, resolved);
            let mut events = Vec::new();
            self.on_left_battlefield(id, &mut events);
            if let Some((card_id, controller)) = leaver {
                events.push(GameEvent::CreatureLeftWithoutDying { card_id, controller });
            }
            self.dispatch_triggers_for_events(&events);
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
        // CR 712.16/712.17 — a melded permanent leaves the battlefield as
        // its two component cards.
        let mut card = card;
        if !card.meld_parts.is_empty() && zone != Zone::Battlefield {
            for part in std::mem::take(&mut card.meld_parts) {
                self.place_card_at_resolved_zone(part, zone);
            }
            return;
        }
        // CR 702.140e — a merged (mutated) permanent leaves as its components.
        if !card.mutate_stack.is_empty() && zone != Zone::Battlefield {
            for part in std::mem::take(&mut card.mutate_stack) {
                self.place_card_at_resolved_zone(part, zone);
            }
            return;
        }
        // CR 702.95h — a card leaving the battlefield is no longer Soulbond-
        // paired. Clear its own link so a later re-entry can re-pair cleanly
        // (the SBA in `check_state_based_actions` clears the partner's side).
        card.soulbond_partner = None;
        // CR 708.10 — a face-down permanent is turned face up as it leaves
        // the battlefield (no-op unless it carries a stashed real definition).
        card.turn_face_up();
        // The graveyard→exile redirect (Rest in Peace / Leyline / Disturb back
        // face, CR 614.6 / 702.146e) and its void-counter rider read the back
        // face, so capture them *before* the CR 712.4 front-face revert.
        let exile_on_graveyard = self.graveyard_exiled_for(&card) || card.disturb_back_exiles();
        let void_counter_on_exile = self.graveyard_exile_redirects(&card).1;
        // CR 710.4 / 712.4 — flip cards and transformed DFCs revert to their
        // unflipped / front face off the battlefield.
        card.revert_flip();
        card.revert_transform();
        // CR 702.160c — a prototype permanent has only its printed
        // (full, colorless) characteristics off the battlefield.
        card.revert_prototype();
        // CR 709.5c — Room unlocked designations are battlefield-only.
        card.reset_room_doors();
        // MKM — a Case's solved designation is battlefield-only.
        card.reset_case();
        // CR 707 — a temporary copy reverts as it leaves.
        self.revert_copy_on_leave(&mut card);
        match zone {
            // CR 614.6 — "shuffle into its owner's library instead"
            // (Darksteel Colossus); the card never touches the graveyard.
            Zone::Graveyard if card.definition.shuffles_into_library_instead => {
                use rand::seq::SliceRandom;
                self.players[owner].library.push(card);
                let mut rng = rand::rng();
                self.players[owner].library.shuffle(&mut rng);
            }
            // CR 614.6 — Rest in Peace / Leyline of the Void redirect the
            // graveyard arrival to exile; CR 702.146e — so does a Disturb
            // back face.
            Zone::Graveyard if exile_on_graveyard => {
                let mut card = card;
                if void_counter_on_exile {
                    card.add_counters(crate::card::CounterType::Void, 1);
                }
                self.exile.push(card)
            }
            Zone::Graveyard => self.players[owner].send_to_graveyard(card),
            Zone::Exile => self.exile.push(card),
            Zone::Hand => self.players[owner].hand.push(card),
            // CR 614.6 — "put it on the bottom of its owner's library
            // instead" (Nissa's Chosen).
            Zone::Library if card.definition.dies_to_library_bottom => {
                let owner = card.owner;
                self.players[owner].library.push(card);
            }
            // Top of owner's library. Replacement effects don't carry
            // a position field today; if a future replacement needs
            // bottom / shuffled, extend the type.
            Zone::Library => {
                // CR 122.2 — counters and battlefield state don't survive
                // the zone change.
                card.counters.clear();
                card.keyword_counters.clear();
                card.damage = 0;
                card.tapped = false;
                self.players[owner].library.insert(0, card)
            }
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
    /// Shriveling Rot mode 2 — while
    /// `creature_deaths_drain_toughness_this_turn` is set, a creature leaving
    /// the battlefield for the graveyard drains its controller for its
    /// *computed* toughness. Called from both death funnels (the SBA lethal
    /// sweep and the destroy/sacrifice route) before the card is removed.
    pub(crate) fn apply_death_toughness_drain(&mut self, id: CardId) {
        if !self.creature_deaths_drain_toughness_this_turn {
            return;
        }
        let Some(controller) = self.battlefield.iter().find(|c| c.id == id).map(|c| c.controller)
        else {
            return;
        };
        let toughness = self
            .computed_permanent(id)
            .map(|cp| cp.toughness)
            .or_else(|| self.battlefield.iter().find(|c| c.id == id).map(|c| c.toughness()))
            .unwrap_or(0);
        if toughness > 0 {
            self.adjust_life(controller, -toughness);
        }
    }

    pub fn remove_to_graveyard_with_triggers(&mut self, id: CardId) -> Vec<GameEvent> {
        // Collect both `CreatureDied` and `PermanentLeavesBattlefield`
        // self-source triggers off the leaving permanent. CreatureDied
        // only matters for creatures (Solitude evoke-sac etc.);
        // PermanentLeavesBattlefield is the broader "when this leaves the
        // battlefield" hook used by Chromatic Star, Roomba-style cards,
        // and any future non-creature die-trigger.
        // Hushbringer (CR 614): creature-death triggers are suppressed while
        // a `SuppressCreatureEtbTriggers { also_dies }` static is in play.
        let dies_suppressed = crate::game::actions::creature_dies_triggers_suppressed(self);
        // CR 700.4 — "dies" means put into a graveyard from the battlefield.
        // Under a graveyard→exile replacement (Rest in Peace, Leyline of the
        // Void, void counters) the card never dies, so its own dies triggers,
        // equipment-granted dies triggers, and the died tally are suppressed
        // (Persist/Undying already no-op — the card never reaches the
        // graveyard to be returned from).
        let exiled_instead = self
            .battlefield
            .iter()
            .find(|c| c.id == id)
            .is_some_and(|c| self.graveyard_exiled_for(c) || c.disturb_back_exiles());
        let dies_suppressed = dies_suppressed || exiled_instead;
        let (leave_triggers, dying_creature_controller): (Vec<DeathTrigger>, Option<usize>) = self
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
                    // CR 603.10a — the dying creature's own self-fire death
                    // triggers. SelfSource always, plus the self-inclusive
                    // scopes ("this or another … you control dies" —
                    // YourControl) so a destroyed/sacrificed aristocrat drains
                    // for its own death, matching the SBA lethal-damage path.
                    .filter(|t| matches!(
                        t.event.scope,
                        EventScope::SelfSource
                            | EventScope::YourControl
                            | EventScope::AnyPlayer
                            | EventScope::ActivePlayer,
                    ))
                    .filter(|t| match t.event.kind {
                        EventKind::PermanentLeavesBattlefield => true,
                        EventKind::CreatureDied => is_creature && !dies_suppressed,
                        // CR 700.4 — "when this is put into a graveyard from the
                        // battlefield" for a non-creature (Wicked Role token's
                        // death-drain). Suppressed under a graveyard→exile
                        // replacement just like CreatureDied.
                        EventKind::PermanentDied => !dies_suppressed,
                        _ => false,
                    })
                    .map(|t| (c.id, t.effect.clone(), c.controller, t.event.filter.clone()))
                    .collect();
                let creature_controller = if is_creature { Some(c.controller) } else { None };
                (triggers, creature_controller)
            })
            .unwrap_or_default();
        // CR 702.6e — Equipment-granted "dies" triggers fire as though
        // printed on the equipped creature (Skullclamp). The SBA lethal
        // path collects these too; without this the Destroy / sacrifice
        // funnels dropped them.
        let mut leave_triggers = leave_triggers;
        if let Some(controller) = dying_creature_controller
            && !dies_suppressed
        {
            for eq in &self.battlefield {
                if eq.attached_to != Some(id) {
                    continue;
                }
                let Some(bonus) = &eq.definition.equipped_bonus else { continue };
                for ta in &bonus.triggered_abilities {
                    if ta.event.kind == EventKind::CreatureDied {
                        leave_triggers.push((id, ta.effect.clone(), controller, ta.event.filter.clone()));
                    }
                }
            }
        }
        if dying_creature_controller.is_some() {
            self.apply_death_toughness_drain(id);
        }
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
            && !exiled_instead
        {
            self.players[controller_idx].creatures_died_this_turn =
                self.players[controller_idx].creatures_died_this_turn.saturating_add(1);
            if self.battlefield.iter().any(|c| c.id == id
                && c.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Zubera))
            {
                self.players[controller_idx].zuberas_died_this_turn =
                    self.players[controller_idx].zuberas_died_this_turn.saturating_add(1);
            }
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
        self.remove_from_battlefield_to_graveyard_raw(id);
        let mut out = Vec::new();
        if let Some((owner, is_land)) = gy_info
            && self.players[owner].graveyard.iter().any(|c| c.id == id)
        {
            out.push(GameEvent::CardPutIntoGraveyard { player: owner, card_id: id, is_land });
        }
        for (source, effect, controller, filter) in leave_triggers {
            // CR 603.4/603.10a — a filtered self-death trigger checks its
            // intervening filter against the dying creature (bound as
            // TriggerSource via the cached death snapshot) before firing.
            if let Some(pred) = &filter {
                let mut ctx = crate::game::effects::EffectContext::for_trigger(
                    source, controller, None, 0,
                );
                ctx.trigger_source = crate::game::effects::event_subject(
                    &GameEvent::CreatureDied { card_id: id },
                    &EventKind::CreatureDied,
                );
                ctx.event_amount = self.event_amount_for(&GameEvent::CreatureDied { card_id: id });
                if !self.evaluate_predicate(pred, &ctx) {
                    continue;
                }
            }
            // Drivnod, Carnage Dominus — a creature dying causes this trigger,
            // so it fires an additional time per Drivnod its controller runs.
            let fires = 1 + if dying_creature_controller.is_some() {
                self.battlefield
                    .iter()
                    .filter(|c| c.controller == controller)
                    .flat_map(|c| &c.definition.static_abilities)
                    .filter(|sa| {
                        matches!(
                            sa.effect,
                            crate::effect::StaticEffect::DoubleControllerDeathTriggers
                        )
                    })
                    .count()
            } else {
                0
            };
            for _ in 0..fires {
                let auto_target =
                    self.auto_target_for_effect_avoiding(&effect, controller, Some(source));
                self.stack.push(
                    TriggerPush::new(source, controller, effect.clone())
                        .target(auto_target)
                        .build(),
                );
            }
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
        if let Some(mut returned) = Self::take_card(&mut self.players[owner].graveyard, id) {
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
