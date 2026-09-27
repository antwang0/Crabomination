//! A repeatable chip at a player — Theater of Horrors' "{3}{R}: 1 damage to
//! target opponent", Chandra, Fire of Kaladesh's "{T}: 1 damage to target
//! player" — was never activated in a 6-seat census (seed 1270001): the
//! removal ping only aims at a face when the shot is lethal. At an opponent's
//! end step the mana and the tap are spare (both come back at the untap), so
//! a pod bot now spends them on the lowest-life opponent. Commander games
//! only, so two-player play is unchanged.

use crate::effect::{Effect, Selector, Value};
use crate::game::GameState;
use crate::game::types::{GameAction, Target};

/// A constant damage aimed at the ability's target slot, alone or leading a
/// sequence (Chandra's transform check follows it).
fn chips_target(e: &Effect) -> bool {
    match e {
        Effect::DealDamage { to: Selector::Target(_) | Selector::TargetFiltered { .. }, amount: Value::Const(n) } => {
            *n > 0
        }
        Effect::Seq(v) => v.first().is_some_and(chips_target),
        _ => false,
    }
}

/// The first accepted chip at an opponent, lowest life first, at an
/// opponent's end step.
pub(super) fn pick_end_step_ping(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || state.active_player_idx == seat || !state.stack.is_empty() {
        return None;
    }
    let mut opps: Vec<usize> =
        (0..state.players.len()).filter(|&q| !state.same_team(q, seat) && state.players[q].is_alive()).collect();
    opps.sort_by_key(|&q| (state.effective_life(q), q));
    for c in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (i, ab) in c.definition.activated_abilities.iter().enumerate() {
            let costs_only_mana = !ab.sac_cost
                && ab.sac_other_filter.is_none()
                && ab.tap_other_filter.is_none()
                && ab.discard_cost.is_none()
                && ab.life_cost == 0
                && ab.energy_cost == 0
                && ab.remove_counter_cost.is_none()
                && ab.remove_all_counters_cost.is_none()
                && ab.remove_counter_among_filter.is_none()
                && ab.exile_other_filter.is_none()
                && ab.remove_counter_x.is_none()
                && !ab.mana_cost.has_x()
                // A free untapped one would be taken every priority.
                && (ab.tap_cost || ab.mana_cost.cmc() > 0);
            if !costs_only_mana || !chips_target(&ab.effect) {
                continue;
            }
            for &q in &opps {
                let action = GameAction::ActivateAbility {
                    card_id: c.id,
                    ability_index: i,
                    target: Some(Target::Player(q)),
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                };
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;
    use crate::mana::Color;

    /// Theater of Horrors pings the lowest-life opponent at an opponent's end
    /// step with the mana spare, not on its controller's turn.
    #[test]
    fn theater_of_horrors_chips_the_lowest_life_opponent() {
        let mut g = crate::game::multi_player_game(4);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        let theater = g.add_card_to_battlefield(0, crate::catalog::theater_of_horrors());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.players[2].life = 12;
        g.step = TurnStep::End;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        assert!(pick_end_step_ping(&g, 0).is_none(), "own turn");
        g.active_player_idx = 1;
        let a = pick_end_step_ping(&g, 0).expect("ping");
        assert!(matches!(a, GameAction::ActivateAbility { card_id, target: Some(Target::Player(2)), .. } if card_id == theater));
        g.perform_action(a).expect("activate");
        crate::game::drain_stack(&mut g);
        assert_eq!(g.players[2].life, 11);
    }
}
