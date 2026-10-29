//! Pods only: "untap target [mana source]" — Arbor Elf's Forest, Kiora's
//! Follower, Aphetto Alchemist, Thousand-Year Elixir on a mana creature,
//! Clock of Omens or Magewright's Stone on a rock, Minamo on a legendary land
//! — on a tapped mana source of the seat's: that source's mana again. The eval
//! prices the untap at nothing, so a census never saw one activated. Taken in
//! the seat's own main phase once it has nothing castable, and only when the
//! resolved untap, its costs paid, makes a hand card castable.
//!
//! Two seats never reach it, so the two-player bench pool is untouched.

use crate::card::CardId;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::actions::is_mana_ability_public;
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
    // Tapped lands, then tapped nonland `{T}` mana sources (rocks, dorks).
    let mut tapped: Vec<(bool, CardId)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.tapped)
        .filter(|c| {
            c.definition.is_land()
                || c.definition.activated_abilities.iter().any(|ab| ab.tap_cost && is_mana_ability_public(&ab.effect))
        })
        .map(|c| (!c.definition.is_land(), c.id))
        .collect();
    if tapped.is_empty() {
        return None;
    }
    tapped.sort();
    for src in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in src.definition.activated_abilities.iter().enumerate() {
            if !untaps_its_target(&ab.effect) || (ab.tap_cost && src.tapped) {
                continue;
            }
            // One probe per printed card: five tapped Forests answer alike.
            let mut tried: Vec<&str> = Vec::new();
            for &(_, mana) in &tapped {
                let Some(name) = state.battlefield_find(mana).map(|c| c.definition.name) else { continue };
                if mana == src.id || tried.contains(&name) {
                    continue;
                }
                tried.push(name);
                let action = GameAction::ActivateAbility {
                    card_id: src.id,
                    ability_index: idx,
                    target: Some(Target::Permanent(mana)),
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                };
                // The board once it resolves, its costs paid.
                let Some(mut after) = state.accept(action.clone()) else { continue };
                if after.resolve_top_of_stack().is_err() || after.battlefield_find(mana).is_none_or(|c| c.tapped) {
                    continue;
                }
                let enables = cast_candidates(&after, seat, w, None)
                    .into_iter()
                    .any(|(a, _)| matches!(a, GameAction::CastSpell { .. }) && after.would_accept(a));
                if enables {
                    return Some(action);
                }
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

    /// A nonland source and a non-`{T}` untapper: Clock of Omens taps two
    /// Ornithopters to untap the tapped Sol Ring, once that pays for a card.
    #[test]
    fn clock_of_omens_untaps_a_sol_ring_for_a_cast() {
        let w = EvalWeights::default();
        let mut g = pod();
        g.add_card_to_battlefield(0, crate::catalog::clock_of_omens());
        g.add_card_to_battlefield(0, crate::catalog::ornithopter());
        g.add_card_to_battlefield(0, crate::catalog::ornithopter());
        let ring = g.add_card_to_battlefield(0, crate::catalog::sol_ring());
        g.battlefield_find_mut(ring).unwrap().tapped = true;
        assert!(pick_untap_ramp(&g, 0, &w).is_none(), "nothing in hand to cast");
        g.add_card_to_hand(0, crate::catalog::mind_stone());
        assert!(matches!(
            pick_untap_ramp(&g, 0, &w),
            Some(GameAction::ActivateAbility { target: Some(Target::Permanent(t)), .. }) if t == ring
        ));
    }
}
