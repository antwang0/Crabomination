//! The bot's sink for "{X}, Remove X [kind] counters from this" activations
//! (Marath, Will of the Wild; Arcbound Javelineer). No generator chose an X
//! for one, so Marath's counters sat unspent and its seat won 6 % of four-seat
//! pods. Every mode at every payable X is dry-run and scored against passing;
//! the best improvement is taken.

use crate::effect::Effect;
use crate::game::GameState;
use crate::game::types::GameAction;

use super::bot::{EvalWeights, eval_material, evaluate_action_outcome};

/// The best-scoring remove-X-counters activation `seat` can make, if any beats
/// doing nothing.
pub(super) fn pick_x_counter_ability(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    let baseline = eval_material(state, seat, w);
    let mut best: Option<(i32, GameAction)> = None;
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            let Some(kind) = ab.remove_counter_x else { continue };
            let have = card.counter_count(kind);
            if have == 0 || ab.sac_cost || ab.exhaust {
                continue;
            }
            let modes: Vec<(Option<usize>, &Effect)> = match &ab.effect {
                Effect::ChooseMode(ms) => ms.iter().enumerate().map(|(i, m)| (Some(i), m)).collect(),
                other => vec![(None, other)],
            };
            for (mode, eff) in modes {
                let target = if eff.requires_target() {
                    match state.auto_target_for_effect(eff, seat) {
                        Some(t) => Some(t),
                        None => continue,
                    }
                } else {
                    None
                };
                for x in (1..=have).rev() {
                    let action = GameAction::ActivateAbility {
                        card_id: card.id,
                        ability_index: idx,
                        target: target.clone(),
                        additional_targets: Vec::new(),
                        x_value: Some(x),
                        mode,
                    };
                    let Some(settled) = state.accept(action.clone()) else { continue };
                    if let Some(ev) = evaluate_action_outcome(state, seat, &action, Some(&settled), w)
                        && ev > baseline
                        && best.as_ref().is_none_or(|(b, _)| ev > *b)
                    {
                        best = Some((ev, action));
                    }
                }
            }
        }
    }
    best.map(|(_, a)| a)
}
