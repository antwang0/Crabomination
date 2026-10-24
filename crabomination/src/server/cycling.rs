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

/// CR 702.29e — a landcycle (`Landcycling` / `Typecycling`, FRA's "basic
/// landcycling {2}" on its six-drops) when `seat` holds no land, has fewer
/// than `ENOUGH_LANDS` out, and cannot cast the card next turn: in its own
/// main phase while the land drop is unused (the fetched land is played on
/// the next tick), else at an opponent's end step. No bot path cycled one:
/// 3,000 FRA sealed duels (2026-10-24) held Hexhaven Battalion in 22 % of the
/// games that drew it and never landcycled anything. Behind the
/// `duel_sinks` seat flag (or a commander, as the cycling above).
pub(super) fn pick_landcycle(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::card::Keyword;
    use crate::game::types::TurnStep;
    if !super::bot::pod_sinks_on(state, seat) || !state.stack.is_empty() {
        return None;
    }
    let p = &state.players[seat];
    let window = if state.active_player_idx == seat {
        matches!(state.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain) && p.can_play_land()
    } else {
        state.step == TurnStep::End
    };
    if !window || p.hand.iter().any(|c| c.definition.is_land()) || !p.library.iter().any(|c| c.definition.is_land()) {
        return None;
    }
    let lands = state.battlefield.iter().filter(|c| c.controller == seat && c.definition.is_land()).count();
    if lands >= ENOUGH_LANDS {
        return None;
    }
    // The costliest card first: it is the one furthest from being cast.
    let mut cards: Vec<(u32, crate::card::CardId, crate::mana::ManaCost)> = p
        .hand
        .iter()
        .filter(|c| c.definition.cost.cmc() as usize > lands + 1)
        .filter_map(|c| {
            let cost = c.definition.keywords.iter().find_map(|k| match k {
                Keyword::Landcycling(mc, _) => Some(mc.clone()),
                Keyword::Typecycling(spec) => Some(spec.0.clone()),
                _ => None,
            })?;
            Some((c.definition.cost.cmc(), c.id, cost))
        })
        .collect();
    cards.sort_by_key(|(cmc, id, _)| (std::cmp::Reverse(*cmc), *id));
    cards
        .into_iter()
        .map(|(_, id, cost)| (GameAction::Landcycle { card_id: id }, cost))
        .find(|(a, cost)| landcycle_accepted(state, seat, a, cost))
        .map(|(a, _)| a)
}

/// A landcycle the engine takes. `would_accept` refuses every one a promptable
/// seat makes with two kinds of basic left: the engine suspends it into the
/// library pick before the cost is paid (`ResumeContext::ActionSearchPick`),
/// and the bot-side guard reads any replay suspension as a cast it may not be
/// able to pay. So the probe may end in exactly that pick, with the cost shown
/// payable up front — the bot answers the search, the replay pays.
fn landcycle_accepted(state: &GameState, seat: usize, action: &GameAction, cost: &crate::mana::ManaCost) -> bool {
    if state.would_accept(action.clone()) {
        return true;
    }
    let mut probe = state.clone();
    probe.perform_action(action.clone()).is_ok()
        && probe
            .pending_decision
            .as_ref()
            .is_some_and(|pd| matches!(pd.decision, crate::decision::Decision::SearchLibrary { player, .. } if player == seat))
        && state.could_pay_cost(seat, cost)
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

    /// CR 702.29e — under `duel_sinks`, a duel bot with no land in hand and
    /// three out basic-landcycles Hexhaven Battalion in its main phase while
    /// the land drop is unused; with the flag off, or a land in hand, it
    /// keeps the card.
    #[test]
    fn a_duel_bot_landcycles_a_six_drop_when_short_of_lands() {
        let mut g = crate::game::two_player_game();
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let battalion = g.add_card_to_hand(0, crate::catalog::hexhaven_battalion());
        g.add_card_to_library(0, crate::catalog::plains());
        g.add_card_to_library(0, crate::catalog::plains());
        for _ in 0..3 {
            g.add_card_to_battlefield(0, crate::catalog::plains());
        }
        assert_eq!(pick_landcycle(&g, 0), None, "flag off");
        g.players[0].duel_sinks = true;
        assert_eq!(pick_landcycle(&g, 0), Some(GameAction::Landcycle { card_id: battalion }));
        // Two kinds of basic left: the engine asks which one first (a
        // promptable seat), and the landcycle is still offered.
        g.add_card_to_library(0, crate::catalog::forest());
        g.players[0].wants_ui = true;
        assert_eq!(pick_landcycle(&g, 0), Some(GameAction::Landcycle { card_id: battalion }), "a pick first");
        g.add_card_to_hand(0, crate::catalog::plains());
        assert_eq!(pick_landcycle(&g, 0), None, "a land in hand already");
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
