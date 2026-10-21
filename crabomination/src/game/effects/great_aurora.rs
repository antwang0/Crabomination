//! `Effect::EachPlayerShufflesHandAndPermanentsThenDraws` — "each player
//! shuffles all cards from their hand and all permanents they own into their
//! library, then draws that many cards" (The Great Aurora). Per the
//! 2015-06-22 rulings the count is the hand cards plus the permanents shuffled
//! in — a token counts though it ceases to exist; a commander its owner sends
//! to the command zone instead (CR 903.9b) isn't shuffled in and doesn't.

use crate::card::CardId;
use crate::effect::{Effect, LibraryPosition, PlayerRef, Selector, Value, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::{GameError, GameEvent, GameState};

impl GameState {
    pub(super) fn each_player_shuffles_hand_and_permanents_then_draws(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let seats = self.apnap_sort(self.living_seats().collect());
        let mut counts: Vec<(usize, i32)> = Vec::new();
        for &p in &seats {
            let perms: Vec<(CardId, bool)> =
                self.battlefield.iter().filter(|c| c.owner == p).map(|c| (c.id, c.is_token)).collect();
            let hand: Vec<CardId> = self.players[p].hand.iter().map(|c| c.id).collect();
            let tokens = perms.iter().filter(|(_, t)| *t).count();
            let before = self.players[p].library.len();
            let dest = ZoneDest::Library { who: PlayerRef::Seat(p), pos: LibraryPosition::Top };
            for id in perms.iter().map(|(id, _)| *id).chain(hand) {
                self.move_card_to(id, &dest, ctx, events);
            }
            let shuffled_in = self.players[p].library.len().saturating_sub(before) + tokens;
            self.shuffle_library(p, events);
            counts.push((p, shuffled_in as i32));
        }
        for (p, n) in counts {
            if n > 0 {
                self.run_effect(
                    &Effect::Draw { who: Selector::Player(PlayerRef::Seat(p)), amount: Value::Const(n) },
                    ctx,
                    events,
                )?;
            }
        }
        Ok(())
    }
}
