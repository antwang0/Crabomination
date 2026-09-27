//! CR 702.140 — no bot built `GameAction::CastMutate`, so the Otrimi seat's
//! mutate creatures only ever entered on their own. The bot now offers one
//! mutate cast per mutate card in hand (and a mutate commander in the command
//! zone, CR 903.8) onto its best host; `score_candidate` prices it against
//! the plain cast. Commander games only, so two-player play is unchanged.

use crate::card::{CardInstance, CreatureType};
use crate::game::GameState;
use crate::game::types::GameAction;

/// A mutate cast for each mutate card `seat` holds, unvalidated (the pick
/// site probes in score order): onto the host
/// that can already attack, then the biggest; over it when the incoming
/// creature is the bigger body (CR 702.140c — the top card's characteristics).
pub(super) fn mutate_candidates(state: &GameState, seat: usize) -> Vec<GameAction> {
    let p = &state.players[seat];
    if p.commanders.is_empty() {
        return Vec::new();
    }
    let size = |c: &CardInstance| c.definition.power + c.definition.toughness;
    let host = state
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == seat
                && c.owner == seat
                && c.definition.is_creature()
                && !c.definition.has_creature_type(CreatureType::Human)
        })
        .max_by_key(|c| (!c.summoning_sick, size(c), std::cmp::Reverse(c.id)));
    let Some(host) = host else { return Vec::new() };
    p.hand
        .iter()
        .chain(p.command.iter().filter(|c| p.commanders.contains(&c.id)))
        .filter(|c| c.definition.mutate.is_some())
        .map(|c| GameAction::CastMutate {
            card_id: c.id,
            target: host.id,
            on_top: size(c) >= size(host),
            x_value: None,
        })
        .collect()
}
