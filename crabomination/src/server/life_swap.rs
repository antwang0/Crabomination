//! "{6}, {T}: Two target players exchange life totals" (Soul Conduit). No
//! picker covered a life exchange — activated abilities go through
//! per-effect pickers — so the bot never used one. Low on life, it names
//! itself and the opponent with the most life (CR 701.12c), when the swing
//! is worth a card's activation.

use crate::effect::Effect;
use crate::game::GameState;
use crate::game::types::{GameAction, Target, TurnStep};

/// The least life gained by a swap that is worth making.
const MIN_SWING: i32 = 8;

/// An activation of a two-player life exchange that names `seat` and its
/// highest-life opponent, when that opponent has [`MIN_SWING`] more life.
pub(super) fn pick_life_swap(state: &GameState, seat: usize) -> Option<GameAction> {
    if !matches!(state.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain) || !state.stack.is_empty() {
        return None;
    }
    let mine = state.effective_life(seat);
    let (opp, theirs) = state
        .opponents_of(seat)
        .into_iter()
        .map(|o| (o, state.effective_life(o)))
        .max_by_key(|&(o, life)| (life, std::cmp::Reverse(o)))?;
    if theirs - mine < MIN_SWING {
        return None;
    }
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if !matches!(ab.effect, Effect::ExchangeLifeTotals { .. }) {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: Some(Target::Player(seat)),
                additional_targets: vec![Target::Player(opp)],
                x_value: None,
                mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}
