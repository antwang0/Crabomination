//! Abilities that function from the command zone and are activated there
//! (CR 113.6c / 903.9) — Derevi, Empyrial Tactician's "{1}{G}{W}{U}: Put
//! Derevi onto the battlefield from the command zone", which owes no commander
//! tax (it isn't a cast) and is instant-speed. No bot path listed a command-zone
//! ability: a 183-deck `--card-census` never activated Derevi's. A pod bot now
//! takes one when the dry run beats passing. Commander games only.

use crate::game::GameState;
use crate::game::types::GameAction;

use super::bot::{EvalWeights, eval_material, evaluate_action_outcome};

/// The best-scoring command-zone activation `seat` can make now, if any.
pub(super) fn pick_command_zone_ability(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || !state.stack.is_empty() {
        return None;
    }
    let candidates: Vec<(crate::card::CardId, usize)> = state.players[seat]
        .command
        .iter()
        .flat_map(|c| {
            c.definition
                .activated_abilities
                .iter()
                .enumerate()
                .filter(|(_, ab)| ab.from_command_zone)
                .map(move |(i, _)| (c.id, i))
        })
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let base = eval_material(state, seat, w);
    let mut best: Option<(i32, GameAction)> = None;
    for (card_id, idx) in candidates {
        let action =
            GameAction::ActivateAbility { card_id, ability_index: idx, target: None, additional_targets: vec![], x_value: None, mode: None };
        let Some(settled) = state.accept(action.clone()) else { continue };
        if let Some(ev) = evaluate_action_outcome(state, seat, &action, Some(&settled), w)
            && ev > base
            && best.as_ref().is_none_or(|(b, _)| ev > *b)
        {
            best = Some((ev, action));
        }
    }
    best.map(|(_, a)| a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;
    use crate::mana::Color;

    /// Derevi in the command zone with four mana up: the bot puts it onto the
    /// battlefield (no tax owed), and not without the mana.
    #[test]
    fn a_pod_bot_puts_derevi_in_from_the_command_zone() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        let derevi = g.seat_commanders(0, vec![crate::catalog::derevi_empyrial_tactician()])[0];
        let w = EvalWeights::default();
        assert_eq!(pick_command_zone_ability(&g, 0, &w), None, "no mana");
        for c in [Color::Green, Color::White, Color::Blue] {
            g.players[0].mana_pool.add(c, 1);
        }
        g.players[0].mana_pool.add_colorless(1);
        assert!(matches!(
            pick_command_zone_ability(&g, 0, &w),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == derevi
        ));
    }
}
