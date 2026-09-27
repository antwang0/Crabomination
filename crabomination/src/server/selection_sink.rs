//! A card-selection mana sink — "scry N", "surveil N", "draw a card, then
//! discard a card" on the bot's own permanent (Castle Vantress, the Strixhaven
//! Campuses, Desolate Lighthouse, Nivix Guildmage, Sensei's Divining Top's
//! rearrange, Mazemind Tome's page-counter scry) — never beat passing in
//! the outcome score, which can't see library order, so a 6-seat
//! `--card-census` (seed 206001) never activated one. The bot now spends
//! leftover mana on one at an opponent's end step, after cycling. Commander
//! games only, so two-player play is unchanged.
//!
//! A mana rock's "sacrifice this: draw N" (the Cluestones, Lockets,
//! Dreamstone Hedron, Mnemonic Sphere) scored below keeping the rock, so a
//! 6-seat census (seed 1270001) never cashed one in. Once the seat has six
//! lands, an opponent's end step with the mana spare now cashes it in.

use crate::effect::{Effect, PlayerRef, Selector};
use crate::game::GameState;
use crate::game::types::GameAction;

/// Scry / surveil / rearrange for yourself, or a loot (draw then discard,
/// both yours). A sequence led by a scry counts (Mazemind Tome's scry, then
/// its page-count check).
fn is_selection(e: &Effect) -> bool {
    match e {
        Effect::Scry { who: PlayerRef::You, .. }
        | Effect::Surveil { who: PlayerRef::You, .. }
        | Effect::RearrangeTop { who: PlayerRef::You, .. } => true,
        Effect::Seq(v) => {
            matches!(
                v.as_slice(),
                [Effect::Draw { who: Selector::You, .. }, Effect::Discard { who: Selector::You, .. }]
            ) || matches!(v.first(), Some(Effect::Scry { who: PlayerRef::You, .. }))
        }
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
                // An untapped rearrange (Sensei's Divining Top) would repeat
                // for every spare mana: once a turn, read off the seat's
                // "activated an artifact's ability this turn".
                let repeats = !ab.tap_cost
                    && matches!(ab.effect, Effect::RearrangeTop { .. })
                    && state.players[seat].artifact_ability_activated_this_turn;
                (costs_only_mana && !repeats && is_selection(&ab.effect)).then_some(GameAction::ActivateAbility {
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

/// Lands `seat` controls past which a rock's mana is worth less than its cards.
const CASH_IN_LANDS: usize = 6;

/// A "{mana}, [{T},] sacrifice this: draw N" on `seat`'s non-creature
/// artifact, at an opponent's end step once the seat has `CASH_IN_LANDS`.
pub(super) fn pick_cash_in(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || state.active_player_idx == seat || !state.stack.is_empty() {
        return None;
    }
    let lands = state.battlefield.iter().filter(|c| c.controller == seat && c.definition.is_land()).count();
    if lands < CASH_IN_LANDS {
        return None;
    }
    state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_artifact() && !c.definition.is_creature())
        .flat_map(|c| {
            c.definition.activated_abilities.iter().enumerate().filter_map(move |(i, ab)| {
                let draws = matches!(ab.effect, Effect::Draw { who: Selector::You, .. });
                let only_mana_tap_sac = ab.sac_cost
                    && ab.sac_other_filter.is_none()
                    && ab.tap_other_filter.is_none()
                    && ab.discard_cost.is_none()
                    && ab.life_cost == 0
                    && ab.energy_cost == 0
                    && ab.remove_counter_cost.is_none()
                    && ab.remove_all_counters_cost.is_none()
                    && ab.remove_counter_among_filter.is_none()
                    && ab.exile_other_filter.is_none()
                    && !ab.mana_cost.has_x();
                (draws && only_mana_tap_sac).then_some(GameAction::ActivateAbility {
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

    /// Dreamstone Hedron is cashed in at an opponent's end step with six
    /// lands and the mana spare, not with five.
    #[test]
    fn dreamstone_hedron_cashes_in_late() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        g.active_player_idx = 1;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        let hedron = g.add_card_to_battlefield(0, crate::catalog::dreamstone_hedron());
        g.clear_sickness(hedron);
        for _ in 0..5 {
            let l = g.add_card_to_battlefield(0, crate::catalog::forest());
            g.clear_sickness(l);
        }
        for _ in 0..5 {
            g.add_card_to_library(0, crate::catalog::grizzly_bears());
        }
        assert!(pick_cash_in(&g, 0).is_none(), "five lands");
        let l = g.add_card_to_battlefield(0, crate::catalog::forest());
        g.clear_sickness(l);
        let a = pick_cash_in(&g, 0).expect("cash in");
        g.perform_action(a).expect("activate");
        crate::game::drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), 3);
        assert!(g.battlefield_find(hedron).is_none());
    }

    /// Sensei's Divining Top rearranges once at an opponent's end step, not
    /// once per spare mana.
    #[test]
    fn senseis_top_rearranges_once_a_turn() {
        let mut g = crate::game::multi_player_game(3);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        g.active_player_idx = 1;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        let top = g.add_card_to_battlefield(0, crate::catalog::senseis_divining_top());
        for _ in 0..5 {
            g.add_card_to_library(0, crate::catalog::grizzly_bears());
        }
        g.players[0].mana_pool.add_colorless(3);
        let a = pick_selection_sink(&g, 0).expect("rearrange");
        assert!(matches!(a, GameAction::ActivateAbility { card_id, ability_index: 0, .. } if card_id == top));
        g.perform_action(a).expect("activate");
        crate::game::drain_stack(&mut g);
        g.priority.player_with_priority = 0;
        assert!(pick_selection_sink(&g, 0).is_none(), "once a turn");
    }
}
