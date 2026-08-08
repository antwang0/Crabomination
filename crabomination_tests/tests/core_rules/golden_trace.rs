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
const DIGESTS: &[(u64, Option<usize>, u32, usize, u64)] = &[
    (1, Some(0), 11, 269, 0x8573_fe96_81c5_5749),
    (2, Some(1), 16, 404, 0x5678_1950_8088_3a6b),
    (3, Some(1), 20, 480, 0x5930_42b4_db37_3c9a),
    (4, Some(0), 13, 330, 0x3947_01c0_e7bc_f137),
    (5, Some(0), 9, 218, 0xef00_5c18_bf9c_9af0),
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
