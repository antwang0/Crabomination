//! Pods only: "discard a card: you may cast a creature spell from your
//! graveyard this turn" (Chainer, Nightmare Adept). The bot already casts
//! from the graveyard once the permission is live (`cast_candidates`), but a
//! 183-deck census never saw the permission bought. Taken in the seat's own
//! main phase once nothing else is castable, with a spare card to discard,
//! when the permission makes a graveyard creature castable.
//!
//! Two seats never reach it, so the two-player bench pool is untouched.

use crate::effect::Effect;
use crate::game::GameState;
use crate::game::types::{GameAction, TurnStep};

use super::bot::{EvalWeights, cast_candidates};

/// Hand size at which the cheapest card is spare even when it isn't a land.
const SPARE_HAND_AT: usize = 3;

pub(super) fn pick_graveyard_cast_permission(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    if state.players.len() <= 2
        || state.active_player_idx != seat
        || !matches!(state.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
        || !state.stack.is_empty()
        || state.graveyard_creature_cast_live(seat)
        || !state.players[seat].graveyard.iter().any(|c| c.definition.is_creature())
    {
        return None;
    }
    let hand = &state.players[seat].hand;
    if !(hand.len() >= SPARE_HAND_AT || hand.iter().any(|c| c.definition.is_land())) {
        return None;
    }
    state.battlefield.iter().filter(|c| c.controller == seat).find_map(|c| {
        c.definition.activated_abilities.iter().enumerate().find_map(|(i, ab)| {
            if !matches!(ab.effect, Effect::MayCastCreatureFromGraveyardThisTurn) {
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
            if !state.would_accept(action.clone()) {
                return None;
            }
            // The board once it resolves: the permission live, the cheapest
            // card discarded.
            let mut after = state.clone();
            after.players[seat].graveyard_creature_cast_turn = Some(state.turn_number);
            if let Some(pos) = (0..after.players[seat].hand.len()).min_by_key(|&k| {
                let h = &after.players[seat].hand[k];
                (h.definition.cost.cmc(), h.id)
            }) {
                let card = after.players[seat].hand.remove(pos);
                after.players[seat].graveyard.push(card);
            }
            let graveyard: Vec<_> = after.players[seat].graveyard.iter().map(|g| g.id).collect();
            let enables = cast_candidates(&after, seat, w, None).into_iter().any(|(a, _)| {
                matches!(a, GameAction::CastSpell { card_id, .. } if graveyard.contains(&card_id)) && after.would_accept(a)
            });
            enables.then_some(action)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chainer discards a spare land for the Craw Wurm in its graveyard, and
    /// not with nothing there to cast.
    #[test]
    fn chainer_buys_the_graveyard_cast() {
        let w = EvalWeights::default();
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let chainer = g.add_card_to_battlefield(0, crate::catalog::chainer_nightmare_adept());
        for _ in 0..6 {
            g.add_card_to_battlefield(0, crate::catalog::forest());
        }
        g.add_card_to_hand(0, crate::catalog::swamp());
        assert!(pick_graveyard_cast_permission(&g, 0, &w).is_none(), "nothing in the graveyard");
        g.add_card_to_graveyard(0, crate::catalog::craw_wurm());
        assert!(matches!(
            pick_graveyard_cast_permission(&g, 0, &w),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == chainer
        ));
    }
}
