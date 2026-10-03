//! The bot's sink for "{X}, Remove X [kind] counters from this / from among
//! your permanents" and "pay X {E}" activations
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
            // "Remove X [kind] counters from this", or from among / from one
            // of the matching permanents you control (Ooze Flux, Moxite
            // Refinery) — capped, each X is a dry run.
            let have = if let Some(kind) = ab.remove_counter_x {
                card.counter_count(kind)
            } else if let Some((kind, filter)) = &ab.remove_counter_among_x {
                let counts = state
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == seat)
                    .filter(|c| state.evaluate_requirement_static_on(filter, c, seat, Some(card.id)))
                    .map(|c| match kind {
                        Some(k) => c.counter_count(*k),
                        None => c.counters.values().sum(),
                    });
                let n: u32 = if ab.remove_counter_among_x_one { counts.max().unwrap_or(0) } else { counts.sum() };
                n.min(12)
            } else if ab.energy_x_cost {
                // "Pay X {E}" (Sphinx of the Revelation, Chthonian Nightmare).
                state.players[seat].energy.min(12)
            } else {
                continue;
            };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::CounterType;
    use crate::game::types::{Target, TurnStep};

    /// Moxite Refinery's X comes from among your permanents' counters; no
    /// generator sized it, so the census never saw it activated. The sink
    /// finds an X that turns an artifact's charge into a creature's +1/+1s.
    #[test]
    fn a_pod_bot_sizes_moxite_refinerys_x() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let mr = g.add_card_to_battlefield(0, crate::catalog::moxite_refinery());
        let eng = g.add_card_to_battlefield(0, crate::catalog::insight_engine());
        g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.battlefield_find_mut(eng).unwrap().add_counters(CounterType::Charge, 3);
        g.players[0].mana_pool.add_colorless(2);
        let pick = pick_x_counter_ability(&g, 0, &EvalWeights::default());
        assert!(
            matches!(pick, Some(GameAction::ActivateAbility { card_id, x_value: Some(x), target: Some(Target::Permanent(_)), .. }) if card_id == mr && x > 0),
            "{pick:?}"
        );
    }

    /// "Pay X {E}" is an X the bot sizes too: Sphinx of the Revelation draws
    /// with the energy it has.
    #[test]
    fn a_pod_bot_sizes_an_energy_x() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let s = g.add_card_to_battlefield(0, crate::catalog::sphinx_of_the_revelation());
        g.clear_sickness(s);
        for _ in 0..5 {
            g.add_card_to_library(0, crate::catalog::island());
        }
        g.players[0].energy = 3;
        for c in [crate::mana::Color::White, crate::mana::Color::Blue] {
            g.players[0].mana_pool.add(c, 3);
        }
        let pick = pick_x_counter_ability(&g, 0, &EvalWeights::default());
        assert!(
            matches!(pick, Some(GameAction::ActivateAbility { card_id, x_value: Some(x), .. }) if card_id == s && x > 0),
            "{pick:?}"
        );
    }
}
