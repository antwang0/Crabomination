//! Pods only: "{T}: untap target Forest" (Arbor Elf) and "{T}: untap another
//! target permanent" (Kiora's Follower) on a tapped land of the seat's — a
//! land's worth of mana again. The eval prices the untap at nothing, so a
//! 183-deck census never saw one activated. Taken in the seat's own main
//! phase once it has nothing castable, and only when the untapped land makes
//! a hand card castable.
//!
//! Two seats never reach it, so the two-player bench pool is untouched.

use crate::card::CardId;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameAction, Target, TurnStep};

use super::bot::{EvalWeights, cast_candidates};

/// An ability whose whole effect untaps its slot-0 target.
fn untaps_its_target(e: &Effect) -> bool {
    matches!(e, Effect::Untap { what: Selector::Target(0) | Selector::TargetFiltered { slot: 0, .. }, .. })
}

pub(super) fn pick_untap_ramp(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    if state.players.len() <= 2
        || state.active_player_idx != seat
        || !matches!(state.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
        || !state.stack.is_empty()
    {
        return None;
    }
    let mut tapped: Vec<CardId> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.tapped && c.definition.is_land())
        .map(|c| c.id)
        .collect();
    if tapped.is_empty() {
        return None;
    }
    tapped.sort();
    for src in state.battlefield.iter().filter(|c| c.controller == seat && !c.tapped) {
        for (idx, ab) in src.definition.activated_abilities.iter().enumerate() {
            if !ab.tap_cost || !untaps_its_target(&ab.effect) {
                continue;
            }
            for &land in &tapped {
                let action = GameAction::ActivateAbility {
                    card_id: src.id,
                    ability_index: idx,
                    target: Some(Target::Permanent(land)),
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                };
                if !state.would_accept(action.clone()) {
                    continue;
                }
                // The board once it resolves: the source tapped, the land not.
                let mut after = state.clone();
                if let Some(c) = after.battlefield_find_mut(src.id) {
                    c.tapped = true;
                }
                if let Some(c) = after.battlefield_find_mut(land) {
                    c.tapped = false;
                }
                let enables = cast_candidates(&after, seat, w, None)
                    .into_iter()
                    .any(|(a, _)| matches!(a, GameAction::CastSpell { .. }) && after.would_accept(a));
                return enables.then_some(action);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pod() -> GameState {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g
    }

    /// Arbor Elf untaps the tapped Forest when that pays for the Llanowar
    /// Elves in hand, and not when nothing in hand needs it, nor in a duel.
    #[test]
    fn arbor_elf_untaps_a_forest_for_a_cast() {
        let w = EvalWeights::default();
        let mut g = pod();
        let elf = g.add_card_to_battlefield(0, crate::catalog::arbor_elf());
        g.clear_sickness(elf);
        let forest = g.add_card_to_battlefield(0, crate::catalog::forest());
        g.battlefield_find_mut(forest).unwrap().tapped = true;
        assert!(pick_untap_ramp(&g, 0, &w).is_none(), "nothing in hand to cast");
        g.add_card_to_hand(0, crate::catalog::llanowar_elves());
        assert!(matches!(
            pick_untap_ramp(&g, 0, &w),
            Some(GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. })
                if card_id == elf && t == forest
        ));
        let mut duel = crate::game::two_player_game();
        duel.active_player_idx = 0;
        duel.step = TurnStep::PreCombatMain;
        let elf = duel.add_card_to_battlefield(0, crate::catalog::arbor_elf());
        duel.clear_sickness(elf);
        let forest = duel.add_card_to_battlefield(0, crate::catalog::forest());
        duel.battlefield_find_mut(forest).unwrap().tapped = true;
        duel.add_card_to_hand(0, crate::catalog::llanowar_elves());
        assert!(pick_untap_ramp(&duel, 0, &w).is_none(), "two seats: untouched");
    }
}
