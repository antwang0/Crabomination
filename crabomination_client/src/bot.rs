use rand::RngExt;

use crabomination::{
    card::{CardDefinition, CardId, Keyword},
    decision::{AutoDecider, Decider},
    game::{GameAction, GameEvent, GameState, Target, TurnStep},
    mana::ManaPool,
};

use crate::game::{PLAYER_0, PLAYER_1};

/// Take one player 1 action and return the resulting events.
/// Called whenever it might be P1's turn to act (checks priority internally).
pub fn p1_take_action<R: RngExt>(state: &mut GameState, rng: &mut R) -> Vec<GameEvent> {
    if state.is_game_over() {
        return vec![];
    }
    // If a decision is pending on P1's spell/ability, auto-answer it. Without
    // this the game would stall — no UI is surfaced for bot-side decisions.
    if let Some(pending) = &state.pending_decision
        && pending.acting_player() == PLAYER_1
    {
        let answer = AutoDecider.decide(&pending.decision);
        return state
            .perform_action(GameAction::SubmitDecision(answer))
            .unwrap_or_default();
    }
    // Only act when P1 actually has priority.
    if state.player_with_priority() != PLAYER_1 {
        return vec![];
    }
    match (state.active_player_idx, state.step) {
        (PLAYER_1, TurnStep::PreCombatMain) | (PLAYER_1, TurnStep::PostCombatMain) => {
            p1_main_phase(state, rng)
        }
        (PLAYER_1, TurnStep::DeclareAttackers) => p1_attack(state),
        // Waiting for P0 to declare blockers — P1 should not act.
        (PLAYER_1, TurnStep::DeclareBlockers) if !state.attacking().is_empty() => vec![],
        // P1 has priority but is not the active player (opponent's turn) — pass.
        _ => state
            .perform_action(GameAction::PassPriority)
            .unwrap_or_default(),
    }
}

/// Called once when entering DeclareBlockers while player 1 is the defending player.
/// (active_player == PLAYER_0, player 1 defends.)
pub fn p1_declare_blocks<R: RngExt>(state: &mut GameState, rng: &mut R) -> Vec<GameEvent> {
    if state.active_player_idx != PLAYER_0 || state.step != TurnStep::DeclareBlockers {
        return vec![];
    }
    let attacking = state.attacking().to_vec();

    // Snapshot blocker and attacker data to avoid borrow conflicts.
    let p1_blockers: Vec<(CardId, bool, bool)> = state
        .battlefield
        .iter()
        .filter(|c| c.owner == PLAYER_1 && c.can_block())
        .map(|c| {
            (
                c.id,
                c.has_keyword(&Keyword::Flying),
                c.has_keyword(&Keyword::Reach),
            )
        })
        .collect();

    // (attacker_id, has_flying)
    let attacker_data: Vec<(CardId, bool)> = attacking
        .iter()
        .filter_map(|&id| {
            state
                .battlefield
                .iter()
                .find(|c| c.id == id)
                .map(|a| (id, a.has_keyword(&Keyword::Flying)))
        })
        .collect();

    let assignments: Vec<(CardId, CardId)> = p1_blockers
        .into_iter()
        .filter_map(|(blocker_id, blocker_flying, blocker_reach)| {
            // Only consider attackers this blocker can legally block.
            let legal: Vec<CardId> = attacker_data
                .iter()
                .filter(|(_, atk_flying)| !atk_flying || blocker_flying || blocker_reach)
                .map(|(id, _)| *id)
                .collect();
            if legal.is_empty() {
                None
            } else {
                Some((blocker_id, legal[rng.random_range(0..legal.len())]))
            }
        })
        .collect();

    state
        .perform_action(GameAction::DeclareBlockers(assignments))
        .unwrap_or_default()
}

// ── Private helpers ────────────────────────────────────────────────────────────

fn p1_main_phase<R: RngExt>(state: &mut GameState, rng: &mut R) -> Vec<GameEvent> {
    // 1. Tap any untapped land first (one per call so the caller can animate).
    //    Mana abilities can be activated any time we have priority.
    let land_id = state
        .battlefield
        .iter()
        .find(|c| c.owner == PLAYER_1 && c.definition.is_land() && !c.tapped)
        .map(|c| c.id);

    if let Some(id) = land_id {
        return state
            .perform_action(GameAction::ActivateAbility {
                card_id: id,
                ability_index: 0,
                target: None,
            })
            .unwrap_or_default();
    }

    // 2. Cast a random affordable card from hand
    let hand: Vec<_> = state.players[PLAYER_1].hand.to_vec();
    let castable: Vec<_> = hand
        .iter()
        .filter(|c| can_afford(&c.definition, &state.players[PLAYER_1].mana_pool))
        .collect();

    if !castable.is_empty() {
        let card = castable[rng.random_range(0..castable.len())].clone();
        let action = if card.definition.is_land() {
            if state.players[PLAYER_1].can_play_land() {
                GameAction::PlayLand(card.id)
            } else {
                return state
                    .perform_action(GameAction::PassPriority)
                    .unwrap_or_default();
            }
        } else {
            let target = choose_target(state, &card.definition, PLAYER_1, rng);
            GameAction::CastSpell {
                card_id: card.id,
                target,
                mode: None,
                x_value: None,
            }
        };

        if let Ok(evs) = state.perform_action(action) {
            return evs;
        }
    }

    // 3. Nothing to do — pass priority
    state
        .perform_action(GameAction::PassPriority)
        .unwrap_or_default()
}

fn p1_attack(state: &mut GameState) -> Vec<GameEvent> {
    let attackers: Vec<CardId> = state
        .battlefield
        .iter()
        .filter(|c| c.owner == PLAYER_1 && c.can_attack())
        .map(|c| c.id)
        .collect();

    let mut all_events = Vec::new();

    // DeclareAttackers may push attack triggers onto the stack
    if let Ok(evs) = state.perform_action(GameAction::DeclareAttackers(attackers)) {
        all_events.extend(evs);
    }

    // Drain the stack (attack triggers) before advancing the step
    while !state.stack.is_empty() {
        match state.perform_action(GameAction::PassPriority) {
            Ok(evs) => all_events.extend(evs),
            Err(_) => break,
        }
    }

    // Now advance past DeclareAttackers
    if let Ok(evs) = state.perform_action(GameAction::PassPriority) {
        all_events.extend(evs);
    }

    all_events
}

// ── Public helpers used by the UI/input system ────────────────────────────────

/// True if the player can pay the card's mana cost from their current pool.
pub fn can_afford(def: &CardDefinition, pool: &ManaPool) -> bool {
    let cost = if def.cost.has_x() {
        // Bot spends all remaining mana on X
        def.cost.with_x_value(0)
    } else {
        def.cost.clone()
    };
    pool.clone().pay(&cost).is_ok()
}

/// Choose a target that satisfies `req`, preferring opponent creatures for damage
/// and defaulting to targeting the opponent player for `Any`.
#[allow(dead_code)]
fn target_for_requirement<R: RngExt>(
    state: &GameState,
    req: &crabomination::card::SelectionRequirement,
    _caster: usize,
    opp: usize,
    rng: &mut R,
) -> Option<Target> {
    use crabomination::card::SelectionRequirement as SR;
    match req {
        SR::Player => Some(Target::Player(opp)),
        SR::Creature => {
            let ids: Vec<_> = state
                .battlefield
                .iter()
                .filter(|c| c.owner == opp && c.definition.is_creature())
                .map(|c| c.id)
                .collect();
            if ids.is_empty() {
                None
            } else {
                Some(Target::Permanent(ids[rng.random_range(0..ids.len())]))
            }
        }
        // For Any, prefer player (direct damage is most impactful for player 1)
        SR::Any | SR::Not(_) | SR::Or(_, _) => Some(Target::Player(opp)),
        // For And requirements, check if creatures are valid
        SR::And(_, _) => {
            let ids: Vec<_> = state
                .battlefield
                .iter()
                .filter(|c| {
                    c.owner == opp
                        && c.definition.is_creature()
                        && state.evaluate_requirement(req, &Target::Permanent(c.id), PLAYER_0)
                })
                .map(|c| c.id)
                .collect();
            if ids.is_empty() {
                None
            } else {
                Some(Target::Permanent(ids[rng.random_range(0..ids.len())]))
            }
        }
        _ => Some(Target::Player(opp)),
    }
}

/// Pick a sensible auto-target for a spell cast by `caster`.
pub fn choose_target<R: RngExt>(
    state: &GameState,
    def: &CardDefinition,
    caster: usize,
    _rng: &mut R,
) -> Option<Target> {
    state.auto_target_for_effect(&def.effect, caster)
}

/// Auto-target for spells player 0 casts (always picks opponent or their creatures).
pub fn p0_auto_target(state: &GameState, def: &CardDefinition) -> Option<Target> {
    choose_target(state, def, PLAYER_0, &mut rand::rng())
}

/// Decide whether player 1 should keep their current hand.
/// Returns `true` to keep, `false` to mulligan.
/// Always keeps after 3 mulligans (forced keep at 4 cards).
pub fn p1_mulligan_decision(state: &GameState, mulligans_taken: usize) -> bool {
    if mulligans_taken >= 3 {
        return true;
    }
    let hand = &state.players[PLAYER_1].hand;
    let lands = hand.iter().filter(|c| c.definition.is_land()).count();
    // Keep if hand has 2–5 lands (playable range).
    (2..=5).contains(&lands)
}
