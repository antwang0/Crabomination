//! CR 702.143 — no bot path foretold a card or cast a foretold one, so every
//! foretell card in the catalog was a plain hard-cast to self-play. The bot
//! now foretells when its main phase has nothing better to do (the {2} is
//! idle mana, and Ranar makes the first one free) and casts foretold cards
//! from exile through the main-phase enumeration.

use crate::game::GameState;
use crate::game::types::GameAction;

/// Foretell the most expensive foretellable hand card when the main-phase
/// pick was a pass. `None` without one — a hand walk, so a deck with no
/// foretell cards pays nothing more.
pub(super) fn pick_foretell(state: &GameState, seat: usize) -> Option<GameAction> {
    state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.foretell_cost.is_some())
        .max_by_key(|c| c.definition.cost.cmc())
        .map(|c| GameAction::Foretell { card_id: c.id })
        .filter(|a| state.would_accept(a.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;
    use crate::mana::Color;

    /// CR 702.143a — with {2} spare and nothing else to do, the bot foretells.
    #[test]
    fn a_foretell_card_is_foretold_with_idle_mana() {
        let mut g = crate::game::two_player_game();
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let card = g.add_card_to_hand(0, crate::catalog::saw_it_coming());
        assert!(pick_foretell(&g, 0).is_none(), "no mana, no foretell");
        g.players[0].mana_pool.add(Color::Blue, 2);
        assert!(matches!(pick_foretell(&g, 0), Some(GameAction::Foretell { card_id }) if card_id == card));
    }
}
