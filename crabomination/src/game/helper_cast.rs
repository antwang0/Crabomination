//! Casts paid partly by tapping helpers or exiling graveyard cards — convoke
//! (CR 702.51), improvise (CR 702.126) and delve (CR 702.66) — from any zone
//! the card may be cast from: the hand (`cast_spell_with_convoke`), the
//! command zone (CR 903.8: the tax is an additional cost, so convoke and delve
//! apply to it too) and the graveyard under a cast permission. Also "you can't
//! spend mana to cast this spell" (Hogaak), where the helpers pay it all.

use crate::card::{CardId, CardInstance, Keyword, KeywordSlice};
use crate::game::GameState;
use crate::game::actions::PaymentReceipt;
use crate::game::types::{GameError, GameEvent, Target};
use crate::mana::{ManaCost, ManaPool, SpellKind};

impl GameState {
    /// `CastSpellConvoke` / `CastSpellDelve` / `CastSpellConvokeDelve`, routed
    /// by the zone `card_id` is in.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn cast_with_helpers(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        convoke: &[CardId],
        delve: &[CardId],
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.players[p].has_in_hand(card_id) && self.players[p].command.iter().any(|c| c.id == card_id) {
            return self.cast_from_command_zone_as(p, card_id, target, additional_targets, mode, x_value, false, convoke, delve);
        }
        if !self.players[p].has_in_hand(card_id)
            && let Some(used_type) = self.graveyard_cast_type_available(p, card_id)
        {
            return self.cast_hopped_from_graveyard(p, card_id, used_type, |g| {
                g.cast_spell_with_convoke(
                    card_id,
                    target,
                    additional_targets,
                    mode,
                    x_value,
                    convoke,
                    delve,
                    crate::game::actions::CastFlags::default(),
                )
            });
        }
        self.cast_spell_with_convoke(
            card_id,
            target,
            additional_targets,
            mode,
            x_value,
            convoke,
            delve,
            crate::game::actions::CastFlags::default(),
        )
    }

    /// Validate a cast's helpers before anything moves: each convoke /
    /// improvise helper is an untapped creature / artifact `p` controls, the
    /// spell has the keyword, each delved card sits in `p`'s graveyard.
    /// Returns whether the spell convokes (a convoking creature may pay a
    /// colored pip; every other helper pays {1}).
    pub(crate) fn check_cast_helpers(
        &self,
        p: usize,
        card: &CardInstance,
        convoke: &[CardId],
        delve: &[CardId],
    ) -> Result<bool, GameError> {
        let has_convoke = card.definition.keywords.has_kw(&Keyword::Convoke)
            || self.players[p].next_spell_convoke_this_turn
            || self.spell_granted_convoke(p, card);
        let has_improvise = card.definition.keywords.has_kw(&Keyword::Improvise)
            || self.players[p].next_spell_improvise_this_turn
            || self.spell_granted_improvise(p, card);
        if !convoke.is_empty() && !has_convoke && !has_improvise {
            return Err(GameError::SorcerySpeedOnly); // reuse: spell can't tap helpers
        }
        for cid in convoke {
            let ok = self.battlefield.iter().any(|c| {
                c.id == *cid
                    && c.controller == p
                    && !c.tapped
                    && ((has_convoke && self.computed_is_creature(c))
                        || (has_improvise && self.computed_has_card_type(c, crate::card::CardType::Artifact)))
            });
            if !ok {
                return Err(GameError::CardNotOnBattlefield(*cid));
            }
        }
        if !delve.is_empty()
            && !card.definition.keywords.has_kw(&Keyword::Delve)
            && !self.controller_grants_spells_delve(p)
        {
            return Err(GameError::SorcerySpeedOnly); // reuse: spell doesn't have delve
        }
        if let Some(cid) = delve.iter().find(|cid| !self.players[p].graveyard.iter().any(|c| c.id == **cid)) {
            return Err(GameError::CardNotInGraveyard(*cid));
        }
        Ok(has_convoke)
    }

    /// CR 702.51 — tap each convoke / improvise helper and credit its mana: a
    /// convoking creature pays a colored pip its colors cover and the cost
    /// still needs, else {1}.
    pub(crate) fn credit_convoke_helpers(&mut self, p: usize, cost: &ManaCost, helpers: &[CardId], has_convoke: bool) {
        for cid in helpers {
            let colors: Vec<crate::mana::Color> = if has_convoke {
                self.computed_permanent(*cid).map(|cp| cp.colors.to_vec()).unwrap_or_default()
            } else {
                Vec::new()
            };
            if let Some(c) = self.battlefield.find_by_id_mut(*cid) {
                c.tapped = true;
            }
            match colors.iter().copied().find(|c| self.cost_still_needs_color(p, cost, *c)) {
                Some(color) => self.players[p].mana_pool.add(color, 1),
                None => self.players[p].mana_pool.add_colorless(1),
            }
        }
    }

    /// "You can't spend mana to cast this spell" — the pool holds only the
    /// convoke credit (the player's own float, `float`, was set aside before
    /// it), and the credit must pay `cost` exactly. On success the float comes
    /// back and the receipt reports no mana spent; on failure the snapshot is
    /// restored (convoke taps undone, float back).
    pub(crate) fn pay_without_mana(
        &mut self,
        p: usize,
        cost: &ManaCost,
        kind: &SpellKind,
        float: ManaPool,
        snapshot: crate::game::actions::PaymentSnapshot,
    ) -> Result<PaymentReceipt, GameError> {
        let mut pool = std::mem::take(&mut self.players[p].mana_pool);
        let paid = pool.pay_for_spell(cost, kind);
        match paid {
            Ok(side_effects) if pool.total() == 0 => {
                self.players[p].mana_pool = float.clone();
                Ok(PaymentReceipt { auto_events: Vec::new(), side_effects, pool_before: float })
            }
            other => {
                self.restore_payment_state(p, snapshot);
                Err(match other {
                    Err(e) => GameError::Mana(e),
                    Ok(_) => GameError::InvalidTarget, // reuse: helpers over-pay the cost
                })
            }
        }
    }

    /// Cast `card_id` from `p`'s graveyard under a cast permission
    /// (`graveyard_cast_type_available`): hop it into the hand for the normal
    /// cast pipeline, put it back on failure, and on success record the
    /// zone change and spend Muldrotha's per-type budget (`used_type`).
    pub(crate) fn cast_hopped_from_graveyard(
        &mut self,
        p: usize,
        card_id: CardId,
        used_type: Option<crate::card::CardType>,
        cast: impl FnOnce(&mut Self) -> Result<Vec<GameEvent>, GameError>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let card = Self::take_card(&mut self.players[p].graveyard, card_id).ok_or(GameError::CardNotInHand(card_id))?;
        self.players[p].hand.push(card);
        self.casting_hop = Some((card_id, crate::game::HopFrom::Graveyard));
        let mut r = cast(self);
        self.casting_hop = None;
        match &mut r {
            Err(_) => {
                if let Some(card) = Self::take_card(&mut self.players[p].hand, card_id) {
                    self.players[p].send_to_graveyard(card);
                }
            }
            Ok(evs) => {
                self.note_left_graveyard(p, card_id, evs);
                if let Some(t) = used_type {
                    self.players[p].graveyard_cast_types_this_turn.push(t);
                }
                self.entered_from_graveyard_this_turn.insert(card_id);
            }
        }
        r
    }

    /// After a helper-paid cast's payment went through (outside the hand
    /// path, which does this inline): exile the delved cards — linked to the
    /// spell when `stamp` — and report the convoke taps (CR 702.51: "becomes
    /// tapped" triggers fire).
    pub(crate) fn finish_helper_payment(
        &mut self,
        p: usize,
        spell: CardId,
        convoke: &[CardId],
        delve: &[CardId],
        has_convoke: bool,
        stamp: Option<CardId>,
    ) -> Vec<GameEvent> {
        let mut events = Vec::new();
        for cid in delve {
            if let Some(mut exiled) = Self::take_card(&mut self.players[p].graveyard, *cid) {
                if stamp.is_some() {
                    exiled.exiled_with = stamp;
                }
                self.exile.push(exiled);
                self.players[p].cards_exiled_this_turn += 1;
                self.note_exiled_from_graveyard(p, *cid, &mut events);
            }
        }
        for &cid in convoke {
            events.push(GameEvent::PermanentTapped { card_id: cid, actor: None, as_attacker: false });
        }
        if has_convoke && !convoke.is_empty() {
            self.convoked_by.push((spell, convoke.to_vec()));
        }
        events
    }
}
