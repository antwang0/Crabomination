//! Pods only: "{T}: draw two cards, then target opponent gains control of
//! this" (Humble Defector). The outcome score prices a 2/1 handed to an
//! opponent above two cards, so a 183-deck census never saw one activated —
//! though handing it over is the card's whole point, and the opponent who
//! gets it can do the same back. A small body (power at most
//! [`MAX_DONATED_POWER`]) is given to the opponent who threatens the seat
//! least; a big one stays home.
//!
//! Two seats never reach it, so the two-player bench pool is untouched.

use crate::effect::{Effect, Selector, Value};
use crate::game::GameState;
use crate::game::types::{GameAction, Target};

/// The most power the bot will hand an opponent for its cards.
const MAX_DONATED_POWER: i32 = 3;

/// "Draw N (N ≥ 2) … an opponent gains control of this".
fn is_donate_draw(e: &Effect) -> bool {
    let Effect::Seq(v) = e else { return false };
    let draws = v.iter().any(|s| {
        matches!(s, Effect::Draw { who: Selector::You, amount: Value::Const(n) } if *n >= 2)
    });
    let donates = v.iter().any(|s| matches!(s, Effect::GainControl { what: Selector::This, to: Some(_), .. }));
    draws && donates
}

pub(super) fn pick_donate_draw(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players.len() <= 2 || state.players[seat].library.len() < 4 {
        return None;
    }
    // The opponent least able to hurt us with it: the fewest creatures.
    let mut opps = state.opponents_of(seat);
    opps.sort_by_key(|&p| {
        (state.battlefield.iter().filter(|c| c.controller == p && state.computed_is_creature(c)).count(), p)
    });
    state.battlefield.iter().filter(|c| c.controller == seat).find_map(|c| {
        let small = state.computed_permanent(c.id).is_some_and(|cp| cp.power <= MAX_DONATED_POWER);
        if !small {
            return None;
        }
        c.definition.activated_abilities.iter().enumerate().find_map(|(i, ab)| {
            if !is_donate_draw(&ab.effect) {
                return None;
            }
            opps.iter().find_map(|&p| {
                let action = GameAction::ActivateAbility {
                    card_id: c.id,
                    ability_index: i,
                    target: Some(Target::Player(p)),
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                };
                state.would_accept(action.clone()).then_some(action)
            })
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;

    /// Humble Defector draws two and goes to the opponent with fewer
    /// creatures, at three seats; never in a duel.
    #[test]
    fn humble_defector_is_handed_to_the_quieter_opponent() {
        for seats in [3, 2] {
            let mut g = crate::game::multi_player_game(seats);
            g.active_player_idx = 0;
            g.step = TurnStep::PostCombatMain;
            g.priority.player_with_priority = 0;
            let d = g.add_card_to_battlefield(0, crate::catalog::humble_defector());
            g.clear_sickness(d);
            for _ in 0..6 {
                g.add_card_to_library(0, crate::catalog::island());
            }
            g.add_card_to_battlefield(1, crate::catalog::grizzly_bears());
            let got = pick_donate_draw(&g, 0);
            if seats == 2 {
                assert!(got.is_none(), "a duel is untouched");
            } else {
                assert!(matches!(got, Some(GameAction::ActivateAbility { card_id, target: Some(Target::Player(2)), .. }) if card_id == d));
            }
        }
    }
}
