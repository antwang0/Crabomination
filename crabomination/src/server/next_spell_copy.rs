//! A "{cost}: when you next cast an instant or sorcery spell this turn, copy
//! it …" activation (Zevlor, Elturel Exile's per-opponent copies) has no value
//! until a spell follows, so the main-phase score never took it: 100 Zevlor
//! pods (seed 9447) never activated it. The bot now arms the watcher first
//! when its hand holds a hostile targeted instant or sorcery it can still
//! afford after paying for the watcher. Commander games only, so two-player
//! play is unchanged.

use crate::effect::Effect;
use crate::game::GameState;
use crate::game::types::{DelayedKind, GameAction};

/// The first accepted watcher activation `seat` can follow with a spell this
/// turn, in its own main phase with an empty stack.
pub(super) fn pick_arm_next_spell_copy(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || !state.stack.is_empty() || state.active_player_idx != seat {
        return None;
    }
    let mana = super::bot::mana_upper_bound(state, seat);
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (index, ab) in card.definition.activated_abilities.iter().enumerate() {
            let Effect::OnYourNextSpellMatchingThisTurn { filter, .. } = &ab.effect else { continue };
            // A source-power cap (Loki's "mana value less than or equal to
            // Loki's power") is read off the hand card now; the rest of a
            // filter may name the spell's targets (Zevlor), which it has none of yet.
            let capped = filter.resolve_source_power(state.source_power_lki(card.id).unwrap_or(0));
            let capped = (&capped != filter).then_some(capped);
            let armed = state.delayed_triggers.iter().any(|dt| {
                dt.source == card.id && matches!(dt.kind, DelayedKind::YourNextSpellMatchingThisTurn(_))
            });
            if armed {
                continue;
            }
            let budget = mana.saturating_sub(ab.mana_cost.cmc());
            let follow_up = state.players[seat].hand.iter().any(|c| {
                let d = &c.definition;
                (d.is_instant() || d.is_sorcery())
                    && capped.as_ref().is_none_or(|f| state.evaluate_requirement_on_card(f, c, seat))
                    && d.effect.requires_target()
                    && !d.effect.prefers_friendly_target()
                    && d.cost.cmc() <= budget
            });
            if !follow_up {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: index,
                target: None,
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
    use crate::mana::Color;

    /// Zevlor arms his watcher with a Bolt in hand and mana for both, once.
    #[test]
    fn zevlor_arms_before_a_bolt() {
        let mut g = crate::game::multi_player_game(4);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        let zev = g.add_card_to_battlefield(0, crate::catalog::zevlor_elturel_exile());
        g.clear_sickness(zev);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(2);
        g.players[0].mana_pool.add(Color::Red, 1);
        assert!(pick_arm_next_spell_copy(&g, 0).is_none(), "no spell to follow");
        g.add_card_to_hand(0, crate::catalog::lightning_bolt());
        let a = pick_arm_next_spell_copy(&g, 0).expect("arm");
        g.perform_action(a).expect("activate");
        crate::game::drain_stack(&mut g);
        assert!(pick_arm_next_spell_copy(&g, 0).is_none(), "already armed (and tapped)");
    }

    /// Loki (power 2) doesn't arm for a hand whose only instant costs more
    /// than his power: the watch would never copy it.
    #[test]
    fn loki_arms_only_for_a_spell_his_power_covers() {
        let mut g = crate::game::multi_player_game(4);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        let loki = g.add_card_to_battlefield(0, crate::catalog::loki_laufeyson());
        g.clear_sickness(loki);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(6);
        g.players[0].mana_pool.add(Color::Red, 2);
        g.add_card_to_hand(0, crate::catalog::lava_axe());
        assert!(pick_arm_next_spell_copy(&g, 0).is_none(), "Lava Axe is above his power");
        g.add_card_to_hand(0, crate::catalog::lightning_bolt());
        assert!(pick_arm_next_spell_copy(&g, 0).is_some(), "Bolt is within it");
    }
}
