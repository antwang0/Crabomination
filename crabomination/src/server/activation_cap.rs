//! CR 104.4b — an optional activation loop the bot isn't winning ends with
//! the bot. Peregrin Took + Nuka-Cola Vending Machine: "sacrifice three Foods:
//! draw a card" pays for itself (each sacrificed Food mints a Treasure, and
//! Peregrin adds a Food to every token batch), so a pod seat activated it
//! until the board cap left the game undecided (seats 2006-2009 seed 41,
//! seats 2138-2143 seed 37). Past `MAX_VOLUNTARY_ACTIVATIONS_PER_TURN`
//! non-mana activations in one turn a Commander bot stops activating — the
//! activation twin of the twelve-combat attack brake. Mana abilities stay
//! free; two-player play is unchanged.
//! The same brake for casts: Inspired Skypainter's prepared Maestro's Gift
//! copying a Skypainter that enters prepared again, doubled by Twinning
//! Staff, cast copies until each action took seconds (six-seat pod seed
//! 1073858, decks 6054,2881,4375,3323,4191,834).

use crate::game::GameState;
use crate::game::types::GameAction;

/// Non-mana activations in one turn past which a Commander bot stops
/// activating voluntarily.
pub(super) const MAX_VOLUNTARY_ACTIVATIONS_PER_TURN: u32 = 40;

/// Spells cast in one turn past which a Commander bot stops casting
/// voluntarily.
pub(super) const MAX_VOLUNTARY_CASTS_PER_TURN: u32 = 40;

/// True when `action` is a cast `seat` (a Commander seat that has already
/// cast `MAX_VOLUNTARY_CASTS_PER_TURN` spells this turn) should not take.
pub(super) fn over_cast_cap(state: &GameState, seat: usize, action: &GameAction) -> bool {
    let player = &state.players[seat];
    action.is_cast()
        && !player.commanders.is_empty()
        && player.spells_cast_this_turn >= MAX_VOLUNTARY_CASTS_PER_TURN
}

/// True when `action` is a non-mana activation `seat` (a Commander seat that
/// has already spent its per-turn budget) should not take.
pub(super) fn over_activation_cap(state: &GameState, seat: usize, action: &GameAction) -> bool {
    let GameAction::ActivateAbility {
        card_id,
        ability_index,
        ..
    } = action
    else {
        return false;
    };
    let player = &state.players[seat];
    if player.commanders.is_empty()
        || player.nonmana_activations_this_turn < MAX_VOLUNTARY_ACTIVATIONS_PER_TURN
    {
        return false;
    }
    let printed = state.battlefield_find(*card_id).and_then(|c| {
        c.definition
            .activated_abilities
            .get(*ability_index)
            .map(|a| a.effect.clone())
    });
    !printed.is_some_and(|effect| crate::game::actions::is_mana_ability_public(&effect))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::multi_player_game;

    fn sol_ring_ish() -> crate::card::CardDefinition {
        use crate::effect::{Effect, ManaPayload, PlayerRef, Value};
        crate::card::CardDefinition {
            name: "Test Rock",
            card_types: vec![crate::card::CardType::Artifact],
            activated_abilities: vec![
                crate::effect::ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Colorless(Value::ONE),
                    },
                    ..Default::default()
                },
                crate::effect::ActivatedAbility {
                    effect: Effect::Draw {
                        who: crate::effect::Selector::You,
                        amount: Value::ONE,
                    },
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    fn activate(card_id: crate::card::CardId, ability_index: usize) -> GameAction {
        GameAction::ActivateAbility {
            card_id,
            ability_index,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        }
    }

    /// CR 104.4b — past the budget a Commander seat's non-mana activation is
    /// held, its mana ability is not, and a seat without a commander is never
    /// braked.
    #[test]
    fn cr_104_4b_the_activation_brake_holds_only_non_mana_commander_activations() {
        let mut g = multi_player_game(3);
        let rock = g.add_card_to_battlefield(0, sol_ring_ish());
        let draw = activate(rock, 1);
        assert!(
            !over_activation_cap(&g, 0, &draw),
            "no commander: never braked"
        );
        g.players[0].commanders.push(rock);
        assert!(!over_activation_cap(&g, 0, &draw), "under budget");
        g.players[0].nonmana_activations_this_turn = MAX_VOLUNTARY_ACTIVATIONS_PER_TURN;
        assert!(over_activation_cap(&g, 0, &draw));
        assert!(
            !over_activation_cap(&g, 0, &activate(rock, 0)),
            "mana abilities stay free"
        );
    }

    /// The tally counts non-mana activations and clears as a turn begins.
    #[test]
    fn cr_104_4b_the_activation_tally_counts_and_resets() {
        let mut g = multi_player_game(2);
        let rock = g.add_card_to_battlefield(0, sol_ring_ish());
        g.add_card_to_library(
            0,
            crate::card::CardDefinition {
                name: "Filler",
                ..Default::default()
            },
        );
        g.active_player_idx = 0;
        g.step = crate::game::types::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(activate(rock, 1)).expect("draw ability");
        assert_eq!(g.players[0].nonmana_activations_this_turn, 1);
        g.begin_turn_resets();
        assert_eq!(g.players[0].nonmana_activations_this_turn, 0);
    }

    /// CR 104.4b — the cast brake: past the budget a Commander seat's cast is
    /// held (a prepare-spell copy included); a seat without a commander never.
    #[test]
    fn cr_104_4b_the_cast_brake_holds_only_commander_casts() {
        let mut g = multi_player_game(3);
        let rock = g.add_card_to_battlefield(0, sol_ring_ish());
        let cast = GameAction::CastPrepareSpell {
            creature_id: rock,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        g.players[0].spells_cast_this_turn = MAX_VOLUNTARY_CASTS_PER_TURN;
        assert!(!over_cast_cap(&g, 0, &cast), "no commander: never braked");
        g.players[0].commanders.push(rock);
        assert!(over_cast_cap(&g, 0, &cast));
        assert!(!over_cast_cap(&g, 0, &activate(rock, 1)), "an activation is the other brake's");
        g.players[0].spells_cast_this_turn = MAX_VOLUNTARY_CASTS_PER_TURN - 1;
        assert!(!over_cast_cap(&g, 0, &cast), "under budget");
    }
}
