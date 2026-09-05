//! Golden traces: the full action/state history of fixed-seed bot games,
//! frozen in the repo.
//!
//! The suite's other tests each pin one rule in a hand-built board. That
//! catches a rule that breaks; it does not catch a *refactor* that quietly
//! changes what a real game does — a reordered iteration, a dropped
//! trigger in a shape nobody wrote a test for, a "pure" optimization that
//! isn't. Those show up here as a diff, because a whole game is compared
//! rather than one assertion.
//!
//! Two properties are being asserted at once:
//!
//! * **Determinism.** Same seed, same trace. The committed file was
//!   produced by a *different process* on a different day, so comparing
//!   against it is a cross-process determinism check: any `HashMap`
//!   iteration order leaking into game logic changes the text, since
//!   `RandomState` reseeds per process.
//! * **Behaviour preservation.** A performance change that moves a trace
//!   isn't behaviour-preserving, whatever the benchmark says.
//!
//! When a rules fix legitimately changes a trace, re-bless it in the same
//! commit and say why in the message:
//!
//! ```text
//! CRAB_BLESS_TRACES=1 cargo nextest run -E 'binary(core_rules)' golden_trace --no-capture
//! ```

use crabomination::catalog as c;
use crabomination::cube::CardFactory;
use crabomination::recommend::trace_game;

/// Turn cap. High enough that these decks finish, low enough that a game
/// which stops finishing fails loudly instead of hanging the suite.
const MAX_ACTIONS: usize = 20_000;

fn deck(spec: &[(CardFactory, usize)]) -> Vec<CardFactory> {
    spec.iter().flat_map(|&(f, n)| std::iter::repeat_n(f, n)).collect()
}

/// Mono-red aggro. Races, so the trace covers attacks, burn and a short
/// game.
fn red() -> Vec<CardFactory> {
    deck(&[
        (c::mountain as CardFactory, 17),
        (c::lightning_bolt, 4),
        (c::shock, 3),
        (c::goblin_guide, 4),
        (c::monastery_swiftspear, 3),
        (c::gray_ogre, 3),
        (c::hill_giant, 3),
        (c::fire_elemental, 2),
        (c::shivan_dragon, 1),
    ])
}

/// Azorius skies. Evasion, removal and a counterspell, so the trace covers
/// the stack, auras and blocks the other deck can't make.
fn white_blue() -> Vec<CardFactory> {
    deck(&[
        (c::plains as CardFactory, 9),
        (c::island, 8),
        (c::wind_drake, 4),
        (c::air_elemental, 3),
        (c::serra_angel, 2),
        (c::baneslayer_angel, 1),
        (c::wall_of_omens, 3),
        (c::pacifism, 3),
        (c::swords_to_plowshares, 3),
        (c::divination, 2),
        (c::counterspell, 2),
    ])
}

const SEED: u64 = 0xC0FFEE;
const GOLDEN: &str = include_str!("golden_trace_seed_c0ffee.txt");

/// Rewrite the committed trace next to this source file. Only runs under
/// `CRAB_BLESS_TRACES=1`; a normal run never touches the repo.
fn bless(name: &str, text: &str) -> bool {
    if std::env::var_os("CRAB_BLESS_TRACES").is_none() {
        return false;
    }
    // `file!()` is workspace-relative but the test's cwd is the package
    // directory, so anchor it on the manifest's parent.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let path = root.join(file!()).parent().unwrap().join(name);
    std::fs::write(&path, text).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    eprintln!("blessed {}", path.display());
    true
}

#[test]
fn red_vs_skies_matches_the_committed_trace() {
    let t = trace_game(&red(), &white_blue(), SEED, MAX_ACTIONS);
    let got = t.text();
    if bless("golden_trace_seed_c0ffee.txt", &got) {
        return;
    }
    if got != GOLDEN {
        // Point at the first divergence rather than dumping two full
        // games: the interesting line is always the first one that moved.
        let (a, b): (Vec<&str>, Vec<&str>) = (GOLDEN.lines().collect(), got.lines().collect());
        let at = a.iter().zip(&b).position(|(x, y)| x != y);
        let msg = match at {
            Some(i) => format!(
                "trace diverges at line {}:\n  expected: {}\n  actual:   {}",
                i + 1,
                a[i],
                b[i]
            ),
            None => format!("trace length changed: expected {} lines, got {}", a.len(), b.len()),
        };
        panic!(
            "{msg}\n\nIf this is an intended rules change, re-bless in the same commit:\n  \
             CRAB_BLESS_TRACES=1 cargo nextest run -E 'binary(core_rules)' golden_trace --no-capture"
        );
    }
}

/// The same seed replayed in-process. Cheap, and it separates two failure
/// modes that would otherwise look identical: a trace that moved between
/// commits (the test above) versus one that isn't reproducible at all.
#[test]
fn the_same_seed_replays_identically() {
    let (a, b) = (
        trace_game(&red(), &white_blue(), 7, MAX_ACTIONS),
        trace_game(&red(), &white_blue(), 7, MAX_ACTIONS),
    );
    if a.digest() != b.digest() {
        let at = a.lines.iter().zip(&b.lines).position(|(x, y)| x != y);
        panic!(
            "same seed produced two different games; first divergence at {:?}:\n  run 1: {:?}\n  \
             run 2: {:?}\n  (lines {} vs {})",
            at,
            at.and_then(|i| a.lines.get(i)),
            at.and_then(|i| b.lines.get(i)),
            a.lines.len(),
            b.lines.len(),
        );
    }
    assert_eq!(a.winner, b.winner);
    assert_eq!(a.turns, b.turns);
}

/// The same seed replayed on several threads *at once*.
///
/// The two tests above replay sequentially on one thread, so neither can see
/// a divergence that only appears when games run **concurrently** — a global
/// cache filled in a different order, a thread-local carried across games, an
/// allocator address that leaks into a key. A ladder run at `--threads 2`
/// flipping one game's winner that `--threads 1` gets right is exactly that
/// shape, and it is invisible to a sequential replay.
///
/// Deliberately runs more threads than a small box has cores, so the workers
/// interleave rather than each getting a quiet core.
#[test]
fn the_same_seed_replays_identically_across_threads() {
    use crabomination::mana::Color;
    use rand::SeedableRng;
    let mut r = rand::rngs::StdRng::seed_from_u64(0xC0BE_5EED);
    let a = crabomination::cube::cube_deck([Color::Red, Color::White], &mut r);
    let b = crabomination::cube::cube_deck([Color::White, Color::Blue], &mut r);
    let traces: Vec<_> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let (a, b) = (&a, &b);
                s.spawn(move || {
                    // Two games per worker, so a stream carried from one game
                    // into the next on the same thread also shows up.
                    [
                        trace_game(a, b, 21, MAX_ACTIONS),
                        trace_game(a, b, 21, MAX_ACTIONS),
                    ]
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    let first = &traces[0];
    for (i, t) in traces.iter().enumerate().skip(1) {
        if t.digest() != first.digest() {
            let at = first.lines.iter().zip(&t.lines).position(|(x, y)| x != y);
            panic!(
                "replay {i} diverged from replay 0 under concurrency; first divergence at \
                 {at:?}:\n  run 0: {:?}\n  run {i}: {:?}\n  (winners {:?} vs {:?})",
                at.and_then(|j| first.lines.get(j)),
                at.and_then(|j| t.lines.get(j)),
                first.winner,
                t.winner,
            );
        }
    }
}

/// The ladder pair that used to break its own mirror, replayed concurrently.
///
/// `bot_ladder --a gang --b gang --decks all --games 400 --seed 21` reported a
/// self-mirror pair that did *not* split — a pair whose two halves are the
/// same game with the seats relabelled. `CRAB_PAIR_SWEEPS=1` named it:
/// archetype "cube RW" (the seed-21 cube pool's third pair), job seed
/// 25769803807, pair 5, pair seed **1663341901257141384**. At `--threads 1` it
/// always split; at 2 and 3 it flipped, in both directions, run to run.
///
/// The cause was `GameState::restart_game` (CR 727): it rebuilds the state
/// with `GameState::new`, whose `GameRng` is `from_entropy`, so a seeded game
/// that restarts drew its post-restart deal from OS entropy. This game
/// restarts, which is why it and only it broke. Sequential replays could not
/// see it because both halves of a pair then drew from the same entropy
/// stream in the same order often enough to agree.
#[test]
fn the_ladder_pair_that_breaks_its_mirror() {
    use crabomination::cube::{cube_deck, random_color_pair};
    use rand::SeedableRng;
    const PAIR_SEED: u64 = 1_663_341_901_257_141_384;
    // `bot_ladder::cube_archetypes(21, 8)`, third entry.
    let deck = {
        let mut r = rand::rngs::StdRng::seed_from_u64(21 ^ 0xC0BE_5EED);
        let mut d = Vec::new();
        for _ in 0..3 {
            let colors = random_color_pair(&mut r);
            d = cube_deck(colors, &mut r);
        }
        d
    };
    let traces: Vec<_> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let deck = &deck;
                s.spawn(move || {
                    [
                        trace_game(deck, deck, PAIR_SEED, 50_000),
                        trace_game(deck, deck, PAIR_SEED, 50_000),
                    ]
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    let first = &traces[0];
    for (i, t) in traces.iter().enumerate().skip(1) {
        if t.digest() != first.digest() {
            let at = first.lines.iter().zip(&t.lines).position(|(x, y)| x != y);
            panic!(
                "replay {i} diverged; first divergence at {at:?}:\n  run 0: {:?}\n  \
                 run {i}: {:?}\n  (winners {:?} vs {:?}, turns {} vs {})",
                at.and_then(|j| first.lines.get(j)),
                at.and_then(|j| t.lines.get(j)),
                first.winner,
                t.winner,
                first.turns,
                t.turns,
            );
        }
    }
}

/// A different seed has to produce a different game — otherwise the two
/// tests above would pass on an engine that ignores its seed entirely.
#[test]
fn different_seeds_produce_different_games() {
    let a = trace_game(&red(), &white_blue(), 7, MAX_ACTIONS);
    let b = trace_game(&red(), &white_blue(), 8, MAX_ACTIONS);
    assert_ne!(a.digest(), b.digest(), "seed is not reaching the shuffle");
}

/// Breadth without bulk: five more seeds pinned by digest. Committing five
/// more full traces would be ~200 KB of text nobody reads; a digest still
/// fails when the engine drifts, and the readable diff is one test up.
///
/// `(seed, winner, turns, actions, digest)`.
// Re-pinned 2026-08-09: `EvalWeights::default()` adopted `determinize: 1`
// (task #25), so the default bot's sims redeal hidden zones and three of
// the five seeded games legitimately take different lines. Deliberate
// behaviour change, not drift.
const DIGESTS: &[(u64, Option<usize>, u32, usize, u64)] = &[
    // Seeds 1, 2 and 5 moved when `determinize_hidden` began sorting
    // hidden zones before redealing them (see the commit). Every winner
    // is unchanged; only the lines differ, which is what an honest-redeal
    // change looks like -- the searches make a different arbitrary guess,
    // not a better or worse one. Seeds 3 and 4 are untouched.
    // Re-blessed 2026-09-05 for the round-56 adoptions (the block chain,
    // +5.8 on the ladder, and the wide attack chain, +0.5): every seat now
    // finds sim-priced blocks — gangs on a bare menu above all — where the
    // greedy trade table declared nothing, so combats trade where they
    // used to connect. Seeds 1 and 2 keep their winners and end a few
    // actions apart (281 -> 277, 384 -> 434); seeds 3 and 4 FLIP to seat
    // 1 and run far longer (21 -> 32 turns, 13 -> 24) — the aggro deck's
    // early swings are now blocked instead of raced, and the skies deck
    // wins the long game. Seed 5 and no other line are untouched. The
    // full committed trace moved at one line, a turn-7 block declaration.
    // Re-blessed 2026-09-05 for the round-60 adoption (the open-board
    // attack shortcut on the default: no loss on sealed, cube and fixed,
    // -4.1 % wall clock): on a board with no opposing creature the greedy
    // declaration is taken without the sim, and these two red decks meet
    // that board every early turn — the sim's occasional hold-back of a
    // Goblin Guide (its trigger cards the opponent) is gone. Seed 1 keeps
    // its winner and runs longer (11 -> 19 turns, 277 -> 460 actions);
    // seed 4 keeps its winner and runs shorter (24 -> 22, 580 -> 541);
    // seed 5 keeps its winner and races faster (9 -> 7, 223 -> 181).
    // Seeds 2 and 3 and the full committed trace are untouched.
    (1, Some(0), 19, 460, 0xd6aa_9217_5911_0df5),
    // Re-blessed Aug 2026 for the CR 510.4/510.5 combat-damage fix: the step
    // loop used to skip a whole attacker/blocker pairing whenever the attacker
    // dealt no damage in that step, so a first striker that failed to kill its
    // blocker was never struck back. Seeds 2 and 3 now trade in combat where
    // a first striker previously walked away, which ends both games in fewer
    // actions (392 -> 384, 459 -> 439). Both winners and both turn counts are
    // unchanged; seeds 1, 4 and 5 are untouched.
    // Re-blessed 2026-08-19 for the desperation-chump-block adoption
    // (round 43, +0.9 on the ladder): seed 2's game contains a board
    // where a seat within two swings of dead now chumps instead of
    // taking it. Same winner, same turn and action counts — only the
    // digest differs, which is what a single changed block looks like.
    // Re-blessed 2026-08-26 for the per-colour affordability budget: the
    // bot's pre-filter used to offer a spell whenever *some* source made
    // each of its colours, so `{G}{G}` off a lone Forest reached the pick
    // site and was thrown away by the engine's payment. Seed 2 contains one
    // such offer; the bot now takes its next line instead. Same winner, same
    // turn count, same action count — only the digest moves, and the filter
    // is sound by measurement (zero cases where `could_pay_cost` accepts a
    // cost the budget rejects, over 21 pool/seed configurations).
    (2, Some(1), 18, 434, 0x9b44_e2cd_0954_bb3e),
    // Re-blessed 2026-08-26 for the combat-planner legality fixes: the block
    // planner used to assemble batches `declare_blockers` rejects (a landwalk
    // it could not see, a gang pass that skipped the pair gate, menace read
    // off the printed keyword set instead of the computed one), and the
    // engine rejects the *batch* — so the defender blocked with nothing. It
    // blocks now, and seed 3 runs two turns longer (19 -> 21, 439 -> 484
    // actions) for the same winner. Seeds 1, 2, 4 and 5 are untouched.
    // Re-blessed 2026-09-04 for the attack-chain adoption (round 55, +2.3
    // on the ladder): the default's attack search now also offers a
    // declaration grown one creature at a time from nobody, and seed 3
    // contains one board where that set out-scores the holdback menu.
    // Same winner, same turn count, one action fewer (484 -> 483); seeds
    // 1, 2, 4 and 5 and the full committed trace are untouched.
    // Re-blessed 2026-09-05 for the round-58 adoption (the wide chain's
    // pair move only from an empty greedy and only after the singles tie;
    // no loss over four ladder seeds, -14.9 % wall clock): seed 3 holds
    // one board where the pair move used to out-score the singles beside
    // a non-empty greedy declaration. Same winner, same turn count, same
    // action count — only the digest moves; seeds 1, 2, 4 and 5 and the
    // full committed trace are untouched.
    (3, Some(1), 32, 737, 0x72af_f312_fcea_3b21),
    // Re-blessed 2026-08-22 for the slot-walk targeting fix: the filtered
    // auto-target path used to take the first legal permanent in
    // battlefield order, so Swords to Plowshares ("target creature", an
    // unrestricted filter) picked by board position rather than by side.
    // Seat 1 now aims it at seat 0 deterministically instead of by luck of
    // ordering, and seed 4's game ends in two fewer actions (330 -> 328).
    // Same winner, same turn count; seeds 1, 2, 3 and 5 are untouched,
    // which is what a targeting fix rather than a rules change looks like.
    (4, Some(1), 22, 541, 0x5b00_4268_cea2_9a41),
    (5, Some(0), 7, 181, 0xbdda_f862_13d8_5da0),
];

#[test]
fn seeded_games_match_their_digests() {
    let mut rows = Vec::new();
    let mut bad = Vec::new();
    for &(seed, winner, turns, actions, digest) in DIGESTS {
        let t = trace_game(&red(), &white_blue(), seed, MAX_ACTIONS);
        rows.push(format!(
            "    ({seed}, {:?}, {}, {}, {:#018x}),",
            t.winner,
            t.turns,
            t.lines.len(),
            t.digest()
        ));
        if (t.winner, t.turns, t.lines.len(), t.digest()) != (winner, turns, actions, digest) {
            bad.push(seed);
        }
    }
    if !bad.is_empty() {
        panic!(
            "seeds {bad:?} drifted. Current values (paste into DIGESTS, with a \
             one-line justification in the commit):\n{}",
            rows.join("\n")
        );
    }
}

/// The pool that broke, and the one the hand-built decks above cannot
/// stand in for: two seeded cube decks, traced twice. `--decks cube` played
/// a different game on every run until `841dd40b`, because `RandomState`
/// reseeds every `HashMap` — per process *and* per instance — and ~110 of
/// the engine's maps are locals the field-by-field survey never reached.
/// The decks here draw from the whole cube pool, so they touch card paths
/// `red()` / `white_blue()` never do.
#[test]
fn a_seeded_cube_pairing_replays_identically() {
    use crabomination::mana::Color;
    use rand::SeedableRng;
    let decks = || {
        let mut r = rand::rngs::StdRng::seed_from_u64(0xC0BE_5EED);
        let a = crabomination::cube::cube_deck([Color::Red, Color::Black], &mut r);
        let b = crabomination::cube::cube_deck([Color::White, Color::Blue], &mut r);
        (a, b)
    };
    let (a1, b1) = decks();
    let (a2, b2) = decks();
    assert_eq!((&a1, &b1), (&a2, &b2), "cube deck construction is not seeded");
    let (t1, t2) = (
        trace_game(&a1, &b1, 11, MAX_ACTIONS),
        trace_game(&a2, &b2, 11, MAX_ACTIONS),
    );
    if t1.digest() != t2.digest() {
        let at = t1.lines.iter().zip(&t2.lines).position(|(x, y)| x != y);
        panic!(
            "a cube game is not reproducible; first divergence at {at:?}:\n  run 1: {:?}\n  \
             run 2: {:?}",
            at.and_then(|i| t1.lines.get(i)),
            at.and_then(|i| t2.lines.get(i)),
        );
    }
}
