//! CR 729 — subgames.
//!
//! A subgame is a whole nested game played with the main game's players,
//! using the cards in their libraries as their decks (CR 729.2). It is
//! piloted end-to-end by bots — there is no way to hand priority in a nested
//! game back to a networked seat — and bounded by an action cap so a stalled
//! nest can't wedge the outer game. Shahrazad is the only card that starts
//! one.

use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::player::Player;
use crate::recommend::STALE_ROUNDS;
use crate::server::bot::{Bot, RandomBot};

/// CR 729.5 — a subgame may itself contain a subgame; the nest is capped so
/// a library full of Shahrazads terminates.
const MAX_DEPTH: u32 = 2;
/// Bounded pilot budget. A subgame that hasn't decided by here is a draw
/// (CR 104.4), which for Shahrazad means every player takes the payout.
const MAX_ACTIONS: usize = 4_000;

impl GameState {
    /// Play a subgame and return its winning **seat index in the outer game**,
    /// or `None` for a draw / stalled nest. Main-game zones are untouched: the
    /// subgame gets fresh instances of the library cards' definitions, and the
    /// outer libraries are shuffled afterwards (CR 729.6).
    pub(crate) fn play_subgame(&mut self) -> Option<usize> {
        if self.subgame_depth >= MAX_DEPTH {
            return None;
        }
        let seats = self.players.len();
        let mut sub = GameState::new(
            (0..seats).map(|i| Player::new(i, self.players[i].name.clone())).collect(),
        );
        sub.subgame_depth = self.subgame_depth + 1;
        // Derived from (and advancing) the outer stream, so a seeded game
        // stays seeded through a Shahrazad.
        sub.rng = self.rng.fork();
        for seat in 0..seats {
            let defs: Vec<crate::card::CardDefinition> =
                self.players[seat].library.iter().map(|c| (*c.definition).clone()).collect();
            // CR 729.3 — a player with no library can't play; they lose the
            // subgame immediately rather than deck out on the first draw.
            for def in defs {
                sub.add_card_to_library(seat, def);
            }
            sub.players[seat].library.shuffle(&mut sub.rng.draw());
        }
        sub.start_mulligan_phase();

        let mut bots: Vec<Box<dyn Bot>> =
            (0..seats).map(|_| Box::new(RandomBot::default()) as Box<dyn Bot>).collect();
        let (mut actions, mut stale) = (0usize, 0usize);
        while !sub.is_game_over() && actions < MAX_ACTIONS && stale < STALE_ROUNDS {
            let mut any = false;
            for (s, bot) in bots.iter_mut().enumerate() {
                let Some(a) = bot.next_action(&sub, s) else { continue };
                if sub.perform_action(a).is_ok() {
                    any = true;
                    actions += 1;
                    if sub.is_game_over() {
                        break;
                    }
                }
            }
            if any { stale = 0 } else { stale += 1 }
        }

        // CR 729.6 — the cards used in the subgame go back where they came
        // from and each player shuffles their library.
        let mut shuffle_events = Vec::new();
        for seat in 0..seats {
            self.shuffle_library(seat, &mut shuffle_events);
        }
        sub.game_over.flatten()
    }
}
