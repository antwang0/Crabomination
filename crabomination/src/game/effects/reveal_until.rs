//! "Reveal cards from the top of your library until you reveal N [kind]
//! cards" — the shared walk behind the reveal-until-to-battlefield effects.

use super::EffectContext;
use crate::card::{CardInstance, SelectionRequirement};
use crate::effect::{PlayerRef, Selector, Value, ZoneDest};
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
        rest_bottom: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        use rand::seq::SliceRandom;
        let p = ctx.controller;
        let need = self.evaluate_value(count, ctx).max(0) as u32;
        if need == 0 {
            return Ok(());
        }
        let (hits, mut rest) =
            self.reveal_until_n(p, need, |g, c| g.evaluate_requirement_on_card(filter, c, p));
        if rest_bottom {
            rest.shuffle(&mut self.rng.draw());
        }
        self.players[p].library.extend(rest);
        let dest = ZoneDest::Battlefield { controller: PlayerRef::Seat(p), tapped: false };
        let placed: Vec<_> = hits.iter().map(|c| c.id).collect();
        for card in hits {
            self.place_card_in_dest(card, p, &dest, events);
        }
        // `Selector::LastMoved` names the cards put onto the battlefield (Dack
        // Fayden goads and hands them out). Guarded like the sibling below:
        // the scratch is a CoW group.
        if !placed.is_empty() {
            self.scratch.last_moved_cards = placed;
        }
        if !rest_bottom {
            self.shuffle_library(p, events);
        }
        self.check_state_based_actions_mid_resolution(events);
        Ok(())
    }

    /// Audacious Reshapers: reveal until one `filter` card, put it onto the
    /// battlefield, the rest on the bottom in a random order, then the
    /// source deals the controller damage equal to the cards revealed (the
    /// hit included; every card when none hits).
    pub(super) fn reveal_until_one_to_battlefield_rest_bottom(
        &mut self,
        filter: &SelectionRequirement,
        damage_controller: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        use rand::seq::SliceRandom;
        let p = ctx.controller;
        let (hits, mut rest) =
            self.reveal_until_n(p, 1, |g, c| g.evaluate_requirement_on_card(filter, c, p));
        let revealed = (hits.len() + rest.len()) as u32;
        rest.shuffle(&mut self.rng.draw());
        self.players[p].library.extend(rest);
        let dest = ZoneDest::Battlefield { controller: PlayerRef::Seat(p), tapped: false };
        let placed: Vec<_> = hits.iter().map(|c| c.id).collect();
        for card in hits {
            self.place_card_in_dest(card, p, &dest, events);
        }
        // `Selector::LastMoved` is the card put onto the battlefield (Shifting
        // Shadow attaches itself to it). Stored only on a hit: the scratch is
        // a CoW group, and an empty-over-empty store would unshare it.
        if !placed.is_empty() {
            self.scratch.last_moved_cards = placed;
        }
        if damage_controller && revealed > 0 {
            self.deal_damage_to_from(super::EntityRef::Player(p), revealed, ctx.source, events);
        }
        self.check_state_based_actions_mid_resolution(events);
        Ok(())
    }

    /// Reality Scramble's reveal: until a card sharing a card type with the
    /// card `with` resolves to (CR 205.2a), onto the battlefield; the rest on
    /// the bottom in a random order.
    pub(super) fn reveal_until_shares_card_type_to_battlefield(
        &mut self,
        with: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let types = self
            .resolve_selector(with, ctx)
            .into_iter()
            .find_map(|e| e.as_card_id())
            .and_then(|id| self.find_card_anywhere(id))
            .map(|c| c.definition.card_types.clone())
            .unwrap_or_default();
        let Some(filter) = types
            .into_iter()
            .map(SelectionRequirement::HasCardType)
            .reduce(|a, b| a.or(b))
        else {
            return Ok(());
        };
        self.reveal_until_matching_to_battlefield(&filter, &Value::ONE, true, ctx, events)
    }
}
