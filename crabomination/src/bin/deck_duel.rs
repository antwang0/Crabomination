//! Play two specific decklists head to head and report an honest edge.
//!
//! `recommend_pool` answers "which build out of this pool", and `bot_ladder`
//! answers "which pilot", mirror-matched so the decks cancel. Neither
//! answers the question you actually have once two judges disagree about a
//! pool: *these two 40-card lists — which one do I sleeve?* That comparison
//! has no candidate enumeration and no mirror; it is one decklist against
//! another, and it needs its own binary.
//!
//! Both lists are piloted identically (the adopted default profile), so the
//! only difference between the seats is the seventy-five cards. Games are
//! played in **antithetic seat pairs** — the same shuffle from both sides —
//! which is worth roughly 2–4× the effective sample here for the same
//! reason it is on the ladder: in a 40-card deck the deal is most of the
//! variance, and replaying it cancels rather than averages. The realized
//! within-pair correlation is measured and printed, never assumed.
//!
//! Decklists are the same format `recommend_pool` reads (`N card name` per
//! line, `#` comments). Basics must be named outright — `8 Plains`, not
//! "8 White basic".
//!
//! ```text
//! deck_duel a.txt b.txt                 # 500 pairs = 1000 games, seed 0
//! deck_duel a.txt b.txt 2000 7          # 2000 pairs, seed 7
//! ```

use crabomination::cube::CardFactory;
use crabomination::recommend::{Pilot, paired_stat, simulate_match_pairs_piloted, wilson};

/// Long enough that a grindy limited game finishes; games that hit it are
/// reported as undecided rather than scored.
const MAX_ACTIONS: usize = 4_000;

fn load(path: &str) -> Vec<CardFactory> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}");
        std::process::exit(2);
    });
    let parse = crabomination::decklist::parse_decklist(&text);
    if !parse.unknown.is_empty() {
        eprintln!("{path}: unresolved names — fix these before playing:");
        for u in &parse.unknown {
            eprintln!("  {u}");
        }
        std::process::exit(1);
    }
    let deck = parse.main;
    // A sideboard in this context is almost certainly a mis-formatted
    // maindeck rather than a real one; folding it in silently would play a
    // deck the file does not describe.
    if !parse.sideboard.is_empty() {
        eprintln!(
            "{path}: {} sideboard cards ignored — deck_duel plays maindecks only",
            parse.sideboard.len()
        );
    }
    if deck.len() < 40 {
        eprintln!("{path}: only {} cards — expected at least 40", deck.len());
        std::process::exit(1);
    }
    deck
}

fn main() {
    // 32 MB because a debug build needs ~6 MB here and the default main
    // thread gets 8 — too close to rely on. `bot_ladder` and
    // `selfplay_train` carry the same workaround; this is what it is for.
    //
    // It is *not* deep recursion: the trace that overflowed 8 MB was 18
    // frames. `run_effect` (the match over every card effect in
    // `effects/mod.rs`) compiles to a ~1.8 MB stack frame at opt-level 0,
    // because rustc without optimization gives every local in every match
    // arm its own slot instead of colouring non-overlapping ones onto the
    // same storage — so the frame is roughly the *sum* over ~900 arms
    // when only one ever runs. Effects resolving sub-effects nest it ~3
    // deep. Release collapses it; the same run is happy on 1 MB.
    //
    // It was 2.56 MB until `TokenDefinition` was boxed out of `Effect`
    // (which took `size_of::<Effect>()` from 1464 to 448 bytes). The
    // remaining headroom is ~17 levels of `run_effect` nesting in debug,
    // which no card comes close to but nothing enforces.
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(run)
        .expect("spawn simulation thread")
        .join()
        .expect("simulation thread panicked");
}

fn run() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.len() < 2 {
        eprintln!("usage: deck_duel <deck_a.txt> <deck_b.txt> [pairs] [seed]");
        eprintln!("  pairs default 500 (each pair is 2 games, seats swapped on one shuffle)");
        std::process::exit(2);
    }
    let pairs: usize = argv.get(2).and_then(|s| s.parse().ok()).unwrap_or(500);
    let seed: u64 = argv.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);

    let (path_a, path_b) = (&argv[0], &argv[1]);
    let (deck_a, deck_b) = (load(path_a), load(path_b));
    println!(
        "A: {} ({} cards)\nB: {} ({} cards)",
        path_a,
        deck_a.len(),
        path_b,
        deck_b.len()
    );
    println!("{pairs} antithetic pairs = {} games, seed {seed}, identical pilots", pairs * 2);

    let tally = simulate_match_pairs_piloted(
        &deck_a,
        &deck_b,
        pairs,
        [Pilot::default(), Pilot::default()],
        MAX_ACTIONS,
        seed,
    );

    let decided = tally.wins_a + tally.wins_b;
    let (lo, hi) = wilson(tally.wins_a, decided, 1.96);
    println!(
        "\nunpaired: A {} - {} B ({} undecided) = {:.1}% [{:.1}%, {:.1}%]",
        tally.wins_a,
        tally.wins_b,
        tally.undecided,
        100.0 * tally.wins_a as f64 / decided.max(1) as f64,
        lo * 100.0,
        hi * 100.0,
    );

    let Some(stat) = paired_stat(&tally.pairs) else {
        println!("too few decided pairs for a paired estimate");
        return;
    };
    let sweeps_a = tally.pairs.iter().filter(|&&s| s > 0).count();
    let sweeps_b = tally.pairs.iter().filter(|&&s| s < 0).count();
    let splits = tally.pairs.iter().filter(|&&s| s == 0).count();
    let (plo, phi) = (stat.p - 1.96 * stat.se, stat.p + 1.96 * stat.se);
    println!(
        "paired:   {} pairs — {sweeps_a} A-sweeps, {sweeps_b} B-sweeps, {splits} splits",
        stat.n
    );
    println!(
        "          A win% {:.1}%  [{:.1}%, {:.1}%]  (+/-{:.2} pts)",
        stat.p * 100.0,
        plo * 100.0,
        phi * 100.0,
        1.96 * stat.se * 100.0
    );
    // The efficiency the pairing actually bought, not the one it should
    // have: rho > 0 would mean pairing cost precision instead of buying it.
    let factor = 1.0 + stat.rho;
    println!(
        "          within-pair rho {:.3} — variance x{:.2} vs independent games, i.e. these {} \
         games carry the precision of {:.0}",
        stat.rho,
        factor,
        stat.n * 2,
        if factor > 1e-9 { stat.n as f64 * 2.0 / factor } else { f64::INFINITY }
    );

    println!(
        "\nverdict: {}",
        if plo > 0.5 {
            format!("A ({path_a}) is stronger — the interval is entirely above 50%")
        } else if phi < 0.5 {
            format!("B ({path_b}) is stronger — the interval is entirely below 50%")
        } else {
            "inconclusive — the interval straddles 50%".to_string()
        }
    );
}
