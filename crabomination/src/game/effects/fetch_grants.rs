//! Haldan, Avid Arcanist's "cards you exiled that have fetch counters": the
//! grants Pako stamps stay live while their holder controls a permanent with
//! `StaticEffect::PlayFetchCounteredExiles`, and wake or park as one enters
//! or leaves (CR 611.3a — a static's effect applies while its source is on
//! the battlefield).

use crate::card::{CardId, MAY_PLAY_DORMANT, MayPlayDuration, MayPlayPermission};
use crate::effect::StaticEffect;
use crate::game::GameState;

impl GameState {
    fn plays_fetch_exiles(def: &crate::card::CardDefinition) -> bool {
        def.static_abilities.iter().any(|sa| matches!(sa.effect, StaticEffect::PlayFetchCounteredExiles))
    }

    /// The seat a `WhileHolderControlsFetchPlayer` grant of `holder`'s runs
    /// under now: `holder` while they control such a permanent.
    pub(crate) fn fetch_grant_seat(&self, holder: usize) -> usize {
        let live = self.battlefield.iter().any(|c| c.controller == holder && Self::plays_fetch_exiles(&c.definition));
        if live { holder } else { MAY_PLAY_DORMANT }
    }

    /// `card` entered or left the battlefield: if it carries the static,
    /// re-read every grant it governs. A guarded walk (CoW zones).
    pub(crate) fn refresh_fetch_grants_for(&mut self, card: CardId) {
        let carries = self
            .battlefield_find(card)
            .or_else(|| self.find_card_anywhere(card))
            .is_some_and(|c| Self::plays_fetch_exiles(&c.definition));
        if !carries {
            return;
        }
        let governed = |c: &crate::card::CardInstance| {
            c.cold_any(|k| {
                k.may_play_until.as_ref().is_some_and(|m| {
                    matches!(m.duration, MayPlayDuration::WhileHolderControlsFetchPlayer { .. })
                })
            })
        };
        if !self.exile.iter().any(governed) {
            return;
        }
        let ids: Vec<CardId> = self.exile.iter().filter(|c| governed(c)).map(|c| c.id).collect();
        for id in ids {
            let Some(perm) = self.exile.iter().find(|c| c.id == id).and_then(|c| c.may_play_until) else { continue };
            let MayPlayDuration::WhileHolderControlsFetchPlayer { holder } = perm.duration else { continue };
            let player = self.fetch_grant_seat(holder);
            if player != perm.player
                && let Some(c) = self.exile.iter_mut().find(|c| c.id == id)
            {
                c.may_play_until = Some(MayPlayPermission { player, ..perm });
            }
        }
    }
}
