//! A self-sacrificing mana rock that replaces itself — Chromatic Star's "when
//! this is put into a graveyard, draw", Chromatic Sphere's draw beside the
//! mana, Wizard's Rockets at X = 0 — sat on pod boards all game (census, seed
//! 29100): the auto-tapper spends a self-consuming source last, so it was
//! never needed. At an opponent's end step with its cost spare, a pod bot
//! cracks it for the card. Commander games only, so two-player play is
//! unchanged.

use crate::card::{EventKind, EventScope};
use crate::effect::{ActivatedAbility, Effect};
use crate::game::GameState;
use crate::game::types::GameAction;

fn draws(e: &Effect) -> bool {
    match e {
        Effect::Draw { .. } => true,
        Effect::Seq(v) => v.iter().any(draws),
        _ => false,
    }
}

fn adds_mana(e: &Effect) -> bool {
    match e {
        Effect::AddMana { .. } => true,
        Effect::Seq(v) => v.iter().any(adds_mana),
        _ => false,
    }
}

/// A mana-making ability that sacrifices its own source and draws, in its
/// body (Chromatic Sphere — a stack ability, CR 605.1a) or through the
/// source's own "put into a graveyard" trigger.
fn cantrip_sac(def: &crate::card::CardDefinition, ab: &ActivatedAbility) -> bool {
    ab.sac_cost
        && adds_mana(&ab.effect)
        && (draws(&ab.effect)
            || def.triggered_abilities.iter().any(|t| {
                matches!(t.event.kind, EventKind::PermanentDied | EventKind::PutIntoGraveyard)
                    && t.event.scope == EventScope::SelfSource
                    && draws(&t.effect)
            }))
}

/// The first accepted crack at an opponent's end step (X = 0 for an {X} cost).
pub(super) fn pick_cantrip_rock(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || state.active_player_idx == seat || !state.stack.is_empty() {
        return None;
    }
    for c in state.battlefield.iter().filter(|c| c.controller == seat && !c.tapped) {
        for (i, ab) in c.definition.activated_abilities.iter().enumerate() {
            if !cantrip_sac(&c.definition, ab) {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: c.id,
                ability_index: i,
                target: None,
                additional_targets: Vec::new(),
                x_value: ab.mana_cost.has_x().then_some(0),
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

    /// Chromatic Star (draw on death), Chromatic Sphere (draw in the body) and
    /// Wizard's Rockets (X = 0) are cracked at an opponent's end step with a
    /// land spare; a two-player seat without a commander keeps them.
    #[test]
    fn cantrip_rocks_are_cracked_at_an_opponents_end_step() {
        for def in [crate::catalog::chromatic_star(), crate::catalog::chromatic_sphere(), crate::catalog::wizards_rockets()] {
            let name = def.name;
            let mut g = crate::game::multi_player_game(3);
            let rock = g.add_card_to_battlefield(0, def);
            g.battlefield_find_mut(rock).unwrap().tapped = false;
            g.add_card_to_battlefield(0, crate::catalog::wastes());
            g.add_card_to_library(0, crate::catalog::forest());
            g.step = TurnStep::End;
            g.active_player_idx = 1;
            g.priority.player_with_priority = 0;
            assert!(pick_cantrip_rock(&g, 0).is_none(), "{name}: no commander, no crack");
            g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
            g.active_player_idx = 0;
            assert!(pick_cantrip_rock(&g, 0).is_none(), "{name}: own turn");
            g.active_player_idx = 1;
            let hand = g.players[0].hand.len();
            let a = pick_cantrip_rock(&g, 0).unwrap_or_else(|| panic!("{name}: crack"));
            g.perform_action(a).expect("activate");
            crate::game::drain_stack(&mut g);
            assert!(g.battlefield_find(rock).is_none(), "{name}: sacrificed");
            assert_eq!(g.players[0].hand.len(), hand + 1, "{name}: drew");
        }
    }
}
