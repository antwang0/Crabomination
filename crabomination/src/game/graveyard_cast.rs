//! "Until end of turn, you may cast [filter] spells from your / any graveyard"
//! as a rules permission for the turn (Gaea's Will, The Great Work III): each
//! grant in `TurnRegistries::graveyard_cast_eot` is stamped as a pay-own-cost
//! `may_play_until` onto every covered card, at resolution and again at each
//! SBA check, so a card put there later in the turn is castable too.

use super::GameState;

impl GameState {
    /// Stamp every live grant onto the covered graveyard cards that carry no
    /// permission yet. A card's one `may_play_until` slot keeps an earlier
    /// grant (another seat's, or a narrower one).
    pub(crate) fn stamp_graveyard_cast_grants(&mut self) {
        use crate::card::{MayPlayDuration, MayPlayPermission};
        let turn = self.turn_number;
        let grants = self.turn.graveyard_cast_eot.clone();
        for (seat, any_graveyard, filter, exile_after) in grants {
            for owner in 0..self.players.len() {
                if owner != seat && !any_graveyard {
                    continue;
                }
                let ids: Vec<crate::card::CardId> = self.players[owner]
                    .graveyard
                    .iter()
                    .filter(|c| c.may_play_until.is_none() && self.evaluate_requirement_on_card(&filter, c, seat))
                    .map(|c| c.id)
                    .collect();
                for id in ids {
                    let Some(card) = self.players[owner].graveyard.iter_mut().find(|c| c.id == id) else { continue };
                    card.may_play_until = Some(MayPlayPermission {
                        cast_only: false,
                        locks_further_casts: false,
                        one_cast_group: None,
                        player: seat,
                        granted_turn: turn,
                        duration: MayPlayDuration::EndOfThisTurn,
                        exile_after,
                        miracle: false,
                        pay_life: false,
                        bottom_after: false,
                        undaunted: false,
                    });
                    card.granted_alt_cast_cost_eot = Some(card.definition.cost.clone());
                }
            }
        }
    }
}
