//! Pods only: "search your library for a land and put it onto the
//! battlefield" activations — Sakura-Tribe Elder, Burnished Hart, Myriad
//! Landscape, Fertilid. The material eval prices a 1/1 above a tapped basic,
//! so the sacrifice pickers never took them (a 3/4/6-seat census: 10 to 25
//! decks each, never activated). A land is permanent mana; take the fetch in
//! the generic sink's windows unless it sacrifices a real threat.
//!
//! Two seats never reach it, so the two-player bench pool is untouched.

use crate::card::{CardInstance, SelectionRequirement as R};
use crate::effect::{ActivatedAbility, Effect, PlayerRef, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameAction, TurnStep};

/// Above this many lands another one isn't worth a card or a body.
const ENOUGH_LANDS: usize = 12;

/// The land filters of `effect`'s "search your library, put it onto your
/// battlefield" steps (one per `Search`); empty when it isn't a land fetch.
fn land_fetches(effect: &Effect) -> Vec<&R> {
    match effect {
        Effect::Search {
            who: PlayerRef::You,
            filter,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, .. },
        } if names_land(filter) => vec![filter],
        Effect::Seq(steps) => {
            let all: Vec<Vec<&R>> = steps.iter().map(land_fetches).collect();
            if all.iter().any(Vec::is_empty) { Vec::new() } else { all.into_iter().flatten().collect() }
        }
        _ => Vec::new(),
    }
}

fn names_land(r: &R) -> bool {
    match r {
        R::Land | R::IsBasicLand | R::HasLandType(_) => true,
        R::And(a, b) => names_land(a) || names_land(b),
        R::Or(a, b) => names_land(a) && names_land(b),
        _ => false,
    }
}

/// Would activating `ab` of `card` give up something worth keeping? Printed
/// power: an anthem pumps every body alike, so a pumped Burnished Hart is
/// still the ramp piece it was built as.
fn costs_a_threat(state: &GameState, card: &CardInstance, ab: &ActivatedAbility) -> bool {
    ab.sac_cost && state.computed_is_creature(card) && card.definition.power >= 3
}

pub(super) fn pick_land_ramp(state: &GameState, seat: usize) -> Option<GameAction> {
    let window = match state.step {
        TurnStep::PostCombatMain => state.active_player_idx == seat,
        TurnStep::End => {
            state.active_player_idx != seat && state.next_alive_seat(state.active_player_idx) == seat
        }
        _ => false,
    };
    if state.players.len() <= 2 || !state.stack.is_empty() || !window {
        return None;
    }
    let lands = state.battlefield.iter().filter(|c| c.controller == seat && c.definition.is_land()).count();
    if lands >= ENOUGH_LANDS {
        return None;
    }
    let library = &state.players[seat].library;
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            let fetches = land_fetches(&ab.effect);
            if fetches.is_empty()
                || ab.from_hand
                || ab.from_graveyard
                || (ab.tap_cost && card.tapped)
                || costs_a_threat(state, card, ab)
                || !fetches.iter().any(|f| library.iter().any(|c| state.evaluate_requirement_on_card(f, c, seat)))
            {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            if state.accept(action.clone()).is_some() {
                return Some(action);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn end_step_before_seat_0(g: &mut GameState) {
        g.active_player_idx = 2;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
    }

    /// Sakura-Tribe Elder is sacrificed for a basic in the end step before its
    /// controller's turn; a 2-player game never asks.
    #[test]
    fn sakura_tribe_elder_fetches_before_our_turn() {
        let mut g = crate::game::multi_player_game(3);
        let ste = g.add_card_to_battlefield(0, crate::catalog::sakura_tribe_elder());
        g.add_card_to_library(0, crate::catalog::forest());
        end_step_before_seat_0(&mut g);
        let got = pick_land_ramp(&g, 0);
        assert!(matches!(got, Some(GameAction::ActivateAbility { card_id, .. }) if card_id == ste), "got {got:?}");
        g.step = TurnStep::PreCombatMain;
        assert!(pick_land_ramp(&g, 0).is_none(), "not a window");
        let mut duel = crate::game::two_player_game();
        duel.add_card_to_battlefield(0, crate::catalog::sakura_tribe_elder());
        duel.add_card_to_library(0, crate::catalog::forest());
        duel.active_player_idx = 1;
        duel.step = TurnStep::End;
        assert!(pick_land_ramp(&duel, 0).is_none(), "two seats: untouched");
    }

    /// No land left to find, or a body worth keeping: no activation.
    #[test]
    fn no_fetch_without_a_land_or_at_the_cost_of_a_threat() {
        let mut g = crate::game::multi_player_game(3);
        g.add_card_to_battlefield(0, crate::catalog::sakura_tribe_elder());
        g.add_card_to_library(0, crate::catalog::grizzly_bears());
        end_step_before_seat_0(&mut g);
        assert!(pick_land_ramp(&g, 0).is_none(), "the library has no basic");
        g.add_card_to_library(0, crate::catalog::forest());
        let mut g2 = crate::game::multi_player_game(3);
        let big = crate::card::CardDefinition { power: 4, toughness: 4, ..crate::catalog::sakura_tribe_elder() };
        g2.add_card_to_battlefield(0, big);
        g2.add_card_to_library(0, crate::catalog::forest());
        end_step_before_seat_0(&mut g2);
        assert!(pick_land_ramp(&g2, 0).is_none(), "a printed 4/4 isn't fodder");
        assert!(pick_land_ramp(&g, 0).is_some(), "the 1/1 is");
    }
    /// Burnished Hart pays {3} from untapped lands for two basics.
    #[test]
    fn burnished_hart_pays_and_fetches() {
        let mut g = crate::game::multi_player_game(3);
        let hart = g.add_card_to_battlefield(0, crate::catalog::burnished_hart());
        for _ in 0..3 {
            g.add_card_to_battlefield(0, crate::catalog::forest());
        }
        g.add_card_to_library(0, crate::catalog::forest());
        g.add_card_to_library(0, crate::catalog::forest());
        end_step_before_seat_0(&mut g);
        let got = pick_land_ramp(&g, 0);
        assert!(matches!(got, Some(GameAction::ActivateAbility { card_id, .. }) if card_id == hart), "got {got:?}");
    }
}
