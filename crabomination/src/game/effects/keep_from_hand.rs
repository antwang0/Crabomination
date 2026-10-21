//! `Effect::EachPlayerKeepsUpToFromHandShufflesRest` — "each player chooses
//! up to N cards in their hand, then shuffles the rest into their library"
//! (Worldpurge). Every living seat chooses in APNAP order (CR 101.4), then
//! each shuffles its own rest in. A headless seat keeps up to three lands,
//! then its priciest spells, then any lands left.

use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::{Effect, LibraryPosition, PlayerRef, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::{GameError, GameEvent, GameState};

/// Lands a headless seat keeps ahead of its spells.
const HEADLESS_LANDS: usize = 3;

impl GameState {
    pub(super) fn each_player_keeps_from_hand_shuffles_rest(
        &mut self,
        keep: u32,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let source = ctx.source.unwrap_or(CardId(0));
        let seats = self.apnap_sort(self.living_seats().collect());
        let mut cursor = 0usize;
        let mut rests: Vec<(usize, Vec<CardId>)> = Vec::new();
        for p in seats {
            let hand: Vec<(CardId, String, bool, u32)> = self.players[p]
                .hand
                .iter()
                .map(|c| (c.id, c.definition.name.to_string(), c.definition.is_land(), c.definition.cost.cmc()))
                .collect();
            if hand.len() <= keep as usize {
                continue;
            }
            let mut spells: Vec<&(CardId, String, bool, u32)> = hand.iter().filter(|h| !h.2).collect();
            spells.sort_by_key(|h| std::cmp::Reverse(h.3));
            let lands = hand.iter().filter(|h| h.2);
            let auto: Vec<CardId> = lands
                .clone()
                .take(HEADLESS_LANDS)
                .chain(spells)
                .chain(lands.skip(HEADLESS_LANDS))
                .map(|h| h.0)
                .take(keep as usize)
                .collect();
            let named = hand.iter().map(|(id, n, _, _)| (*id, n.clone())).collect();
            let Some(kept) = self.ask_seat_cards_logged(
                &mut cursor,
                p,
                format!("Choose up to {keep} cards to keep; the rest are shuffled into your library"),
                source,
                named,
                0,
                keep,
                PickValue::Gain,
                effect,
                auto,
            ) else {
                return Ok(());
            };
            rests.push((p, hand.iter().map(|h| h.0).filter(|id| !kept.contains(id)).collect()));
        }
        self.clear_answer_log();
        for (p, rest) in rests {
            let dest = ZoneDest::Library { who: PlayerRef::Seat(p), pos: LibraryPosition::Top };
            for id in rest {
                self.move_card_to(id, &dest, ctx, events);
            }
            self.shuffle_library(p, events);
        }
        Ok(())
    }
}
