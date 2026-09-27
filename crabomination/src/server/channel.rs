//! Channel (CR 207.2c, an ability word: "{cost}, Discard this card: …") — the Kamigawa channel
//! lands (Boseiju, Otawara, Eiganjo, Sokenzan, Takenuma) are Commander staples
//! no bot path activated: a 183-deck `--card-census` never channelled one. A
//! pod bot now spends a spare one (six lands out, the cycling bar) when the
//! dry run beats passing. Commander games only, so two-player play is unchanged.

use crate::game::GameState;
use crate::game::types::GameAction;

use super::bot::{EvalWeights, eval_material, evaluate_action_outcome};
use super::cycling::ENOUGH_LANDS;

/// The best-scoring channel activation of a spare land in `seat`'s hand, if
/// one beats doing nothing.
pub(super) fn pick_channel(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    if state.players[seat].commanders.is_empty() || !state.stack.is_empty() {
        return None;
    }
    let channels: Vec<(crate::card::CardId, usize)> = state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.is_land())
        .flat_map(|c| {
            c.definition
                .activated_abilities
                .iter()
                .enumerate()
                .filter(|(_, ab)| ab.from_hand && ab.discard_self_cost)
                .map(move |(i, _)| (c.id, i))
        })
        .collect();
    if channels.is_empty() {
        return None;
    }
    let lands = state.battlefield.iter().filter(|c| c.controller == seat && c.definition.is_land()).count();
    if lands < ENOUGH_LANDS {
        return None;
    }
    let mut best: Option<(i32, GameAction)> = None;
    for (card_id, idx) in channels {
        let ab = &state.players[seat].hand.iter().find(|c| c.id == card_id)?.definition.activated_abilities[idx];
        let (target, additional_targets) = if ab.effect.requires_target() {
            match state.auto_targets_for_effect_all_slots_x(&ab.effect, seat, None, false, Some(card_id), None) {
                (Some(t), extra) => (Some(t), extra),
                (None, _) => continue,
            }
        } else {
            (None, Vec::new())
        };
        let action =
            GameAction::ActivateAbility { card_id, ability_index: idx, target, additional_targets, x_value: None, mode: None };
        let Some(settled) = state.accept(action.clone()) else { continue };
        // The land is spare, so the bar is the board without it, not with it
        // in hand: the evaluator prices a card in hand above most answers.
        let mut spent = state.clone();
        spent.players[seat].hand.retain(|c| c.id != card_id);
        let base = eval_material(&spent, seat, w);
        if let Some(ev) = evaluate_action_outcome(state, seat, &action, Some(&settled), w)
            && ev > base
            && best.as_ref().is_none_or(|(b, _)| ev > *b)
        {
            best = Some((ev, action));
        }
    }
    best.map(|(_, a)| a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::{Target, TurnStep};
    use crate::mana::Color;

    /// A spare Boseiju is channelled at an opponent's artifact in a pod, and
    /// kept while the bot is short of lands or in a game without commanders.
    #[test]
    fn a_pod_bot_channels_a_spare_boseiju() {
        let mut g = crate::game::multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        let boseiju = g.add_card_to_hand(0, crate::catalog::boseiju_who_endures());
        let rock = g.add_card_to_battlefield(1, crate::catalog::sol_ring());
        for _ in 0..5 {
            g.add_card_to_battlefield(0, crate::catalog::forest());
        }
        g.players[0].mana_pool.add(Color::Green, 2);
        let w = EvalWeights::default();
        assert_eq!(pick_channel(&g, 0, &w), None, "no commander: not a pod");
        let cmd = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.players[0].commanders.push(cmd);
        assert_eq!(pick_channel(&g, 0, &w), None, "five lands: keep it as a land");
        g.add_card_to_battlefield(0, crate::catalog::forest());
        assert!(matches!(
            pick_channel(&g, 0, &w),
            Some(GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. })
                if card_id == boseiju && t == rock
        ));
    }
}
