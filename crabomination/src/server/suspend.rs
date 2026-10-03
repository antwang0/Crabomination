//! A card with suspend and no mana cost (Wheel of Fate, Ancestral Vision,
//! Living End) can only be cast by suspending it (CR 702.62a), and no bot
//! path suspended anything — Wheel of Fate was never played in 300 four-seat
//! pods (seed 10130, card census). The bot suspends such a card as soon as
//! it can; a castable card with suspend is left to the cast enumeration.

use crate::card::Keyword;
use crate::game::GameState;
use crate::game::types::GameAction;

/// The first accepted suspend of a hand card that has no mana cost. `None`
/// without one in hand — a hand walk, so the common case costs nothing more.
pub(super) fn pick_suspend_only(state: &GameState, seat: usize) -> Option<GameAction> {
    state.players[seat]
        .hand
        .iter()
        .filter(|c| {
            c.definition.cost.symbols.is_empty()
                && c.definition.keywords.iter().any(|k| matches!(k, Keyword::Suspend(..)))
        })
        .map(|c| GameAction::Suspend { card_id: c.id })
        .find(|a| state.would_accept(a.clone()))
}

/// Suspend X (Aeon Chronicler — N = 0 with an {X} in the cost) with mana the
/// bot would otherwise leave idle: the largest accepted X up to
/// [`SUSPEND_X_CAP`] (each X is a time counter and, for Aeon Chronicler, a
/// card drawn on the way in). `None` without such a card in hand.
pub(super) fn pick_suspend_x(state: &GameState, seat: usize) -> Option<GameAction> {
    let card = state.players[seat].hand.iter().find(|c| {
        c.definition.keywords.iter().any(|k| matches!(k, Keyword::Suspend(0, cost) if cost.has_x()))
    })?;
    (1..=SUSPEND_X_CAP)
        .rev()
        .map(|x_value| GameAction::SuspendX { card_id: card.id, x_value })
        .find(|a| state.would_accept(a.clone()))
}

/// The largest X [`pick_suspend_x`] tries: past four turns of counters a
/// suspended card is out of the game too long to matter.
const SUSPEND_X_CAP: u32 = 4;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;
    use crate::mana::Color;

    /// CR 702.62a — Wheel of Fate has no mana cost, so suspending it is the
    /// only way to cast it: the bot does, once it can pay {1}{R}.
    #[test]
    fn a_suspend_only_card_is_suspended() {
        let mut g = crate::game::two_player_game();
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let wheel = g.add_card_to_hand(0, crate::catalog::wheel_of_fate());
        g.add_card_to_hand(0, crate::catalog::grizzly_bears());
        assert_eq!(pick_suspend_only(&g, 0), None, "no mana");
        g.players[0].mana_pool.add(Color::Red, 2);
        assert!(matches!(pick_suspend_only(&g, 0), Some(GameAction::Suspend { card_id }) if card_id == wheel));
    }

    /// CR 702.62a — Suspend X: the bot names the largest X its mana pays for
    /// ({X}{3}{U} with six mana is X = 2), and nothing without the {3}{U}.
    #[test]
    fn suspend_x_takes_the_largest_affordable_x() {
        let mut g = crate::game::two_player_game();
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let aeon = g.add_card_to_hand(0, crate::catalog::aeon_chronicler());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3);
        assert_eq!(pick_suspend_x(&g, 0), None, "X can't be 0");
        g.players[0].mana_pool.add_colorless(2);
        assert_eq!(pick_suspend_x(&g, 0), Some(GameAction::SuspendX { card_id: aeon, x_value: 2 }));
    }
}
