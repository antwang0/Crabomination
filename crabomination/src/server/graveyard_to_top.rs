//! Pods only: "{cost}, {T}: put target [artifact / enchantment] card from
//! your graveyard on top of your library" (Academy Ruins, Hall of Heliod's
//! Generosity). The outcome score can't see a library's order, so a
//! 183-deck census never saw one activated. At the end step of the opponent
//! seated just before the bot — the tapped land untaps in a moment, and the
//! card is the next draw — the most expensive matching card goes back on top.
//!
//! Two seats never reach it, so the two-player bench pool is untouched.

use crate::effect::{Effect, LibraryPosition, PlayerRef, Selector, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameAction, Target, TurnStep};

/// Cheapest card worth a draw step.
const MIN_MANA_VALUE: u32 = 2;

/// "Put target card from your graveyard on top of your library".
fn recurs_to_top(e: &Effect) -> bool {
    matches!(
        e,
        Effect::Move {
            what: Selector::Target(0) | Selector::TargetFiltered { slot: 0, .. },
            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
        }
    )
}

pub(super) fn pick_graveyard_to_top(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players.len() <= 2
        || state.step != TurnStep::End
        || state.active_player_idx == seat
        || state.next_alive_seat(state.active_player_idx) != seat
        || !state.stack.is_empty()
    {
        return None;
    }
    let mut picks: Vec<(u32, crate::card::CardId)> = state.players[seat]
        .graveyard
        .iter()
        .filter(|c| !c.is_token)
        .map(|c| (c.definition.cost.cmc(), c.id))
        .filter(|&(mv, _)| mv >= MIN_MANA_VALUE)
        .collect();
    picks.sort_by_key(|&(mv, id)| (std::cmp::Reverse(mv), id));
    state.battlefield.iter().filter(|c| c.controller == seat).find_map(|c| {
        c.definition.activated_abilities.iter().enumerate().find_map(|(i, ab)| {
            if !recurs_to_top(&ab.effect) || ab.sac_cost || ab.sac_other_filter.is_some() {
                return None;
            }
            picks.iter().find_map(|&(_, id)| {
                let action = GameAction::ActivateAbility {
                    card_id: c.id,
                    ability_index: i,
                    target: Some(Target::Permanent(id)),
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

    /// Academy Ruins puts the costliest artifact card on top at the end step
    /// before the bot's turn, and not at another seat's.
    #[test]
    fn academy_ruins_recurs_the_best_artifact() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 2;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        let ruins = g.add_card_to_battlefield(0, crate::catalog::academy_ruins());
        g.clear_sickness(ruins);
        g.add_card_to_graveyard(0, crate::catalog::sol_ring());
        let stone = g.add_card_to_graveyard(0, crate::catalog::mind_stone());
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
        assert!(matches!(
            pick_graveyard_to_top(&g, 0),
            Some(GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. })
                if card_id == ruins && t == stone
        ));
        g.active_player_idx = 1;
        assert!(pick_graveyard_to_top(&g, 0).is_none(), "seat 1 isn't just before us");
    }
}
