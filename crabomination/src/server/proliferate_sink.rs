//! A proliferate ability as a mana sink (Contagion Engine, Karn's Bastion,
//! Yawgmoth's {B}{B}) — none was activated in a 48-deck `--card-census`
//! (seed 10169). Taken only when proliferating helps the seat: a permanent of
//! its own carries a +1/+1 or loyalty counter, or an opponent has poison.

use crate::card::CounterType;
use crate::effect::{ActivatedAbility, Effect};
use crate::game::GameState;
use crate::game::types::GameAction;

/// The ability only proliferates (one or more times).
pub(super) fn only_proliferates(ab: &ActivatedAbility) -> bool {
    match &ab.effect {
        Effect::Proliferate => true,
        Effect::Seq(v) => !v.is_empty() && v.iter().all(|e| matches!(e, Effect::Proliferate)),
        _ => false,
    }
}

fn proliferate_helps(state: &GameState, seat: usize) -> bool {
    let ours = state.battlefield.iter().filter(|c| c.controller == seat).any(|c| {
        c.counter_count(CounterType::PlusOnePlusOne) > 0 || c.counter_count(CounterType::Loyalty) > 0
    });
    ours || state.opponents_of(seat).into_iter().any(|p| state.players[p].poison_counters > 0)
}

pub(super) fn pick_proliferate(state: &GameState, seat: usize) -> Option<GameAction> {
    if !proliferate_helps(state, seat) {
        return None;
    }
    state.battlefield.iter().filter(|c| c.controller == seat).find_map(|c| {
        c.definition.activated_abilities.iter().enumerate().find_map(|(i, ab)| {
            if !only_proliferates(ab) || ab.sac_cost || ab.sac_other_filter.is_some() {
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
            state.would_accept(action.clone()).then_some(action)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;

    /// Karn's Bastion proliferates once a creature of ours carries a +1/+1
    /// counter, and not before.
    #[test]
    fn karns_bastion_proliferates_with_a_counter_to_grow() {
        let mut g = crate::game::two_player_game();
        g.step = TurnStep::PostCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let bastion = g.add_card_to_battlefield(0, crate::catalog::karns_bastion());
        g.players[0].mana_pool.add_colorless(4);
        let bear = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        assert!(pick_proliferate(&g, 0).is_none(), "nothing to proliferate");
        g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        assert!(matches!(pick_proliferate(&g, 0), Some(GameAction::ActivateAbility { card_id, .. }) if card_id == bastion));
    }
}
