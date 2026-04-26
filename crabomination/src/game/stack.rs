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
        let triggers: Vec<(CardId, Effect, usize)> = self
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
                    })
                    .map(|t| (c.id, t.effect.clone(), c.controller))
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
            self.stack.push(StackItem::Trigger {
                source,
                controller,
                effect: Box::new(effect),
                target,
                mode: None,
            });
        }

        for (source, effect, controller) in triggers {
            let auto_target = self.auto_target_for_effect(&effect, controller);
            self.stack.push(StackItem::Trigger {
                source,
                controller,
                effect: Box::new(effect),
                target: auto_target,
                mode: None,
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
                uncounterable: _,
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
                        });
                    }

                    // Push ETB triggers onto the stack.
                    for effect in etb_triggers {
                        let auto_target = self.auto_target_for_effect(&effect, caster);
                        self.stack.push(StackItem::Trigger {
                            source: card_id,
                            controller: caster,
                            effect: Box::new(effect),
                            target: auto_target,
                            mode: None,
                        });
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
                        for (src, effect) in other_triggers {
                            let auto_target = self.auto_target_for_effect(&effect, caster);
                            self.stack.push(StackItem::Trigger {
                                source: src,
                                controller: caster,
                                effect: Box::new(effect),
                                target: auto_target,
                                mode: None,
                            });
                        }
                    }
                } else {
                    let chosen_mode = mode.unwrap_or(0);
                    let mut spell_events =
                        self.continue_spell_resolution(card, caster, target, chosen_mode, None)?;
                    events.append(&mut spell_events);
                    if self.pending_decision.is_some() {
                        return Ok(events);
                    }
                }

                // SpellCast triggers fire after the spell resolves (e.g. Prowess).
                self.fire_spell_cast_triggers(caster, is_noncreature);
            }
            StackItem::Trigger {
                source,
                controller,
                effect,
                target,
                mode,
            } => {
                let chosen_mode = mode.unwrap_or(0);
                let mut trig_events = self.continue_trigger_resolution(
                    source,
                    controller,
                    *effect,
                    target,
                    chosen_mode,
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
        for card in &mut self.battlefield {
            if card.owner == p {
                card.tapped = false;
                card.summoning_sick = false;
            }
        }
        self.players[p].lands_played_this_turn = 0;
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
            events.push(GameEvent::CreatureDied { card_id: id });
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
            self.remove_from_battlefield_to_graveyard(id);
            // Push Dies triggers to the stack for resolution.
            for (source, effect, controller) in die_triggers {
                let auto_target = self.auto_target_for_effect(&effect, controller);
                self.stack.push(StackItem::Trigger {
                    source,
                    controller,
                    effect: Box::new(effect),
                    target: auto_target,
                    mode: None,
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
                    returned.damage = 0;
                    returned.summoning_sick = true;
                    returned.add_counters(crate::card::CounterType::MinusOneMinusOne, 1);
                    let rid = returned.id;
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
                returned.damage = 0;
                returned.summoning_sick = true;
                returned.add_counters(crate::card::CounterType::PlusOnePlusOne, 1);
                let rid = returned.id;
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
            self.remove_from_battlefield_to_graveyard(id);
        }

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
        let die_triggers: Vec<(CardId, Effect, usize)> = self
            .battlefield
            .iter()
            .find(|c| c.id == id)
            .map(|c| {
                c.definition
                    .triggered_abilities
                    .iter()
                    .filter(|t| t.event.kind == EventKind::CreatureDied)
                    .map(|t| (c.id, t.effect.clone(), c.controller))
                    .collect()
            })
            .unwrap_or_default();
        self.remove_from_battlefield_to_graveyard(id);
        for (source, effect, controller) in die_triggers {
            let auto_target = self.auto_target_for_effect(&effect, controller);
            self.stack.push(StackItem::Trigger {
                source,
                controller,
                effect: Box::new(effect),
                target: auto_target,
                mode: None,
            });
        }
        vec![] // Trigger events are on the stack; callers resolve them via pass_priority.
    }
}
