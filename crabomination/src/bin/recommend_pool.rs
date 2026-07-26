//! Rank sealed-deck builds for a pool from the terminal.
//!
//! ```text
//! cargo run -p crabomination --bin recommend_pool -- pool.txt [seed] [games_per_pairing]
//! ```
//!
//! The pool file is a plain decklist ("2 Zimone's Experiment" per line,
//! Arena/MTGO shapes accepted). Unresolved names are reported and the run
//! aborts — a sealed ranking with silently missing cards is worse than
//! no ranking.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use crabomination::recommend::{self, SimConfig};

/// Case/space-insensitive label key: `"u/b + g"` ≡ `"U/B+g"`.
fn normalize_label(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_lowercase()
}

/// Per-card attribution across a refined variant fleet: each card's mean
/// win rate over variants that play it vs those that don't. Deck-level
/// results can't credit single cards; this can (noisily — the counts are
/// shown for a reason). Only names appearing in AND missing from ≥3
/// variants are comparable.
fn print_attribution(rec: &recommend::Recommendation) {
    let n = rec.evals.len();
    if n < 6 {
        return;
    }
    let mut per_card: HashMap<&'static str, (Vec<f64>, Vec<f64>)> = HashMap::new();
    let all_names: std::collections::HashSet<&'static str> = rec.candidates[..n]
        .iter()
        .flat_map(|c| c.main.iter().chain(c.duals.iter()).map(|&f| f().name))
        .collect();
    for (i, c) in rec.candidates[..n].iter().enumerate() {
        let wr = rec.evals[i].win_rate();
        let in_deck: std::collections::HashSet<&'static str> =
            c.main.iter().chain(c.duals.iter()).map(|&f| f().name).collect();
        for name in &all_names {
            let entry = per_card.entry(name).or_default();
            if in_deck.contains(name) {
                entry.0.push(wr);
            } else {
                entry.1.push(wr);
            }
        }
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    let mut rows: Vec<(&str, f64, usize, f64, usize, f64)> = per_card
        .into_iter()
        .filter(|(_, (i, o))| i.len() >= 3 && o.len() >= 3)
        .map(|(name, (i, o))| {
            let (mi, mo) = (mean(&i), mean(&o));
            (name, mi, i.len(), mo, o.len(), mi - mo)
        })
        .collect();
    rows.sort_by(|a, b| b.5.partial_cmp(&a.5).unwrap_or(std::cmp::Ordering::Equal));
    println!("\nper-card attribution (mean win rate with vs without; noisy — mind the counts):");
    for (name, mi, ni, mo, no, d) in rows {
        println!(
            "  {:+5.1}  {:5.1}% (in {ni:>2})  vs {:5.1}% (out {no:>2})  {name}",
            d * 100.0,
            mi * 100.0,
            mo * 100.0,
        );
    }
}

fn print_ranking(rec: &recommend::Recommendation) {
    for (rank, &i) in rec.ranking.iter().enumerate() {
        let e = &rec.evals[i];
        let c = &rec.candidates[i];
        let tag = match e.eliminated_round {
            Some(r) => format!("eliminated round {r}"),
            None => String::from("finalist"),
        };
        println!(
            "  {}. {:>16}  {:5.1}% ± {:4.1}  (n={:4}, {tag})",
            rank + 1,
            c.label,
            e.win_rate() * 100.0,
            e.ci_halfwidth(1.96) * 100.0,
            e.decided(),
        );
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!(
            "usage: recommend_pool <pool.txt> [seed] [games_per_pairing] [candidate_cap] \
             [pin,labels] [refine_top] [variants_per_shape]"
        );
        std::process::exit(2);
    };
    let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let games: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(20);
    let cap: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(8);
    // Comma-separated labels ("U/B+g,B/G+u") to force into the simulated
    // set regardless of static rank — for testing a pet build the static
    // score undervalues.
    let pins: Vec<String> = args
        .next()
        .map(|s| s.split(',').map(|p| normalize_label(p)).collect())
        .unwrap_or_default();
    // Stage-2 refinement: variants per top shape. 0 disables (default).
    let refine_top: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let variants: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(6);
    let racing_rounds: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(3);

    // `random:SEED` generates a synthetic sealed pool (6 SOS packs) instead
    // of reading a file — for calibrating what a typical pool's best build
    // scores against the same field.
    let pool: Vec<_> = if let Some(pool_seed) = path.strip_prefix("random:") {
        use rand::SeedableRng;
        let pool_seed: u64 = pool_seed.parse().unwrap_or_else(|_| {
            eprintln!("random:<u64 seed> expected, got {path}");
            std::process::exit(2);
        });
        let mut rng = rand::rngs::StdRng::seed_from_u64(pool_seed);
        let sos = crabomination::draft::sos_draft_pool();
        let pool: Vec<_> =
            (0..6).flat_map(|_| crabomination::draft::generate_sos_pack(&sos, &mut rng)).collect();
        println!("pool: {} cards (synthetic, pool seed {pool_seed})", pool.len());
        pool
    } else {
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("cannot read {path}: {e}");
            std::process::exit(2);
        });
        let parse = crabomination::decklist::parse_decklist(&text);
        if !parse.unknown.is_empty() {
            eprintln!("unresolved names — fix these before ranking:");
            for u in &parse.unknown {
                eprintln!("  {u}");
            }
            std::process::exit(1);
        }
        let mut pool = parse.main;
        pool.extend(parse.sideboard);
        println!("pool: {} cards, all resolved", pool.len());
        pool
    };

    let cfg = SimConfig {
        seed,
        games_per_pairing: games,
        candidate_cap: cap,
        refine_top,
        variants_per_shape: variants,
        racing_rounds,
        ..Default::default()
    };
    let mut candidates = recommend::enumerate_candidates(&pool, &cfg);
    // Pull pinned labels up into the simulated top-K (keeping their
    // relative static order below the organic qualifiers).
    if !pins.is_empty() {
        let mut missing: Vec<&String> = pins
            .iter()
            .filter(|p| !candidates.iter().any(|c| normalize_label(&c.label) == **p))
            .collect();
        if !missing.is_empty() {
            missing.sort();
            eprintln!("warning: pinned label(s) not among enumerated candidates: {missing:?}");
        }
        // Pins first (each group keeps its static order); the cap then
        // covers all pins plus the best organic qualifiers.
        candidates.sort_by_key(|c| {
            let pinned = pins.contains(&normalize_label(&c.label));
            (!pinned, std::cmp::Reverse(c.static_score))
        });
    }
    println!("\ntop candidates by static score{}:", if pins.is_empty() { "" } else { " (pins first)" });
    for c in candidates.iter().take(cap.max(10)) {
        println!("  {:>10}  score {:>4}  ({} spells)", c.label, c.static_score, c.main.len());
    }

    println!(
        "\nsimulating top {} vs a {}-deck gauntlet (seed {seed}, racing {}) …",
        cfg.candidate_cap.min(candidates.len()),
        cfg.gauntlet_size,
        if cfg.racing { "on" } else { "off" },
    );
    let progress = AtomicUsize::new(0);
    let rec = recommend::recommend_prepared(candidates, &cfg, |evals| {
        // One status line every ~40 finished jobs.
        let n = progress.fetch_add(1, Ordering::Relaxed);
        if n % 40 == 0 {
            let total: u32 = evals.iter().map(|e| e.decided() + e.undecided).sum();
            eprint!("\r  {total} games played …");
        }
    });
    eprintln!();

    println!("\nfinal ranking (win rate vs the field ± 95% CI):");
    print_ranking(&rec);

    // Stage 2: variant refinement of the winning shapes.
    let rec = if refine_top > 0 {
        println!(
            "\nrefining top {refine_top} shapes × {variants} variants (same gauntlet) …",
        );
        let refined = recommend::refine(&pool, &rec, &cfg, |evals| {
            let n = progress.fetch_add(1, Ordering::Relaxed);
            if n % 40 == 0 {
                let total: u32 = evals.iter().map(|e| e.decided() + e.undecided).sum();
                eprint!("\r  {total} variant games played …");
            }
        });
        eprintln!();
        println!("\nrefined ranking (win rate vs the field ± 95% CI):");
        print_ranking(&refined);
        print_attribution(&refined);
        refined
    } else {
        rec
    };

    let best = &rec.candidates[rec.ranking[0]];
    println!("\nrecommended build — {} ({} spells + {} lands):", best.label, best.main.len(), cfg.total_lands);
    let mut counts: HashMap<&str, u32> = HashMap::new();
    for &f in &best.main {
        *counts.entry(f().name).or_insert(0) += 1;
    }
    let mut lines: Vec<(u32, &str)> = counts.into_iter().map(|(n, c)| (c, n)).collect();
    lines.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(b.1)));
    for (n, name) in lines {
        println!("  {n} {name}");
    }
    let mut dual_counts: HashMap<&str, u32> = HashMap::new();
    for &f in &best.duals {
        *dual_counts.entry(f().name).or_insert(0) += 1;
    }
    let mut dual_lines: Vec<(&str, u32)> = dual_counts.into_iter().collect();
    dual_lines.sort();
    for (name, n) in dual_lines {
        println!("  {n} {name}");
    }
    let mut basics: Vec<_> = best.basics.iter().collect();
    basics.sort_by_key(|(c, _)| format!("{c:?}"));
    for (c, n) in basics {
        if *n > 0 {
            println!("  {n} {c:?} basic");
        }
    }
    let pin_arg =
        if pins.is_empty() { String::new() } else { format!(" \"{}\"", pins.join(",")) };
    println!("\nreproduce with: recommend_pool {path} {seed} {games} {cap}{pin_arg}");
}
