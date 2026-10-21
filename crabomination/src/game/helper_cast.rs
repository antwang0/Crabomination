//! Casts paid partly by tapping helpers or exiling graveyard cards — convoke
//! (CR 702.51), improvise (CR 702.126) and delve (CR 702.66) — from any zone
//! the card may be cast from: the hand (`cast_spell_with_convoke`), the
//! command zone (CR 903.8: the tax is an additional cost, so convoke and delve
//! apply to it too — on a hop, with every other hand-pipeline variant) and the
//! graveyard under a cast permission. Also "you can't spend mana to cast this
//! spell" (Hogaak), where the helpers pay it all.

use crate::card::CardId;
use crate::game::GameState;
use crate::game::actions::PaymentReceipt;
use crate::game::types::{GameError, GameEvent, Target};
use crate::mana::{ManaCost, ManaPool, SpellKind};

impl GameState {
    /// `CastSpellConvoke` / `CastSpellDelve` / `CastSpellConvokeDelve` from the
    /// hand or a graveyard cast permission (the dispatcher hops a commander in
    /// from the command zone first).
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

    /// CR 903.8 — cast a commander from the command zone through a
    /// hand-pipeline variant: hop it into the hand (the pipeline adds the tax
    /// and counts the cast, `HopFrom::CommandZone`), and put it back where it
    /// was if the cast fails or leaves it in hand (a pending float-spend
    /// question re-runs the action from the command zone).
    pub(crate) fn cast_hopped_from_command_zone(
        &mut self,
        p: usize,
        card_id: CardId,
        cast: impl FnOnce(&mut Self) -> Result<Vec<GameEvent>, GameError>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let at = self.players[p].command.iter().position(|c| c.id == card_id).ok_or(GameError::CardNotInHand(card_id))?;
        let card = self.players[p].command.remove(at);
        self.players[p].hand.push(card);
        self.casting_hop = Some((card_id, crate::game::HopFrom::CommandZone));
        let r = cast(self);
        self.casting_hop = None;
        if let Some(card) = Self::take_card(&mut self.players[p].hand, card_id) {
            let at = at.min(self.players[p].command.len());
            self.players[p].command.insert(at, card);
            self.offboard_keyword_grants = true;
        }
        r
    }
}
