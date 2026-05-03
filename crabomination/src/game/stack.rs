use super::*;
use crate::card::{Keyword, Supertype};
use crate::effect::{Effect, EventKind, EventScope};
use crate::game::types::{DelayedKind, DelayedTrigger};

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

        let mut events = vec![];

        if self.step == TurnStep::Cleanup {
            self.do_cleanup();
        }

        // Skip FirstStrikeDamage if no first/double-strike creatures are in combat.
        let mut next = self.step.next();
        if next == TurnStep::FirstStrikeDamage && !self.has_first_strikers() {
            next = next.next(); // skip directly to CombatDamage
        }

        self.step = next;
        events.push(GameEvent::StepChanged(next));

        match next {
            // Untap has no priority window — auto-execute and move on.
            TurnStep::Untap => {
                self.do_untap();
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
                    match self.players[p].draw_top() {
                        Some(id) => events.push(GameEvent::CardDrawn {
                            player: p,
                            card_id: id,
                        }),
                        None => {
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
                }
                self.give_priority_to_active();
            }
            TurnStep::Upkeep => {
                self.fire_step_triggers(TurnStep::Upkeep);
                self.give_priority_to_active();
            }
            TurnStep::PreCombatMain => {
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
                self.give_priority_to_active();
            }
            TurnStep::CombatDamage => {
                let mut combat_events = self.resolve_combat()?;
                events.append(&mut combat_events);
                self.give_priority_to_active();
            }
            TurnStep::End => {
                self.fire_step_triggers(TurnStep::End);
                self.give_priority_to_active();
            }
            TurnStep::Cleanup => {
                // Reset per-turn spell counter.
                self.spells_cast_this_turn = 0;
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
        let mut triggers: Vec<(CardId, Effect, usize)> = self
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
                    })
                    .map(|t| (c.id, t.effect.clone(), c.controller))
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
                        triggers.push((c.id, t.effect.clone(), c.owner));
                    }
                }
            }
        }

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

        for (source, effect, controller, target) in delayed_to_fire {
            // If the delayed trigger was registered with no captured
            // target (e.g. rebound's re-cast), pick a fresh auto-target at
            // fire time so the trigger doesn't enter the stack with a
            // None target slot.
            let target = target.or_else(|| {
                self.auto_target_for_effect_avoiding(&effect, controller, Some(source))
            });
            self.stack.push(StackItem::Trigger {
                source,
                controller,
                effect: Box::new(effect),
                target,
                mode: None,
                x_value: 0,
                converged_value: 0,
                // Delayed trigger — subject defaults to source permanent
                // (the original trigger's stored subject is not currently
                // threaded through `delayed_triggers`).
                subject: Some(crate::game::effects::EntityRef::Permanent(source)),
            });
        }

        for (source, effect, controller) in triggers {
            // Triggered abilities pass the trigger source so the picker
            // can avoid pumping the source itself when a better target
            // exists (Strixhaven Magecraft / Repartee, etc.).
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
                // Step trigger — subject defaults to source permanent.
                subject: Some(crate::game::effects::EntityRef::Permanent(source)),
            });
        }
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
                mode,
                x_value,
                converged_value,
                uncounterable: _,
                face,
                is_copy,
            } => {
                let card = *card;
                let card_id = card.id;
                let is_noncreature = !card.definition.is_creature();

                if card.definition.is_permanent() {
                    // Collect ETB triggers before moving card into battlefield.
                    let etb_triggers: Vec<Effect> = card
                        .definition
                        .triggered_abilities
                        .iter()
                        .filter(|t| t.event.kind == EventKind::EntersBattlefield
                            && matches!(t.event.scope, EventScope::SelfSource))
                        .map(|t| t.effect.clone())
                        .collect();
                    let evoked = card.evoked;
                    let mut card = card;
                    // Aura targeting: an Aura spell pre-binds its enchanted
                    // permanent to its `attached_to` field at the moment it
                    // enters the battlefield, so the orphaned-aura SBA
                    // (CR 704.5m — "an Aura that isn't attached to an
                    // object or player is put into its owner's graveyard")
                    // doesn't immediately graveyard the aura between bf
                    // entry and the cast-target snapshot. The cast's
                    // target (slot 0) is always a permanent for Auras
                    // (CR 303.4f).
                    if card.definition.is_aura()
                        && card.attached_to.is_none()
                        && let Some(crate::game::types::Target::Permanent(t_id)) = &target
                    {
                        card.attached_to = Some(*t_id);
                    }
                    self.battlefield.push(card);
                    events.push(GameEvent::PermanentEntered { card_id });

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
                            subject: Some(crate::game::effects::EntityRef::Permanent(card_id)),
                        });
                    }

                    // Push ETB triggers onto the stack — Elesh Norn
                    // replacement adjusts the trigger count (0 = suppressed
                    // by opponent's Norn, 1+N = each of your Norns adds an
                    // extra fire). The spell's `x_value` is threaded so
                    // ETB-trigger expressions like `Effect::AddCounter
                    // { amount: Value::XFromCost }` (Pterafractyl, Static
                    // Prison) read the actual paid X.
                    let etb_multiplier =
                        crate::game::actions::etb_trigger_multiplier(self, caster);
                    for effect in etb_triggers {
                        let auto_target = self.auto_target_for_effect_avoiding(
                            &effect,
                            caster,
                            Some(card_id),
                        );
                        for _ in 0..etb_multiplier {
                            self.stack.push(StackItem::Trigger {
                                source: card_id,
                                controller: caster,
                                effect: Box::new(effect.clone()),
                                target: auto_target.clone(),
                                mode: None,
                                x_value,
                                converged_value,
                                // ETB SelfSource — subject is the
                                // entering permanent.
                                subject: Some(crate::game::effects::EntityRef::Permanent(card_id)),
                            });
                        }
                    }

                    // AnotherOfYours creature ETB triggers.
                    if self
                        .battlefield
                        .last()
                        .map(|c| c.id == card_id && c.definition.is_creature())
                        .unwrap_or(false)
                    {
                        let other_triggers: Vec<(CardId, Effect)> = self
                            .battlefield
                            .iter()
                            .filter(|c| c.id != card_id && c.controller == caster)
                            .flat_map(|c| {
                                c.definition
                                    .triggered_abilities
                                    .iter()
                                    .filter(|t| t.event.kind == EventKind::EntersBattlefield
                                        && matches!(t.event.scope, EventScope::AnotherOfYours))
                                    .map(|t| (c.id, t.effect.clone()))
                            })
                            .collect();
                        // Elesh Norn replacement: each listener's trigger
                        // count is determined by the listener's controller
                        // (which equals `caster` here, so we reuse the
                        // multiplier we'd compute for self-source above).
                        let aoy_multiplier =
                            crate::game::actions::etb_trigger_multiplier(self, caster);
                        for (src, effect) in other_triggers {
                            let auto_target =
                                self.auto_target_for_effect_avoiding(&effect, caster, Some(src));
                            for _ in 0..aoy_multiplier {
                                self.stack.push(StackItem::Trigger {
                                    source: src,
                                    controller: caster,
                                    effect: Box::new(effect.clone()),
                                    target: auto_target.clone(),
                                    mode: None,
                                    x_value: 0,
                                    converged_value: 0,
                                    // AnotherOfYours ETB — subject is
                                    // the entering creature.
                                    subject: Some(crate::game::effects::EntityRef::Permanent(card_id)),
                                });
                            }
                        }
                    }
                } else {
                    let chosen_mode = mode.unwrap_or(0);
                    // Stamp the cast face from the StackItem onto the
                    // resolving card via the kicked-flashback marker if
                    // applicable. The Flashback face is the only one
                    // that needs special routing on resolve (exile, not
                    // graveyard); Back-face MDFCs always exile-or-not
                    // based on their own keywords.
                    //
                    // `is_copy` is threaded through so a copied spell
                    // ceases to exist on resolution (no graveyard / exile
                    // step) — see `continue_spell_resolution_with_face`.
                    let mut spell_events = self.continue_spell_resolution_with_face_copy(
                        card,
                        caster,
                        target,
                        chosen_mode,
                        x_value,
                        converged_value,
                        None,
                        face,
                        is_copy,
                    )?;
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
                subject,
            } => {
                let chosen_mode = mode.unwrap_or(0);
                let mut trig_events = self.continue_trigger_resolution(
                    source,
                    controller,
                    *effect,
                    target,
                    chosen_mode,
                    x_value,
                    converged_value,
                    subject,
                )?;
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

    pub(crate) fn do_untap(&mut self) {
        let p = self.active_player_idx;
        // Untap permanents YOU CONTROL on your untap step, not just
        // those you originally owned. A creature you've stolen
        // (Threaten / Mind Control) untaps on your turn; one of yours
        // that's been stolen does not. Filtering by `owner` here would
        // leave stolen permanents permanently tapped (or, conversely,
        // un-tap a stolen permanent on the wrong player's turn).
        for card in &mut self.battlefield {
            if card.controller == p {
                card.tapped = false;
                card.summoning_sick = false;
            }
        }
        self.players[p].lands_played_this_turn = 0;
        self.players[p].spells_cast_this_turn = 0;
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
        // Reset the "cards exiled this turn" tally; powers Strixhaven
        // "if one or more cards were put into exile this turn" payoffs
        // (Ennis the Debate Moderator) per turn.
        self.players[p].cards_exiled_this_turn = 0;
        // Reset per-spell-type tallies (instant/sorcery vs creature
        // casts). These refine `spells_cast_this_turn` for cards that
        // need exact-type filtering (Potioner's Trove "instant or
        // sorcery only" gate, future Magecraft variants).
        self.players[p].instants_or_sorceries_cast_this_turn = 0;
        self.players[p].creatures_cast_this_turn = 0;
        // Clear Teferi, Time Raveler's "you may cast sorceries as though they
        // had flash" flag — it expires on the start of your next turn.
        self.players[p].sorceries_as_flash = false;
    }

    pub(crate) fn do_cleanup(&mut self) {
        // Clear temporary pump effects (CardInstance-level bonuses still used as base)
        for card in &mut self.battlefield {
            card.clear_end_of_turn_effects();
        }
        // Expire UntilEndOfTurn continuous effects from the layer system
        self.expire_end_of_turn_effects();
        // Clear all damage from creatures
        for card in &mut self.battlefield {
            card.damage = 0;
        }
        // Empty mana pools
        for player in &mut self.players {
            player.mana_pool.empty();
        }
        // Advance to the next non-eliminated player's turn (TurnStarted
        // fires on Untap entry).
        self.active_player_idx = self.next_alive_seat(self.active_player_idx);
        self.turn_number += 1;
        self.give_priority_to_active();
    }

    // ── State-based actions ───────────────────────────────────────────────────

    pub(crate) fn check_state_based_actions(&mut self) -> Vec<GameEvent> {
        let mut events = vec![];

        // +1/+1 and -1/-1 counters cancel each other out (CR 704.5q / 704.5r).
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
                if let Some(v) = card.counters.get_mut(&crate::card::CounterType::PlusOnePlusOne) { *v -= cancel; }
                if let Some(v) = card.counters.get_mut(&crate::card::CounterType::MinusOneMinusOne) { *v -= cancel; }
                card.counters.retain(|_, n| *n > 0);
            }
        }

        // Legend rule: if two+ legendaries with the same name share a controller,
        // keep the newest (highest CardId) and sacrifice the rest.
        let legend_victims: Vec<CardId> = {
            let mut seen: std::collections::HashMap<(usize, &str), CardId> =
                std::collections::HashMap::new();
            let mut victims = Vec::new();
            // Sort by id descending so we keep the newest.
            let mut legendaries: Vec<_> = self
                .battlefield
                .iter()
                .filter(|c| c.definition.supertypes.contains(&Supertype::Legendary))
                .collect();
            legendaries.sort_by_key(|b| std::cmp::Reverse(b.id));
            for c in legendaries {
                let key = (c.controller, c.definition.name);
                if let std::collections::hash_map::Entry::Vacant(e) = seen.entry(key) {
                    e.insert(c.id);
                } else {
                    victims.push(c.id);
                }
            }
            victims
        };
        for id in legend_victims {
            let (is_creature, is_planeswalker) = self.battlefield
                .iter()
                .find(|c| c.id == id)
                .map(|c| (c.definition.is_creature(), c.definition.is_planeswalker()))
                .unwrap_or((false, false));
            if is_creature {
                events.push(GameEvent::CreatureDied { card_id: id });
            } else if is_planeswalker {
                events.push(GameEvent::PlaneswalkerDied { card_id: id });
            }
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
                // Lethal damage kills non-indestructible creatures.
                !c.has_keyword(&Keyword::Indestructible) && (c.damage as i32) >= computed_toughness
            })
            .map(|c| c.id)
            .collect();

        for id in dead {
            events.push(GameEvent::CreatureDied { card_id: id });
            // Collect Dies triggers and Persist/Undying info before removing from battlefield.
            let (
                die_triggers,
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
                    let triggers: Vec<(CardId, Effect, usize)> = c
                        .definition
                        .triggered_abilities
                        .iter()
                        .filter(|t| t.event.kind == EventKind::CreatureDied)
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
            // Bump the controller's per-turn died-creature tally for
            // Witherbloom "if a creature died under your control this
            // turn" payoffs (Essenceknit Scholar).
            if controller_idx < self.players.len() {
                self.players[controller_idx].creatures_died_this_turn =
                    self.players[controller_idx].creatures_died_this_turn.saturating_add(1);
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
                    // Death trigger — subject is the dying creature
                    // (== source for SelfSource Dies; for AnotherOfYours
                    // we'd want the actual dying card, but this path
                    // currently only fires SelfSource).
                    subject: Some(crate::game::effects::EntityRef::Card(id)),
                });
            }
            // Persist: return to battlefield with -1/-1 counter if it had no -1/-1 counter.
            if has_persist && minus_count == 0 {
                // Find the card in owner's graveyard and return it.
                if let Some(pos) = self.players[owner]
                    .graveyard
                    .iter()
                    .position(|c| c.id == id)
                {
                    let mut returned = self.players[owner].graveyard.remove(pos);
                    self.players[owner].cards_left_graveyard_this_turn =
                        self.players[owner].cards_left_graveyard_this_turn.saturating_add(1);
                    returned.damage = 0;
                    returned.summoning_sick = true;
                    returned.add_counters(crate::card::CounterType::MinusOneMinusOne, 1);
                    let rid = returned.id;
                    events.push(GameEvent::CardLeftGraveyard { player: owner, card_id: rid });
                    self.battlefield.push(returned);
                    events.push(GameEvent::PermanentEntered { card_id: rid });
                }
            }
            // Undying: return to battlefield with +1/+1 counter if it had no +1/+1 counter.
            else if has_undying && plus_count == 0
                && let Some(pos) = self.players[owner]
                    .graveyard
                    .iter()
                    .position(|c| c.id == id)
            {
                let mut returned = self.players[owner].graveyard.remove(pos);
                self.players[owner].cards_left_graveyard_this_turn =
                    self.players[owner].cards_left_graveyard_this_turn.saturating_add(1);
                returned.damage = 0;
                returned.summoning_sick = true;
                returned.add_counters(crate::card::CounterType::PlusOnePlusOne, 1);
                let rid = returned.id;
                events.push(GameEvent::CardLeftGraveyard { player: owner, card_id: rid });
                self.battlefield.push(returned);
                events.push(GameEvent::PermanentEntered { card_id: rid });
            }
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

        // Auras with no valid attachment target go to their owner's
        // graveyard (CR 704.5m). Note: the cast-time pre-attach in
        // `resolve_top_of_stack` snapshots the target onto
        // `attached_to` before this SBA fires, so a freshly-resolved
        // Aura with a legal target survives.
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
            self.remove_from_battlefield_to_graveyard(id);
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
        // removed from turn/priority rotation; the game ends when ≤ 1 alive.
        for i in 0..self.players.len() {
            if self.players[i].eliminated {
                continue;
            }
            let lost = self.players[i].life <= 0 || self.players[i].poison_counters >= 10;
            if lost {
                self.players[i].eliminated = true;
            }
        }

        if self.game_over.is_none() {
            let alive: Vec<usize> = (0..self.players.len())
                .filter(|i| !self.players[*i].eliminated)
                .collect();
            match alive.len() {
                0 => {
                    self.game_over = Some(None);
                    events.push(GameEvent::GameOver { winner: None });
                }
                1 => {
                    let winner = alive[0];
                    self.game_over = Some(Some(winner));
                    events.push(GameEvent::GameOver { winner: Some(winner) });
                }
                _ => {}
            }
        }

        events
    }

    pub(crate) fn remove_from_battlefield_to_graveyard(&mut self, id: CardId) {
        if let Some(pos) = self.battlefield.iter().position(|c| c.id == id) {
            let card = self.battlefield.remove(pos);
            let owner = card.owner;
            self.remove_effects_from_source(id);
            self.players[owner].send_to_graveyard(card);
        }
    }

    pub(crate) fn remove_from_battlefield_to_exile(&mut self, id: CardId) {
        if let Some(pos) = self.battlefield.iter().position(|c| c.id == id) {
            let card = self.battlefield.remove(pos);
            self.remove_effects_from_source(id);
            self.exile.push(card);
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
        let (leave_triggers, dying_creature_controller): (Vec<(CardId, Effect, usize)>, Option<usize>) = self
            .battlefield
            .iter()
            .find(|c| c.id == id)
            .map(|c| {
                let is_creature = c.definition.is_creature();
                let triggers = c.definition
                    .triggered_abilities
                    .iter()
                    .filter(|t| matches!(t.event.scope, EventScope::SelfSource))
                    .filter(|t| match t.event.kind {
                        EventKind::PermanentLeavesBattlefield => true,
                        EventKind::CreatureDied => is_creature,
                        _ => false,
                    })
                    .map(|t| (c.id, t.effect.clone(), c.controller))
                    .collect();
                let creature_controller = if is_creature { Some(c.controller) } else { None };
                (triggers, creature_controller)
            })
            .unwrap_or_default();
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
        self.remove_from_battlefield_to_graveyard(id);
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
                // Leaves-battlefield trigger — subject is the leaving
                // permanent itself.
                subject: Some(crate::game::effects::EntityRef::Card(id)),
            });
        }
        vec![] // Trigger events are on the stack; callers resolve them via pass_priority.
    }
}
