//! A card-selection mana sink — "scry N", "surveil N", "draw a card, then
//! discard a card" on the bot's own permanent (Castle Vantress, the Strixhaven
//! Campuses, Desolate Lighthouse, Nivix Guildmage) — never beat passing in
//! the outcome score, which can't see library order, so a 6-seat
//! `--card-census` (seed 206001) never activated one. The bot now spends
//! leftover mana on one at an opponent's end step, after cycling. Commander
//! games only, so two-player play is unchanged.

use crate::effect::{Effect, PlayerRef, Selector};
use crate::game::GameState;
use crate::game::types::GameAction;

/// Scry / surveil for yourself, or a loot (draw then discard, both yours).
fn is_selection(e: &Effect) -> bool {
    match e {
        Effect::Scry { who: PlayerRef::You, .. } | Effect::Surveil { who: PlayerRef::You, .. } => true,
        Effect::Seq(v) => matches!(
            v.as_slice(),
            [Effect::Draw { who: Selector::You, .. }, Effect::Discard { who: Selector::You, .. }]
        ),
        _ => false,
    }
}

/// The first accepted selection sink `seat` can activate, paid with mana
/// and at most a tap of its own source.
pub(super) fn pick_selection_sink(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() {
        return None;
    }
    state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat)
        .flat_map(|c| {
            c.definition.activated_abilities.iter().enumerate().filter_map(move |(i, ab)| {
                let costs_only_mana = !ab.sac_cost
                    && ab.sac_other_filter.is_none()
                    && ab.tap_other_filter.is_none()
                    && ab.discard_cost.is_none()
                    && ab.life_cost == 0
                    && ab.energy_cost == 0
                    && ab.remove_counter_x.is_none();
                (costs_only_mana && is_selection(&ab.effect)).then_some(GameAction::ActivateAbility {
                    card_id: c.id,
                    ability_index: i,
                    target: None,
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                })
            })
        })
        .find(|a| state.would_accept(a.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::TurnStep;
    use crate::mana::Color;

    /// Castle Vantress scries at an opponent's end step in a Commander game.
    #[test]
    fn castle_vantress_scries_with_leftover_mana() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 1;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        let castle = g.add_card_to_battlefield(0, crate::catalog::castle_vantress());
        g.players[0].mana_pool.add(Color::Blue, 4);
        assert!(pick_selection_sink(&g, 0).is_none(), "outside Commander");
        g.seat_commanders(0, vec![crate::catalog::llanowar_elves()]);
        assert!(matches!(pick_selection_sink(&g, 0), Some(GameAction::ActivateAbility { card_id, .. }) if card_id == castle));
    }
}
