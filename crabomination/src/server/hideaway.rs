//! Pods only: a hideaway land's "{color}, {T}: you may play the exiled card
//! without paying its mana cost if [condition]" (Mosswort Bridge, Spinerock
//! Knoll, Windbrisk Heights, Shelldock Isle). A 183-deck census (6 seats,
//! seed 290000, 30 games a group) never saw one activated — Mosswort Bridge
//! in eleven decks: the sink chain asked only once nothing was castable, by
//! when the land had been tapped for mana. A free spell is taken in the
//! seat's own main phase, ahead of its casts, as soon as the condition holds
//! (Windbrisk's and Spinerock's hold after combat). A hidden land is left:
//! its play needs the turn's land drop.
//!
//! Two seats never reach it, so the two-player bench pool is untouched.

use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameAction, TurnStep};

/// Whether `e` casts the card exiled with its source for free.
fn casts_the_hidden_card(e: &Effect) -> bool {
    match e {
        Effect::CastWithoutPayingImmediate { what: Selector::CardExiledWithSource, .. } => true,
        Effect::Seq(steps) => steps.iter().any(casts_the_hidden_card),
        _ => false,
    }
}

pub(super) fn pick_hideaway_play(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players.len() <= 2
        || state.active_player_idx != seat
        || !matches!(state.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
        || !state.stack.is_empty()
    {
        return None;
    }
    // One pass over exile (short) rather than one per permanent: this runs
    // ahead of every main-phase cast decision a pod seat makes.
    state.exile.iter().filter(|e| !e.definition.is_land()).find_map(|hidden| {
        let c = state.battlefield_find(hidden.exiled_with?)?;
        if c.controller != seat || c.tapped {
            return None;
        }
        c.definition.activated_abilities.iter().enumerate().find_map(|(i, ab)| {
            if !casts_the_hidden_card(&ab.effect) {
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

    /// Mosswort Bridge plays its hidden Craw Wurm once creatures it sees have
    /// 10 power, and not before.
    #[test]
    fn mosswort_bridge_plays_its_hidden_card_at_ten_power() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bridge = g.add_card_to_battlefield(0, crate::catalog::mosswort_bridge());
        g.add_card_to_battlefield(0, crate::catalog::forest());
        let hidden = g.add_card_to_exile(0, crate::catalog::craw_wurm());
        g.exile.iter_mut().find(|c| c.id == hidden).unwrap().exiled_with = Some(bridge);
        g.add_card_to_battlefield(0, crate::catalog::craw_wurm());
        assert!(pick_hideaway_play(&g, 0).is_none(), "6 power isn't 10");
        g.add_card_to_battlefield(0, crate::catalog::craw_wurm());
        assert!(matches!(
            pick_hideaway_play(&g, 0),
            Some(GameAction::ActivateAbility { card_id, ability_index: 1, .. }) if card_id == bridge
        ));
    }
}
