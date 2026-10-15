//! CR 702.190 — the bot's sneak window. Sneak is cast during the declare
//! blockers step by returning an unblocked attacker, so it shares ninjutsu's
//! window (`bot.rs::pick_ninjutsu`); without this a printed sneak card was
//! only ever hard-cast, and Ninja Teen's graveyard sneak never at all.

use crate::card::CardId;
use crate::game::GameState;
use crate::game::types::{GameAction, TurnStep};

/// A sneak cast worth making now: a hand card with printed sneak, or a
/// graveyard creature under a `GraveyardCreaturesHaveSneak` grant, at least
/// as strong as the weakest unblocked attacker it sends home. Biggest first;
/// the engine's `would_accept` checks timing, mana and the return cost.
pub(super) fn pick_sneak(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.step != TurnStep::DeclareBlockers || !state.blockers_declared() || !state.stack.is_empty() {
        return None;
    }
    let player = &state.players[seat];
    let from_hand = player
        .hand
        .iter()
        .filter(|c| c.definition.alternative_cost.as_ref().is_some_and(|a| a.sneak));
    let grant = state.graveyard_sneak_cost(seat).is_some();
    let from_graveyard = player
        .graveyard
        .iter()
        .filter(|c| grant && state.graveyard_sneak_alt(seat, c.id).is_some());
    let carriers: Vec<&crate::card::CardInstance> = from_hand.chain(from_graveyard).collect();
    if carriers.is_empty() {
        return None;
    }
    let power_of = |id: CardId| state.computed_permanent(id).map_or(0, |cp| cp.power);
    let give_up = state
        .attacking()
        .iter()
        .map(|a| a.attacker)
        .filter(|&id| {
            state.battlefield_find(id).is_some_and(|c| c.controller == seat)
                && state.blocker_count_of(id) == 0
                && !state.blocked_attackers.contains(&id)
        })
        .map(power_of)
        .min()?;
    let mut picks: Vec<(i32, CardId)> = carriers
        .into_iter()
        .filter(|c| !c.definition.effect.requires_target() && c.definition.power >= give_up)
        .map(|c| (-c.definition.power, c.id))
        .collect();
    picks.sort_unstable();
    picks
        .into_iter()
        .map(|(_, card_id)| GameAction::CastSpellAlternative {
            card_id,
            pitch_card: None,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .find(|a| state.would_accept(a.clone()))
}
