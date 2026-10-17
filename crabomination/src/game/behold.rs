//! CR 701.4 — behold: "choose a [quality] permanent you control or reveal a
//! [quality] card from your hand". Knowledge-only; nothing moves.

use super::GameState;
use crate::card::{CardId, CreatureType, SelectionRequirement};

impl GameState {
    /// Can `p` behold a `filter` object? `except` is the spell being cast,
    /// which can't reveal itself from hand.
    pub(crate) fn can_behold(&self, filter: &SelectionRequirement, p: usize, except: Option<CardId>) -> bool {
        self.battlefield.iter().any(|c| c.controller == p && self.evaluate_requirement_static_on(filter, c, p, None))
            || self.players.get(p).is_some_and(|pl| {
                pl.hand.iter().any(|c| Some(c.id) != except && self.evaluate_requirement_on_card(filter, c, p))
            })
    }

    /// The creature type `p` can behold the most of (`count` at least) among
    /// creatures they control and creature cards in hand (`except` aside);
    /// ties go to the first type met. A changeling counts toward every type.
    pub(crate) fn beheld_creature_type(&self, p: usize, except: Option<CardId>, count: u32) -> Option<CreatureType> {
        let mine: Vec<CardId> =
            self.battlefield.iter().filter(|c| c.controller == p && self.permanent_is_creature(c.id)).map(|c| c.id).collect();
        let hand: Vec<&crate::card::CardInstance> = self
            .players
            .get(p)?
            .hand
            .iter()
            .filter(|c| Some(c.id) != except && c.definition.is_creature())
            .collect();
        let mut seen: Vec<CreatureType> = Vec::new();
        for ct in mine
            .iter()
            .filter_map(|id| self.battlefield_find(*id))
            .chain(hand.iter().copied())
            .flat_map(|c| c.definition.subtypes.creature_types.iter().copied())
        {
            if !seen.contains(&ct) {
                seen.push(ct);
            }
        }
        let tally = |ct: CreatureType| {
            mine.iter().filter(|id| self.permanent_has_creature_type(**id, ct)).count()
                + hand.iter().filter(|c| self.card_off_battlefield_has_creature_type(c, ct)).count()
        };
        let mut best: Option<(CreatureType, usize)> = None;
        for ct in seen {
            let n = tally(ct);
            if n >= count as usize && best.is_none_or(|(_, b)| n > b) {
                best = Some((ct, n));
            }
        }
        best.map(|(ct, _)| ct)
    }
}
