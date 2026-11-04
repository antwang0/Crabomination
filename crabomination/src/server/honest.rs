//! The honest redeal: each opponent's hidden cards dealt from the game's
//! card pool, weighted by the table net's belief head, instead of
//! reshuffled from their real deck.
//!
//! [`determinize_hidden`](super::bot::determinize_hidden) never reads the
//! opponent's hand, but it redeals from their *true* unseen cards — hand
//! plus library — so every search in every mode knows the opponent's exact
//! decklist and only mis-sorts hand against library (ML_NOTES round 39).
//! Against a human in the client that is the human's sealed or cube build,
//! which no player at the table could know. This redeal keeps only what
//! the seat can see:
//!
//! * the public zones, untouched, and the opponent's hand and library
//!   *sizes* (public numbers);
//! * the game's card pool (`table_selfplay::PoolIx`): the set for sealed,
//!   the cube list, the precons' union for Commander — what a player knows
//!   could be in a deck;
//! * the table net's belief (`TableNet::belief`), read off the seat's own
//!   view (`encode_table`, which shows no other seat's hidden cards): per
//!   opponent, a distribution over the pool for a card in hand and one for
//!   a card in library.
//!
//! Each hidden card is drawn from its zone's distribution, with the deck
//! rules a player knows applied: in Commander, the commander's color
//! identity (CR 903.4) and singleton (CR 903.5b); elsewhere at most
//! [`MAX_COPIES`] of a card; basic lands unlimited (CR 100.2a); copies
//! already visible in the opponent's public zones count against the cap.
//! The new cards take the hidden cards' `CardId`s (shuffled), so the id set
//! — a function of public history — is unchanged.
//!
//! Approximations, all toward knowing less than a player would: a card the
//! seat saw and lost track of (a Duress reveal, a bounce) is re-dealt; a
//! card known to sit on top of a library (a public "put on top") is not
//! pinned there; face-down cards keep their identity, as under
//! `determinize_hidden`.
//!
//! Without a table net carrying the belief head [`redeal`] returns `false`
//! and the caller keeps its old redeal; [`stats`] counts both, so a run
//! can say which one it played.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};

use crate::card::{CardId, CardInstance};
use crate::fxhash::HashMap;
use crate::game::GameState;
use crate::game::types::StackItem;
use crate::table_selfplay::PoolIx;

use super::encode_table::table_slots;
use super::net_eval;

/// Copies of one non-basic card a non-Commander deck is dealt at most —
/// the constructed limit (CR 100.2a). Sealed has no limit, but more than
/// four of a card from six packs is a world not worth sampling.
pub const MAX_COPIES: u16 = 4;

static REDEALS: AtomicU64 = AtomicU64::new(0);
static FALLBACKS: AtomicU64 = AtomicU64::new(0);
static OFF_POOL: AtomicU64 = AtomicU64::new(0);

/// `(redeals, fallbacks, off_pool)`: honest redeals done; calls that found
/// no table net or belief head and left the caller's redeal to it; public
/// cards an opponent showed that the redeal's pool does not contain — a
/// nonzero count says the pool is the wrong one for the game.
pub fn stats() -> (u64, u64, u64) {
    (REDEALS.load(Relaxed), FALLBACKS.load(Relaxed), OFF_POOL.load(Relaxed))
}

/// A distribution over the pool, as cumulative weights.
struct Dist {
    cum: Vec<f64>,
}

impl Dist {
    /// `softmax(logits)` over the cards `allow` keeps; `None` when it keeps
    /// none.
    fn new(logits: &[f32], allow: impl Fn(usize) -> bool) -> Option<Self> {
        let max = logits.iter().enumerate().filter(|&(i, _)| allow(i)).map(|(_, &l)| l).fold(f32::NEG_INFINITY, f32::max);
        if !max.is_finite() {
            return None;
        }
        let mut acc = 0.0f64;
        let cum = logits
            .iter()
            .enumerate()
            .map(|(i, &l)| {
                if allow(i) {
                    acc += ((l - max) as f64).exp();
                }
                acc
            })
            .collect();
        Some(Self { cum })
    }

    fn draw(&self, rng: &mut StdRng) -> usize {
        let total = *self.cum.last().expect("a kept card");
        let u = rng.random_range(0.0..total);
        self.cum.partition_point(|&c| c <= u).min(self.cum.len() - 1)
    }

    /// A draw restricted to the cards `open` keeps, exactly, by one pass
    /// over the pool; `None` when they carry no weight.
    fn draw_open(&self, rng: &mut StdRng, open: impl Fn(usize) -> bool) -> Option<usize> {
        let weight = |i: usize| if open(i) { self.cum[i] - if i == 0 { 0.0 } else { self.cum[i - 1] } } else { 0.0 };
        let total: f64 = (0..self.cum.len()).map(weight).sum();
        if total <= 0.0 {
            return None;
        }
        let mut u = rng.random_range(0.0..total);
        let mut last = None;
        for i in 0..self.cum.len() {
            let w = weight(i);
            if w > 0.0 {
                last = Some(i);
                if u < w {
                    return Some(i);
                }
                u -= w;
            }
        }
        last
    }
}

/// One decision's belief: the slots it was read for and, per opponent slot
/// (from 1), the hand and library distributions.
struct Belief {
    slots: Vec<usize>,
    zones: Vec<(Dist, Dist)>,
}

thread_local! {
    /// The last few beliefs by state. A search redeals at the top of every
    /// rollout from the same root, so a decision pays one forward pass per
    /// thread, not one per rollout.
    static CACHE: RefCell<Vec<(u64, Rc<Belief>)>> = const { RefCell::new(Vec::new()) };
}

const CACHE_LEN: usize = 8;

fn belief(g: &GameState, seat: usize, pool: &'static PoolIx) -> Option<Rc<Belief>> {
    let (t, net) = net_eval::table_belief(g, seat, pool)?;
    let key = t.fingerprint()
        ^ (net as u64).rotate_left(17)
        ^ (std::sync::Arc::as_ptr(&pool.pool) as u64).rotate_left(41)
        ^ (seat as u64).rotate_left(7);
    if let Some(b) = CACHE.with(|c| c.borrow().iter().find(|(k, _)| *k == key).map(|(_, b)| b.clone())) {
        return Some(b);
    }
    let b = Rc::new(build_belief(g, seat, pool, &net_eval::table_belief_logits(&t)?)?);
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() >= CACHE_LEN {
            c.remove(0);
        }
        c.push((key, b.clone()));
    });
    Some(b)
}

/// The belief head's logits as distributions, per opponent slot: in
/// Commander, only cards inside the opponent's commander identity.
fn build_belief(g: &GameState, seat: usize, pool: &PoolIx, logits: &[(Vec<f32>, Vec<f32>)]) -> Option<Belief> {
    let slots = table_slots(g, seat);
    if logits.len() + 1 != slots.len() {
        return None;
    }
    let mut zones = Vec::with_capacity(logits.len());
    for (k, (hand, lib)) in logits.iter().enumerate() {
        let pl = &g.players[slots[k + 1]];
        let identity = pl.commander_identity;
        let commander = !pl.commanders.is_empty();
        let allow = |i: usize| !commander || pool.identity(i).is_subset_of(identity);
        zones.push((Dist::new(hand, allow)?, Dist::new(lib, allow)?));
    }
    Some(Belief { slots, zones })
}

/// The copies of each pool card `p` owns in zones every seat sees.
fn public_copies(g: &GameState, p: usize, pool: &PoolIx) -> HashMap<u16, u16> {
    let mut out = HashMap::default();
    let mut see = |c: &CardInstance| {
        if c.is_token || c.face_down || c.owner != p {
            return;
        }
        // The card as printed: a transformed or flipped permanent shows its
        // other face's name (CR 712, 710), the pool holds the front's.
        let printed = c.front_face.as_ref().or(c.unflipped_def.as_ref()).map_or(c.definition.name, |d| d.name);
        match pool.of(printed) {
            Some(i) => *out.entry(i).or_insert(0) += 1,
            None => {
                OFF_POOL.fetch_add(1, Relaxed);
            }
        }
    };
    g.battlefield.iter().for_each(&mut see);
    g.exile.iter().for_each(&mut see);
    g.players[p].graveyard.iter().for_each(&mut see);
    g.players[p].command.iter().for_each(&mut see);
    for it in g.stack.iter() {
        if let StackItem::Spell { card, .. } = it {
            see(card);
        }
    }
    out
}

/// Redeal `g`'s hidden zones as `seat` may know them: `seat`'s own library
/// shuffled (its list is its own), every opponent's hand and library dealt
/// from the pool by the belief. `false`, with `g` untouched, when no table
/// net with a belief head is loaded — the caller then redeals its old way.
///
/// A function of the information set, the salt and the net, like
/// `determinize_hidden`: the belief reads `seat`'s view, the zones' sizes
/// and the public zones are public, and `seat`'s library is canonicalised
/// by id before its shuffle.
pub(crate) fn redeal(g: &mut GameState, seat: usize, salt: u64) -> bool {
    let Some(pool) = net_eval::redeal_pool(g) else {
        FALLBACKS.fetch_add(1, Relaxed);
        return false;
    };
    let Some(b) = belief(g, seat, pool) else {
        FALLBACKS.fetch_add(1, Relaxed);
        return false;
    };
    REDEALS.fetch_add(1, Relaxed);
    deal(g, seat, salt, pool, &b);
    true
}

/// [`redeal`] by a belief already read.
fn deal(g: &mut GameState, seat: usize, salt: u64, pool: &PoolIx, b: &Belief) {
    let mut rng = StdRng::seed_from_u64(
        salt ^ 0x484F_4E45_5354_0000 ^ ((g.turn_number as u64) << 32) ^ ((seat as u64) << 16) ^ g.step as u64,
    );
    g.players[seat].library.sort_unstable_by_key(|c| c.id.0);
    g.players[seat].library.shuffle(&mut rng);
    for (k, &p) in b.slots.iter().enumerate().skip(1) {
        let (hand_d, lib_d) = &b.zones[k - 1];
        let commander = !g.players[p].commanders.is_empty();
        let mut copies = public_copies(g, p, pool);
        let cap = |i: usize| {
            if pool.is_basic(i) {
                u16::MAX
            } else if commander {
                1
            } else {
                MAX_COPIES
            }
        };
        // Rejection against the cap: a capped card is usually a small share
        // of the mass, so a few tries find an open one. A belief that
        // concentrates on capped cards gets an exact draw over the open ones
        // instead, and one with no open card left (never, while basics are
        // in the pool) the last draw over the cap.
        let mut deal = |d: &Dist, rng: &mut StdRng| {
            let open = |copies: &HashMap<u16, u16>, i: usize| copies.get(&(i as u16)).copied().unwrap_or(0) < cap(i);
            let mut i = d.draw(rng);
            let mut tries = 0;
            while !open(&copies, i) {
                tries += 1;
                if tries == 16 {
                    i = d.draw_open(rng, |j| open(&copies, j)).unwrap_or(i);
                    break;
                }
                i = d.draw(rng);
            }
            *copies.entry(i as u16).or_insert(0) += 1;
            i
        };
        let (n, m) = (g.players[p].hand.len(), g.players[p].library.len());
        let mut dealt: Vec<usize> = Vec::with_capacity(n + m);
        for j in 0..n + m {
            dealt.push(deal(if j < n { hand_d } else { lib_d }, &mut rng));
        }
        let mut ids: Vec<CardId> = g.players[p].hand.iter().chain(g.players[p].library.iter()).map(|c| c.id).collect();
        ids.sort_unstable_by_key(|id| id.0);
        ids.shuffle(&mut rng);
        drop(g.players[p].hand.drain(..));
        drop(g.players[p].library.drain(..));
        for (j, (id, i)) in ids.into_iter().zip(dealt).enumerate() {
            let mut card = CardInstance::new(id, pool.def(i), p);
            if j < n {
                g.players[p].hand.push(card);
            } else {
                // Already in the library, not put there (CR 400.7): a redeal
                // is no game action, so no arrival may trigger off it.
                card.in_library = true;
                g.players[p].library.push(card);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table_selfplay::{Mode, mode_pool, mode_template};

    /// `mode`'s opening table with seven cards in every hand.
    fn dealt(mode: Mode, seed: u64) -> GameState {
        let mut g = mode_template(mode, seed);
        for p in &mut g.players {
            let split = p.library.len() - 7;
            let hand: Vec<_> = p.library.split_off(split);
            p.hand.extend(hand);
        }
        g
    }

    /// Fixed made-up logits, one `(hand, library)` per opponent.
    fn logits(g: &GameState, pool: &PoolIx, f: impl Fn(usize) -> f32) -> Vec<(Vec<f32>, Vec<f32>)> {
        let l: Vec<f32> = (0..pool.len()).map(f).collect();
        (1..g.players.len()).map(|_| (l.clone(), l.clone())).collect()
    }

    fn zone(cards: &[CardInstance]) -> Vec<(u32, &'static str)> {
        cards.iter().map(|c| (c.id.0, c.definition.name)).collect()
    }

    /// The point of the redeal: the opponent's real hand and library never
    /// reach it. Two tables that differ only in what the opponent is
    /// actually holding — same ids, same sizes, other cards — redeal to the
    /// same cards in the same places, and nothing public moves.
    #[test]
    fn the_honest_redeal_reads_no_hidden_card() {
        let vocab = net_eval::vocab();
        let pool = mode_pool(Mode::Cube, vocab);
        let real = dealt(Mode::Cube, 5);
        let mut other = real.clone();
        let opp = &mut other.players[1];
        let swap = |cards: &[CardInstance]| -> Vec<CardInstance> {
            cards.iter().enumerate().map(|(j, c)| CardInstance::new(c.id, pool.def((j * 13 + 5) % pool.len()), 1)).collect()
        };
        let (hand, lib) = (swap(&opp.hand), swap(&opp.library));
        drop(opp.hand.drain(..));
        drop(opp.library.drain(..));
        opp.hand.extend(hand);
        opp.library.extend(lib);
        assert_ne!(zone(&real.players[1].library), zone(&other.players[1].library), "the two tables hide different cards");

        let b = build_belief(&real, 0, pool, &logits(&real, pool, |i| (i % 7) as f32 * 0.3)).expect("a belief");
        let (mut a, mut c) = (real.clone(), other.clone());
        deal(&mut a, 0, 7, pool, &b);
        deal(&mut c, 0, 7, pool, &b);
        assert_eq!(zone(&a.players[1].hand), zone(&c.players[1].hand), "the hand dealt is the same");
        assert_eq!(zone(&a.players[1].library), zone(&c.players[1].library), "and the library");
        assert_ne!(zone(&a.players[1].library), zone(&real.players[1].library), "the deal is not the real deck");

        let ids = |g: &GameState| {
            let mut v: Vec<u32> = g.players[1].hand.iter().chain(g.players[1].library.iter()).map(|c| c.id.0).collect();
            v.sort_unstable();
            v
        };
        assert_eq!(ids(&a), ids(&real), "the hidden ids are the ones that were hidden");
        assert_eq!(a.players[1].hand.len(), 7);
        assert_eq!(a.players[1].library.len(), real.players[1].library.len());
        assert_eq!(zone(&a.players[0].hand), zone(&real.players[0].hand), "our own hand stays ours");
        let mut ours = zone(&a.players[0].library);
        let mut was = zone(&real.players[0].library);
        ours.sort_unstable();
        was.sort_unstable();
        assert_eq!(ours, was, "our own library is only shuffled");
        assert!(a.players[1].hand.iter().all(|c| c.owner == 1), "dealt cards belong to the opponent");
        assert_eq!(a.players[1].library.arrivals(), real.players[1].library.arrivals(), "a redeal puts nothing into a library");
    }

    /// Commander's deck rules are known to everyone: what is dealt to a pod
    /// opponent sits inside their commander's color identity, and no
    /// non-basic card appears twice or beside the commander itself.
    #[test]
    fn a_commander_redeal_keeps_identity_and_singleton() {
        let vocab = net_eval::vocab();
        let pool = mode_pool(Mode::Pod(4), vocab);
        let mut g = dealt(Mode::Pod(4), 3);
        let b = build_belief(&g, 0, pool, &logits(&g, pool, |_| 0.0)).expect("a belief");
        deal(&mut g, 0, 11, pool, &b);
        for p in 1..g.players.len() {
            let pl = &g.players[p];
            let mut seen: std::collections::HashSet<&str> = pl.command.iter().map(|c| c.definition.name).collect();
            for c in pl.hand.iter().chain(pl.library.iter()) {
                let i = pool.of(c.definition.name).expect("dealt from the pool") as usize;
                assert!(pool.identity(i).is_subset_of(pl.commander_identity), "{} is outside seat {p}'s identity", c.definition.name);
                if !pool.is_basic(i) {
                    assert!(seen.insert(c.definition.name), "{} dealt twice to seat {p}", c.definition.name);
                }
            }
        }
    }

    /// A belief sure of one card still deals a legal deck: that card up to
    /// the copy limit, less the copies already on the battlefield, and the
    /// rest of the hidden cards from the remaining mass.
    #[test]
    fn a_confident_belief_stops_at_the_copy_limit() {
        let vocab = net_eval::vocab();
        let pool = mode_pool(Mode::SealedSos, vocab);
        let mut g = dealt(Mode::SealedSos, 9);
        let sure = (0..pool.len()).find(|&i| !pool.is_basic(i)).expect("a non-basic");
        let shown = CardInstance::new(CardId(9_000_000), pool.def(sure), 1);
        g.battlefield.push(shown);
        let b = build_belief(&g, 0, pool, &logits(&g, pool, |i| if i == sure { 40.0 } else { 0.0 })).expect("a belief");
        deal(&mut g, 0, 3, pool, &b);
        let pl = &g.players[1];
        let name = pool.def(sure).name;
        let n = pl.hand.iter().chain(pl.library.iter()).filter(|c| c.definition.name == name).count();
        assert_eq!(n, MAX_COPIES as usize - 1, "four copies in all, one of them public");
    }

    #[test]
    fn a_distribution_draws_only_what_it_allows_in_proportion() {
        let d = Dist::new(&[0.0, 5.0, (3.0f32).ln(), 0.0], |i| i != 1).expect("three kept");
        let mut rng = StdRng::seed_from_u64(1);
        let mut n = [0usize; 4];
        for _ in 0..50_000 {
            n[d.draw(&mut rng)] += 1;
        }
        assert_eq!(n[1], 0, "a masked card is never drawn, however large its logit");
        let r = n[2] as f64 / n[0] as f64;
        assert!((r - 3.0).abs() < 0.15, "weights follow the softmax: {n:?}");
        assert!(Dist::new(&[1.0, 2.0], |_| false).is_none(), "nothing kept is no distribution");
    }
}
