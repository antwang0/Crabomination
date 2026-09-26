//! Pods only: the last-resort activation pass for the non-mana abilities no
//! shape generator in `main_phase_action_with` covers — an untap (Arbor Elf),
//! a put-onto-the-battlefield (Elvish Piper), a scry, tutor or animate rock.
//! The ability census (`bot_ladder --card-census`) counted ~1,500 printed
//! abilities of played cards that no pod seat ever activated. Each candidate
//! is dry-run and scored against passing; the best improvement is taken, in
//! the post-combat main phase.
//!
//! Two seats never reach it, so the two-player bench pool is untouched.

use crate::game::GameState;
use crate::game::types::{GameAction, TurnStep};

use super::bot::{
    EvalWeights, ability_sink_bits, action_outcome_is_temporary, eval_material, evaluate_action_outcome, mana_upper_bound,
};
use crate::effect::Effect;

/// The best-scoring uncovered activation `seat` can make in its main phase,
/// if any beats doing nothing.
pub(super) fn pick_generic_ability(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    // The post-combat main phase only: leftover mana, once a turn, with
    // combat settled — half the probes of asking in both main phases.
    if state.players.len() <= 2 || !state.stack.is_empty() || state.step != TurnStep::PostCombatMain {
        return None;
    }
    let mut baseline: Option<i32> = None;
    let mut mana: Option<u32> = None;
    let mut best: Option<(i32, GameAction)> = None;
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            // A shape a generator above owns, a mana ability, or one whose
            // tap it can't pay this turn.
            if ability_sink_bits(ab) != 0
                || crate::game::actions::is_mana_ability(&ab.effect)
                || ab.remove_counter_x.is_some()
                || ab.from_hand
                || ab.from_graveyard
                || ab.from_exile
                || ab.from_command_zone
                || (ab.tap_cost && card.tapped)
            {
                continue;
            }
            // A cost-free ability could be taken every tick; the no-progress
            // watch would catch it, but it never needs to start.
            if !ab.tap_cost && ab.mana_cost.symbols.is_empty() && !ab.sac_cost && ab.sac_other_filter.is_none() {
                continue;
            }
            // What the material eval can't see (a library order, a shield
            // held for a removal spell) always scores as passing: skip the
            // probe.
            if matches!(
                ab.effect,
                Effect::Scry { .. }
                    | Effect::Surveil { .. }
                    | Effect::RearrangeTop { .. }
                    | Effect::Regenerate { .. }
                    | Effect::Untap { .. }
                    | Effect::Tap { .. }
            ) {
                continue;
            }
            let cmc = ab.mana_cost.cmc();
            if cmc > 0 && cmc > *mana.get_or_insert_with(|| mana_upper_bound(state, seat)) {
                continue;
            }
            let (target, additional_targets) = if ab.effect.requires_target() {
                match state.auto_targets_for_effect_all_slots_sourced(&ab.effect, seat, None, Some(card.id)) {
                    (Some(t), extra) => (Some(t), extra),
                    (None, _) => continue,
                }
            } else {
                (None, Vec::new())
            };
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target,
                additional_targets,
                x_value: None,
                mode: None,
            };
            // An until-end-of-turn gain reads as permanent to the evaluator.
            if action_outcome_is_temporary(state, &action) {
                continue;
            }
            let Some(settled) = state.accept(action.clone()) else { continue };
            let base = *baseline.get_or_insert_with(|| eval_material(state, seat, w));
            if let Some(ev) = evaluate_action_outcome(state, seat, &action, Some(&settled), w)
                && ev > base
                && best.as_ref().is_none_or(|(b, _)| ev > *b)
            {
                best = Some((ev, action));
            }
        }
    }
    best.map(|(_, a)| a)
}
