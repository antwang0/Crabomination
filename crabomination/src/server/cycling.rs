//! CR 702.29 — the bot's only cycling path. Nothing constructed
//! `GameAction::Cycle`, so cycling lands and granted cycling (Rhet-Tomb
//! Mystic) were dead cards in self-play. Commander pods only: a duel's bot
//! keeps its committed behaviour (`--bench` byte-identical), and a pod's long
//! games are where a spare land or an uncastable fatty is worth a new card.

use crate::game::GameState;
use crate::game::types::GameAction;

use super::bot::{EvalWeights, eval_material, evaluate_action_outcome};

/// Lands on the battlefield past which another land in hand is spare.
pub(super) const ENOUGH_LANDS: usize = 6;

/// A cycle worth making at an opponent's end step, or `None`: a card whose
/// own "when you cycle this" trigger settles above passing
/// ([`pick_cycle_trigger`]); else a land once `seat` controls six, or a
/// nonland card whose mana value is more than two above `seat`'s land count.
/// A free cycle (Gavi, New Perspectives) loosens both bars by two. The dry
/// run is the gate (cost, Stabilizer).
pub(super) fn pick_cycle(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    use crate::card::Keyword;
    if state.players[seat].commanders.is_empty() {
        return None;
    }
    if let Some(cycle) = pick_cycle_trigger(state, seat, w) {
        return Some(cycle);
    }
    let lands = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_land())
        .count();
    let slack = if state.cycling_is_free(seat) { 2 } else { 0 };
    state.players[seat]
        .hand
        .iter()
        .filter(|c| {
            c.definition.keywords.iter().any(|k| matches!(k, Keyword::Cycling(_) | Keyword::CyclingLife(_)))
                || state.granted_cycling_for(seat, c).is_some()
        })
        .filter(|c| {
            if c.definition.is_land() {
                lands + slack >= ENOUGH_LANDS
            } else {
                c.definition.cost.cmc() as usize + slack > lands + 2
            }
        })
        .map(|c| GameAction::Cycle { card_id: c.id, x_value: None })
        .find(|a| state.would_accept(a.clone()))
}

/// CR 702.29c — a card that triggers on its own cycling (Decree of Pain's
/// −2/−2, Krosan Tusker's land search, the Gempalms) cycled for its trigger:
/// the cycle is dry-run, the trigger settled, and the best outcome above
/// passing taken. The land-count bar never let one through — a 9,200-game
/// census of every target deck cycled no Decree of Pain in six decks.
fn pick_cycle_trigger(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    use crate::card::{EventKind, EventScope};
    let mut base: Option<i32> = None;
    let mut best: Option<(i32, GameAction)> = None;
    for c in state.players[seat].hand.iter() {
        let own_trigger = c.definition.triggered_abilities.iter().any(|t| {
            matches!(t.event.kind, EventKind::CardCycled) && matches!(t.event.scope, EventScope::SelfSource)
        });
        if !own_trigger {
            continue;
        }
        let action = GameAction::Cycle { card_id: c.id, x_value: None };
        let Some(settled) = state.accept(action.clone()) else { continue };
        let base = *base.get_or_insert_with(|| eval_material(state, seat, w));
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
    use crate::game::multi_player_game;
    use crate::game::types::TurnStep;
    use crate::mana::Color;

    /// A spare cycling land is cycled in a pod once six lands are out, and
    /// never in a game without commanders.
    #[test]
    fn a_pod_bot_cycles_a_spare_land() {
        let mut g = multi_player_game(3);
        g.active_player_idx = 1;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        let desert = g.add_card_to_hand(0, crate::catalog::desert_of_the_true());
        g.add_card_to_library(0, crate::catalog::plains());
        for _ in 0..5 {
            g.add_card_to_battlefield(0, crate::catalog::plains());
        }
        g.players[0].mana_pool.add(Color::White, 2);
        let w = EvalWeights::default();
        assert_eq!(pick_cycle(&g, 0, &w), None, "no commander: not a pod");
        let cmd = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.players[0].commanders.push(cmd);
        assert_eq!(pick_cycle(&g, 0, &w), None, "five lands: keep it");
        g.add_card_to_battlefield(0, crate::catalog::plains());
        assert_eq!(pick_cycle(&g, 0, &w), Some(GameAction::Cycle { card_id: desert, x_value: None }));
    }

    /// CR 702.29c — Decree of Pain is cycled at an opponent's end step for
    /// its −2/−2 when that kills an opponent's board; the land-count bar alone
    /// (eight lands out) would keep it.
    #[test]
    fn a_pod_bot_cycles_decree_of_pain_for_its_trigger() {
        let mut g = multi_player_game(3);
        g.active_player_idx = 1;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        let decree = g.add_card_to_hand(0, crate::catalog::decree_of_pain());
        g.add_card_to_library(0, crate::catalog::swamp());
        for _ in 0..8 {
            g.add_card_to_battlefield(0, crate::catalog::swamp());
        }
        let cmd = g.add_card_to_battlefield(0, crate::catalog::hill_giant());
        g.players[0].commanders.push(cmd);
        // Untapped Swamps, nothing floating: the cycle cost auto-taps.
        let w = EvalWeights::default();
        assert_eq!(pick_cycle(&g, 0, &w), None, "nothing of theirs dies");
        for _ in 0..3 {
            g.add_card_to_battlefield(1, crate::catalog::grizzly_bears());
        }
        assert_eq!(pick_cycle(&g, 0, &w), Some(GameAction::Cycle { card_id: decree, x_value: None }));
    }
}
