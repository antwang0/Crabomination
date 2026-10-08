//! "Spells you cast this turn that are black and/or red cost {X} less" —
//! Rowan, Scion of War's {T}, X the life you lost this turn. A pod census of
//! the Rowan seat never activated it: the main-phase enumeration scores a
//! turn-long discount at nothing. A pod bot now takes it in its precombat
//! main when the discount is at least `MIN_DISCOUNT` and a matching spell
//! in hand costs that much. Commander games only, so two-player play is
//! unchanged.

use crate::effect::Effect;
use crate::game::GameState;
use crate::game::effects::EffectContext;
use crate::game::types::{GameAction, TurnStep};

/// A discount below this isn't worth tapping the source (Rowan stops
/// attacking).
const MIN_DISCOUNT: i32 = 2;

pub(super) fn pick_spell_discount(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty()
        || state.active_player_idx != seat
        || state.step != TurnStep::PreCombatMain
        || !state.stack.is_empty()
    {
        return None;
    }
    for c in state.battlefield.iter().filter(|c| c.controller == seat && !c.tapped) {
        for (i, ab) in c.definition.activated_abilities.iter().enumerate() {
            let Effect::SpellsCostLessThisTurnByValue { filter, amount } = &ab.effect else { continue };
            let ctx = EffectContext::for_ability(c.id, seat, None);
            let n = state.evaluate_value(amount, &ctx);
            if n < MIN_DISCOUNT {
                continue;
            }
            let wanted = state.players[seat].hand.iter().any(|h| {
                !h.definition.is_land()
                    && h.definition.cost.cmc() as i32 >= n
                    && state.evaluate_requirement_on_card(filter, h, seat)
            });
            if !wanted {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: c.id,
                ability_index: i,
                target: None,
                additional_targets: Vec::new(),
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

    /// Rowan taps for the discount after its controller lost 3 life, with a
    /// black spell of mana value 3 or more in hand; not with none lost.
    #[test]
    fn rowan_takes_the_discount_after_losing_life() {
        for (lost, want) in [(3, true), (0, false)] {
            let mut g = crate::game::multi_player_game(4);
            g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
            let rowan = g.add_card_to_battlefield(0, crate::catalog::rowan_scion_of_war());
            g.clear_sickness(rowan);
            g.add_card_to_hand(0, crate::catalog::vilis_broker_of_blood());
            g.players[0].life_lost_this_turn = lost;
            g.step = TurnStep::PreCombatMain;
            g.active_player_idx = 0;
            g.priority.player_with_priority = 0;
            assert_eq!(pick_spell_discount(&g, 0).is_some(), want, "lost {lost}");
        }
    }
}
