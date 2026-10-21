//! A modal one-sided sweep as an activated ability — Thanos, the Mad Titan's
//! power-up "choose odd or even, destroy each other creature whose mana value
//! is of the chosen quality" — was activated by no bot path (6-seat census,
//! seed 9407). The bot now fires one in its main phase when the better mode
//! takes more of its opponents' creatures (by mana value) than its own.
//! Commander games only, so two-player play is unchanged.

use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameAction, Target};

/// The destroy filter of each mode, when every mode of `e`'s `ChooseMode`
/// (top level or inside a `Seq`) is a board-wide destroy.
fn wipe_modes(e: &Effect) -> Option<Vec<&crate::card::SelectionRequirement>> {
    match e {
        Effect::Seq(steps) => steps.iter().find_map(wipe_modes),
        Effect::ChooseMode(modes) => modes
            .iter()
            .map(|m| match m {
                Effect::Destroy { what: Selector::EachPermanent(f) }
                | Effect::DestroyNoRegen { what: Selector::EachPermanent(f) } => Some(f),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}

/// The best mode's activation on one of `seat`'s permanents, when its net
/// mana value destroyed (opponents' minus `seat`'s) is positive.
pub(super) fn pick_modal_wipe(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || !state.stack.is_empty() {
        return None;
    }
    state.battlefield.iter().filter(|c| c.controller == seat).find_map(|src| {
        src.definition.activated_abilities.iter().enumerate().find_map(|(index, ab)| {
            let modes = wipe_modes(&ab.effect)?;
            let net = |f: &crate::card::SelectionRequirement| -> i32 {
                state
                    .battlefield
                    .iter()
                    .filter(|c| state.evaluate_requirement_static(f, &Target::Permanent(c.id), seat, Some(src.id)))
                    .map(|c| {
                        let mv = c.definition.cost.cmc() as i32 + 1;
                        if c.controller == seat { -mv } else { mv }
                    })
                    .sum()
            };
            let (mode, score) = modes.iter().enumerate().map(|(i, f)| (i, net(f))).max_by_key(|&(i, s)| (s, std::cmp::Reverse(i)))?;
            if score <= 0 {
                return None;
            }
            let action = GameAction::ActivateAbility {
                card_id: src.id,
                ability_index: index,
                target: None,
                additional_targets: Vec::new(),
                x_value: None,
                mode: Some(mode),
            };
            state.would_accept(action.clone()).then_some(action)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;

    /// Thanos's power-up picks the parity that costs the opponents more: two
    /// opposing two-drops (even) against the bot's own five-drop (odd).
    #[test]
    fn thanos_snaps_the_parity_that_hurts_the_table() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
        let thanos = g.add_card_to_battlefield(0, crate::catalog::thanos_the_mad_titan());
        g.add_card_to_battlefield(0, crate::catalog::serra_angel());
        g.add_card_to_battlefield(1, crate::catalog::grizzly_bears());
        g.add_card_to_battlefield(2, crate::catalog::grizzly_bears());
        for c in crate::mana::Color::ALL {
            g.players[0].mana_pool.add(c, 1);
        }
        g.players[0].mana_pool.add_colorless(1);
        let pick = pick_modal_wipe(&g, 0);
        assert!(
            matches!(pick, Some(GameAction::ActivateAbility { card_id, mode: Some(1), .. }) if card_id == thanos),
            "even: {pick:?}"
        );
        g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        assert!(pick_modal_wipe(&g, 0).is_none(), "a wipe that costs the bot as much isn't worth it");
    }
}
