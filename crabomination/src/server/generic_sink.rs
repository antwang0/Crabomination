//! Pods only: the last-resort activation pass for the non-mana abilities no
//! shape generator in `main_phase_action_with` covers — an untap (Arbor Elf),
//! a put-onto-the-battlefield (Elvish Piper), a scry, tutor or animate rock.
//! The ability census (`bot_ladder --card-census`) counted ~1,500 printed
//! abilities of played cards that no pod seat ever activated. Each candidate
//! is dry-run and scored against passing; the best improvement is taken, in
//! the post-combat main phase or the end step just before the seat's turn.
//!
//! Two seats never reach it, so the two-player bench pool is untouched.

use crate::game::GameState;
use crate::game::types::{GameAction, Target, TurnStep};

use super::bot::{
    EvalWeights, ability_sink_bits, action_outcome_is_temporary, eval_material, evaluate_action_outcome, mana_upper_bound,
};
use crate::effect::Effect;

/// The best-scoring uncovered activation `seat` can make in its main phase,
/// if any beats doing nothing.
pub(super) fn pick_generic_ability(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    // The post-combat main phase: leftover mana, once a turn, with combat
    // settled — half the probes of asking in both main phases. And the end
    // step of the opponent seated just before us: a creature tapped there
    // untaps before it could block, so "tap five untapped Vampires"
    // (Captivating Vampire) and "tap three untapped Zombies" (Cryptbreaker)
    // are free — neither was ever activated in a 6-seat census.
    let window = match state.step {
        TurnStep::PostCombatMain => state.active_player_idx == seat,
        TurnStep::End => {
            state.active_player_idx != seat && state.next_alive_seat(state.active_player_idx) == seat
        }
        _ => false,
    };
    if state.players.len() <= 2 || !state.stack.is_empty() || !window {
        return None;
    }
    let mut baseline: Option<i32> = None;
    let mut mana: Option<u32> = None;
    let mut best: Option<(i32, GameAction)> = None;
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            // A shape a generator above owns (they don't choose an X, so an
            // `{X}` one stays here), a mana ability, or one whose tap it
            // can't pay this turn.
            if (ability_sink_bits(ab) != 0 && !ab.mana_cost.has_x())
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
            if !ab.tap_cost
                && ab.mana_cost.symbols.is_empty()
                && !ab.sac_cost
                && ab.sac_other_filter.is_none()
                && ab.tap_others_cost.is_none()
                && ab.tap_n_filter.is_none()
            {
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
            let budget = *mana.get_or_insert_with(|| mana_upper_bound(state, seat));
            if cmc > 0 && cmc > budget {
                continue;
            }
            // `{X}`: the largest payable X first, at most three that find a
            // target (Geth's X is its target's mana value).
            let xs: Vec<Option<u32>> = if ab.mana_cost.has_x() {
                (1..=budget.saturating_sub(cmc)).rev().map(Some).collect()
            } else {
                vec![None]
            };
            let mut probes = 0;
            for x in xs {
                if probes == 3 {
                    break;
                }
                let (target, additional_targets) = if ab.effect.requires_target() {
                    match state.auto_targets_for_effect_all_slots_x(&ab.effect, seat, None, false, Some(card.id), x) {
                        (Some(t), extra) => (Some(t), extra),
                        (None, _) => continue,
                    }
                } else {
                    (None, Vec::new())
                };
                // "Target player …" auto-aims at an opponent; a slot that is
                // a gift (Fertilid's land search) is worth asking about us too.
                let mut targets = vec![target];
                if let Some(Target::Player(p)) = targets[0]
                    && p != seat
                    && additional_targets.is_empty()
                {
                    targets.push(Some(Target::Player(seat)));
                }
                let mut stop = false;
                for target in targets {
                    let action = GameAction::ActivateAbility {
                        card_id: card.id,
                        ability_index: idx,
                        target,
                        additional_targets: additional_targets.clone(),
                        x_value: x,
                        mode: None,
                    };
                    // An until-end-of-turn gain reads as permanent to the evaluator.
                    if action_outcome_is_temporary(state, &action) {
                        stop = true;
                        break;
                    }
                    probes += 1;
                    let Some(settled) = state.accept(action.clone()) else { continue };
                    // A Class level buys static and triggered abilities the
                    // material eval can't price; one that can be paid is taken.
                    if matches!(ab.effect, Effect::AdvanceClassLevel) {
                        return Some(action);
                    }
                    let base = *baseline.get_or_insert_with(|| eval_material(state, seat, w));
                    if let Some(ev) = evaluate_action_outcome(state, seat, &action, Some(&settled), w)
                        && ev > base
                        && best.as_ref().is_none_or(|(b, _)| ev > *b)
                    {
                        best = Some((ev, action));
                    }
                }
                if stop {
                    break;
                }
                // Without `{X}` there is one action to try; with it and no
                // target, the largest X is the one worth asking about.
                if x.is_none() || !ab.effect.requires_target() {
                    break;
                }
            }
        }
    }
    best.map(|(_, a)| a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{ActivatedAbility, CardDefinition, CardType};
    use crate::effect::{PlayerRef, Selector, Value};

    /// "Target player draws a card" auto-aims at an opponent; the sink also
    /// asks about its own seat and takes the draw for itself.
    #[test]
    fn a_target_player_gift_is_aimed_at_the_bot() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        let rod = g.add_card_to_battlefield(
            0,
            CardDefinition {
                name: "Test Scholar's Rod",
                card_types: vec![CardType::Artifact],
                activated_abilities: vec![ActivatedAbility {
                    mana_cost: crate::mana::cost(&[crate::mana::generic(1)]),
                    tap_cost: true,
                    effect: Effect::Draw {
                        who: Selector::Player(PlayerRef::Target(0)),
                        amount: Value::ONE,
                    },
                    ..Default::default()
                }],
                ..Default::default()
            },
        );
        for seat in 0..3 {
            for _ in 0..3 {
                g.add_card_to_library(seat, crate::catalog::island());
            }
        }
        g.players[0].mana_pool.add_colorless(1);
        let got = pick_generic_ability(&g, 0, &EvalWeights::default());
        assert!(
            matches!(got, Some(GameAction::ActivateAbility { card_id, target: Some(Target::Player(0)), .. }) if card_id == rod),
            "got {got:?}"
        );
    }

    /// Captivating Vampire at the end step of the opponent seated before
    /// us: tapping five Vampires is free there, so the steal is taken — and
    /// not at an earlier opponent's end step (the Vampires would sit tapped
    /// through the next opponent's turn).
    #[test]
    fn captivating_vampire_steals_at_the_end_step_before_our_turn() {
        let mut g = crate::game::multi_player_game(4);
        let cv = g.add_card_to_battlefield(0, crate::catalog::captivating_vampire());
        for _ in 0..4 {
            g.add_card_to_battlefield(0, crate::catalog::vampire_nighthawk());
        }
        g.add_card_to_battlefield(3, crate::catalog::serra_angel());
        g.add_card_to_battlefield(1, crate::catalog::serra_angel());
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        g.active_player_idx = 2;
        assert!(pick_generic_ability(&g, 0, &EvalWeights::default()).is_none(), "seat 3 still to go");
        g.active_player_idx = 3;
        let got = pick_generic_ability(&g, 0, &EvalWeights::default());
        assert!(matches!(got, Some(GameAction::ActivateAbility { card_id, .. }) if card_id == cv), "got {got:?}");
        g.perform_action(got.unwrap()).expect("activate");
        crate::game::drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Serra Angel"), "an Angel stolen");
    }
}
