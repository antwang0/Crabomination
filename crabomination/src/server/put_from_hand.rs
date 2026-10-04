//! Pods only: "put a creature card from your hand onto the battlefield"
//! activations — Sneak Attack, Elvish Piper, Quicksilver Amulet. A 183-deck
//! census never saw one activated: the generic sink asks only after combat
//! and reads Sneak Attack's sacrifice-at-end-step as a temporary gain, so the
//! haste swing it exists for was never considered.
//!
//! A permanent put (Elvish Piper) deploys the costliest matching creature
//! the cast path left in hand, in either main phase. A haste-and-sacrifice
//! put (Sneak Attack) is taken before combat only, for a creature with
//! enough power to be worth a one-turn swing.
//!
//! Two seats never reach it, so the two-player bench pool is untouched.

use crate::effect::{Effect, PlayerRef};
use crate::game::GameState;
use crate::game::types::{GameAction, TurnStep};

/// The least power a one-turn (sacrificed or returned) creature must bring.
const MIN_SWING_POWER: i32 = 4;

pub(super) fn pick_put_from_hand(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players.len() <= 2 || !state.stack.is_empty() || state.active_player_idx != seat {
        return None;
    }
    let precombat = match state.step {
        TurnStep::PreCombatMain => true,
        TurnStep::PostCombatMain => false,
        _ => return None,
    };
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            let Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter,
                haste,
                sacrifice_eot,
                return_eot,
                ..
            } = &ab.effect
            else {
                continue;
            };
            if ab.tap_cost && card.tapped {
                continue;
            }
            let temporary = *sacrifice_eot || *return_eot;
            if temporary && !(*haste && precombat) {
                continue;
            }
            // The filter the resolver will read (Aether Vial's counter-sized
            // mana value, a chosen creature type), so a pick that would put
            // nothing is never paid for — Sneak Attack has no tap to stop a
            // second, empty activation.
            let filter = filter
                .resolve_source_counters(&|k| card.counter_count(k))
                .resolve_x(0)
                .resolve_chosen_creature_type(card.chosen_creature_type);
            let best = state.players[seat]
                .hand
                .iter()
                .filter(|c| c.definition.is_creature())
                .filter(|c| state.evaluate_requirement_on_card(&filter, c, seat))
                .max_by_key(|c| (c.definition.cost.cmc(), c.definition.power));
            let Some(best) = best else { continue };
            let worth = if temporary {
                best.definition.power >= MIN_SWING_POWER
            } else {
                best.definition.cost.cmc() > ab.mana_cost.cmc() + 1
            };
            if !worth {
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

    fn pod_main(step: TurnStep) -> GameState {
        let mut g = crate::game::multi_player_game(4);
        g.active_player_idx = 0;
        g.step = step;
        g.priority.player_with_priority = 0;
        g
    }

    /// Sneak Attack puts a Shivan Dragon in for a hasty swing before combat,
    /// not after it, and not a 2/2.
    #[test]
    fn sneak_attack_cheats_a_big_creature_in_before_combat() {
        let mut g = pod_main(TurnStep::PreCombatMain);
        let sneak = g.add_card_to_battlefield(0, crate::catalog::sneak_attack());
        g.add_card_to_hand(0, crate::catalog::grizzly_bears());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        assert!(pick_put_from_hand(&g, 0).is_none(), "a 2/2 isn't worth the sacrifice");
        g.add_card_to_hand(0, crate::catalog::shivan_dragon());
        assert!(matches!(
            pick_put_from_hand(&g, 0),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == sneak
        ));
        g.step = TurnStep::PostCombatMain;
        assert!(pick_put_from_hand(&g, 0).is_none(), "no swing left after combat");
    }

    /// Elvish Piper deploys a creature it can't cheaply cast, in either main.
    #[test]
    fn elvish_piper_deploys_the_costliest_creature() {
        let mut g = pod_main(TurnStep::PostCombatMain);
        let piper = g.add_card_to_battlefield(0, crate::catalog::elvish_piper());
        g.clear_sickness(piper);
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.add_card_to_hand(0, crate::catalog::craw_wurm());
        assert!(matches!(
            pick_put_from_hand(&g, 0),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == piper
        ));
        // Two seats: the bench pool never asks.
        let mut duel = crate::game::two_player_game();
        duel.active_player_idx = 0;
        duel.step = TurnStep::PostCombatMain;
        let piper = duel.add_card_to_battlefield(0, crate::catalog::elvish_piper());
        duel.clear_sickness(piper);
        duel.add_card_to_hand(0, crate::catalog::craw_wurm());
        assert!(pick_put_from_hand(&duel, 0).is_none());
    }
}
