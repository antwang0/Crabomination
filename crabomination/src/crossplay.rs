//! Cross-binary ladder: *this* build's bot against another build's, in
//! lockstep over a pipe.
//!
//! `bot_ladder --a X --b Y` compares two evaluation *profiles* inside one
//! binary, and `scripts/ab_wall.py` times two binaries without letting them
//! meet. Neither answers the question a code change asks — "does the bot
//! this build plays win against the bot the last build played" — so every
//! change that moved play has been justified by argument and invariants
//! instead. This module is that missing measurement.
//!
//! **Both processes run the whole game.** The parent spawns the peer binary
//! with its own argv plus `--peer`, so both build the same archetype field,
//! the same templates and the same shuffles from the same seed. Each round
//! each process polls only the seat it pilots and publishes the action it
//! chose; the other side reads it off the pipe and runs it. Two mirror
//! states, one action stream, and no `GameState` ever crosses the wire —
//! the messages are one `Option<GameAction>` per seat poll.
//!
//! **The mirrors are checked, not assumed.** Every published action carries
//! the sender's [`state_digest`] of the state it was chosen in; the receiver
//! compares it to its own before running anything. A mismatch means the two
//! builds' *engines* disagree about what an action does, which voids the
//! comparison rather than one game of it — so a fault aborts the run and
//! names the seat, the poll and both digests. That is also the limit of the
//! tool, stated plainly: it gates a change to how the bot *chooses*, and
//! reports a change to how the engine *resolves* as a fault.
//!
//! **The null control is free.** Run a binary against a copy of itself and
//! every pair must split, exactly as `--bench`'s self-mirror does: same
//! shuffle, same jitter seed, roles swapped between the two games of a pair.
//! A cross run that reads anything but 50.0 % against itself is a bug in
//! this file.
//!
//! Numbers from here are **not** comparable to the in-process ladder's: one
//! process interleaves both seats' tie-break draws on a single jitter
//! stream, and two processes each draw their own (salted by seat, so the
//! two seats stay independent and a pair's second game replays the first
//! with the binaries swapped).

use std::io::{BufRead, BufReader, Read, Write};

use serde::{Deserialize, Serialize};

use crate::game::{GameAction, GameState};

/// Wire format version. Bumped whenever [`Msg`] changes shape; the
/// handshake refuses a peer that speaks a different one, because the
/// alternative is a silently mis-parsed action stream.
pub const PROTO: u32 = 1;

/// One line of the protocol. Line-delimited JSON: the volume is one small
/// message per seat poll, and a transcript that can be read with `head` is
/// worth more here than the bytes a binary encoding would save.
#[derive(Serialize, Deserialize, Debug)]
pub enum Msg {
    /// Parent -> peer, once.
    Hello { proto: u32, field: u64 },
    /// Peer -> parent, once.
    HelloOk { proto: u32, field: u64, build: String },
    /// Parent -> peer: play this chunk of the queue. `units` is pairs under
    /// `--paired` and games otherwise, matching the in-process job.
    Job { arch: usize, units: usize, seed: u64 },
    /// Parent -> peer: the queue is empty.
    Done,
    /// One seat's poll: the action it submitted (`None` = no move) and the
    /// sender's digest of the state it chose in.
    Act { action: Option<GameAction>, pre: u64 },
    /// The sender's mirror does not match its peer's. The run is void.
    Fault { at: usize, seat: usize, mine: u64, theirs: u64 },
}

/// Why a cross run stopped early. Every variant voids the whole run, not
/// one game: a divergence means the two builds disagree about the engine,
/// and an I/O error means the peer is gone.
#[derive(Debug)]
pub enum CrossFault {
    /// The digest the peer sent for its state is not the digest of ours.
    Diverged { at: usize, seat: usize, mine: u64, theirs: u64 },
    /// The peer detected the divergence first and told us.
    PeerDiverged { at: usize, seat: usize, mine: u64, theirs: u64 },
    Io(String),
    Protocol(String),
}

impl std::fmt::Display for CrossFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrossFault::Diverged { at, seat, mine, theirs } => write!(
                f,
                "engines diverged at poll {at} of this game (seat {seat}): \
                 our state {mine:#018x}, peer's {theirs:#018x}"
            ),
            CrossFault::PeerDiverged { at, seat, mine, theirs } => write!(
                f,
                "peer reports divergence at poll {at} of this game (seat {seat}): \
                 its state {theirs:#018x}, ours {mine:#018x}"
            ),
            CrossFault::Io(e) => write!(f, "peer link: {e}"),
            CrossFault::Protocol(e) => write!(f, "peer protocol: {e}"),
        }
    }
}

/// A line-delimited JSON channel to the other build.
pub struct CrossLink {
    r: BufReader<Box<dyn Read + Send>>,
    w: Box<dyn Write + Send>,
    line: String,
}

impl CrossLink {
    pub fn new(r: Box<dyn Read + Send>, w: Box<dyn Write + Send>) -> Self {
        Self { r: BufReader::new(r), w, line: String::new() }
    }

    /// This process's stdin/stdout, for `--peer`.
    pub fn stdio() -> Self {
        Self::new(Box::new(std::io::stdin()), Box::new(std::io::stdout()))
    }

    pub fn send(&mut self, m: &Msg) -> Result<(), CrossFault> {
        let mut s = serde_json::to_string(m).map_err(|e| CrossFault::Protocol(e.to_string()))?;
        s.push('\n');
        self.w.write_all(s.as_bytes()).map_err(|e| CrossFault::Io(e.to_string()))?;
        // Every message is a turn in a strictly alternating exchange, so
        // there is never a second one to batch with it — an unflushed
        // write is a deadlock, not a saving.
        self.w.flush().map_err(|e| CrossFault::Io(e.to_string()))
    }

    pub fn recv(&mut self) -> Result<Msg, CrossFault> {
        self.line.clear();
        let n = self.r.read_line(&mut self.line).map_err(|e| CrossFault::Io(e.to_string()))?;
        if n == 0 {
            return Err(CrossFault::Io("peer closed the link".into()));
        }
        serde_json::from_str(&self.line).map_err(|e| CrossFault::Protocol(e.to_string()))
    }
}

/// The peer half of a game in progress: which seat this process pilots and
/// the link the other seat's actions arrive on.
///
/// Held for the whole run (one per worker) and re-seated per game, so the
/// play loop takes a plain `&mut` and the link's buffers outlive the game.
pub struct CrossGame {
    link: CrossLink,
    /// The seat this process pilots. Set per game by the pairs loop.
    pub seat: usize,
    /// Set once; every later call is a no-op and the run is over.
    pub fault: Option<CrossFault>,
    /// Seat polls exchanged this game — *not* the accepted-action count,
    /// which skips the polls that return nothing. It is the coordinate a
    /// fault is reported at, and the only thing that makes a divergence
    /// locatable.
    at: usize,
}

impl CrossGame {
    pub fn new(link: CrossLink) -> Self {
        Self { link, seat: 0, fault: None, at: 0 }
    }

    pub fn link_mut(&mut self) -> &mut CrossLink {
        &mut self.link
    }

    /// Start a game with this process piloting `seat`.
    pub fn seat_at(&mut self, seat: usize) {
        self.seat = seat;
        self.at = 0;
    }

    fn set(&mut self, f: CrossFault) {
        if self.fault.is_none() {
            self.fault = Some(f);
        }
    }

    /// Publish this seat's poll. `pre` is our digest of the state the bot
    /// chose in.
    pub fn publish(&mut self, action: Option<&GameAction>, pre: u64) {
        if self.fault.is_some() {
            return;
        }
        self.at += 1;
        let m = Msg::Act { action: action.cloned(), pre };
        if let Err(e) = self.link.send(&m) {
            self.set(e);
        }
    }

    /// Read the peer seat's poll, checking that it saw the state we see.
    pub fn receive(&mut self, pre: u64) -> Option<GameAction> {
        if self.fault.is_some() {
            return None;
        }
        self.at += 1;
        let (at, seat) = (self.at, 1 - self.seat);
        match self.link.recv() {
            Ok(Msg::Act { action, pre: theirs }) => {
                if theirs != pre {
                    // Tell the peer before we stop reading: it is about to
                    // block on our next publish, and a bare exit would show
                    // up there as a broken pipe with no cause attached.
                    let _ = self
                        .link
                        .send(&Msg::Fault { at, seat, mine: pre, theirs });
                    self.set(CrossFault::Diverged { at, seat, mine: pre, theirs });
                    return None;
                }
                action
            }
            Ok(Msg::Fault { at, seat, mine, theirs }) => {
                // `mine`/`theirs` are the *sender's* frame of reference.
                self.set(CrossFault::PeerDiverged { at, seat, mine: theirs, theirs: mine });
                None
            }
            Ok(other) => {
                self.set(CrossFault::Protocol(format!("expected Act, got {other:?}")));
                None
            }
            Err(e) => {
                self.set(e);
                None
            }
        }
    }
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a over everything about a game two builds must agree on before
/// either can be said to have out-played the other.
///
/// Deliberately the *observable* state and not the whole struct: it is
/// computed twice per round in both processes, so it reads the zones'
/// lengths and each permanent's mutable fields rather than serializing
/// anything. What it cannot see (per-resolution scratch, the ~78
/// `#[serde(skip)]` fields) is reset by the next resolution and cannot
/// separate two games without first moving something this does see.
pub fn state_digest(g: &GameState) -> u64 {
    let mut h = FNV_OFFSET;
    let mut mix = |v: u64| {
        h ^= v;
        h = h.wrapping_mul(FNV_PRIME);
    };
    mix(g.turn_number as u64);
    mix(g.step as u64);
    mix(g.priority.player_with_priority as u64);
    mix(g.priority.consecutive_passes as u64);
    mix(g.active_player_idx as u64);
    mix(g.stack.len() as u64);
    mix(g.exile.len() as u64);
    mix(match g.game_over {
        None => 0,
        Some(None) => 1,
        Some(Some(s)) => 2 + s as u64,
    });
    for p in &g.players {
        mix(p.life as i64 as u64);
        mix(p.library.len() as u64);
        mix(p.hand.len() as u64);
        mix(p.graveyard.len() as u64);
        for c in p.hand.iter() {
            mix(c.id.0 as u64);
        }
        for c in p.graveyard.iter() {
            mix(c.id.0 as u64);
        }
    }
    for c in g.battlefield.iter() {
        mix(c.id.0 as u64);
        mix(c.controller as u64);
        mix(c.tapped as u64);
        mix(c.damage as u64);
        mix((c.power_bonus + c.perm_power_bonus) as i64 as u64);
        mix((c.toughness_bonus + c.perm_toughness_bonus) as i64 as u64);
        mix(c.counters.len() as u64);
        mix(c.counters.values().map(|n| *n as u64).sum::<u64>());
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cube::CardFactory;
    use crate::recommend::{Pilot, simulate_match_pairs_cross};

    /// Two `CrossLink`s wired to each other over a pair of OS pipes.
    fn linked() -> (CrossLink, CrossLink) {
        let (a_r, b_w) = std::io::pipe().expect("pipe");
        let (b_r, a_w) = std::io::pipe().expect("pipe");
        (
            CrossLink::new(Box::new(a_r), Box::new(a_w)),
            CrossLink::new(Box::new(b_r), Box::new(b_w)),
        )
    }

    fn deck() -> Vec<CardFactory> {
        use crate::catalog as c;
        let mut d: Vec<CardFactory> = Vec::new();
        for _ in 0..17 {
            d.push(c::mountain as CardFactory);
        }
        for _ in 0..12 {
            d.push(c::gray_ogre);
        }
        for _ in 0..11 {
            d.push(c::lightning_bolt);
        }
        d
    }

    /// The whole point of the digest: a board that moved has to change it.
    #[test]
    fn digest_moves_with_the_board() {
        let g = crate::recommend::build_match_template(&deck(), &deck());
        let base = state_digest(&g);
        assert_eq!(base, state_digest(&g.clone()), "a clone is the same state");
        let mut h = g.clone();
        h.players[0].life -= 1;
        assert_ne!(base, state_digest(&h), "a life change is observable");
        let mut t = g.clone();
        t.turn_number += 1;
        assert_ne!(base, state_digest(&t), "a turn change is observable");
    }

    /// A peer that saw a different state is caught on the first exchange,
    /// and told, rather than having its action run against ours.
    #[test]
    fn a_mismatched_digest_faults_both_ends() {
        let (a, b) = linked();
        let mut sender = CrossGame::new(a);
        let mut receiver = CrossGame::new(b);
        receiver.seat_at(1);
        sender.publish(None, 0xdead_beef);
        assert!(receiver.receive(0x0bad_c0de).is_none());
        match receiver.fault {
            Some(CrossFault::Diverged { seat, mine, theirs, .. }) => {
                assert_eq!((seat, mine, theirs), (0, 0x0bad_c0de, 0xdead_beef));
            }
            other => panic!("expected Diverged, got {other:?}"),
        }
        // The detector owes its peer the reason; without it the far side
        // only ever sees a broken pipe.
        sender.seat_at(0);
        assert!(sender.receive(0xdead_beef).is_none());
        assert!(matches!(sender.fault, Some(CrossFault::PeerDiverged { .. })));
    }

    /// A matched digest is invisible: the action goes through untouched.
    #[test]
    fn a_matched_digest_passes_the_action_through() {
        let (a, b) = linked();
        let mut sender = CrossGame::new(a);
        let mut receiver = CrossGame::new(b);
        receiver.seat_at(1);
        sender.publish(Some(&GameAction::PassPriority), 77);
        assert!(matches!(receiver.receive(77), Some(GameAction::PassPriority)));
        assert!(receiver.fault.is_none());
    }

    /// End to end: two `CrossGame`s in two threads play the same seeded
    /// pair, one seat each, and agree on both games without a fault. This
    /// is the in-process form of `bot_ladder --vs` against itself, and the
    /// same null — one engine on both ends, so every pair splits.
    #[test]
    fn a_cross_pair_agrees_and_splits() {
        let (a, b) = linked();
        let d = deck();
        let run = |mut cx: CrossGame, i_am_a: bool| {
            let d = d.clone();
            std::thread::Builder::new()
                .stack_size(32 * 1024 * 1024)
                .spawn(move || {
                    let t = simulate_match_pairs_cross(
                        &d,
                        &d,
                        1,
                        [Pilot::default(), Pilot::default()],
                        50_000,
                        4242,
                        Some(&mut cx),
                        i_am_a,
                    );
                    (t.wins_a, t.wins_b, t.undecided, t.pairs, cx.fault.is_some())
                })
                .expect("spawn cross worker")
        };
        let ha = run(CrossGame::new(a), true);
        let hb = run(CrossGame::new(b), false);
        let ra = ha.join().expect("side A");
        let rb = hb.join().expect("side B");
        assert!(!ra.4 && !rb.4, "no fault: {ra:?} {rb:?}");
        assert_eq!(ra, rb, "both processes score the pair identically");
        assert_eq!(ra.2, 0, "both games decided");
        assert_eq!(ra.3, vec![0], "one engine on both ends splits the pair");
    }
}
