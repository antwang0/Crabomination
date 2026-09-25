//! CR 702.29 — the bot's only cycling path. Nothing constructed
//! `GameAction::Cycle`, so cycling lands and granted cycling (Rhet-Tomb
//! Mystic) were dead cards in self-play. Commander pods only: a duel's bot
//! keeps its committed behaviour (`--bench` byte-identical), and a pod's long
//! games are where a spare land or an uncastable fatty is worth a new card.

use crate::game::GameState;
use crate::game::types::GameAction;

/// Lands on the battlefield past which another land in hand is spare.
const ENOUGH_LANDS: usize = 6;

/// A cycle worth making at an opponent's end step, or `None`: a land once
/// `seat` controls six, or a nonland card whose mana value is more than two
/// above `seat`'s land count. A free cycle (Gavi, New Perspectives) loosens
/// both bars by two. The dry run is the gate (cost, Stabilizer).
pub(super) fn pick_cycle(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::card::Keyword;
    if state.players[seat].commanders.is_empty() {
        return None;
    }
    let lands = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_land())
        .count();
    let slack = if state.cycling_is_free(seat) { 2 } else { 0 };
    state.players[seat]
        .hand
        .iter()
        .filter(|c| {
            c.definition.keywords.iter().any(|k| matches!(k, Keyword::Cycling(_) | Keyword::CyclingLife(_)))
                || state.granted_cycling_for(seat, c).is_some()
        })
        .filter(|c| {
            if c.definition.is_land() {
                lands + slack >= ENOUGH_LANDS
            } else {
                c.definition.cost.cmc() as usize + slack > lands + 2
            }
        })
        .map(|c| GameAction::Cycle { card_id: c.id, x_value: None })
        .find(|a| state.would_accept(a.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::multi_player_game;
    use crate::game::types::TurnStep;
    use crate::mana::Color;

    /// A spare cycling land is cycled in a pod once six lands are out, and
    /// never in a game without commanders.
    #[test]
    fn a_pod_bot_cycles_a_spare_land() {
        let mut g = multi_player_game(3);
        g.active_player_idx = 1;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        let desert = g.add_card_to_hand(0, crate::catalog::desert_of_the_true());
        g.add_card_to_library(0, crate::catalog::plains());
        for _ in 0..5 {
            g.add_card_to_battlefield(0, crate::catalog::plains());
        }
        g.players[0].mana_pool.add(Color::White, 2);
        assert_eq!(pick_cycle(&g, 0), None, "no commander: not a pod");
        let cmd = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.players[0].commanders.push(cmd);
        assert_eq!(pick_cycle(&g, 0), None, "five lands: keep it");
        g.add_card_to_battlefield(0, crate::catalog::plains());
        assert_eq!(pick_cycle(&g, 0), Some(GameAction::Cycle { card_id: desert, x_value: None }));
    }
}
