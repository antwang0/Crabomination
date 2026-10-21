//! Myra the Magnificent's "{X}, {T}: exile target instant or sorcery card
//! with mana value X from your graveyard onto an Attraction without a midway
//! counter" pays off only on later visits, so the main-phase score never took
//! it: 40 Myra pods (seed 9460) never activated it. The bot now activates it
//! in its own main phase when it controls a midway-less Attraction, picking
//! the graveyard spell with the greatest mana value it can pay X for.
//! Commander games only, so two-player play is unchanged.

use crate::card::{ArtifactSubtype, CounterType};
use crate::effect::Effect;
use crate::game::GameState;
use crate::game::types::{GameAction, Target};

/// The best accepted midway activation for `seat`, in its own main phase with
/// an empty stack.
pub(super) fn pick_exile_onto_attraction(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || !state.stack.is_empty() || state.active_player_idx != seat {
        return None;
    }
    let open = state.battlefield.iter().any(|c| {
        c.controller == seat
            && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Attraction)
            && c.counter_count(CounterType::Midway) == 0
    });
    if !open {
        return None;
    }
    let mana = super::bot::mana_upper_bound(state, seat);
    for card in state.battlefield.iter().filter(|c| c.controller == seat && !c.tapped) {
        for (index, ab) in card.definition.activated_abilities.iter().enumerate() {
            if !matches!(ab.effect, Effect::ExileOntoAttractionWithMidway { .. }) {
                continue;
            }
            let budget = mana.saturating_sub(ab.mana_cost.cmc());
            let mut spells: Vec<_> = state.players[seat]
                .graveyard
                .iter()
                .filter(|c| (c.definition.is_instant() || c.definition.is_sorcery()) && c.definition.cost.cmc() <= budget)
                .map(|c| (c.definition.cost.cmc(), c.id))
                .collect();
            spells.sort_by_key(|s| std::cmp::Reverse(s.0));
            for (mv, id) in spells {
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: index,
                    target: Some(Target::Permanent(id)),
                    additional_targets: Vec::new(),
                    x_value: Some(mv),
                    mode: None,
                };
                if state.would_accept(action.clone()) {
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
    use crate::game::types::TurnStep;
    use crate::mana::Color;

    /// Myra stashes the costlier of two graveyard spells onto her Attraction,
    /// and only once one is open.
    #[test]
    fn myra_stashes_the_biggest_affordable_spell() {
        let mut g = crate::game::multi_player_game(4);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        let myra = g.add_card_to_battlefield(0, crate::catalog::myra_the_magnificent());
        g.clear_sickness(myra);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.add_card_to_graveyard(0, crate::catalog::opt());
        let div = g.add_card_to_graveyard(0, crate::catalog::divination());
        g.add_card_to_graveyard(0, crate::catalog::concentrate());
        g.players[0].mana_pool.add(Color::Blue, 3);
        assert!(pick_exile_onto_attraction(&g, 0).is_none(), "no Attraction yet");
        g.add_card_to_battlefield(0, crate::catalog::balloon_stand());
        let a = pick_exile_onto_attraction(&g, 0).expect("activate");
        assert!(matches!(a, GameAction::ActivateAbility { target: Some(Target::Permanent(t)), x_value: Some(3), .. } if t == div));
    }
}
