//! Pods only: a graveyard ability usable only during its controller's upkeep
//! (Eternal Dragon's "{3}{W}{W}: return this from your graveyard to your
//! hand. Activate only during your upkeep"). The bot passes its own upkeep,
//! and the main-phase graveyard walk never sees a window the ability can use,
//! so a 183-deck census never saw one activated. It is taken when the seat's
//! mana covers it with [`SPARE`] to spare for the turn ahead.
//!
//! Two seats never reach it, so the two-player bench pool is untouched.

use crate::card::Predicate;
use crate::game::GameState;
use crate::game::types::{GameAction, TurnStep};

/// Mana the seat keeps for its main phases after paying.
const SPARE: u32 = 3;

/// Does `p` (an ability condition) name the upkeep step?
fn names_upkeep(p: &Predicate) -> bool {
    match p {
        Predicate::CurrentStepIs(TurnStep::Upkeep) => true,
        Predicate::All(v) | Predicate::Any(v) => v.iter().any(names_upkeep),
        _ => false,
    }
}

pub(super) fn pick_upkeep_graveyard_ability(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players.len() <= 2
        || state.active_player_idx != seat
        || state.step != TurnStep::Upkeep
        || !state.stack.is_empty()
    {
        return None;
    }
    let budget = super::bot::mana_upper_bound(state, seat);
    state.players[seat].graveyard.iter().find_map(|c| {
        c.definition.activated_abilities.iter().enumerate().find_map(|(i, ab)| {
            if !ab.from_graveyard
                || ab.effect.requires_target()
                || !ab.condition.as_ref().is_some_and(names_upkeep)
                || ab.mana_cost.cmc() + SPARE > budget
            {
                return None;
            }
            let action = GameAction::ActivateAbility {
                card_id: c.id,
                ability_index: i,
                target: None,
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            state.would_accept(action.clone()).then_some(action)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Eternal Dragon comes back to hand in its owner's upkeep with mana to
    /// spare, at three seats; not in a duel, and not short of mana.
    #[test]
    fn eternal_dragon_returns_in_upkeep() {
        for (seats, lands, want) in [(3, 8, true), (3, 6, false), (2, 8, false)] {
            let mut g = crate::game::multi_player_game(seats);
            g.active_player_idx = 0;
            g.step = TurnStep::Upkeep;
            g.priority.player_with_priority = 0;
            let dragon = g.add_card_to_graveyard(0, crate::catalog::eternal_dragon());
            for _ in 0..lands {
                let l = g.add_card_to_battlefield(0, crate::catalog::plains());
                g.clear_sickness(l);
            }
            let got = pick_upkeep_graveyard_ability(&g, 0);
            assert_eq!(
                matches!(got, Some(GameAction::ActivateAbility { card_id, .. }) if card_id == dragon),
                want,
                "{seats} seats, {lands} lands"
            );
        }
    }
}
