//! "Reveal cards from the top of your library until you reveal N [kind]
//! cards" — the shared walk behind the reveal-until-to-battlefield effects.

use super::EffectContext;
use crate::card::{CardInstance, SelectionRequirement};
use crate::effect::{PlayerRef, Value, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// Takes cards off the top of `seat`'s library until `need` of them pass
    /// `hit`, or the library runs out. Returns `(hits, rest)` in reveal order.
    pub(super) fn reveal_until_n(
        &mut self,
        seat: usize,
        need: u32,
        mut hit: impl FnMut(&Self, &CardInstance) -> bool,
    ) -> (Vec<CardInstance>, Vec<CardInstance>) {
        let (mut hits, mut rest) = (Vec::new(), Vec::new());
        while (hits.len() as u32) < need && !self.players[seat].library.is_empty() {
            let card = self.players[seat].library.remove(0);
            if hit(self, &card) { hits.push(card) } else { rest.push(card) }
        }
        (hits, rest)
    }

    /// Synthetic Destiny's end-step half: every revealed `filter` card onto
    /// the battlefield under the controller, the rest shuffled back in.
    pub(super) fn reveal_until_matching_to_battlefield(
        &mut self,
        filter: &SelectionRequirement,
        count: &Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let p = ctx.controller;
        let need = self.evaluate_value(count, ctx).max(0) as u32;
        if need == 0 {
            return Ok(());
        }
        let (hits, rest) =
            self.reveal_until_n(p, need, |g, c| g.evaluate_requirement_on_card(filter, c, p));
        self.players[p].library.extend(rest);
        let dest = ZoneDest::Battlefield { controller: PlayerRef::Seat(p), tapped: false };
        for card in hits {
            self.place_card_in_dest(card, p, &dest, events);
        }
        self.shuffle_library(p, events);
        self.check_state_based_actions_into(events);
        Ok(())
    }
}
