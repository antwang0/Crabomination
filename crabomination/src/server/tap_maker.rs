//! Pods only: a creature whose `{T}` ability makes a token stays home from an
//! attack that kills nobody. The token sink (`pick_token_maker`) asks in the
//! post-combat main phase and at the end step before the seat's turn, and an
//! attacker is tapped at both — Bloodline Keeper's "{T}: create a 2/2 black
//! Vampire with flying" went unactivated across a 3,000-game six-seat census
//! because the Keeper swung for 3 every turn instead.
//!
//! Two seats never reach it, so the two-player bench pool is untouched.

use crate::game::GameState;
use crate::game::types::{Attack, AttackTarget, GameAction};

use super::bot::{EvalWeights, ability_makes_token, sink_sacrifice_cost};

/// `attacks` without the creatures that would rather tap for a token, unless
/// the declaration as given is lethal to some defender.
pub(super) fn hold_tap_token_makers(
    state: &GameState,
    seat: usize,
    attacks: Vec<Attack>,
    w: &EvalWeights,
) -> Vec<Attack> {
    if state.players.len() <= 2 || attacks.is_empty() {
        return attacks;
    }
    if super::renewal_guard::board_is_saturated(state, seat) || super::renewal_guard::board_is_cluttered(state, seat) {
        return attacks;
    }
    // A swing that kills a defender keeps every body in it.
    let lethal = state.players.iter().enumerate().any(|(p, pl)| {
        p != seat
            && pl.is_alive()
            && attacks
                .iter()
                .filter(|a| a.target == AttackTarget::Player(p))
                .filter_map(|a| state.computed_permanent(a.attacker))
                .fold(0i32, |n, cp| n.saturating_add(cp.power.max(0)))
                >= state.effective_life(p)
    });
    if lethal {
        return attacks;
    }
    let makes_tokens = |id| {
        let Some(card) = state.battlefield_find(id) else { return false };
        card.definition.activated_abilities.iter().enumerate().any(|(idx, ab)| {
            ab.tap_cost
                && !ab.exhaust
                && !ab.sorcery_speed
                && !sink_sacrifice_cost(ab, w)
                && !ab.effect.requires_target()
                && ability_makes_token(&ab.effect)
                && state.would_accept(GameAction::ActivateAbility {
                    card_id: id,
                    ability_index: idx,
                    target: None,
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                })
        })
    };
    let kept: Vec<Attack> = attacks.iter().filter(|a| !makes_tokens(a.attacker)).copied().collect();
    if kept.len() == attacks.len() || !state.would_accept(GameAction::DeclareAttackers(kept.clone())) {
        return attacks;
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::game::types::TurnStep;

    /// A Bloodline Keeper that can tap for a Vampire stays home from a swing
    /// that kills nobody, at four seats; a duel keeps the attack.
    #[test]
    fn bloodline_keeper_makes_a_vampire_instead_of_swinging() {
        for seats in [4, 2] {
            let mut g = crate::game::multi_player_game(seats);
            g.active_player_idx = 0;
            g.step = TurnStep::DeclareAttackers;
            g.priority.player_with_priority = 0;
            let keeper = g.add_card_to_battlefield(0, catalog::bloodline_keeper());
            let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(keeper);
            g.clear_sickness(bears);
            let at = |id| Attack { attacker: id, target: AttackTarget::Player(1) };
            let held = hold_tap_token_makers(&g, 0, vec![at(keeper), at(bears)], &EvalWeights::default());
            if seats == 2 {
                assert_eq!(held.len(), 2, "a duel is untouched");
            } else {
                assert_eq!(held, vec![at(bears)], "the Keeper stays home");
            }
        }
    }
}
