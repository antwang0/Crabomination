//! The Grand Larceny effects: playing the cards other players own.
//!
//! - Smirking Spelljacker's "exile target spell an opponent controls", linked
//!   to the source so its attack trigger can cast it.
//! - Thief of Sanity / Siphon Insight: look at the top N of a player's
//!   library, exile one face down (castable by you with mana of any type),
//!   the rest to their graveyard or the bottom.
//! - Extract Brain: the opponent chooses X cards from their hand; you may
//!   cast one of them free.
//! - Mind's Dilation: a player exiles their top card; if it's nonland, you
//!   may cast it free.
//! - Nashi, Moon Sage's Scion: every player exiles their top card; you may
//!   play them this turn, paying life rather than mana for a spell.
//! - Thieving Amalgam / Orochi Soul-Reaver: manifest the top card of another
//!   player's library under your control (CR 701.34).

use crate::card::{CardId, MayPlayDuration, MayPlayPermission, Zone};
use crate::effect::{Effect, PlayerRef, Selector};
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, StackItem, Target};
use crate::game::{GameError, GameState};

impl GameState {
    /// `Effect::ExileSpellLinked` — each resolved spell leaves the stack for
    /// exile (not countered: an uncounterable spell goes too), stamped
    /// `exiled_with` the source.
    pub(super) fn exile_spell_linked(
        &mut self,
        what: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let ids: Vec<CardId> = self.resolve_selector(what, ctx).into_iter().filter_map(|e| e.as_card_id()).collect();
        for cid in ids {
            let Some(pos) =
                self.stack.iter().position(|si| matches!(si, StackItem::Spell { card, .. } if card.id == cid))
            else {
                continue;
            };
            if let StackItem::Spell { mut card, .. } = self.stack.remove(pos) {
                // CR 707.10a — a copy (minted as a token) ceases to exist off
                // the stack.
                if card.is_token {
                    continue;
                }
                card.exiled_with = ctx.source;
                self.exile.push(*card);
                events.push(GameEvent::PermanentExiled { card_id: cid });
            }
        }
        Ok(())
    }

    /// Stamp "you may play it while it stays exiled, with mana of any type"
    /// (CR 609.4b: the mana value as generic) on an exiled card.
    fn grant_any_type_play(&mut self, id: CardId, to: usize) {
        let turn = self.turn_number;
        if let Some(card) = self.exile.iter_mut().find(|c| c.id == id) {
            card.may_play_until = Some(MayPlayPermission {
                player: to,
                granted_turn: turn,
                duration: MayPlayDuration::WhileExiled,
                exile_after: false,
                miracle: false,
                pay_life: false,
            });
            card.granted_alt_cast_cost_eot =
                Some(crate::mana::ManaCost::new(vec![crate::mana::generic(card.definition.cost.cmc())]));
        }
    }

    /// The engine's pick among cards to take: the highest mana value
    /// nonland, else a land.
    fn best_card_to_take<'a>(cards: impl Iterator<Item = &'a crate::card::CardInstance>) -> Option<CardId> {
        cards.max_by_key(|c| (!c.definition.is_land(), c.definition.cost.cmc())).map(|c| c.id)
    }

    /// `Effect::LookTopExileOneFaceDownMayPlay`.
    pub(super) fn look_top_exile_one_face_down_may_play(
        &mut self,
        who: &PlayerRef,
        count: &crate::effect::Value,
        rest_to_graveyard: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(seat) = self.resolve_player(who, ctx) else { return Ok(()) };
        let n = self.evaluate_value(count, ctx).max(0) as usize;
        let looked: Vec<CardId> = self.players[seat].library.iter().take(n).map(|c| c.id).collect();
        let Some(pick) = Self::best_card_to_take(self.players[seat].library.iter().take(n)) else {
            return Ok(());
        };
        if let Some(mut card) = Self::take_card(&mut self.players[seat].library, pick) {
            card.exiled_with = ctx.source;
            card.face_down = true;
            self.exile.push(card);
            events.push(GameEvent::PermanentExiled { card_id: pick });
            self.grant_any_type_play(pick, ctx.controller);
        }
        let dest = if rest_to_graveyard {
            crate::effect::ZoneDest::Graveyard
        } else {
            crate::effect::ZoneDest::Library {
                who: PlayerRef::Seat(seat),
                pos: crate::effect::LibraryPosition::Bottom,
            }
        };
        for id in looked.into_iter().filter(|id| *id != pick) {
            self.move_card_to(id, &dest, ctx, events);
        }
        Ok(())
    }

    /// Cast `id` (in `zone`) without paying its mana cost — "you may": the
    /// free-cast handler asks the controller itself.
    fn may_cast_free(
        &mut self,
        id: CardId,
        zone: Zone,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let c = EffectContext { targets: vec![Target::Permanent(id)], ..ctx.clone() };
        self.run_effect(
            &Effect::CastWithoutPayingImmediate {
                what: Selector::Target(0),
                source_zone: zone,
                exile_after: false,
                copy: false,
                reduce_generic: 0,
                pay_own_cost: false,
            },
            &c,
            events,
        )
    }

    /// `Effect::OpponentChoosesXFromHandCastOneFree` — the opponent names X
    /// cards (their pick: lands, then the cheapest); you may cast the best
    /// nonland among them free.
    pub(super) fn opponent_chooses_x_from_hand_cast_one_free(
        &mut self,
        who: &PlayerRef,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(seat) = self.resolve_player(who, ctx) else { return Ok(()) };
        let x = ctx.x_value as usize;
        let mut hand: Vec<&crate::card::CardInstance> = self.players[seat].hand.iter().collect();
        hand.sort_by_key(|c| (!c.definition.is_land(), c.definition.cost.cmc()));
        let pick = Self::best_card_to_take(hand.into_iter().take(x).filter(|c| !c.definition.is_land()));
        match pick {
            Some(id) => self.may_cast_free(id, Zone::Hand, ctx, events),
            None => Ok(()),
        }
    }

    /// `Effect::ExileTopMayCastFreeIfNonland`.
    pub(super) fn exile_top_may_cast_free_if_nonland(
        &mut self,
        who: &PlayerRef,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(seat) = self.resolve_player(who, ctx) else { return Ok(()) };
        let Some((id, land)) = self.players[seat].library.first().map(|c| (c.id, c.definition.is_land())) else {
            return Ok(());
        };
        self.move_card_to(id, &crate::effect::ZoneDest::Exile, ctx, events);
        if land || !self.exile.iter().any(|c| c.id == id) {
            return Ok(());
        }
        self.may_cast_free(id, Zone::Exile, ctx, events)
    }

    /// `Effect::ExileTopOfEachLibraryMayPlayForLife`.
    pub(super) fn exile_top_of_each_library_may_play_for_life(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let mut exiled = Vec::new();
        for seat in 0..self.players.len() {
            if !self.players[seat].is_alive() {
                continue;
            }
            if let Some(id) = self.players[seat].library.first().map(|c| c.id) {
                self.move_card_to(id, &crate::effect::ZoneDest::Exile, ctx, events);
                exiled.push(Target::Permanent(id));
            }
        }
        if exiled.is_empty() {
            return Ok(());
        }
        let n = exiled.len() as u8;
        let c = EffectContext { targets: exiled, ..ctx.clone() };
        for slot in 0..n {
            self.run_effect(
                &Effect::GrantMayPlayForLife { what: Selector::Target(slot), duration: MayPlayDuration::EndOfThisTurn },
                &c,
                events,
            )?;
        }
        Ok(())
    }

    /// `Effect::ManifestTopOfLibraryUnderYou` — CR 701.34 with the card's
    /// owner and the manifesting player apart.
    pub(super) fn manifest_top_of_library_under_you(
        &mut self,
        who: &PlayerRef,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(seat) = self.resolve_player(who, ctx) else { return Ok(()) };
        let Some(id) = self.players[seat].library.first().map(|c| c.id) else { return Ok(()) };
        if let Some(c) = self.players[seat].library.iter_mut().find(|c| c.id == id) {
            c.turn_face_down();
        }
        self.move_card_to(
            id,
            &crate::effect::ZoneDest::Battlefield { controller: PlayerRef::Seat(ctx.controller), tapped: false },
            ctx,
            events,
        );
        Ok(())
    }
}
