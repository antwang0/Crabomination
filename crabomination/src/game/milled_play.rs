//! Coram, the Undertaker — "During each of your turns, you may play a land
//! and cast a spell from among cards in graveyards that were put there from
//! libraries this turn." A permission over EVERY graveyard, with its own
//! one-land / one-spell budget each turn; the card hops into the caster's
//! hand for the normal pipeline (its owner doesn't change, so it goes back
//! to its owner's graveyard) exactly like Muldrotha's own-graveyard hop.

use crate::card::CardId;
use crate::effect::StaticEffect;
use crate::game::GameState;
use crate::game::actions::CastFlags;
use crate::game::types::{GameError, GameEvent, Target};

impl GameState {
    /// The owner of `card_id` when `p` may play it under Coram's permission
    /// right now: `p`'s turn, `p` controls the static, and the card sits in a
    /// graveyard it was milled into this turn.
    pub(crate) fn milled_play_owner(&self, p: usize, card_id: CardId) -> Option<usize> {
        if !self.has_milled_play_permission(p) {
            return None;
        }
        (0..self.players.len()).find(|&o| {
            self.players[o].milled_ids_this_turn.contains(&card_id)
                && self.players[o].graveyard.iter().any(|c| c.id == card_id)
        })
    }

    /// `p`'s turn and `p` controls the static — the cheap gate callers check
    /// before walking graveyards.
    pub(crate) fn has_milled_play_permission(&self, p: usize) -> bool {
        self.active_player_idx == p
            && self.battlefield.iter().any(|c| {
                c.controller == p
                    && c.definition
                        .static_abilities
                        .iter()
                        .any(|sa| matches!(sa.effect, StaticEffect::MayPlayCardsMilledThisTurn))
            })
    }

    /// Cast a milled card under the permission, or `None` when it doesn't
    /// apply (the caller falls through to its other routes).
    pub(crate) fn try_cast_milled(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Option<Result<Vec<GameEvent>, GameError>> {
        let p = self.priority.player_with_priority;
        if self.players[p].milled_spell_cast_this_turn {
            return None;
        }
        let owner = self.milled_play_owner(p, card_id)?;
        let blocked = self.players[owner].graveyard.iter().any(|c| {
            c.id == card_id
                && (c.definition.is_land()
                    || self.cast_from_zone_blocked(p, &c.definition, crate::card::Zone::Graveyard))
        });
        if blocked {
            return None;
        }
        let card = Self::take_card(&mut self.players[owner].graveyard, card_id)?;
        self.players[p].hand.push(card);
        self.casting_hop = Some((card_id, crate::game::HopFrom::Graveyard));
        let r = self.cast_spell_with_convoke(
            card_id,
            target,
            additional_targets,
            mode,
            x_value,
            &[],
            &[],
            CastFlags::default(),
        );
        self.casting_hop = None;
        match &r {
            Err(_) => {
                if let Some(card) = Self::take_card(&mut self.players[p].hand, card_id) {
                    self.players[owner].send_to_graveyard(card);
                }
            }
            Ok(_) => {
                self.players[p].milled_spell_cast_this_turn = true;
                self.entered_from_graveyard_this_turn.insert(card_id);
            }
        }
        Some(r)
    }

    /// Play a milled land under the permission (it still uses the land drop,
    /// CR 305.2), or `None` when it doesn't apply.
    pub(crate) fn try_play_milled_land(&mut self, card_id: CardId) -> Option<Result<Vec<GameEvent>, GameError>> {
        let p = self.priority.player_with_priority;
        if self.players[p].milled_land_played_this_turn {
            return None;
        }
        let owner = self.milled_play_owner(p, card_id)?;
        if !self.players[owner].graveyard.iter().any(|c| c.id == card_id && c.definition.is_land()) {
            return None;
        }
        if !self.can_cast_sorcery_speed(p) {
            return Some(Err(GameError::SorcerySpeedOnly));
        }
        if !self.can_player_play_land(p) {
            return Some(Err(GameError::AlreadyPlayedLand));
        }
        let card = Self::take_card(&mut self.players[owner].graveyard, card_id)?;
        self.players[p].milled_land_played_this_turn = true;
        self.entered_from_graveyard_this_turn.insert(card_id);
        Some(self.place_land_card(p, card))
    }
}
