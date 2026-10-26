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

/// `EvalWeights::hand_abilities`, duels: a nonland card's "{cost}, discard
/// this card: target creature gets -N/-M" (Proft, Sinister Mastermind's {B}:
/// -3/-1) aimed at the opponent's best creature it kills, while the card's
/// own cast condition (Proft's threshold) keeps it from being cast. Once the
/// card can be cast, casting it is the better use, so it is kept.
pub(super) fn pick_hand_ability(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    use crate::effect::{Effect, Value};
    use crate::game::types::Target;
    if !w.hand_abilities || state.players.len() != 2 || !state.stack.is_empty() {
        return None;
    }
    let opp = 1 - seat;
    for c in state.players[seat].hand.iter().filter(|c| !c.definition.is_land()) {
        let castable_later = c.definition.cast_condition.as_ref().is_none_or(|cond| {
            state.evaluate_predicate(cond, &crate::game::effects::EffectContext::for_trigger(c.id, seat, None, 0))
        });
        if castable_later {
            continue;
        }
        for (idx, ab) in c.definition.activated_abilities.iter().enumerate() {
            if !(ab.from_hand && ab.discard_self_cost) {
                continue;
            }
            let Effect::PumpPT { toughness: Value::Const(t), .. } = &ab.effect else { continue };
            if *t >= 0 {
                continue;
            }
            // The opponent's creature it kills with the most power + toughness.
            let victim = state
                .battlefield
                .iter()
                .filter(|p| p.controller == opp)
                .filter_map(|p| {
                    let cp = state.computed_permanent(p.id)?;
                    (state.computed_is_creature(p) && cp.toughness <= -*t)
                        .then_some((p.id, cp.power + cp.toughness))
                })
                .max_by_key(|&(_, size)| size);
            let Some((victim, _)) = victim else { continue };
            let action = GameAction::ActivateAbility {
                card_id: c.id,
                ability_index: idx,
                target: Some(Target::Permanent(victim)),
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

    /// `hand_abilities`: Proft, Sinister Mastermind below threshold kills an
    /// opposing X/1 from hand for {B}; at threshold it is kept to be cast.
    #[test]
    fn proft_below_threshold_kills_an_x1_from_hand() {
        let mut g = crate::game::two_player_game();
        g.active_player_idx = 0;
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        let proft = g.add_card_to_hand(0, crate::catalog::proft_sinister_mastermind());
        let elf = g.add_card_to_battlefield(1, crate::catalog::llanowar_elves());
        g.add_card_to_battlefield(1, crate::catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Black, 1);
        let on = EvalWeights { hand_abilities: true, ..EvalWeights::default() };
        assert_eq!(pick_hand_ability(&g, 0, &EvalWeights::default()), None, "flag off");
        assert!(matches!(
            pick_hand_ability(&g, 0, &on),
            Some(GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. }) if card_id == proft && t == elf
        ), "the 1/1 dies to -3/-1, the 2/2 doesn't");
        for _ in 0..7 {
            g.add_card_to_graveyard(0, crate::catalog::grizzly_bears());
        }
        assert_eq!(pick_hand_ability(&g, 0, &on), None, "threshold: cast it instead");
    }
}
