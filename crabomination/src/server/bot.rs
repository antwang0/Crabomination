//! In-process bots for server-hosted matches.
//!
//! Unlike networked clients, a bot reads the full authoritative [`GameState`]
//! each tick and returns the next [`GameAction`] it wants the server to
//! perform. The match actor polls every bot seat to a fixed point after every
//! state change, so a bot just needs to make *some* forward-progressing
//! decision (including `PassPriority`) whenever it holds priority.

use rand::{RngExt, rng};

use crate::card::{CardDefinition, CardId, Keyword};
use crate::decision::{AutoDecider, Decider};
use crate::game::{Attack, AttackTarget, GameAction, GameState, Target, TurnStep};
use crate::mana::ManaPool;

/// Drives one seat without a human client. Implementations see the full
/// `GameState` and return the single next action they'd like to submit.
pub trait Bot: Send {
    /// Return `Some(action)` to submit, or `None` if it's not this bot's turn
    /// to act right now (no priority, waiting on an opponent decision, game
    /// already over, etc.).
    fn next_action(&mut self, state: &GameState, seat: usize) -> Option<GameAction>;
}

/// Random-play reference bot. Taps lands, plays a random affordable card from
/// hand, attacks with everything that can, assigns blockers at random, and
/// auto-answers any decisions with [`AutoDecider`].
///
/// The bot keeps a little internal flag state so it only submits
/// `DeclareAttackers`/`DeclareBlockers` once per combat phase — the match
/// actor polls it repeatedly, so without these flags it would re-submit every
/// tick.
pub struct RandomBot {
    last_step_key: Option<(u32, TurnStep, usize)>,
    attackers_declared: bool,
    blocks_declared: bool,
}

impl RandomBot {
    pub fn new() -> Self {
        Self {
            last_step_key: None,
            attackers_declared: false,
            blocks_declared: false,
        }
    }

    fn sync_step(&mut self, state: &GameState) {
        let key = (state.turn_number, state.step, state.active_player_idx);
        if self.last_step_key != Some(key) {
            self.last_step_key = Some(key);
            self.attackers_declared = false;
            self.blocks_declared = false;
        }
    }
}

impl Default for RandomBot {
    fn default() -> Self {
        Self::new()
    }
}

impl Bot for RandomBot {
    fn next_action(&mut self, state: &GameState, seat: usize) -> Option<GameAction> {
        if state.is_game_over() {
            return None;
        }
        self.sync_step(state);

        // Any pending decision addressed to us: auto-answer it.
        if let Some(pending) = &state.pending_decision {
            if pending.acting_player() == seat {
                let answer = AutoDecider.decide(&pending.decision);
                return Some(GameAction::SubmitDecision(answer));
            }
            return None;
        }

        if state.player_with_priority() != seat {
            return None;
        }

        let is_active = state.active_player_idx == seat;

        match state.step {
            TurnStep::DeclareBlockers if !is_active => {
                if !self.blocks_declared && !state.attacking().is_empty() {
                    self.blocks_declared = true;
                    Some(GameAction::DeclareBlockers(pick_blocks(state, seat)))
                } else {
                    Some(GameAction::PassPriority)
                }
            }
            TurnStep::DeclareAttackers if is_active => {
                if !self.attackers_declared {
                    self.attackers_declared = true;
                    // Pick the next alive opponent as the default attack
                    // target; in multiplayer this is just the next seat.
                    let target_player = state.next_alive_seat(seat);
                    let attacks: Vec<Attack> = state
                        .battlefield
                        .iter()
                        .filter(|c| c.owner == seat && c.can_attack())
                        .map(|c| Attack {
                            attacker: c.id,
                            target: AttackTarget::Player(target_player),
                        })
                        .collect();
                    Some(GameAction::DeclareAttackers(attacks))
                } else {
                    Some(GameAction::PassPriority)
                }
            }
            TurnStep::PreCombatMain | TurnStep::PostCombatMain if is_active => {
                Some(main_phase_action(state, seat))
            }
            _ => Some(GameAction::PassPriority),
        }
    }
}

fn main_phase_action(state: &GameState, seat: usize) -> GameAction {
    // Tap the first untapped land, one call at a time so each mana ability
    // surfaces as its own event.
    if let Some(id) = state
        .battlefield
        .iter()
        .find(|c| c.owner == seat && c.definition.is_land() && !c.tapped)
        .map(|c| c.id)
    {
        return GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
        };
    }

    // Build list of castable non-land spells, excluding those that need a target
    // when no legal target exists (would loop forever on rejection).
    let castable: Vec<_> = state.players[seat]
        .hand
        .iter()
        .filter(|c| !c.definition.is_land())
        .filter(|c| can_afford(&c.definition, &state.players[seat].mana_pool))
        .filter(|c| {
            // Skip targeted spells when no legal auto-target exists, or
            // we'd loop forever submitting cast attempts that bounce on
            // SelectionRequirementViolated.
            if c.definition.effect.requires_target() {
                state.auto_target_for_effect(&c.definition.effect, seat).is_some()
            } else {
                true
            }
        })
        .collect();

    // Play a land if possible.
    if state.players[seat].can_play_land()
        && let Some(land) = state.players[seat].hand.iter().find(|c| c.definition.is_land())
    {
        return GameAction::PlayLand(land.id);
    }

    if !castable.is_empty() {
        let mut r = rng();
        let card = castable[r.random_range(0..castable.len())];
        let target = state.auto_target_for_effect(&card.definition.effect, seat);
        return GameAction::CastSpell {
            card_id: card.id,
            target,
            mode: None,
            x_value: None,
        };
    }

    GameAction::PassPriority
}

fn pick_blocks(state: &GameState, seat: usize) -> Vec<(CardId, CardId)> {
    // Bot only blocks attackers that are targeting *this* seat (or a
    // planeswalker controlled by this seat).
    let attacker_data: Vec<(CardId, bool)> = state
        .attacking()
        .iter()
        .filter(|atk| state.defender_for(atk.target) == Some(seat))
        .filter_map(|atk| {
            state
                .battlefield
                .iter()
                .find(|c| c.id == atk.attacker)
                .map(|a| (atk.attacker, a.has_keyword(&Keyword::Flying)))
        })
        .collect();

    let blockers: Vec<(CardId, bool, bool)> = state
        .battlefield
        .iter()
        .filter(|c| c.owner == seat && c.can_block())
        .map(|c| {
            (
                c.id,
                c.has_keyword(&Keyword::Flying),
                c.has_keyword(&Keyword::Reach),
            )
        })
        .collect();

    let mut r = rng();
    blockers
        .into_iter()
        .filter_map(|(blocker_id, blocker_flying, blocker_reach)| {
            let legal: Vec<CardId> = attacker_data
                .iter()
                .filter(|(_, atk_flying)| !atk_flying || blocker_flying || blocker_reach)
                .map(|(id, _)| *id)
                .collect();
            if legal.is_empty() {
                None
            } else {
                Some((blocker_id, legal[r.random_range(0..legal.len())]))
            }
        })
        .collect()
}

/// True if the player can pay the card's mana cost from their current pool.
pub fn can_afford(def: &CardDefinition, pool: &ManaPool) -> bool {
    let cost = if def.cost.has_x() {
        def.cost.with_x_value(0)
    } else {
        def.cost.clone()
    };
    pool.clone().pay(&cost).is_ok()
}

/// Pick a sensible auto-target for a spell cast by `caster` using the
/// engine's shared targeting heuristic.
pub fn choose_target(state: &GameState, def: &CardDefinition, caster: usize) -> Option<Target> {
    state.auto_target_for_effect(&def.effect, caster)
}
