//! Core MTG game engine.
//!
//! # Design notes
//! - **Simplified stack**: instants and sorceries resolve immediately on cast
//!   (no priority loop between players).  The `stack` field is reserved for
//!   future expansion.
//! - **Combat damage**: first-strike/double-strike are not split into separate
//!   sub-steps; a DoubleStrike creature deals its power damage twice.
//! - **Hexproof/Shroud**: validated at targeting time.
//! - **Menace**: enforced in `declare_blockers` — an attacker with Menace must
//!   be blocked by ≥ 2 creatures or not blocked at all.
//! - **Dies triggers**: fired when a creature moves from battlefield to
//!   graveyard (via damage, destroy, or state-based actions).
//! - All actions are performed by the *active player* except `declare_blockers`
//!   which is called by whoever controls the defending creatures.

pub(crate) mod actions;
pub(crate) mod combat;
pub(crate) mod effects;
pub mod layers;
pub(crate) mod stack;
#[cfg(test)]
#[path = "../tests/game.rs"]
mod tests;
pub mod types;

pub use types::*;

use crate::card::{CardDefinition, CardId, CardInstance, CardType, Keyword, SelectionRequirement};
use crate::decision::{AutoDecider, Decider, Decision, DecisionAnswer};
use crate::effect::Effect;
use crate::game::effects::EffectContext;
use crate::game::layers::{
    AffectedPermanents, ComputedPermanent, ContinuousEffect, EffectDuration, Layer, Modification,
    PtSublayer, apply_layers,
};
use crate::player::Player;
use std::collections::HashMap;

// ── Game state ────────────────────────────────────────────────────────────────

pub struct GameState {
    pub players: Vec<Player>,
    /// All permanents currently in play.
    pub battlefield: Vec<CardInstance>,
    /// Cards that have been exiled.
    pub exile: Vec<CardInstance>,
    /// The stack of spells and triggered abilities waiting to resolve (LIFO).
    pub stack: Vec<StackItem>,
    pub step: TurnStep,
    /// Index into `players` of the player whose turn it is.
    pub active_player_idx: usize,
    pub turn_number: u32,
    /// `None` while the game is ongoing; `Some(None)` for a draw;
    /// `Some(Some(i))` when player `i` has won.
    pub game_over: Option<Option<usize>>,
    /// Priority state — tracks who can act and when the stack resolves.
    pub priority: PriorityState,
    /// Active continuous effects from resolved spells, abilities, and static abilities.
    pub continuous_effects: Vec<ContinuousEffect>,
    #[allow(dead_code)]
    pub(crate) next_effect_timestamp: u64,
    pub(crate) next_id: u32,
    /// Cards declared as attackers this combat.
    pub(crate) attacking: Vec<CardId>,
    /// Blocker → attacker mapping for the current combat.
    pub(crate) block_map: HashMap<CardId, CardId>,
    /// Set to true once `declare_blockers` has been called during the current DeclareBlockers step.
    pub(crate) blockers_declared: bool,
    /// Skip the draw on the very first turn (turn 1, first player).
    pub(crate) skip_first_draw: bool,
    /// Count of spells cast this turn (for Storm and related effects).
    pub spells_cast_this_turn: u32,
    /// Resolves player choices encountered during effect resolution. Used for
    /// *non-suspending* decisions (e.g. `AddManaAnyColor` auto-picks a color).
    /// Suspending decisions (currently Scry) surface through `pending_decision`
    /// instead; the UI/bot replies via `submit_decision`.
    pub decider: Box<dyn Decider + Send + Sync>,
    /// Set when effect resolution needs player input. Check each frame in the
    /// client to render the appropriate decision modal; clear via
    /// `submit_decision`. While `Some`, no other game actions are permitted.
    pub pending_decision: Option<PendingDecision>,
    /// One-shot signal from `resolve_effect` to the enclosing resolver when an
    /// effect needs to suspend. Callers check this after each effect call, wrap
    /// it up in `pending_decision` with the full resume context, and return.
    /// `remaining` carries any sibling effects still queued behind the one that
    /// suspended (e.g. `Draw` after `Scry` in a Seq).
    pub(crate) suspend_signal: Option<(Decision, PendingEffectState, Effect)>,
}

impl GameState {
    /// Create a fresh game.  `player_names` must have at least 2 entries.
    pub fn new(players: Vec<Player>) -> Self {
        Self {
            players,
            battlefield: Vec::new(),
            exile: Vec::new(),
            stack: Vec::new(),
            step: TurnStep::Untap,
            active_player_idx: 0,
            turn_number: 1,
            game_over: None,
            priority: PriorityState::new(0),
            continuous_effects: Vec::new(),
            next_effect_timestamp: 1,
            next_id: 1,
            attacking: Vec::new(),
            block_map: HashMap::new(),
            blockers_declared: false,
            skip_first_draw: true,
            spells_cast_this_turn: 0,
            decider: Box::new(AutoDecider),
            pending_decision: None,
            suspend_signal: None,
        }
    }

    /// The player who currently holds priority.
    pub fn player_with_priority(&self) -> usize {
        self.priority.player_with_priority
    }

    /// Give priority to the active player and reset consecutive passes.
    pub(crate) fn give_priority_to_active(&mut self) {
        self.priority.player_with_priority = self.active_player_idx;
        self.priority.consecutive_passes = 0;
    }

    // ── Layer system ──────────────────────────────────────────────────────────

    /// Compute the current derived state of all battlefield permanents after
    /// applying all active continuous effects in layer order.
    pub fn compute_battlefield(&self) -> Vec<ComputedPermanent> {
        // Include static-ability effects from permanents currently on the battlefield.
        let mut all_effects: Vec<ContinuousEffect> = self.continuous_effects.clone();
        for card in &self.battlefield {
            let ts = card.id.0 as u64; // stable ordering by card id for static abilities
            let effects = static_ability_to_effects(card, ts);
            all_effects.extend(effects);
        }
        apply_layers(&self.battlefield, &all_effects)
    }

    /// Get the computed state of a single permanent (or None if not on battlefield).
    pub fn computed_permanent(&self, id: CardId) -> Option<ComputedPermanent> {
        self.compute_battlefield().into_iter().find(|c| c.id == id)
    }

    /// Add a transient continuous effect (from a spell/ability resolution).
    pub fn add_continuous_effect(&mut self, effect: ContinuousEffect) {
        self.continuous_effects.push(effect);
    }

    /// Allocate a new monotonically-increasing timestamp.
    #[allow(dead_code)]
    pub(crate) fn next_timestamp(&mut self) -> u64 {
        let ts = self.next_effect_timestamp;
        self.next_effect_timestamp += 1;
        ts
    }

    /// Remove all continuous effects whose source is `id` (source left battlefield).
    pub(crate) fn remove_effects_from_source(&mut self, id: CardId) {
        self.continuous_effects.retain(|e| e.source != id);
    }

    /// Expire all `UntilEndOfTurn` continuous effects (called during Cleanup).
    pub(crate) fn expire_end_of_turn_effects(&mut self) {
        self.continuous_effects
            .retain(|e| e.duration != EffectDuration::UntilEndOfTurn);
    }

    /// True if the stack is empty and it is `player`'s main phase — sorcery timing.
    pub fn can_cast_sorcery_speed(&self, player: usize) -> bool {
        self.stack.is_empty()
            && self.step.is_main_phase()
            && self.active_player_idx == player
            && self.priority.player_with_priority == player
    }

    pub(crate) fn next_id(&mut self) -> CardId {
        let id = CardId(self.next_id);
        self.next_id += 1;
        id
    }

    // ── Public setup helpers (useful in tests) ────────────────────────────────

    /// Add a card to a player's hand without going through library/draw mechanics.
    pub fn add_card_to_hand(&mut self, player_idx: usize, def: CardDefinition) -> CardId {
        let id = self.next_id();
        self.players[player_idx]
            .hand
            .push(CardInstance::new(id, def, player_idx));
        id
    }

    /// Put a card directly onto the battlefield (enters with summoning sickness unless cleared).
    pub fn add_card_to_battlefield(&mut self, player_idx: usize, def: CardDefinition) -> CardId {
        let id = self.next_id();
        self.battlefield
            .push(CardInstance::new(id, def, player_idx));
        id
    }

    /// Add a card to a player's library (top of deck).
    pub fn add_card_to_library(&mut self, player_idx: usize, def: CardDefinition) -> CardId {
        let id = self.next_id();
        self.players[player_idx].add_to_library_bottom(id, def);
        id
    }

    /// Clear summoning sickness from a permanent (convenience for tests).
    pub fn clear_sickness(&mut self, id: CardId) {
        if let Some(c) = self.battlefield_find_mut(id) {
            c.summoning_sick = false;
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over.is_some()
    }

    /// Cards currently declared as attackers in this combat step.
    pub fn attacking(&self) -> &[CardId] {
        &self.attacking
    }

    /// True if `blocker_id` can legally block at least one current attacker.
    pub fn can_block_any_attacker(&self, blocker_id: CardId) -> bool {
        let Some(blocker) = self.battlefield.iter().find(|c| c.id == blocker_id) else {
            return false;
        };
        if !blocker.can_block() {
            return false;
        }
        let computed = self.compute_battlefield();
        let blocker_computed = computed.iter().find(|c| c.id == blocker_id);
        let Some(blocker_cp) = blocker_computed else {
            return false;
        };
        self.attacking.iter().any(|&atk_id| {
            let attacker = self.battlefield.iter().find(|c| c.id == atk_id);
            let atk_kws = computed
                .iter()
                .find(|c| c.id == atk_id)
                .map(|c| c.keywords.as_slice())
                .unwrap_or(&[]);
            attacker
                .map(|atk| can_block_attacker_computed(blocker, atk, blocker_cp, atk_kws))
                .unwrap_or(false)
        })
    }

    /// True if `blocker_id` can legally block `attacker_id`.
    pub fn blocker_can_block_attacker(&self, blocker_id: CardId, attacker_id: CardId) -> bool {
        let Some(blocker) = self.battlefield.iter().find(|c| c.id == blocker_id) else {
            return false;
        };
        let Some(attacker) = self.battlefield.iter().find(|c| c.id == attacker_id) else {
            return false;
        };
        let computed = self.compute_battlefield();
        let blocker_cp = computed.iter().find(|c| c.id == blocker_id);
        let Some(blocker_cp) = blocker_cp else {
            return false;
        };
        let atk_kws = computed
            .iter()
            .find(|c| c.id == attacker_id)
            .map(|c| c.keywords.as_slice())
            .unwrap_or(&[]);
        can_block_attacker_computed(blocker, attacker, blocker_cp, atk_kws)
    }

    // ── Main action dispatch ──────────────────────────────────────────────────

    pub fn perform_action(&mut self, action: GameAction) -> Result<Vec<GameEvent>, GameError> {
        if self.is_game_over() {
            return Err(GameError::GameAlreadyOver);
        }
        // Routing for decision answers is unconditional; everything else must
        // wait until the pending decision is resolved.
        if let GameAction::SubmitDecision(answer) = action {
            return self.submit_decision(answer);
        }
        if self.pending_decision.is_some() {
            return Err(GameError::DecisionPending);
        }
        let events = match action {
            GameAction::PlayLand(id) => self.play_land(id),
            GameAction::CastSpell {
                card_id,
                target,
                mode,
                x_value,
            } => self.cast_spell(card_id, target, mode, x_value),
            GameAction::CastFlashback {
                card_id,
                target,
                mode,
                x_value,
            } => self.cast_flashback(card_id, target, mode, x_value),
            GameAction::ActivateAbility {
                card_id,
                ability_index,
                target,
            } => self.activate_ability(card_id, ability_index, target),
            GameAction::ActivateLoyaltyAbility {
                card_id,
                ability_index,
                target,
            } => self.activate_loyalty_ability(card_id, ability_index, target),
            GameAction::DeclareAttackers(ids) => self.declare_attackers(ids),
            GameAction::DeclareBlockers(assignments) => self.declare_blockers(assignments),
            GameAction::PassPriority => self.pass_priority(),
            GameAction::SubmitDecision(_) => unreachable!(),
        }?;
        self.dispatch_triggers_for_events(&events);
        Ok(events)
    }

    /// Walk the battlefield looking for triggered abilities whose `EventSpec`
    /// matches any of `events`, and push matching triggers onto the stack.
    ///
    /// Events already handled by hardcoded trigger sites (ETB, attackers,
    /// spell-cast, dies, step changes) are skipped here to avoid double-firing.
    /// Everything else (TurnBegins, CardDrawn, LandPlayed, LifeGained, etc.)
    /// gains trigger capability through this path.
    pub(crate) fn dispatch_triggers_for_events(&mut self, events: &[GameEvent]) {
        if events.is_empty() {
            return;
        }
        let mut matched: Vec<(CardId, Effect, usize)> = Vec::new();
        for card in &self.battlefield {
            for ta in &card.definition.triggered_abilities {
                for ev in events {
                    if is_event_hardcoded(ev) {
                        continue;
                    }
                    if crate::game::effects::event_matches_spec(ev, &ta.event, card) {
                        matched.push((card.id, ta.effect.clone(), card.controller));
                        break;
                    }
                }
            }
        }
        for (source, effect, controller) in matched {
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

    /// Activate a loyalty ability on a planeswalker (sorcery speed, once per turn).
    pub fn activate_loyalty_ability(
        &mut self,
        card_id: CardId,
        ability_index: usize,
        target: Option<Target>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        let pos = self
            .battlefield
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotOnBattlefield(card_id))?;
        if self.battlefield[pos].controller != p {
            return Err(GameError::NotYourPriority);
        }
        if self.battlefield[pos].used_loyalty_ability_this_turn {
            return Err(GameError::CardIsTapped(card_id)); // reuse error for now
        }
        if !self.battlefield[pos].definition.is_planeswalker() {
            return Err(GameError::InvalidTarget);
        }

        let ability = self.battlefield[pos]
            .definition
            .loyalty_abilities
            .get(ability_index)
            .cloned()
            .ok_or(GameError::AbilityIndexOutOfBounds)?;

        // Apply loyalty cost.
        let current_loyalty =
            self.battlefield[pos].counter_count(crate::card::CounterType::Loyalty) as i32;
        let new_loyalty = current_loyalty + ability.loyalty_cost;
        if new_loyalty < 0 {
            return Err(GameError::InvalidTarget); // not enough loyalty
        }
        self.battlefield[pos]
            .counters
            .insert(crate::card::CounterType::Loyalty, new_loyalty as u32);
        self.battlefield[pos].used_loyalty_ability_this_turn = true;

        let loyalty_change = ability.loyalty_cost;
        let mut events = vec![
            GameEvent::LoyaltyAbilityActivated {
                planeswalker: card_id,
                loyalty_change,
            },
            GameEvent::LoyaltyChanged {
                card_id,
                new_loyalty,
            },
        ];

        // Push ability effects onto the stack.
        self.stack.push(StackItem::Trigger {
            source: card_id,
            controller: p,
            effect: Box::new(ability.effect),
            target,
            mode: None,
        });
        self.give_priority_to_active();

        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);
        Ok(events)
    }

    /// Submit an answer to the currently-pending decision and resume resolution.
    /// Fails if no decision is pending, or the answer shape doesn't match the
    /// decision kind.
    pub fn submit_decision(&mut self, answer: DecisionAnswer) -> Result<Vec<GameEvent>, GameError> {
        let pd = self
            .pending_decision
            .take()
            .ok_or(GameError::NoDecisionPending)?;
        let mut events = match pd.resume {
            ResumeContext::Spell {
                card,
                caster,
                target,
                mode,
                in_progress,
                remaining,
            } => {
                let mut evs = self.apply_pending_effect_answer(in_progress, &answer)?;
                let mut more =
                    self.continue_spell_resolution(*card, caster, target, mode, Some(remaining))?;
                evs.append(&mut more);
                evs
            }
            ResumeContext::Trigger {
                source,
                controller,
                target,
                mode,
                in_progress,
                remaining,
            } => {
                let mut evs = self.apply_pending_effect_answer(in_progress, &answer)?;
                let mut more = self.continue_trigger_resolution(
                    source, controller, remaining, target, mode,
                )?;
                evs.append(&mut more);
                evs
            }
            ResumeContext::Ability {
                source,
                controller,
                target,
                in_progress,
                remaining,
            } => {
                let mut evs = self.apply_pending_effect_answer(in_progress, &answer)?;
                let mut more = self.continue_ability_resolution(
                    source, controller, remaining, target,
                )?;
                evs.append(&mut more);
                evs
            }
        };
        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);
        self.dispatch_triggers_for_events(&events);
        Ok(events)
    }

    /// Complete the suspended effect using the player's answer. Returns the
    /// events generated by the now-finished effect (e.g. `ScryPerformed`).
    pub(crate) fn apply_pending_effect_answer(
        &mut self,
        state: PendingEffectState,
        answer: &DecisionAnswer,
    ) -> Result<Vec<GameEvent>, GameError> {
        match state {
            PendingEffectState::ScryPeeked { count, player } => {
                let DecisionAnswer::ScryOrder { kept_top, bottom } = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut remaining: Vec<CardInstance> =
                    self.players[player].library.drain(..count).collect();
                let mut top_cards = Vec::with_capacity(kept_top.len());
                for id in kept_top {
                    if let Some(pos) = remaining.iter().position(|c| c.id == *id) {
                        top_cards.push(remaining.remove(pos));
                    }
                }
                let mut bottom_cards = Vec::with_capacity(bottom.len());
                for id in bottom {
                    if let Some(pos) = remaining.iter().position(|c| c.id == *id) {
                        bottom_cards.push(remaining.remove(pos));
                    }
                }
                // Cards listed in neither bucket default to top (end of top).
                top_cards.extend(remaining);
                let bottomed = bottom_cards.len();
                let lib = &mut self.players[player].library;
                for c in bottom_cards {
                    lib.push(c);
                }
                for c in top_cards.into_iter().rev() {
                    lib.insert(0, c);
                }
                Ok(vec![GameEvent::ScryPerformed {
                    player,
                    looked_at: count,
                    bottomed,
                }])
            }
            PendingEffectState::SurveilPeeked { count, player } => {
                // Surveil: player chooses which cards go to the graveyard; rest go to top.
                let DecisionAnswer::ScryOrder {
                    kept_top,
                    bottom: to_graveyard,
                } = answer
                else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut remaining: Vec<CardInstance> =
                    self.players[player].library.drain(..count).collect();
                let mut top_cards = Vec::with_capacity(kept_top.len());
                for id in kept_top {
                    if let Some(pos) = remaining.iter().position(|c| c.id == *id) {
                        top_cards.push(remaining.remove(pos));
                    }
                }
                let mut graveyard_cards = Vec::with_capacity(to_graveyard.len());
                for id in to_graveyard {
                    if let Some(pos) = remaining.iter().position(|c| c.id == *id) {
                        graveyard_cards.push(remaining.remove(pos));
                    }
                }
                top_cards.extend(remaining);
                let graveyarded = graveyard_cards.len();
                for c in graveyard_cards {
                    self.players[player].graveyard.push(c);
                }
                let lib = &mut self.players[player].library;
                for c in top_cards.into_iter().rev() {
                    lib.insert(0, c);
                }
                Ok(vec![GameEvent::SurveilPerformed {
                    player,
                    looked_at: count,
                    graveyarded,
                }])
            }
            PendingEffectState::SearchPending { player, to } => {
                let DecisionAnswer::Search(chosen_id) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = vec![];
                if let Some(card_id) = chosen_id
                    && let Some(pos) = self.players[player].library.iter().position(|c| c.id == *card_id) {
                    let card = self.players[player].library.remove(pos);
                    self.place_card_in_dest(card, player, &to, &mut events);
                }
                Ok(events)
            }
            PendingEffectState::PutOnLibraryPending { player, .. } => {
                let DecisionAnswer::PutOnLibrary(chosen) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = vec![];
                self.execute_put_on_library(player, chosen, &mut events);
                Ok(events)
            }
        }
    }

    /// Resolve a spell's effect tree. On suspension, installs a
    /// `pending_decision` and returns events accumulated so far. `override_effect`
    /// is used on resume to continue with whatever Seq tail was left after the
    /// suspending effect — pass `None` for the initial resolution and `Some(...)`
    /// when continuing from `submit_decision`.
    pub(crate) fn continue_spell_resolution(
        &mut self,
        card: CardInstance,
        caster: usize,
        target: Option<Target>,
        mode: usize,
        override_effect: Option<Effect>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let effect = override_effect.unwrap_or_else(|| card.definition.effect.clone());
        let ctx = EffectContext::for_spell(caster, target.clone(), mode, 0);
        let events = self.resolve_effect(&effect, &ctx)?;
        if let Some((decision, in_progress, remaining)) = self.suspend_signal.take() {
            self.pending_decision = Some(PendingDecision {
                decision,
                resume: ResumeContext::Spell {
                    card: Box::new(card),
                    caster,
                    target,
                    mode,
                    in_progress,
                    remaining,
                },
            });
            return Ok(events);
        }
        self.players[caster].send_to_graveyard(card);
        Ok(events)
    }

    /// Resolve a triggered ability's effect tree.
    pub(crate) fn continue_trigger_resolution(
        &mut self,
        source: CardId,
        controller: usize,
        effect: crate::effect::Effect,
        target: Option<Target>,
        mode: usize,
    ) -> Result<Vec<GameEvent>, GameError> {
        let ctx = EffectContext::for_trigger(source, controller, target.clone(), mode);
        let events = self.resolve_effect(&effect, &ctx)?;
        if let Some((decision, in_progress, remaining)) = self.suspend_signal.take() {
            self.pending_decision = Some(PendingDecision {
                decision,
                resume: ResumeContext::Trigger {
                    source,
                    controller,
                    target,
                    mode,
                    in_progress,
                    remaining,
                },
            });
        }
        Ok(events)
    }

    /// Resolve an activated ability's effect tree.
    pub(crate) fn continue_ability_resolution(
        &mut self,
        source: CardId,
        controller: usize,
        effect: crate::effect::Effect,
        target: Option<Target>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let ctx = EffectContext::for_ability(source, controller, target.clone());
        let events = self.resolve_effect(&effect, &ctx)?;
        if let Some((decision, in_progress, remaining)) = self.suspend_signal.take() {
            self.pending_decision = Some(PendingDecision {
                decision,
                resume: ResumeContext::Ability {
                    source,
                    controller,
                    target,
                    in_progress,
                    remaining,
                },
            });
        }
        Ok(events)
    }

    /// Evaluate whether `target` satisfies `req` given the current game state.
    ///
    /// `controller` is the player who controls the spell or ability (used for
    /// `ControlledByYou` / `ControlledByOpponent` checks).
    pub fn evaluate_requirement(
        &self,
        req: &SelectionRequirement,
        target: &Target,
        controller: usize,
    ) -> bool {
        self.evaluate_requirement_static(req, target, controller)
    }

    pub(crate) fn battlefield_find(&self, id: CardId) -> Option<&CardInstance> {
        self.battlefield.iter().find(|c| c.id == id)
    }

    pub(crate) fn battlefield_find_mut(&mut self, id: CardId) -> Option<&mut CardInstance> {
        self.battlefield.iter_mut().find(|c| c.id == id)
    }

    /// Returns true if the permanent `id` has `kw` after all layer effects are applied.
    /// Falls back to `false` if the permanent is not on the battlefield.
    #[allow(dead_code)]
    pub(crate) fn permanent_has_keyword(&self, id: CardId, kw: &Keyword) -> bool {
        self.computed_permanent(id)
            .map(|c| c.keywords.contains(kw))
            .unwrap_or(false)
    }

    /// Returns the computed power of permanent `id`, or 0 if not on battlefield.
    #[allow(dead_code)]
    pub(crate) fn computed_power(&self, id: CardId) -> i32 {
        self.computed_permanent(id).map(|c| c.power).unwrap_or(0)
    }

    /// Returns the computed toughness of permanent `id`, or 0 if not on battlefield.
    #[allow(dead_code)]
    pub(crate) fn computed_toughness(&self, id: CardId) -> i32 {
        self.computed_permanent(id)
            .map(|c| c.toughness)
            .unwrap_or(0)
    }
}

/// Events already fired by hardcoded sites. Skip them in the unified dispatcher
/// to prevent double-firing. Dies triggers are captured by SBA before the
/// creature leaves the battlefield, so `SelfSource` Dies triggers wouldn't
/// match after-the-fact anyway — keep them hardcoded.
fn is_event_hardcoded(ev: &GameEvent) -> bool {
    matches!(
        ev,
        GameEvent::PermanentEntered { .. }
            | GameEvent::AttackerDeclared(_)
            | GameEvent::SpellCast { .. }
            | GameEvent::CreatureDied { .. }
            | GameEvent::StepChanged(_)
    )
}

// ── Static ability conversion ─────────────────────────────────────────────────

/// Convert a `StaticAbility` from a source permanent into `ContinuousEffect`s.
/// Takes the full `CardInstance` so Equipment/Aura abilities can use `attached_to`.
fn static_ability_to_effects(card: &CardInstance, timestamp: u64) -> Vec<ContinuousEffect> {
    use crate::effect::StaticEffect;
    let source = card.id;

    card.definition
        .static_abilities
        .iter()
        .flat_map(|sa| match &sa.effect {
            StaticEffect::PumpPT { applies_to, power, toughness } => {
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::Modify),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::ModifyPowerToughness(*power, *toughness),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::GrantKeyword { applies_to, keyword } => {
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(keyword.clone()),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::EntersTapped { .. }
            | StaticEffect::ExtraLandPerTurn
            | StaticEffect::CostReduction { .. } => vec![],
        })
        .collect()
}

/// Translate a selector into a `layers::AffectedPermanents` description for
/// those `StaticEffect` variants that express broad "lord-like" scope. Returns
/// `None` if the selector shape isn't representable in the layer system yet.
fn selector_to_affected(
    sel: &crate::effect::Selector,
    card: &CardInstance,
) -> Option<AffectedPermanents> {
    use crate::effect::Selector;
    let controller = card.controller;
    match sel {
        Selector::This => Some(AffectedPermanents::Source),
        Selector::AttachedTo(inner) => {
            if matches!(inner.as_ref(), Selector::This)
                && let Some(attached_id) = card.attached_to
            {
                Some(AffectedPermanents::Specific(vec![attached_id]))
            } else {
                None
            }
        }
        Selector::EachPermanent(req) => affected_from_requirement(req, controller),
        _ => None,
    }
}

fn affected_from_requirement(
    req: &SelectionRequirement,
    source_controller: usize,
) -> Option<AffectedPermanents> {
    use SelectionRequirement as R;
    // Decompose And-trees to extract controller filter + card-type filter.
    let mut ctrl: Option<Option<usize>> = None; // Outer Some(None) = all players; Some(Some(n)) = specific player
    let mut types: Vec<CardType> = vec![];
    let mut creature_type: Option<crate::card::CreatureType> = None;
    let mut walk = vec![req];
    while let Some(r) = walk.pop() {
        match r {
            R::And(a, b) => {
                walk.push(a);
                walk.push(b);
            }
            R::ControlledByYou => ctrl = Some(Some(source_controller)),
            R::ControlledByOpponent => {
                return Some(AffectedPermanents::AllOpponents {
                    source_controller,
                    card_types: if types.is_empty() { vec![] } else { types.clone() },
                });
            }
            R::Creature => types.push(CardType::Creature),
            R::Artifact => types.push(CardType::Artifact),
            R::Enchantment => types.push(CardType::Enchantment),
            R::Planeswalker => types.push(CardType::Planeswalker),
            R::Land => types.push(CardType::Land),
            R::HasCardType(t) => types.push(t.clone()),
            R::HasCreatureType(ct) => creature_type = Some(*ct),
            R::Any | R::Permanent => {}
            _ => return None,
        }
    }
    if let Some(ct) = creature_type {
        return Some(AffectedPermanents::AllWithCreatureType { controller: ctrl.flatten(), creature_type: ct });
    }
    Some(AffectedPermanents::All {
        controller: ctrl.unwrap_or(None),
        card_types: types,
    })
}


// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns true if `blocker` is legally allowed to block `attacker`.
/// Uses `blocker_kws` / `attacker_kws` as the effective keyword sets
/// (from `ComputedPermanent`) instead of the raw definition keywords.
pub(crate) fn can_block_attacker_computed(
    blocker: &CardInstance,
    attacker: &CardInstance,
    blocker_computed: &ComputedPermanent,
    attacker_kws: &[Keyword],
) -> bool {
    let blocker_kws = &blocker_computed.keywords;
    // Unblockable: can't be blocked at all.
    if attacker_kws.contains(&Keyword::Unblockable) {
        return false;
    }
    // Flying: can only be blocked by fliers or reach.
    if attacker_kws.contains(&Keyword::Flying)
        && !blocker_kws.contains(&Keyword::Flying)
        && !blocker_kws.contains(&Keyword::Reach)
    {
        return false;
    }
    // Horsemanship: can only be blocked by other Horsemanship creatures.
    if attacker_kws.contains(&Keyword::Horsemanship)
        && !blocker_kws.contains(&Keyword::Horsemanship)
    {
        return false;
    }
    // Shadow: can only block/be blocked by other shadow creatures.
    if attacker_kws.contains(&Keyword::Shadow) && !blocker_kws.contains(&Keyword::Shadow) {
        return false;
    }
    if blocker_kws.contains(&Keyword::Shadow) && !attacker_kws.contains(&Keyword::Shadow) {
        return false;
    }
    // Skulk: can't be blocked by creatures with greater power.
    if attacker_kws.contains(&Keyword::Skulk) && blocker_computed.power > attacker.power() {
        return false;
    }
    // Intimidate: can only be blocked by artifact creatures or creatures sharing a color.
    if attacker_kws.contains(&Keyword::Intimidate) {
        let blocker_is_artifact = blocker.definition.is_artifact();
        let shares_color = blocker_computed.colors.iter().any(|c| {
            attacker.definition.cost.symbols.iter().any(|s| {
                use crate::mana::ManaSymbol;
                matches!(s, ManaSymbol::Colored(ac) if ac == c)
            })
        });
        if !blocker_is_artifact && !shares_color {
            return false;
        }
    }
    // Protection: attacker has protection from a color that appears in the blocker's cost.
    use crate::mana::ManaSymbol;
    for kw in attacker_kws {
        if let Keyword::Protection(color) = kw
            && blocker
                .definition
                .cost
                .symbols
                .iter()
                .any(|s| matches!(s, ManaSymbol::Colored(c) if c == color))
        {
            return false;
        }
    }
    true
}
