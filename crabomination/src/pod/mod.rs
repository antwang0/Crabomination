//! N-player Commander pods for bot self-play — the format's smoke test.
//!
//! [`crate::recommend`]'s play loop is two-seat by signature (`[Pilot; 2]`,
//! `build_match_template(seat0, seat1)`), and a Commander game is three or four
//! seats with a command zone seated before the first draw. This module is the
//! N-seat sibling: same fixed points (`stop_reason`, `STALE_ROUNDS`, the
//! settled-state adoption), same seeded-shuffle discipline, different arity.
//!
//! It is not on the 2-player throughput path and does not touch it.

pub mod decks;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};

use crate::cube::CardFactory;
use crate::game::GameState;
use crate::player::Player;
use crate::recommend::{Pilot, StopReason, stop_reason};
use crate::server::bot::Bot;

/// One Commander deck: the commander(s) that start in the command zone plus
/// the rest of the 99. Kept as factories so a pod template is built once and
/// cloned per game, the way [`crate::recommend::build_match_template`] is.
#[derive(Clone, Copy)]
pub struct PodDeck {
    pub name: &'static str,
    /// One commander, or two under Partner / Background (CR 702.124).
    pub commanders: &'static [CardFactory],
    /// The rest of the deck — `commanders.len() + main.len()` must be 100.
    pub main: &'static [CardFactory],
}

impl PodDeck {
    /// CR 903.5a — the deck is exactly 100 cards, commanders included.
    pub fn card_count(&self) -> usize {
        self.commanders.len() + self.main.len()
    }
}

/// One finished pod game.
#[derive(Debug, Clone, Copy)]
pub struct PodOutcome {
    /// The surviving seat, or `None` for a draw or an undecided game.
    pub winner: Option<usize>,
    pub actions: usize,
    pub turns: u32,
    pub stop: StopReason,
}

/// What a batch of pod games did. The Commander smoke test's whole report:
/// a panic aborts the process, so everything here is about the games that
/// finished and the ones that did not decide.
#[derive(Debug, Default, Clone)]
pub struct PodTally {
    pub games: u32,
    /// Wins by seat index.
    pub wins: Vec<u32>,
    /// `is_game_over()` with no winner (CR 104.4 draw).
    pub draws: u32,
    /// Ran out of the action budget.
    pub action_capped: u32,
    /// Blew past `MAX_BATTLEFIELD`.
    pub board_capped: u32,
    /// [`crate::recommend::STALE_ROUNDS`] rounds with no bot able to move.
    pub no_legal_move: u32,
    pub total_turns: u64,
    pub total_actions: u64,
}

impl PodTally {
    fn record(&mut self, o: &PodOutcome) {
        self.games += 1;
        self.total_turns += u64::from(o.turns);
        self.total_actions += o.actions as u64;
        match o.stop {
            StopReason::GameOver => match o.winner {
                Some(s) => {
                    if let Some(n) = self.wins.get_mut(s) {
                        *n += 1;
                    }
                }
                None => self.draws += 1,
            },
            StopReason::ActionCap => self.action_capped += 1,
            StopReason::BoardCap => self.board_capped += 1,
            StopReason::NoLegalMove => self.no_legal_move += 1,
        }
    }

    /// Games that ended without a winner, for any reason. The number the
    /// smoke test watches.
    pub fn undecided(&self) -> u32 {
        self.draws + self.action_capped + self.board_capped + self.no_legal_move
    }

    /// Fold another worker's tally in. Order-independent, so the smoke
    /// test's thread split does not change what it reports.
    pub fn merge(&mut self, other: &PodTally) {
        self.games += other.games;
        if self.wins.len() < other.wins.len() {
            self.wins.resize(other.wins.len(), 0);
        }
        for (a, b) in self.wins.iter_mut().zip(&other.wins) {
            *a += b;
        }
        self.draws += other.draws;
        self.action_capped += other.action_capped;
        self.board_capped += other.board_capped;
        self.no_legal_move += other.no_legal_move;
        self.total_turns += other.total_turns;
        self.total_actions += other.total_actions;
    }

    pub fn mean_turns(&self) -> f64 {
        if self.games == 0 { 0.0 } else { self.total_turns as f64 / f64::from(self.games) }
    }
}

/// The target decks a Commander pod run plays. Each is a legal 100-card list
/// (`cr_903_5a_target_decks_are_legal_commander_decks` proves it), so the pod
/// run is a smoke test of the format and not of the deck builder.
pub fn target_decks() -> Vec<PodDeck> {
    vec![
        PodDeck { name: "Sigarda (GW)", commanders: decks::SIGARDA_COMMANDERS, main: decks::SIGARDA_MAIN },
        PodDeck { name: "Judith (BR)", commanders: decks::JUDITH_COMMANDERS, main: decks::JUDITH_MAIN },
        PodDeck { name: "Hanna (UW)", commanders: decks::HANNA_COMMANDERS, main: decks::HANNA_MAIN },
        PodDeck { name: "Tatyova (GU)", commanders: decks::TATYOVA_COMMANDERS, main: decks::TATYOVA_MAIN },
        PodDeck { name: "Krark/Rograkh (R)", commanders: decks::KRARK_COMMANDERS, main: decks::KRARK_MAIN },
    ]
}

/// `seats` decks for a pod, cycling [`target_decks`] when there are fewer
/// lists than seats. A pod of identical lists is still a valid smoke test —
/// mirror pods are what the two-player ladder uses for the same reason.
pub fn pod_field(seats: usize) -> Vec<PodDeck> {
    let decks = target_decks();
    (0..seats).map(|i| decks[i % decks.len()]).collect()
}

/// An unshuffled N-seat Commander state with every library loaded and every
/// command zone seated — the clone-me template for [`play_one_pod_game`].
///
/// `apply_format` runs before the commanders are seated so the 40-life and
/// multiplayer-draw rules are in place when the zone is populated.
pub fn build_pod_template(decks: &[PodDeck]) -> GameState {
    let players = (0..decks.len()).map(|i| Player::new(i, format!("Seat {i}"))).collect();
    let mut g = GameState::new(players);
    g.apply_format(crate::format::Format::Commander);
    for (seat, deck) in decks.iter().enumerate() {
        for &f in deck.main {
            g.add_card_to_library(seat, crate::cube::card_arc(f));
        }
        g.seat_commanders(seat, deck.commanders.iter().map(|f| f()).collect());
        g.players[seat].wants_ui = true;
    }
    g
}

/// Play one seeded pod game from a prebuilt template.
///
/// The seat loop is [`crate::recommend`]'s, generalised past two seats: poll
/// each live seat in turn, adopt the bot's settled state when it hands one
/// back, and stop on the first of game-over / action cap / board cap /
/// staleness that [`stop_reason`] reports.
pub fn play_one_pod_game(
    template: &GameState,
    pilots: &[Pilot],
    max_actions: usize,
    seed: u64,
) -> PodOutcome {
    crate::server::bot::set_jitter_seed(Some(seed));
    let mut g = template.clone();
    let mut shuffle = StdRng::seed_from_u64(seed);
    // `zip`, not `players[seat]`: a caller that hands over more pilots than
    // the template has seats gets the extras ignored rather than a panic.
    for (player, pilot) in g.players.iter_mut().zip(pilots) {
        if let Some(w) = pilot.weights() {
            player.smart_tap = w.smart_tap;
            player.converge_rarest = w.converge_rarest;
        }
        player.hostile_player_targets = match pilot {
            Pilot::Scored(w) => w.hostile_player_targets,
            Pilot::Mcts(cfg) => cfg.weights.hostile_player_targets,
            Pilot::Uniform => false,
        };
        player.library.shuffle(&mut shuffle);
    }
    // A seeded deal implies a seeded game — mulligan reshuffles and every
    // other in-game roll come off the state's own stream (see
    // `play_one_game_traced`, which this mirrors).
    g.rng.reseed(shuffle.random());
    g.start_mulligan_phase();

    let mut bots: Vec<Box<dyn Bot>> =
        pilots.iter().take(g.players.len()).map(|p| p.build()).collect();
    let (mut actions, mut stale) = (0usize, 0usize);
    while stop_reason(&g, actions, max_actions, stale).is_none() {
        let mut any = false;
        for (seat, bot) in bots.iter_mut().enumerate() {
            // Eliminated seats are polled like any other: the bot answers
            // `None` for a seat without priority, and CR 800.4a has already
            // taken their objects and any decision addressed to them. Skipping
            // them here instead was a deadlock — a pending decision the loop
            // never polled suppressed every other seat's actions.
            let Some(step) = bot.next_action_settled(&g, seat) else { continue };
            let crate::server::bot::BotStep { action, settled } = step;
            let ok = if let Some(settled) = settled {
                g = *settled;
                true
            } else {
                match g.perform_action(action) {
                    Ok(events) => {
                        g.recycle_events(events);
                        true
                    }
                    Err(_) => false,
                }
            };
            if ok {
                any = true;
                actions += 1;
                if g.is_game_over() {
                    break;
                }
            }
        }
        if any { stale = 0 } else { stale += 1 }
    }
    crate::server::bot::set_jitter_seed(None);
    let stop = stop_reason(&g, actions, max_actions, stale).unwrap_or(StopReason::NoLegalMove);
    // `CRAB_CAP_DIAG` is the two-player loop's knob and it says the same thing
    // here: what was an undecided game actually doing. One `OnceLock` read a
    // game, on the undecided ones only.
    if crate::recommend::cap_diag_floor().is_some() && !matches!(stop, StopReason::GameOver) {
        eprintln!("pod {stop:?} seed {seed}: {}", crate::recommend::cap_diagnosis(&g, actions));
    }
    PodOutcome { winner: g.game_over.flatten(), actions, turns: g.turn_number, stop }
}

/// Play games `first .. first + count`, rotating the decks through the seats
/// so turn order is not confounded with deck strength (seat 0 is worth a lot
/// in a pod). Game `i` puts deck `d` in seat `(d + i) % n`, and the tally
/// reports wins by *deck*.
///
/// Game `i`'s seed is a pure function of `seed_base` and `i`, so a worker
/// pool can split the range any way it likes and still reproduce the run.
pub fn run_pod_games(
    decks: &[PodDeck],
    first: u32,
    count: u32,
    seed_base: u64,
    max_actions: usize,
    pilot: Pilot,
) -> PodTally {
    let n = decks.len();
    let mut tally = PodTally { wins: vec![0; n], ..Default::default() };
    if n < 2 {
        return tally;
    }
    let pilots = vec![pilot; n];
    // One template per rotation, not per game: a `GameState` clone is a
    // reference bump per zone where rebuilding 100 card definitions a seat
    // is not.
    let templates: Vec<GameState> = (0..n)
        .map(|rot| {
            let seated: Vec<PodDeck> = (0..n).map(|seat| decks[(seat + n - rot) % n]).collect();
            build_pod_template(&seated)
        })
        .collect();
    for i in first..first.saturating_add(count) {
        let rot = (i as usize) % n;
        let seed = seed_base.wrapping_add(u64::from(i).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let o = play_one_pod_game(&templates[rot], &pilots, max_actions, seed);
        // Seat `s` in rotation `rot` holds deck `(s + n - rot) % n`.
        let by_deck = PodOutcome { winner: o.winner.map(|s| (s + n - rot) % n), ..o };
        tally.record(&by_deck);
    }
    tally
}

/// [`run_pod_games`] over `0 .. games`.
pub fn run_pod(
    decks: &[PodDeck],
    games: u32,
    seed_base: u64,
    max_actions: usize,
    pilot: Pilot,
) -> PodTally {
    run_pod_games(decks, 0, games, seed_base, max_actions, pilot)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rofellos_pod(seats: usize) -> Vec<PodDeck> {
        pod_field(seats)
    }

    /// CR 903.5a/903.5b — every target deck is 100 cards, singleton, on
    /// identity and led by a legal commander. The pod run is only a smoke
    /// test of the *format* if its decks are legal to begin with.
    #[test]
    fn cr_903_5a_target_decks_are_legal_commander_decks() {
        for d in target_decks() {
            assert_eq!(d.card_count(), 100, "{}: deck size", d.name);
            let deck = crate::format::Deck {
                main: d.main.iter().map(|f| f()).collect(),
                commanders: d.commanders.iter().map(|f| f()).collect(),
                ..Default::default()
            };
            if let Err(e) = crate::format::validate_commander_deck(&deck) {
                panic!("{} is not a legal Commander deck: {e:?}", d.name);
            }
        }
    }

    /// CR 903.7 — every seat starts with its commander in the command zone,
    /// and CR 903.6 gives every seat 40 life.
    #[test]
    fn cr_903_7_every_seat_starts_with_a_seated_commander() {
        let g = build_pod_template(&rofellos_pod(4));
        assert_eq!(g.players.len(), 4);
        for p in &g.players {
            assert_eq!(p.life, 40);
            assert_eq!(p.command.len(), 1);
            assert_eq!(p.commanders.len(), 1);
            assert_eq!(p.library.len(), 99);
        }
    }

    /// CR 702.124b/d — the partner seat is the pod's only two-commander deck,
    /// and it is here so the pair is exercised by real games rather than only
    /// by a fixture: both commanders start in the command zone, its 99 is 98
    /// (CR 702.124b counts both toward the 100), and the games finish.
    ///
    /// Krark/Rograkh is last in `target_decks`, so `pod_field(4)` never draws
    /// it and the committed outcome table is untouched by its existence; this
    /// test names the field explicitly.
    #[test]
    fn cr_702_124b_a_two_commander_seat_plays_a_pod_game() {
        let field = target_decks();
        let partner = *field.last().expect("the partner deck is last");
        assert_eq!(partner.commanders.len(), 2);
        assert_eq!(partner.card_count(), 100, "CR 702.124b counts both commanders");

        let decks = vec![partner, field[0], field[1], field[2]];
        let t = build_pod_template(&decks);
        assert_eq!(t.players[0].command.len(), 2, "both begin in the command zone");
        assert_eq!(t.players[0].commanders.len(), 2);
        assert_eq!(t.players[0].library.len(), 98);
        for p in &t.players {
            assert_eq!(p.life, 40);
        }

        let pilots = vec![Pilot::default(); 4];
        for seed in [0xC0FFEE_u64, 43, 4242] {
            let o = play_one_pod_game(&t, &pilots, 50_000, seed);
            assert!(o.winner.is_some(), "seed {seed} left the pod undecided");
            assert!(o.turns > 0);
        }
    }

    /// The pod loop is reproducible: same seed, same outcome. Cross-process
    /// determinism is what the seeded smoke test rests on.
    #[test]
    fn a_seeded_pod_game_replays_identically() {
        let decks = rofellos_pod(3);
        let t = build_pod_template(&decks);
        let pilots = vec![Pilot::default(); 3];
        let a = play_one_pod_game(&t, &pilots, 3_000, 0xC0FFEE);
        let b = play_one_pod_game(&t, &pilots, 3_000, 0xC0FFEE);
        assert_eq!((a.winner, a.actions, a.turns), (b.winner, b.actions, b.turns));
    }

    /// The Commander guardrail the two-player golden traces are, at pod
    /// scale: a committed outcome per fixed seed, so a change that moves pod
    /// play shows up here as a diff rather than as a number in a smoke-test
    /// log a reviewer has to remember. The committed values were produced by
    /// a different process on a different day, which makes this a
    /// cross-process determinism check too.
    ///
    /// The triple, not a line-per-action trace: a pod game is ~2,000 actions,
    /// so a real trace would be a 400 KB file that every Commander commit
    /// re-blesses, and the winner/turns/actions triple moves on exactly the
    /// changes a trace would move on.
    ///
    /// When a rules change legitimately moves one, re-bless it in the same
    /// commit and say why in the message.
    #[test]
    fn cr_903_seeded_pod_outcomes_match_the_committed_table() {
        // (seed, winner, turns, actions)
        // Re-blessed for the auto-targeter's ranked "target opponent": the
        // same `default_hostile_opponent` the defender already used now fills
        // an open opponent slot on every cast too, so the two seeded games
        // that were already off the seat order diverge again (92→63, 79→69
        // turns; same winners). ⚠ Three games is a re-bless gate, not a
        // measurement — the aggregate is 32,000 pod games a side, 2/3/4/5
        // seats at two seeds, and it reads **identical** at two seats (the
        // control: a duel has one candidate and the ranking never runs),
        // -0.06/0.00 turns at three, -0.24/-0.17 at four and -0.37/-0.43 at
        // five, 100 % decided with zero stalls on both sides.
        const GOLDEN: [(u64, Option<usize>, u32, usize); 3] = [
            (0xC0FFEE, Some(3), 63, 3005),
            (43, Some(2), 38, 1668),
            (4242, Some(0), 69, 2882),
        ];
        let decks = rofellos_pod(4);
        let t = build_pod_template(&decks);
        let pilots = vec![Pilot::default(); 4];
        let got: Vec<(u64, Option<usize>, u32, usize)> = GOLDEN
            .iter()
            .map(|&(seed, ..)| {
                let o = play_one_pod_game(&t, &pilots, 50_000, seed);
                (seed, o.winner, o.turns, o.actions)
            })
            .collect();
        assert_eq!(got.as_slice(), GOLDEN.as_slice(), "pod outcomes moved");
    }

    /// A four-seat pod plays to a finish without panicking — the whole point
    /// of the module.
    #[test]
    fn a_four_seat_pod_finishes() {
        let t = run_pod(&rofellos_pod(4), 2, 7, 20_000, Pilot::default());
        assert_eq!(t.games, 2);
        assert!(t.total_turns > 0);
    }
}
