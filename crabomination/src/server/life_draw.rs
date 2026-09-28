//! Pods only: "Pay 1 life: exile the top card of your library face down; put
//! it into your hand at the beginning of your next end step" (Necropotence).
//! The card arrives after the material eval's horizon, so no picker ever paid
//! the life — and Necropotence skips its controller's draw step, so a seat
//! that never activates it only loses cards (`--card-census`: never activated).

use crate::effect::{Effect, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameAction, TurnStep};

/// The life a seat keeps back: a dozen, or the opposing board's power and
/// four more, whichever is higher.
fn life_floor(state: &GameState, seat: usize) -> i32 {
    let power: i32 = state
        .battlefield
        .iter()
        .filter(|c| c.controller != seat && !state.same_team(c.controller, seat) && c.definition.is_creature())
        .filter_map(|c| state.computed_permanent(c.id).map(|cp| cp.power.max(0)))
        .fold(0i32, i32::saturating_add);
    power.saturating_add(4).max(12)
}

/// True when `effect` is a delayed draw: its card reaches the hand at a later
/// end step.
fn delayed_draw(effect: &Effect) -> bool {
    match effect {
        Effect::Seq(v) => v.iter().any(delayed_draw),
        Effect::AtNextEndStep { body } => matches!(**body, Effect::Move { to: ZoneDest::Hand(_), .. }),
        _ => false,
    }
}

/// A life-only activation that banks a card for the end step, while the seat
/// stays above its life floor and under seven cards in hand counting the ones
/// already banked.
pub(super) fn pick_life_draw(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players.len() <= 2
        || state.active_player_idx != seat
        || !matches!(state.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
        || !state.stack.is_empty()
    {
        return None;
    }
    let player = state.players.get(seat)?;
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if ab.life_cost == 0
                || !ab.mana_cost.symbols.is_empty()
                || ab.tap_cost
                || ab.sac_cost
                || !delayed_draw(&ab.effect)
            {
                continue;
            }
            let banked = state.exile.iter().filter(|c| c.owner == seat && c.exiled_with == Some(card.id)).count();
            if player.hand.len() + banked >= 7
                || player.library.is_empty()
                || player.life.saturating_sub(ab.life_cost as i32) <= life_floor(state, seat)
            {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: vec![],
                x_value: None,
                mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::game::multi_player_game;

    /// Necropotence banks cards until hand plus banked reaches seven, then
    /// stops; a two-seat game never asks.
    #[test]
    fn necropotence_banks_cards_up_to_a_full_hand() {
        let mut g = multi_player_game(4);
        let necro = g.add_card_to_battlefield(0, catalog::necropotence());
        for _ in 0..10 {
            g.add_card_to_library(0, catalog::swamp());
        }
        for _ in 0..4 {
            g.add_card_to_hand(0, catalog::swamp());
        }
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let mut paid = 0;
        while let Some(a) = pick_life_draw(&g, 0) {
            g.perform_action(a).expect("activation");
            crate::game::drain_stack(&mut g);
            paid += 1;
            assert!(paid <= 3, "stops at a full hand");
        }
        assert_eq!(paid, 3, "four in hand + three banked");
        assert_eq!(g.exile.iter().filter(|c| c.exiled_with == Some(necro)).count(), 3);

        let mut g = crate::game::two_player_game();
        g.add_card_to_battlefield(0, catalog::necropotence());
        g.add_card_to_library(0, catalog::swamp());
        g.step = TurnStep::PreCombatMain;
        assert!(pick_life_draw(&g, 0).is_none(), "pods only");
    }
}
