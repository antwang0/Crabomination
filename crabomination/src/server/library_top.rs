//! CR 401.6 — "you may cast [spells] from the top of your library" (Melek,
//! Izzet Paragon; Thundermane Dragon; Mystic Forge; Courser of Kruphix's
//! lands). The bot's cast and land enumerators read the hand, so no seat
//! ever played off the top: a 183-deck `--card-census --a dflt` never saw
//! Melek's or Thundermane Dragon's "cast from your library" trigger fire.
//! A pod bot now plays the top card when its permission covers it and the
//! turn is otherwise idle. Commander games only.

use crate::game::GameState;
use crate::game::types::GameAction;

/// A land drop or cast of `seat`'s playable library top, if one is accepted.
pub(super) fn pick_library_top(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || !state.stack.is_empty() {
        return None;
    }
    let top = state.players[seat].library.first()?;
    if !state.library_top_playable(seat, top.id) {
        return None;
    }
    let action = if top.definition.is_land() {
        GameAction::PlayLand(top.id)
    } else {
        let (target, additional_targets) = state.auto_targets_for_effect_all_slots(&top.definition.effect, seat, None);
        GameAction::CastSpell { card_id: top.id, target, additional_targets, mode: None, x_value: None }
    };
    state.would_accept(action.clone()).then_some(action)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;
    use crate::mana::Color;

    /// Thundermane Dragon: a big creature on top is cast in a pod, a small
    /// one (outside the permission) is not, nor anything without commanders.
    #[test]
    fn thundermane_dragon_casts_the_top_creature_in_a_pod() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, crate::catalog::thundermane_dragon());
        let bear = g.add_card_to_library(0, crate::catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 8);
        let cmd = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.players[0].commanders.push(cmd);
        assert_eq!(pick_library_top(&g, 0), None, "power 2: not covered");
        g.players[0].library.retain(|c| c.id != bear);
        let wurm = g.add_card_to_library(0, crate::catalog::craw_wurm());
        assert!(matches!(pick_library_top(&g, 0), Some(GameAction::CastSpell { card_id, .. }) if card_id == wurm));
        g.players[0].commanders.clear();
        assert_eq!(pick_library_top(&g, 0), None, "no commander: not a pod");
    }
}
