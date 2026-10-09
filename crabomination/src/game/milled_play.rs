//! Coram, the Undertaker — "During each of your turns, you may play a land
//! and cast a spell from among cards in graveyards that were put there from
//! libraries this turn." A permission over EVERY graveyard, with its own
//! one-land / one-spell budget each turn; the card hops into the caster's
//! hand for the normal pipeline (its owner doesn't change, so it goes back
//! to its owner's graveyard) exactly like Muldrotha's own-graveyard hop.
//! Kagha, Shadow Archdruid's narrower grant (own graveyard, a land or a
//! permanent spell, once a turn) rides the same routes on its own budget.

use crate::card::CardId;
use crate::effect::StaticEffect;
use crate::game::GameState;
use crate::game::actions::CastFlags;
use crate::game::types::{GameError, GameEvent, Target};

impl GameState {
    /// Does `p` control a permanent with static `want` (and is it `p`'s turn)?
    fn milled_static(&self, p: usize, want: fn(&StaticEffect) -> bool) -> bool {
        self.active_player_idx == p
            && self.battlefield.iter().any(|c| {
                c.controller == p && c.definition.static_abilities.iter().any(|sa| want(&sa.effect))
            })
    }

    fn coram_live(&self, p: usize) -> bool {
        self.milled_static(p, |e| matches!(e, StaticEffect::MayPlayCardsMilledThisTurn))
    }

    /// Kagha's grant reaches `card_id` now: its budget unused, and a land or
    /// permanent card milled into `p`'s own graveyard this turn.
    fn kagha_reaches(&self, p: usize, card_id: CardId) -> bool {
        !self.players[p].milled_once_used_this_turn
            && self.milled_static(p, |e| matches!(e, StaticEffect::MayPlayOwnMilledPermanentOncePerTurn))
            && self.players[p].milled_ids_this_turn.contains(&card_id)
            && self.players[p].graveyard.iter().any(|c| c.id == card_id && c.definition.is_permanent())
    }

    /// The owner of `card_id` when `p` may play it under a milled-card
    /// permission right now: Coram's (any graveyard) or Kagha's (`p`'s own).
    pub(crate) fn milled_play_owner(&self, p: usize, card_id: CardId) -> Option<usize> {
        if self.coram_live(p)
            && let Some(o) = (0..self.players.len()).find(|&o| {
                self.players[o].milled_ids_this_turn.contains(&card_id)
                    && self.players[o].graveyard.iter().any(|c| c.id == card_id)
            })
        {
            return Some(o);
        }
        self.kagha_reaches(p, card_id).then_some(p)
    }

    /// `p`'s turn and `p` controls a milled-play static — the cheap gate
    /// callers check before walking graveyards.
    pub(crate) fn has_milled_play_permission(&self, p: usize) -> bool {
        self.milled_static(p, |e| {
            matches!(e, StaticEffect::MayPlayCardsMilledThisTurn | StaticEffect::MayPlayOwnMilledPermanentOncePerTurn)
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
        // Coram's spell budget first, then Kagha's once-a-turn play.
        let coram = !self.players[p].milled_spell_cast_this_turn && self.coram_live(p);
        let kagha = !coram && self.kagha_reaches(p, card_id);
        if !coram && !kagha {
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
        let mut r = self.cast_spell_with_convoke(
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
        match &mut r {
            Err(_) => {
                if let Some(card) = Self::take_card(&mut self.players[p].hand, card_id) {
                    self.players[owner].send_to_graveyard(card);
                }
            }
            Ok(evs) => {
                if kagha {
                    self.players[p].milled_once_used_this_turn = true;
                } else {
                    self.players[p].milled_spell_cast_this_turn = true;
                }
                self.entered_from_graveyard_this_turn.insert(card_id);
                self.note_left_graveyard(owner, card_id, evs);
            }
        }
        Some(r)
    }

    /// Play a milled land under the permission (it still uses the land drop,
    /// CR 305.2), or `None` when it doesn't apply.
    pub(crate) fn try_play_milled_land(&mut self, card_id: CardId) -> Option<Result<Vec<GameEvent>, GameError>> {
        let p = self.priority.player_with_priority;
        let coram = !self.players[p].milled_land_played_this_turn && self.coram_live(p);
        let kagha = !coram && self.kagha_reaches(p, card_id);
        if !coram && !kagha {
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
        if kagha {
            self.players[p].milled_once_used_this_turn = true;
        } else {
            self.players[p].milled_land_played_this_turn = true;
        }
        self.entered_from_graveyard_this_turn.insert(card_id);
        let mut r = self.place_land_card(p, card, false);
        if let Ok(evs) = &mut r {
            self.note_left_graveyard(owner, card_id, evs);
        }
        Some(r)
    }
}
