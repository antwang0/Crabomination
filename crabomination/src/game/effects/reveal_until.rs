//! "Reveal cards from the top of your library until you reveal N [kind]
//! cards" — the shared walk behind the reveal-until-to-battlefield effects.

use super::EffectContext;
use crate::card::{CardId, CardInstance, SelectionRequirement};
use crate::effect::{PlayerRef, Selector, Value, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// A revealed card that went from `p`'s library into a graveyard joins
    /// the "put into a graveyard from a library" family (Polluted Cistern).
    pub(super) fn note_revealed_into_graveyard(&self, p: usize, cid: CardId, events: &mut Vec<GameEvent>) {
        if self.players.iter().any(|pl| pl.graveyard.iter().any(|c| c.id == cid)) {
            events.push(GameEvent::CardRevealedIntoGraveyard { player: p, card_id: cid });
        }
    }

    /// `Effect::RevealUntilNFound` — Old Stickfingers: the `find` cards to
    /// `to`, the rest to the bottom in a random order.
    pub(super) fn reveal_until_n_found(
        &mut self,
        who: &PlayerRef,
        find: &SelectionRequirement,
        count: &Value,
        to: &ZoneDest,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        use rand::seq::SliceRandom;
        let Some(p) = self.resolve_player(who, ctx) else { return Ok(()) };
        let need = self.evaluate_value(count, ctx).max(0) as u32;
        if need == 0 {
            return Ok(());
        }
        let dest = self.resolve_zonedest_player(to, ctx);
        let me = ctx.controller;
        let (hits, mut rest) = self.reveal_until_n(p, need, |g, c| g.evaluate_requirement_on_card(find, c, me));
        for card in hits {
            let cid = card.id;
            self.place_card_in_dest(card, p, &dest, events);
            self.scratch.last_moved_cards.push(cid);
            self.note_revealed_into_graveyard(p, cid, events);
        }
        rest.shuffle(&mut self.rng.draw());
        self.players[p].library.extend(rest);
        self.check_state_based_actions_mid_resolution(events);
        Ok(())
    }

    /// `Effect::RevealUntilNonlandsExileOnePerColor` — Sanar's vivid reveal.
    /// The picks are asked before any card moves, so a suspended ask re-runs
    /// over the same library top.
    pub(super) fn reveal_until_nonlands_exile_per_color(
        &mut self,
        count: &Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &crate::effect::Effect,
    ) -> Result<(), GameError> {
        use crate::mana::Color;
        let p = ctx.controller;
        let need = self.evaluate_value(count, ctx).max(0) as usize;
        if need == 0 {
            return Ok(());
        }
        let lib = &self.players[p].library;
        let (mut n, mut nonlands) = (0, 0);
        while n < lib.len() && nonlands < need {
            nonlands += usize::from(!lib[n].definition.is_land());
            n += 1;
        }
        // Mana value first, so a first-option decider takes the biggest.
        let mut revealed: Vec<(CardId, Vec<Color>, u32, String)> = lib
            .iter()
            .take(n)
            .map(|c| (c.id, c.definition.printed_colors(), c.definition.cost.cmc(), c.definition.name.to_string()))
            .collect();
        revealed.sort_by_key(|r| std::cmp::Reverse(r.2));
        let recolored = self.card_color_change_unscoped();
        let mut colors: Vec<Color> = Vec::new();
        for c in self.battlefield.iter().filter(|c| c.controller == p) {
            let cs = if recolored { self.card_colors_anywhere(c.id) } else { c.definition.printed_colors() };
            for col in cs {
                if !colors.contains(&col) {
                    colors.push(col);
                }
            }
        }
        colors.sort_by_key(|c| *c as u8);
        let source = ctx.source.unwrap_or(CardId(0));
        let (mut cursor, mut chosen) = (0, Vec::new());
        for color in colors {
            let cands: Vec<&(CardId, Vec<Color>, u32, String)> =
                revealed.iter().filter(|r| r.1.contains(&color) && !chosen.contains(&r.0)).collect();
            if cands.is_empty() {
                continue;
            }
            let mut ballot: Vec<String> = cands.iter().map(|r| r.3.clone()).collect();
            ballot.push("No card".to_string());
            let ids: Vec<CardId> = cands.iter().map(|r| r.0).collect();
            let Some(idx) =
                self.ask_seat_option(&mut cursor, p, format!("Exile a {color:?} card?"), source, ballot, effect)
            else {
                return Ok(());
            };
            if let Some(&id) = ids.get(idx) {
                chosen.push(id);
            }
        }
        self.clear_answer_log();
        let taken: Vec<CardInstance> = (0..n).map(|_| self.players[p].library.remove(0)).collect();
        let mut rest = Vec::new();
        for card in taken {
            if !chosen.contains(&card.id) {
                rest.push(card);
                continue;
            }
            let cid = card.id;
            self.place_card_in_dest(card, p, &ZoneDest::Exile, events);
            let granted_turn = self.turn_number;
            if let Some(c) = self.exile.iter_mut().find(|c| c.id == cid) {
                c.may_play_until = Some(crate::card::MayPlayPermission {
                    player: p,
                    granted_turn,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    exile_after: false,
                    miracle: false,
                    pay_life: false,
                    cast_only: true,
                    locks_further_casts: false,
                    one_cast_group: None,
                    bottom_after: false,
                    undaunted: false,
                });
                c.granted_alt_cast_cost_eot = Some(c.definition.cost.clone());
            }
        }
        self.players[p].library.extend(rest);
        self.shuffle_library(p, events);
        Ok(())
    }

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
        may: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &crate::effect::Effect,
    ) -> Result<(), GameError> {
        use rand::seq::SliceRandom;
        let p = ctx.controller;
        // "You may put it": asked before anything moves (the hit is read by
        // position), so a re-run replays the answer.
        let put = if may
            && let Some(hit) = self.players[p].library.iter().find(|c| self.evaluate_requirement_on_card(filter, c, p))
        {
            let name = hit.definition.name.to_string();
            let mut cursor = 0;
            let Some(yes) = self.ask_seat_bool(
                &mut cursor,
                p,
                format!("Put {name} onto the battlefield?"),
                ctx.source.unwrap_or(crate::card::CardId(0)),
                effect,
                crate::decision::OptionalKind::FreeUpside,
            ) else {
                return Ok(());
            };
            self.clear_answer_log();
            yes
        } else {
            true
        };
        let (hits, mut rest) =
            self.reveal_until_n(p, 1, |g, c| g.evaluate_requirement_on_card(filter, c, p));
        let (hits, mut rest) = if put { (hits, rest) } else { (Vec::new(), { rest.extend(hits); rest }) };
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
