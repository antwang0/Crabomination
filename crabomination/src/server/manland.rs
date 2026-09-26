//! A creature-land's "becomes an N/N creature until end of turn" (Raging
//! Ravine, Mutavault, Celestial Colonnade, Den of the Bugbear) was activated by
//! no bot path — a 6-seat `--card-census` over every deck (seed 206001) never
//! animated any of 30-odd of them. The bot now animates one with idle
//! first-main mana so the attack planner can send it. Commander games only,
//! so two-player play is unchanged.

use crate::effect::{Duration, Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameAction, TurnStep};

/// Does this ability turn its own noncreature source into a creature for the turn?
fn animates_self(e: &Effect) -> bool {
    matches!(
        e,
        Effect::BecomeCreature { what: Selector::This, duration: Duration::EndOfTurn, .. }
            | Effect::AnimateAsCreature { what: Selector::This, duration: Duration::EndOfTurn }
    )
}

/// The first accepted self-animation of an untapped noncreature permanent
/// `seat` has controlled since the turn began, in `seat`'s first main phase.
pub(super) fn pick_manland(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty()
        || state.step != TurnStep::PreCombatMain
        || state.active_player_idx != seat
        || !state.stack.is_empty()
    {
        return None;
    }
    state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && !c.tapped && !c.summoning_sick && !c.definition.is_creature())
        .flat_map(|c| {
            c.definition.activated_abilities.iter().enumerate().filter_map(move |(i, ab)| {
                (animates_self(&ab.effect) && !ab.tap_cost && !ab.sac_cost).then_some(GameAction::ActivateAbility {
                    card_id: c.id,
                    ability_index: i,
                    target: None,
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                })
            })
        })
        .find(|a| state.would_accept(a.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mana::Color;

    /// Mutavault animates in a Commander game's first main, not when it came
    /// in this turn, and not in a duel.
    #[test]
    fn a_manland_is_animated_with_idle_mana() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let vault = g.add_card_to_battlefield(0, crate::catalog::mutavault());
        g.players[0].mana_pool.add(Color::Green, 1);
        assert!(pick_manland(&g, 0).is_none(), "outside Commander");
        g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
        g.battlefield_find_mut(vault).unwrap().summoning_sick = true;
        assert!(pick_manland(&g, 0).is_none(), "entered this turn");
        g.clear_sickness(vault);
        assert!(matches!(pick_manland(&g, 0), Some(GameAction::ActivateAbility { card_id, .. }) if card_id == vault));
    }
}
