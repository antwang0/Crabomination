use rand::RngExt;

use crabomination::{
    card::{CardDefinition, CardId, Keyword, SpellEffect},
    game::{GameAction, GameEvent, GameState, Target, TurnStep},
    mana::ManaPool,
};

use crate::game::{BOT, HUMAN};

/// Take one bot action and return the resulting events.
/// Should be called when `state.active_player_idx == BOT`.
pub fn bot_take_action<R: RngExt>(state: &mut GameState, rng: &mut R) -> Vec<GameEvent> {
    if state.is_game_over() {
        return vec![];
    }
    match (state.active_player_idx, state.step) {
        (BOT, TurnStep::PreCombatMain) | (BOT, TurnStep::PostCombatMain) => {
            bot_main_phase(state, rng)
        }
        (BOT, TurnStep::DeclareAttackers) => bot_attack(state),
        // Bot is the attacker here; human must declare blockers before we can advance.
        (BOT, TurnStep::DeclareBlockers) if !state.attacking().is_empty() => vec![],
        (BOT, _) => state.perform_action(GameAction::PassPriority).unwrap_or_default(),
        _ => vec![],
    }
}

/// Called once when entering DeclareBlockers while the bot is the defending player.
/// (active_player == HUMAN, bot defends.)
pub fn bot_declare_blocks<R: RngExt>(state: &mut GameState, rng: &mut R) -> Vec<GameEvent> {
    if state.active_player_idx != HUMAN || state.step != TurnStep::DeclareBlockers {
        return vec![];
    }
    let attacking = state.attacking().to_vec();

    // Snapshot blocker and attacker data to avoid borrow conflicts.
    let bot_blockers: Vec<(CardId, bool, bool)> = state
        .battlefield
        .iter()
        .filter(|c| c.owner == BOT && c.can_block())
        .map(|c| (c.id, c.has_keyword(&Keyword::Flying), c.has_keyword(&Keyword::Reach)))
        .collect();

    // (attacker_id, has_flying)
    let attacker_data: Vec<(CardId, bool)> = attacking
        .iter()
        .filter_map(|&id| {
            state.battlefield.iter().find(|c| c.id == id)
                .map(|a| (id, a.has_keyword(&Keyword::Flying)))
        })
        .collect();

    let assignments: Vec<(CardId, CardId)> = bot_blockers
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

fn bot_main_phase<R: RngExt>(state: &mut GameState, rng: &mut R) -> Vec<GameEvent> {
    // 1. Tap any untapped land first (one per call so the caller can animate)
    let land_id = state
        .battlefield
        .iter()
        .find(|c| c.owner == BOT && c.definition.is_land() && !c.tapped)
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
    let hand: Vec<_> = state.players[BOT].hand.iter().cloned().collect();
    let castable: Vec<_> = hand
        .iter()
        .filter(|c| can_afford(&c.definition, &state.players[BOT].mana_pool))
        .collect();

    if !castable.is_empty() {
        let card = castable[rng.random_range(0..castable.len())].clone();
        let action = if card.definition.is_land() {
            if state.players[BOT].can_play_land() {
                GameAction::PlayLand(card.id)
            } else {
                return state.perform_action(GameAction::PassPriority).unwrap_or_default();
            }
        } else {
            let target = choose_target(state, &card.definition, BOT, rng);
            GameAction::CastSpell { card_id: card.id, target }
        };

        if let Ok(evs) = state.perform_action(action) {
            return evs;
        }
    }

    // 3. Nothing to do — pass priority
    state.perform_action(GameAction::PassPriority).unwrap_or_default()
}

fn bot_attack(state: &mut GameState) -> Vec<GameEvent> {
    let attackers: Vec<CardId> = state
        .battlefield
        .iter()
        .filter(|c| c.owner == BOT && c.can_attack())
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
    pool.clone().pay(&def.cost).is_ok()
}

/// Choose a target that satisfies `req`, preferring opponent creatures for damage
/// and defaulting to targeting the opponent player for `Any`.
fn target_for_requirement<R: RngExt>(
    state: &GameState,
    req: &crabomination::card::SelectionRequirement,
    caster: usize,
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
            if ids.is_empty() { None } else { Some(Target::Permanent(ids[rng.random_range(0..ids.len())])) }
        }
        // For Any, prefer player (direct damage is most impactful for bot)
        SR::Any | SR::Not(_) | SR::Or(_, _) => Some(Target::Player(opp)),
        // For And requirements, check if creatures are valid
        SR::And(_, _) => {
            let ids: Vec<_> = state
                .battlefield
                .iter()
                .filter(|c| c.owner == opp && c.definition.is_creature() && state.evaluate_requirement(req, &Target::Permanent(c.id)))
                .map(|c| c.id)
                .collect();
            if ids.is_empty() { None } else { Some(Target::Permanent(ids[rng.random_range(0..ids.len())])) }
        }
        _ => Some(Target::Player(opp)),
    }
}

/// Pick a sensible auto-target for a spell cast by `caster`.
pub fn choose_target<R: RngExt>(
    state: &GameState,
    def: &CardDefinition,
    caster: usize,
    rng: &mut R,
) -> Option<Target> {
    let opp = 1 - caster;
    for effect in &def.spell_effects {
        match effect {
            SpellEffect::DealDamage { target: req, .. }
            | SpellEffect::DestroyCreature { target: req } => {
                return target_for_requirement(state, req, caster, opp, rng);
            }
            SpellEffect::PumpCreature { .. } => {
                return state
                    .battlefield
                    .iter()
                    .find(|c| c.owner == caster && c.definition.is_creature())
                    .map(|c| Target::Permanent(c.id));
            }
            _ => {}
        }
    }
    None
}

/// Auto-target for spells the human player casts (always picks opponent or their creatures).
pub fn human_auto_target(state: &GameState, def: &CardDefinition) -> Option<Target> {
    choose_target(state, def, HUMAN, &mut rand::rng())
}
