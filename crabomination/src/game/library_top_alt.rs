//! CR 118.9 / 401.6 — an alternative cost paid for a spell cast off the top
//! of the library (One with the Multiverse's once-a-turn free cast "from your
//! hand or the top of your library"). The card hops into hand for the alt-cast
//! pipeline, tagged `HopFrom::LibraryTop`, exactly as `cast_spell` does for a
//! normal library-top cast, and goes back on top if the cast fails.

use super::GameState;
use super::actions::AltCastZone;
use super::types::{GameError, GameEvent, Target};
use crate::card::CardId;

impl GameState {
    /// `card_id` is the top card of `p`'s library and a library-top
    /// permission covers it.
    pub(crate) fn alt_cast_from_library_top(&self, p: usize, card_id: CardId) -> bool {
        !self.players[p].hand.iter().any(|c| c.id == card_id)
            && self.players[p].library.first().is_some_and(|c| {
                c.id == card_id && !self.cast_from_zone_blocked(p, &c.definition, crate::card::Zone::Library)
            })
            && self.library_top_playable(p, card_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn cast_alternative_from_library_top(
        &mut self,
        card_id: CardId,
        pitch_card: Option<CardId>,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let card = self.players[p].library.remove(0);
        self.players[p].hand.push(card);
        self.casting_hop = Some((card_id, crate::game::HopFrom::LibraryTop));
        let r = self.cast_spell_alternative_from(
            AltCastZone::Hand, card_id, pitch_card, target, additional_targets, mode, x_value,
        );
        self.casting_hop = None;
        if r.is_err()
            && let Some(card) = Self::take_card(&mut self.players[p].hand, card_id)
        {
            self.players[p].library.insert(0, card);
        }
        r
    }
}
