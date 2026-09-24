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
}
