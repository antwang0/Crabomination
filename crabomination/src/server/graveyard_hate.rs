//! A repeatable "exile target card from a graveyard" (Ghost Vacuum's {T},
//! Scavenging Ooze's {G}) was never activated in a 6-seat census (seed
//! 2750001, three decks):
//! nothing asks for graveyard hate outside a removal read. At an opponent's
//! end step the tap and mana are spare, so a pod bot exiles an opponent's
//! best graveyard card: a creature card first (the reanimation target), then
//! the greatest mana value. Commander games only, so two-player play is
//! unchanged.

use crate::effect::{Effect, Selector, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameAction, Target};

/// Exiles one targeted card from ANY graveyard (not "your graveyard" — that
/// is fuel, not hate).
fn exiles_a_graveyard_card(e: &Effect) -> bool {
    match e {
        Effect::Move { what: Selector::TargetFiltered { slot: 0, filter }, to: ZoneDest::Exile | ZoneDest::ExileWithSourceStamp } => {
            let f = format!("{filter:?}");
            f.contains("InGraveyard") && !f.contains("InYourGraveyard")
        }
        // Scavenging Ooze's exile leads its counter / life rider.
        Effect::Seq(v) => v.first().is_some_and(exiles_a_graveyard_card),
        _ => false,
    }
}

/// The first accepted graveyard exile aimed at an opponent's card, at an
/// opponent's end step.
pub(super) fn pick_graveyard_hate(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || state.active_player_idx == seat || !state.stack.is_empty() {
        return None;
    }
    let abilities: Vec<(crate::card::CardId, usize)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat)
        .flat_map(|c| {
            c.definition.activated_abilities.iter().enumerate().filter_map(move |(i, ab)| {
                (super::end_step_ping::costs_only_mana(ab) && exiles_a_graveyard_card(&ab.effect)).then_some((c.id, i))
            })
        })
        .collect();
    if abilities.is_empty() {
        return None;
    }
    let mut cards: Vec<(bool, u32, crate::card::CardId)> = (0..state.players.len())
        .filter(|&q| !state.same_team(q, seat))
        .flat_map(|q| state.players[q].graveyard.iter())
        .map(|c| (c.definition.is_creature(), c.definition.cost.cmc(), c.id))
        .collect();
    cards.sort_by(|a, b| b.cmp(a));
    for (card_id, i) in abilities {
        for &(_, _, target) in cards.iter().take(4) {
            let action = GameAction::ActivateAbility {
                card_id,
                ability_index: i,
                target: Some(Target::Permanent(target)),
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
    use crate::game::types::TurnStep;

    /// Ghost Vacuum exiles the opponent's creature card at their end step,
    /// not on its controller's turn.
    #[test]
    fn ghost_vacuum_eats_the_reanimation_target() {
        let mut g = crate::game::multi_player_game(4);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        let vac = g.add_card_to_battlefield(0, crate::catalog::ghost_vacuum());
        g.add_card_to_graveyard(2, crate::catalog::lightning_bolt());
        let fatty = g.add_card_to_graveyard(2, crate::catalog::serra_angel());
        g.step = TurnStep::End;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        assert!(pick_graveyard_hate(&g, 0).is_none(), "own turn");
        g.active_player_idx = 1;
        let a = pick_graveyard_hate(&g, 0).expect("hate");
        assert!(matches!(a, GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. }
            if card_id == vac && t == fatty));
        g.perform_action(a).expect("activate");
        crate::game::drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == fatty));
    }
}
