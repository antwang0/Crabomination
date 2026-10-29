//! Rate Reality Fracture cards by simulated sealed play, for comparison
//! against an external pick order (Draftsim's FRA ratings).
//!
//! Opens `--decks` FRA sealed pools (six 14-card boosters: one rare or
//! mythic, three uncommons, ten commons, rarities from the ratings file),
//! builds each with the engine's v3 sealed builder, then plays `--rounds`
//! random pairings of the decks, `--pairs` seat-swapped game pairs each,
//! under one pilot profile (`dflt` or `net67`). Writes three tables to
//! `--out DIR` for `scripts/fra_card_eval.py`:
//!
//! * `cards.tsv`  — per card: rarity, the builder's static score
//!   (`base_score + quality`), pools it appeared in, decks it made.
//! * `decks.tsv`  — deck id, then its 40 card names.
//! * `games.tsv`  — deck a, deck b, games a won, games b won, undecided.
//!
//! Usage: `fra_card_eval --ratings FRA.tsv --out DIR [--pilot dflt|net67]
//! [--decks 2000] [--rounds 20] [--pairs 2] [--seed 1] [--explore 0]
//! [--threads N]`. `--explore N` swaps N of each deck's spells for left-out
//! on-colour pool cards, so cards the builder never plays still get rated.
//! `net67` loads `CRAB_NET` (default `nets/champion.safetensors`).
//!
//! `fra_card_eval --build POOL.txt [--candidates 64]` is the sealed helper:
//! the pool (a decklist — one card per line, `N Name`, Arena or MTGO export)
//! built and every pool card ranked by its measured value, marked `*` where
//! the build plays it. By default it builds every two-colour pair (and the
//! builder's own pick), races them against one fixed field (`--field 400
//! --pairs 4 --seed 43`, piloted by `--pilot`, default `fra-fixes`) and
//! keeps the best: the builder's colour choice by summed card score is
//! 2.9 / 3.5 points worse than choosing by play (`--audit-builder`, 200 / 150
//! pools, chosen on one field and judged on another), and 12 points off on
//! one real pool. Then it polishes the winner by play ([`polish`], up to
//! `--polish-rounds 4` swaps, `--no-polish` skips it; about four minutes):
//! on the user's pool the polished build gained +3.62 ±1.47 on a field no
//! choice saw and tied the user's hand-tuned list (+0.56 ±1.77). `--static`
//! is the old rated build (best of N by static score, under a second);
//! `--colors WG` confines the static build to those colours.
//!
//! `fra_card_eval --audit-builder [--pools 200] [--field 60] [--pairs 2]
//! [--seed 43] --out FILE.tsv` measures the builder itself: per random FRA
//! pool, its pick against variants of it (one more land in place of the
//! weakest spell; every two-colour pair; with `CRAB_AUDIT_LANDS`, fixed
//! 17- and 18-land builds; with `CRAB_AUDIT_SWAPS`, the swaps that tell
//! those apart; `CRAB_AUDIT_NO_PAIRS` skips the pairs), each raced on two
//! independent fields.
//!
//! `fra_card_eval --gauntlet A.txt B.txt ... [--field 400] [--pairs 4]
//! [--seed 43] [--pilot dflt] [--field-pilot P]` plays each 40-card list
//! (piloted by `--pilot`, the field by `--field-pilot`, default the same) against one fixed
//! field of FRA sealed opponents (`fra_mode::FraPacks` pools, each the
//! best-of-16 rated build), seat-swapped pairs, the same opponent and seed
//! for every list. The unit is the opponent: each list's win rate is the
//! opponent-level mean, and every later list is also read as its paired
//! difference against the first.
//!
//! `--set sos` rates Secrets of Strixhaven instead (`draft::SosPacks` pools,
//! no `--ratings` file); `--ab` takes it too.
//!
//! `fra_card_eval --ab [--pools 2000] [--pairs 8] [--seed 43] [--pilot P] [--best N]`
//! is the builder race instead: per FRA pool (`fra_mode::FraPacks`, no
//! ratings file), the v3 build against the rated build
//! (`SimConfig::rated_cards`, `card_ratings`), seat-swapped pairs under one
//! pilot. The unit is the pool — one deck pair, one draw from each builder —
//! so the verdict is the pool-level t interval, not a game-level one
//! (round "builder v3": 12 pools carried ±6.5 however many games each got).

use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

use crabomination::card::CardDefinition;
use crabomination::catalog::sets::all_factories::all_catalog_card_factories;
use crabomination::mana::{Color, ColorSet};
use crabomination::recommend::{Pilot, simulate_match_pairs_piloted};
use crabomination::server::EvalWeights;
use rand::rngs::StdRng;
use rand::seq::{IndexedRandom, SliceRandom};
use rand::{RngExt, SeedableRng};

type Factory = fn() -> CardDefinition;

struct Card {
    name: String,
    rarity: char,
    f: Factory,
}

fn arg(name: &str) -> Option<String> {
    let a: Vec<String> = std::env::args().collect();
    a.iter().position(|x| x == name).and_then(|i| a.get(i + 1).cloned())
}

fn num(name: &str, default: usize) -> usize {
    arg(name).map(|v| v.parse().expect("numeric flag")).unwrap_or(default)
}

/// `--set sos`: Secrets of Strixhaven pools (`draft::SosPacks`) instead of
/// FRA's. The rating run, `--ab` and nothing else read it.
fn sos() -> bool {
    arg("--set").as_deref() == Some("sos")
}

/// Six boosters of the `--set`.
fn roll_pool(rng: &mut StdRng) -> Vec<Factory> {
    if sos() {
        let packs = crabomination::draft::SosPacks::sos_default();
        return (0..6).flat_map(|_| packs.roll(rng)).collect();
    }
    let packs = crabomination::fra_mode::FraPacks::get();
    (0..6).flat_map(|_| packs.roll(rng)).collect()
}

/// The SOS rating run's card list: every nonland card on the SOS sheet
/// (no external ratings file; rarity unknown, written `?`).
fn sos_cards() -> Vec<Card> {
    let mut seen = std::collections::HashSet::new();
    crabomination::draft::sos_draft_pool_ref()
        .iter()
        .copied()
        .filter(|f| !crabomination::cube::card_brief(*f).is_land && seen.insert(*f as usize))
        .map(|f| Card { name: crabomination::cube::card_brief(f).def.name.to_string(), rarity: '?', f })
        .collect()
}

fn load_cards(path: &str) -> Vec<Card> {
    let mut by_name: HashMap<String, Factory> = HashMap::new();
    for f in all_catalog_card_factories() {
        let d = f();
        let front = d.name.split(" // ").next().unwrap_or(d.name).to_string();
        by_name.entry(d.name.to_string()).or_insert(f);
        by_name.entry(front).or_insert(f);
    }
    let text = std::fs::read_to_string(path).expect("ratings file");
    let mut out = Vec::new();
    for line in text.lines().skip(1) {
        let col: Vec<&str> = line.split('\t').collect();
        if col.len() < 8 || col[7] != "base" {
            continue;
        }
        let name = col[0].replace('_', " ");
        let rarity = col[5].chars().next().unwrap_or('C');
        match by_name.get(&name) {
            Some(&f) => out.push(Card { name, rarity, f }),
            None => eprintln!("not in catalog: {name}"),
        }
    }
    out
}

fn open_pack(by_rarity: &HashMap<char, Vec<usize>>, rng: &mut StdRng) -> Vec<usize> {
    let mut pack = Vec::with_capacity(14);
    let rare = if rng.random_ratio(1, 8) && !by_rarity[&'M'].is_empty() { 'M' } else { 'R' };
    pack.push(*by_rarity[&rare].choose(rng).unwrap());
    for (r, n) in [('U', 3), ('C', 10)] {
        pack.extend(by_rarity[&r].sample(rng, n).copied());
    }
    pack
}

/// `--ab`: the rated builder against the v3 builder it would replace.
/// `--best N` races the client's sealed-opponent recipes instead: best of
/// N rated builds against `selfplay::v3_opponent_build` (best of N, curve);
/// `--best N --shape` races the rated judge with its deck-shape term
/// (`card_ratings::shape_penalty`) against the judge without it;
/// `--best N --slots` half the candidates built to that shape
/// (`selfplay::build_candidates_shaped`) against none, the judge's term on.
fn race_builders(pilot: Pilot, pools: usize, pairs: usize, seed: u64, threads: usize, best: usize) {
    let shape = std::env::args().any(|a| a == "--shape");
    let slots = std::env::args().any(|a| a == "--slots");
    // `--old-duals`: the rated builder against itself blind to the "{T}: add
    // {W} or {U}" duals it could not see before 2026-10-29.
    let old_duals = std::env::args().any(|a| a == "--old-duals");
    // `--trial polish|floor|both`: the rated builder with the judge polish
    // and/or the double-pip land floor against itself with neither.
    let trial: Option<(bool, bool)> = arg("--trial").map(|t| match t.as_str() {
        "polish" => (true, false),
        "floor" => (false, true),
        "both" => (true, true),
        other => panic!("--trial {other}: polish, floor or both"),
    });
    // `--table FILE` (`scripts/fra_card_table.py RATINGS MIN --tsv`): the
    // rated builder on a re-rated table against the same builder on the
    // compiled one — a re-rate's paired gate.
    let table: Option<crabomination::card_ratings::TableOverride> = arg("--table").map(|path| {
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let map = text
            .lines()
            .filter_map(|l| l.split_once('\t'))
            .map(|(n, v)| (n.to_string(), v.trim().parse::<i32>().ok()))
            .collect();
        std::sync::Arc::new(map)
    });
    let next = AtomicUsize::new(0);
    let rows = std::sync::Mutex::new(Vec::with_capacity(pools));
    let start = std::time::Instant::now();
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= pools {
                    break;
                }
                let salt = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (i as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93);
                let mut rng = StdRng::seed_from_u64(salt);
                let pool = roll_pool(&mut rng);
                let rated = |pool: &[Factory]| {
                    if best > 0 {
                        crabomination::selfplay::rated_sealed_build(pool, best, salt ^ 1)
                    } else {
                        crabomination::selfplay::heuristic_sealed_build_rated(pool, salt ^ 1)
                    }
                };
                let (old, new) = if let Some((polish, floor)) = trial {
                    let arm = |p: bool, f: bool| {
                        crabomination::selfplay::with_judge_polish(p, || {
                            crabomination::recommend::with_pip_floor(f, || rated(&pool))
                        })
                    };
                    (arm(false, false), arm(polish, floor))
                } else if old_duals {
                    (crabomination::recommend::with_old_duals(|| rated(&pool)), rated(&pool))
                } else if let Some(t) = &table {
                    (rated(&pool), crabomination::card_ratings::with_table(t.clone(), || rated(&pool)))
                } else if best > 0 && slots {
                    (
                        crabomination::selfplay::rated_sealed_build_slots(&pool, best, salt ^ 1, false),
                        crabomination::selfplay::rated_sealed_build_slots(&pool, best, salt ^ 1, true),
                    )
                } else if best > 0 && shape {
                    (
                        crabomination::selfplay::rated_sealed_build_shaped(&pool, best, salt ^ 1, false),
                        crabomination::selfplay::rated_sealed_build_shaped(&pool, best, salt ^ 1, true),
                    )
                } else if best > 0 {
                    (
                        crabomination::selfplay::v3_opponent_build(&pool, best, salt ^ 1, ColorSet::empty()),
                        crabomination::selfplay::rated_sealed_build(&pool, best, salt ^ 1),
                    )
                } else {
                    (
                        crabomination::selfplay::heuristic_sealed_build_v3(&pool, salt ^ 1),
                        crabomination::selfplay::heuristic_sealed_build_rated(&pool, salt ^ 1),
                    )
                };
                let same = {
                    let key = |d: &[Factory]| {
                        let mut v: Vec<usize> = d.iter().map(|f| *f as usize).collect();
                        v.sort_unstable();
                        v
                    };
                    key(&old) == key(&new)
                };
                let t = simulate_match_pairs_piloted(&new, &old, pairs, [pilot, pilot], 50_000, salt ^ 2);
                let n = done_count(&rows, (t.wins_a, t.wins_b, same));
                if n % 250 == 0 {
                    eprintln!("{n}/{pools} pools, {:.0}s", start.elapsed().as_secs_f64());
                }
            });
        }
    });
    let rows = rows.into_inner().unwrap();
    let identical = rows.iter().filter(|r| r.2).count();
    let shares: Vec<f64> = rows
        .iter()
        .filter(|r| r.0 + r.1 > 0)
        .map(|r| 100.0 * r.0 as f64 / (r.0 + r.1) as f64)
        .collect();
    let n = shares.len() as f64;
    let mean = shares.iter().sum::<f64>() / n;
    let sd = (shares.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
    let half = 1.96 * sd / n.sqrt();
    let (wa, wb): (u32, u32) = rows.iter().fold((0, 0), |a, r| (a.0 + r.0, a.1 + r.1));
    println!(
        "{}: {mean:.2}% [{:.2}, {:.2}] pool-level 95% (n={} pools, {identical} identical decks); games {wa}-{wb} ({:.2}%)",
        if table.is_some() { "table vs compiled" } else { "rated vs v3" },
        mean - half,
        mean + half,
        shares.len(),
        100.0 * wa as f64 / (wa + wb) as f64
    );
}

/// `CRAB_NET` (default the committed champion) into the net leaf's slot.
fn load_net() {
    let path = std::env::var("CRAB_NET").unwrap_or_else(|_| "nets/champion.safetensors".into());
    crabomination::server::net_eval::load_slot(crabomination::server::net_eval::SLOT_BEST, std::path::Path::new(&path))
        .expect("load net");
    eprintln!("loaded value net from {path}");
}

fn done_count<T>(rows: &std::sync::Mutex<Vec<T>>, row: T) -> usize {
    let mut r = rows.lock().unwrap();
    r.push(row);
    r.len()
}

/// `"WG"` / `"ug"` as a colour set.
fn parse_colors(spec: &str) -> ColorSet {
    let mut set = ColorSet::empty();
    for ch in spec.chars() {
        set.insert(match ch.to_ascii_uppercase() {
            'W' => Color::White,
            'U' => Color::Blue,
            'B' => Color::Black,
            'R' => Color::Red,
            'G' => Color::Green,
            other => panic!("--colors {spec}: {other:?} is not one of WUBRG"),
        });
    }
    set
}

/// A decklist file's main deck and sideboard, warning on unknown names.
fn read_list(path: &str) -> (Vec<Factory>, Vec<Factory>) {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let parsed = crabomination::decklist::parse_decklist(&text);
    for u in &parsed.unknown {
        eprintln!("{path}: not in the catalog: {} {:?} (did you mean {:?})", u.count, u.name, u.suggestions);
    }
    (parsed.main, parsed.sideboard)
}

/// `--build` (unless `--static`): how the candidate builds are raced.
struct ByPlay {
    pilot: Pilot,
    field: usize,
    pairs: usize,
    seed: u64,
    threads: usize,
}

/// `--build`: the rated build in each two-colour pair, plus the builder's
/// own pick, all played against one fixed field; the best by win share.
///
/// The builder picks colours by its summed card scores, and that sum is not
/// a deck's strength: on the user's pool (2026-10-25) its pick was W/U at
/// 50.1 % against the field while its own W/G build in the same pool won
/// 62.7 %.
fn build_by_play(pool: &[Factory], candidates: usize, bp: &ByPlay) -> Vec<Factory> {
    use crabomination::selfplay::rated_sealed_build_in_colors;
    let wubrg = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
    let mut cands: Vec<(String, Vec<Factory>)> =
        vec![("builder's pick".to_string(), rated_sealed_build_in_colors(pool, candidates, 0x5EA1, ColorSet::empty()))];
    for i in 0..5 {
        for j in i + 1..5 {
            let mut set = ColorSet::empty();
            set.insert(wubrg[i]);
            set.insert(wubrg[j]);
            let name = format!("{}{}", wubrg[i].short_name(), wubrg[j].short_name());
            cands.push((name, rated_sealed_build_in_colors(pool, candidates, 0x5EA1, set)));
        }
    }
    let key = |d: &[Factory]| {
        let mut k: Vec<&'static str> = d.iter().map(|f| crabomination::cube::card_brief(*f).def.name).collect();
        k.sort_unstable();
        k
    };
    let mut seen: Vec<Vec<&'static str>> = Vec::new();
    cands.retain(|(_, d)| {
        let k = key(d);
        let fresh = d.len() == 40 && !seen.contains(&k);
        if fresh {
            seen.push(k);
        }
        fresh
    });
    let lists: Vec<Vec<Factory>> = cands.iter().map(|(_, d)| d.clone()).collect();
    let rows = field_rows(&lists, bp.pilot, bp.pilot, bp.field, bp.pairs, bp.seed, bp.threads);
    let mut scored: Vec<(f64, f64, usize)> =
        (0..cands.len()).map(|d| { let (m, h) = field_share(&rows, d); (m, h, d) }).collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    println!(
        "Builds raced against {} FRA sealed opponents x {} seat-swapped pairs (seed {}; 95% intervals):",
        bp.field, bp.pairs, bp.seed
    );
    for &(m, h, d) in &scored {
        println!("  {m:6.2} ±{h:4.2}  {}", cands[d].0);
    }
    println!();
    cands.swap_remove(scored[0].2).1
}

/// A card's measured value (`FRA_SIM_VALUE`), when it has one.
fn sim_value(f: Factory) -> Option<f32> {
    let name = crabomination::cube::card_brief(f).def.name;
    crabomination::card_ratings::FRA_SIM_VALUE.iter().find(|(n, _)| *n == name).map(|(_, v)| *v)
}

/// `--build` after the by-play race: swap search by play (`--no-polish`
/// skips it). Per-card values add up, so they cannot see a deck's shape — a
/// creature count, a curve, a spell that needs a target. Each round makes
/// every swap of the deck's six lowest-valued spells for the six best
/// unplayed spells in its colours (plus a spell for a basic and a basic
/// for a spell), screens them on one field, re-races the top three against
/// the current deck on a fresh field, and keeps the best that wins there.
/// The kept deck is then judged against the unpolished one on a third field
/// neither choice saw.
fn polish(pool: &[Factory], start: Vec<Factory>, bp: &ByPlay, rounds: usize) -> Vec<Factory> {
    let brief = |f: Factory| crabomination::cube::card_brief(f);
    let name = |f: Factory| brief(f).def.name;
    let value = |f: Factory| sim_value(f).unwrap_or(0.0);
    let screen = |lists: &[Vec<Factory>], seed: u64| {
        let rows = field_rows(lists, bp.pilot, bp.pilot, (bp.field / 2).max(50), 2, seed, bp.threads);
        (0..lists.len()).map(|d| field_share(&rows, d).0).collect::<Vec<f64>>()
    };
    // List `d`'s paired gain over list 0 on a field: (mean, 95% half-width).
    let paired = |rows: &[Vec<(u32, u32)>], d: usize| {
        let diffs: Vec<f64> = rows.iter().filter_map(|r| Some(win_share(r[d])? - win_share(r[0])?)).collect();
        mean_ci(&diffs)
    };
    let mut deck = start.clone();
    println!("Polishing by play ({} screening opponents x 2 pairs, re-raced on fresh fields of {} x {}):", (bp.field / 2).max(50), bp.field, bp.pairs);
    for round in 0..rounds {
        let mut colors = ColorSet::empty();
        for f in deck.iter().filter(|f| brief(**f).is_land) {
            for c in brief(*f).produces.iter() {
                colors.insert(c);
            }
        }
        let mut left: Vec<Factory> = pool.to_vec();
        for f in &deck {
            if let Some(i) = left.iter().position(|g| *g as usize == *f as usize) {
                left.swap_remove(i);
            }
        }
        let mut ins: Vec<Factory> = Vec::new();
        for f in left.iter().copied().filter(|f| !brief(*f).is_land && brief(*f).fits(colors)) {
            if !ins.iter().any(|g| name(*g) == name(f)) {
                ins.push(f);
            }
        }
        ins.sort_by(|a, b| value(*b).total_cmp(&value(*a)));
        ins.truncate(6);
        let mut outs: Vec<usize> = Vec::new();
        let mut spells: Vec<usize> = (0..deck.len()).filter(|&i| !brief(deck[i]).is_land).collect();
        spells.sort_by(|&a, &b| value(deck[a]).total_cmp(&value(deck[b])));
        for i in spells {
            if outs.len() < 6 && !outs.iter().any(|&j| name(deck[j]) == name(deck[i])) {
                outs.push(i);
            }
        }
        let lands = deck.iter().filter(|f| brief(**f).is_land).count();
        let basics: Vec<usize> = (0..deck.len())
            .filter(|&i| brief(deck[i]).is_land && brief(deck[i]).def.supertypes.contains(&crabomination::card::Supertype::Basic))
            .collect();
        // The heaviest colour's basic, for a spell → land swap.
        let heavy = {
            let mut pips: Vec<(Color, u32)> = colors.iter().map(|c| (c, 0)).collect();
            for f in deck.iter().filter(|f| !brief(**f).is_land) {
                for (c, n) in pips.iter_mut() {
                    *n += brief(*f).pips.get(*c);
                }
            }
            pips.into_iter().max_by_key(|p| p.1).map(|p| crabomination::draft::basic_land_factory(p.0))
        };
        let mut cands: Vec<(String, Vec<Factory>)> = Vec::new();
        for &o in &outs {
            for &i in &ins {
                let mut d = deck.clone();
                d[o] = i;
                cands.push((format!("-{} +{}", name(deck[o]), name(i)), d));
            }
            if let Some(h) = heavy.filter(|_| lands < 18) {
                let mut d = deck.clone();
                d[o] = h;
                cands.push((format!("-{} +{}", name(deck[o]), name(h)), d));
            }
        }
        if lands > 15 {
            let mut seen: Vec<&str> = Vec::new();
            for &b in &basics {
                if seen.contains(&name(deck[b])) {
                    continue;
                }
                seen.push(name(deck[b]));
                for &i in ins.iter().take(3) {
                    let mut d = deck.clone();
                    d[b] = i;
                    cands.push((format!("-{} +{}", name(deck[b]), name(i)), d));
                }
            }
        }
        if cands.is_empty() {
            break;
        }
        let mut lists: Vec<Vec<Factory>> = vec![deck.clone()];
        lists.extend(cands.iter().map(|(_, d)| d.clone()));
        let scores = screen(&lists, bp.seed ^ 0x9011_0000 ^ round as u64);
        let mut order: Vec<usize> = (1..lists.len()).filter(|&d| scores[d] > scores[0]).collect();
        order.sort_by(|&a, &b| scores[b].total_cmp(&scores[a]));
        order.truncate(3);
        if order.is_empty() {
            println!("  round {}: no swap of {} screens above the deck ({:.2})", round + 1, cands.len(), scores[0]);
            break;
        }
        let mut finals: Vec<Vec<Factory>> = vec![deck.clone()];
        finals.extend(order.iter().map(|&d| lists[d].clone()));
        let rows = field_rows(&finals, bp.pilot, bp.pilot, bp.field, bp.pairs, bp.seed ^ 0x9022_0000 ^ round as u64, bp.threads);
        let best = (1..finals.len())
            .map(|k| (k, paired(&rows, k)))
            .max_by(|a, b| a.1.0.total_cmp(&b.1.0))
            .unwrap();
        let (k, (gain, half)) = best;
        let label = &cands[order[k - 1] - 1].0;
        if gain <= 0.0 {
            println!("  round {}: best of {} screened swaps, {label}, re-races {gain:+.2} ±{half:.2}: stop", round + 1, cands.len());
            break;
        }
        println!("  round {}: {label}  {gain:+.2} ±{half:.2} on a fresh field ({} swaps screened)", round + 1, cands.len());
        deck = finals.swap_remove(k);
    }
    if deck.iter().map(|f| *f as usize).ne(start.iter().map(|f| *f as usize)) {
        let rows = field_rows(&[start, deck.clone()], bp.pilot, bp.pilot, bp.field, bp.pairs, bp.seed ^ 0x9033_0000, bp.threads);
        let (g, h) = paired(&rows, 1);
        println!("  polished vs unpolished on a field neither saw: {g:+.2} ±{h:.2}");
    }
    println!();
    deck
}

/// `--build`: the rated build of a player's pool, and the pool ranked.
fn build_pool(path: &str, candidates: usize, colors: ColorSet, by_play: Option<ByPlay>) {
    let (main, side) = read_list(path);
    let pool: Vec<Factory> = main.iter().chain(&side).copied().collect();
    let mut deck = match &by_play {
        Some(bp) if colors.is_empty() => build_by_play(&pool, candidates, bp),
        _ => crabomination::selfplay::rated_sealed_build_in_colors(&pool, candidates, 0x5EA1, colors),
    };
    if let Some(bp) = &by_play
        && std::env::args().all(|a| a != "--no-polish")
    {
        deck = polish(&pool, deck, bp, num("--polish-rounds", 4));
    }
    let brief = |f: Factory| crabomination::cube::card_brief(f);
    let mut spells: Vec<Factory> = deck.iter().copied().filter(|f| !brief(*f).is_land).collect();
    spells.sort_by_key(|f| (brief(*f).cmc, brief(*f).def.name));
    let lands: Vec<Factory> = deck.iter().copied().filter(|f| brief(*f).is_land).collect();
    let count = |list: &[Factory]| {
        let mut out: Vec<(&'static str, usize)> = Vec::new();
        for f in list {
            let n = brief(*f).def.name;
            match out.iter_mut().find(|(m, _)| *m == n) {
                Some(e) => e.1 += 1,
                None => out.push((n, 1)),
            }
        }
        out
    };
    println!("Deck ({} spells, {} lands) from a {}-card pool:", spells.len(), lands.len(), pool.len());
    for (n, k) in count(&spells) {
        println!("{k} {n}");
    }
    for (n, k) in count(&lands) {
        println!("{k} {n}");
    }
    println!("\nPool by measured value (win-rate points per copy; * = in the deck, - = unrated):");
    let mut seen: Vec<Factory> = Vec::new();
    for f in &pool {
        if !seen.iter().any(|g| *g as usize == *f as usize) && !brief(*f).is_land {
            seen.push(*f);
        }
    }
    let value = |f: Factory| {
        crabomination::card_ratings::FRA_SIM_VALUE.iter().find(|(n, _)| *n == brief(f).def.name).map(|(_, v)| *v)
    };
    seen.sort_by(|a, b| value(*b).unwrap_or(f32::MIN).total_cmp(&value(*a).unwrap_or(f32::MIN)));
    for f in seen {
        let mark = if deck.iter().any(|d| *d as usize == f as usize) { '*' } else { ' ' };
        let v = value(f).map_or("    -".to_string(), |v| format!("{v:+5.2}"));
        let b = brief(f);
        let colors: String = b.pip_colors.iter().map(|c| c.short_name()).collect();
        println!("{mark} {v}  {colors:<3} {}  {}", b.cmc, b.def.name);
    }
}

/// `--gauntlet`: each list against one fixed field of rated FRA builds.
fn gauntlet(paths: &[String], pilot: Pilot, field_pilot: Pilot, field: usize, pairs: usize, seed: u64, threads: usize) {
    let lists: Vec<Vec<Factory>> = paths
        .iter()
        .map(|p| {
            let (main, _) = read_list(p);
            if main.len() != 40 {
                eprintln!("{p}: {} cards, not 40", main.len());
            }
            main
        })
        .collect();
    let rows = field_rows(&lists, pilot, field_pilot, field, pairs, seed, threads);
    println!("{field} opponents x {pairs} seat-swapped pairs, seed {seed}; opponent-level 95% intervals");
    for (d, path) in paths.iter().enumerate() {
        let (m, h) = field_share(&rows, d);
        let (w, l) = rows.iter().fold((0, 0), |a, r| (a.0 + r[d].0, a.1 + r[d].1));
        let mut line = format!("{m:6.2} ±{h:4.2}  games {w}-{l}");
        if d > 0 {
            let diffs: Vec<f64> =
                rows.iter().filter_map(|r| Some(win_share(r[d])? - win_share(r[0])?)).collect();
            let (dm, dh) = mean_ci(&diffs);
            line += &format!("  vs first {dm:+6.2} ±{dh:4.2}");
        }
        println!("{line}  {path}");
    }
}

/// A list's win percentage against one opponent, when they played.
fn win_share(r: (u32, u32)) -> Option<f64> {
    (r.0 + r.1 > 0).then(|| 100.0 * r.0 as f64 / (r.0 + r.1) as f64)
}

/// Mean and 95% half-width (normal) of `xs`.
fn mean_ci(xs: &[f64]) -> (f64, f64) {
    let n = xs.len() as f64;
    let m = xs.iter().sum::<f64>() / n;
    let sd = (xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
    (m, 1.96 * sd / n.sqrt())
}

/// List `d`'s mean win share over the field, with its opponent-level interval.
fn field_share(rows: &[Vec<(u32, u32)>], d: usize) -> (f64, f64) {
    let shares: Vec<f64> = rows.iter().filter_map(|r| win_share(r[d])).collect();
    mean_ci(&shares)
}

/// Every list against the same `field` FRA sealed opponents (each the
/// best-of-16 rated build of a seeded pool), `pairs` seat-swapped pairs
/// apiece: one row per opponent of (wins, losses), in `lists` order.
fn field_rows(
    lists: &[Vec<Factory>],
    pilot: Pilot,
    field_pilot: Pilot,
    field: usize,
    pairs: usize,
    seed: u64,
    threads: usize,
) -> Vec<Vec<(u32, u32)>> {
    let packs = crabomination::fra_mode::FraPacks::get();
    let next = AtomicUsize::new(0);
    // Per opponent: (wins, losses) for every list, in `paths` order.
    let rows = std::sync::Mutex::new(Vec::with_capacity(field));
    let start = std::time::Instant::now();
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::Relaxed);
                if j >= field {
                    break;
                }
                let salt = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (j as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93);
                let mut rng = StdRng::seed_from_u64(salt);
                let pool: Vec<Factory> = (0..6).flat_map(|_| packs.roll(&mut rng)).collect();
                let opp = crabomination::selfplay::rated_sealed_build(&pool, 16, salt ^ 1);
                let row: Vec<(u32, u32)> = lists
                    .iter()
                    .map(|deck| {
                        let t = simulate_match_pairs_piloted(deck, &opp, pairs, [pilot, field_pilot], 50_000, salt ^ 2);
                        (t.wins_a, t.wins_b)
                    })
                    .collect();
                let n = done_count(&rows, row);
                if n % 50 == 0 {
                    eprintln!("{n}/{field} opponents, {:.0}s", start.elapsed().as_secs_f64());
                }
            });
        }
    });
    rows.into_inner().unwrap()
}

/// `--audit-builder`: per random FRA pool, the builder's pick against its
/// own variants (one more land; each two-colour pair), every list raced on
/// two independent fields so a choice made on field A is judged on field B.
/// One TSV row per (pool, list): `pool name lands colors shareA shareB`.
fn audit_builder(pilot: Pilot, pools: usize, field: usize, pairs: usize, seed: u64, threads: usize, out: &str) {
    use crabomination::selfplay::{rated_sealed_build_in_colors, rated_sealed_build_lands};
    let packs = crabomination::fra_mode::FraPacks::get();
    let brief = |f: Factory| crabomination::cube::card_brief(f);
    let value = |f: Factory| {
        crabomination::card_ratings::FRA_SIM_VALUE.iter().find(|(n, _)| *n == brief(f).def.name).map_or(0.0, |(_, v)| *v)
    };
    let colors_of = |d: &[Factory]| {
        let mut set = ColorSet::empty();
        for f in d.iter().filter(|f| !brief(**f).is_land) {
            for c in brief(*f).pip_colors.iter() {
                set.insert(c);
            }
        }
        set.iter().map(|c| c.short_name()).collect::<String>()
    };
    // The deck with its lowest-valued spell swapped for a basic of its
    // heaviest colour.
    let plus_land = |d: &[Factory]| -> Vec<Factory> {
        let mut d = d.to_vec();
        let Some(worst) = (0..d.len())
            .filter(|&i| !brief(d[i]).is_land)
            .min_by(|&a, &b| value(d[a]).total_cmp(&value(d[b])))
        else {
            return d;
        };
        let mut pips = [0u32; 5];
        let wubrg = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
        for f in d.iter().filter(|f| !brief(**f).is_land) {
            for (k, c) in wubrg.iter().enumerate() {
                pips[k] += brief(*f).pips.get(*c);
            }
        }
        let heavy = (0..5).max_by_key(|&k| pips[k]).unwrap();
        d[worst] = crabomination::draft::basic_land_factory(wubrg[heavy]);
        d
    };
    let mut w = std::io::BufWriter::new(std::fs::File::create(out).expect("--out file"));
    writeln!(w, "pool\tname\tlands\tcolors\tshareA\tshareB").unwrap();
    let start = std::time::Instant::now();
    for p in 0..pools {
        let salt = seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (p as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25);
        let mut rng = StdRng::seed_from_u64(salt);
        let pool: Vec<Factory> = (0..6).flat_map(|_| packs.roll(&mut rng)).collect();
        let pick = rated_sealed_build_in_colors(&pool, 64, 0x5EA1, ColorSet::empty());
        // `CRAB_AUDIT_CARD=Name`: how often the pick plays a card the pool has.
        if let Ok(card) = std::env::var("CRAB_AUDIT_CARD") {
            let has = |d: &[Factory]| d.iter().any(|f| brief(*f).def.name == card);
            if has(&pool) {
                println!("{card}\tpool {p}\tplayed {}", has(&pick));
            }
        }
        let mut cands: Vec<(String, Vec<Factory>)> =
            vec![("pick".into(), pick.clone()), ("pick+land".into(), plus_land(&pick))];
        if std::env::var("CRAB_AUDIT_SWAPS").is_ok() {
            // Which part of `pick+land` pays: dropping the worst spell, the
            // extra land, or the extra source of the heaviest colour.
            let wubrg = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
            let spells: Vec<usize> = (0..pick.len()).filter(|&i| !brief(pick[i]).is_land).collect();
            let worst = *spells.iter().min_by(|&&a, &&b| value(pick[a]).total_cmp(&value(pick[b]))).unwrap();
            let mut pips = [0u32; 5];
            for &i in &spells {
                for (k, c) in wubrg.iter().enumerate() {
                    pips[k] += brief(pick[i]).pips.get(*c);
                }
            }
            let mut order: Vec<usize> = (0..5).filter(|&k| pips[k] > 0).collect();
            order.sort_by_key(|&k| std::cmp::Reverse(pips[k]));
            let basic = |k: usize| crabomination::draft::basic_land_factory(wubrg[k]);
            // Worst spell → a basic of the lightest colour.
            if let Some(&light) = order.last() {
                let mut d = pick.clone();
                d[worst] = basic(light);
                cands.push(("worst->light".into(), d));
            }
            // A light-colour basic → a heavy-colour basic, land count unchanged.
            if order.len() >= 2 {
                let (heavy, light) = (order[0], order[order.len() - 1]);
                if let Some(i) = pick.iter().position(|f| *f as usize == basic(light) as usize) {
                    let mut d = pick.clone();
                    d[i] = basic(heavy);
                    cands.push(("light->heavy".into(), d));
                }
            }
            // Worst spell → the best unplayed spell in the deck's colours.
            let mut left: Vec<Factory> = pool.clone();
            for f in &pick {
                if let Some(i) = left.iter().position(|g| *g as usize == *f as usize) {
                    left.swap_remove(i);
                }
            }
            let mut deck_colors = ColorSet::empty();
            for &i in &spells {
                for c in brief(pick[i]).pip_colors.iter() {
                    deck_colors.insert(c);
                }
            }
            if let Some(best) = left
                .iter()
                .filter(|f| !brief(**f).is_land && brief(**f).pip_colors.is_subset_of(deck_colors))
                .max_by(|a, b| value(**a).total_cmp(&value(**b)))
            {
                let mut d = pick.clone();
                d[worst] = *best;
                cands.push(("worst->spell".into(), d));
            }
        }
        if std::env::var("CRAB_AUDIT_LANDS").is_ok() {
            for lands in [17, 18] {
                cands.push((format!("pick{lands}"), rated_sealed_build_lands(&pool, 64, 0x5EA1, ColorSet::empty(), lands)));
            }
        }
        let wubrg = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
        let pairs_too = std::env::var("CRAB_AUDIT_NO_PAIRS").is_err();
        for i in (0..5).filter(|_| pairs_too) {
            for j in i + 1..5 {
                let mut set = ColorSet::empty();
                set.insert(wubrg[i]);
                set.insert(wubrg[j]);
                let d = match std::env::var("CRAB_AUDIT_LANDS") {
                    Ok(_) => rated_sealed_build_lands(&pool, 64, 0x5EA1, set, 17),
                    Err(_) => rated_sealed_build_in_colors(&pool, 64, 0x5EA1, set),
                };
                if d.len() == 40 {
                    cands.push((format!("{}{}", wubrg[i].short_name(), wubrg[j].short_name()), d));
                }
            }
        }
        let lists: Vec<Vec<Factory>> = cands.iter().map(|(_, d)| d.clone()).collect();
        let a = field_rows(&lists, pilot, pilot, field, pairs, seed ^ (p as u64) << 20, threads);
        let b = field_rows(&lists, pilot, pilot, field, pairs, seed ^ 0x5555 ^ (p as u64) << 20, threads);
        for (d, (name, deck)) in cands.iter().enumerate() {
            let lands = deck.iter().filter(|f| brief(**f).is_land).count();
            writeln!(w, "{p}\t{name}\t{lands}\t{}\t{:.3}\t{:.3}", colors_of(deck), field_share(&a, d).0, field_share(&b, d).0)
                .unwrap();
        }
        w.flush().unwrap();
        eprintln!("pool {}/{pools}, {:.0}s", p + 1, start.elapsed().as_secs_f64());
    }
}

fn pilot_named(name: &str) -> Pilot {
    match name {
        "dflt" => Pilot::Scored(EvalWeights::default()),
        "net67" => {
            load_net();
            Pilot::Scored(EvalWeights::net_on_default())
        }
        "burn-arms" => Pilot::Scored(EvalWeights::burn_creature_arms_on()),
        "fra-fixes" => Pilot::Scored(EvalWeights::fra_fixes_on()),
        "free-attacks" => Pilot::Scored(EvalWeights { free_attacks: true, ..EvalWeights::fra_fixes_on() }),
        "trick-riders" => Pilot::Scored(EvalWeights { free_attacks: true, trick_riders: true, ..EvalWeights::fra_fixes_on() }),
        "fra-fixes-noburn" => Pilot::Scored(EvalWeights { burn_for_lethal: false, ..EvalWeights::fra_fixes_on() }),
        "mcts-dflt-256" | "mcts-burn-256" | "mcts-fixes-256" => Pilot::Mcts(crabomination::server::MctsConfig {
            iterations: 256,
            horizon_turns: 3,
            weights: match name {
                "mcts-burn-256" => EvalWeights::burn_creature_arms_on(),
                "mcts-fixes-256" => EvalWeights::fra_fixes_on(),
                _ => EvalWeights::default(),
            },
            ..crabomination::server::MctsConfig::default()
        }),
        other => panic!("pilot {other}: dflt, burn-arms, fra-fixes, net67, mcts-dflt-256, mcts-burn-256 or mcts-fixes-256"),
    }
}

fn main() {
    let threads = num("--threads", std::thread::available_parallelism().map_or(8, |n| n.get()));
    if let Some(path) = arg("--build") {
        let by_play = std::env::args().all(|a| a != "--static").then(|| ByPlay {
            pilot: pilot_named(arg("--pilot").as_deref().unwrap_or("fra-fixes")),
            field: num("--field", 400),
            pairs: num("--pairs", 4),
            seed: num("--seed", 43) as u64,
            threads,
        });
        build_pool(&path, num("--candidates", 64), arg("--colors").map_or(ColorSet::empty(), |c| parse_colors(&c)), by_play);
        return;
    }
    if std::env::args().any(|a| a == "--gauntlet") {
        let a: Vec<String> = std::env::args().collect();
        let i = a.iter().position(|x| x == "--gauntlet").unwrap();
        let paths: Vec<String> = a[i + 1..].iter().take_while(|x| !x.starts_with("--")).cloned().collect();
        let name = arg("--pilot").unwrap_or_else(|| "dflt".into());
        let pilot = pilot_named(&name);
        let field_pilot = pilot_named(&arg("--field-pilot").unwrap_or(name));
        gauntlet(&paths, pilot, field_pilot, num("--field", 400), num("--pairs", 4), num("--seed", 43) as u64, threads);
        return;
    }
    if std::env::args().any(|a| a == "--audit-builder") {
        let pilot = pilot_named(arg("--pilot").as_deref().unwrap_or("fra-fixes"));
        let out = arg("--out").expect("--out FILE.tsv");
        audit_builder(pilot, num("--pools", 200), num("--field", 60), num("--pairs", 2), num("--seed", 43) as u64, threads, &out);
        return;
    }
    if std::env::args().any(|a| a == "--ab") {
        let pilot = pilot_named(arg("--pilot").as_deref().unwrap_or("dflt"));
        race_builders(pilot, num("--pools", 2000), num("--pairs", 8), num("--seed", 43) as u64, threads, num("--best", 0));
        return;
    }
    let out = std::path::PathBuf::from(arg("--out").expect("--out DIR"));
    let pilot_name = arg("--pilot").unwrap_or_else(|| "dflt".into());
    let n_decks = num("--decks", 2000);
    let rounds = num("--rounds", 20);
    let pairs = num("--pairs", 2);
    let seed = num("--seed", 1) as u64;
    let explore = num("--explore", 0);
    let pilot = pilot_named(&pilot_name);

    let cards = if sos() { sos_cards() } else { load_cards(&arg("--ratings").expect("--ratings FRA.tsv")) };
    eprintln!("{} cards resolved", cards.len());
    let index: HashMap<usize, usize> = cards.iter().enumerate().map(|(i, c)| (c.f as usize, i)).collect();
    let mut by_rarity: HashMap<char, Vec<usize>> = HashMap::new();
    for r in ['C', 'U', 'R', 'M'] {
        by_rarity.insert(r, Vec::new());
    }
    for (i, c) in cards.iter().enumerate() {
        by_rarity.entry(c.rarity).or_default().push(i);
    }

    // Pools and builds.
    let mut rng = StdRng::seed_from_u64(seed);
    let mut in_pool = vec![0u32; cards.len()];
    let mut in_deck = vec![0u32; cards.len()];
    let mut decks: Vec<Vec<Factory>> = Vec::with_capacity(n_decks);
    for d in 0..n_decks {
        // SOS pools come from its own pack roller (lands and all); a card
        // outside `cards` (a land) is simply not counted.
        let pool: Vec<Factory> = if sos() {
            roll_pool(&mut rng)
        } else {
            (0..6).flat_map(|_| open_pack(&by_rarity, &mut rng)).map(|i| cards[i].f).collect()
        };
        let mut seen = vec![false; cards.len()];
        for f in &pool {
            if let Some(&i) = index.get(&(*f as usize))
                && !seen[i]
            {
                seen[i] = true;
                in_pool[i] += 1;
            }
        }
        let mut deck = crabomination::selfplay::heuristic_sealed_build_v3(&pool, seed ^ (d as u64) << 20);
        // `--explore N`: N of the deck's spells swapped for pool cards the
        // build left out, inside the deck's colours. The v3 builder all but
        // never plays some cards (Way of the Wildspeaker made 17 of 16,000
        // decks), so they never reach the 30-deck cut; the regression
        // credits a swapped card the same way as a chosen one.
        // Its own stream, so the pools match a run without exploration.
        let mut xrng = StdRng::seed_from_u64(seed ^ 0xE491_0000 ^ d as u64);
        for _ in 0..explore {
            let b = |f: Factory| crabomination::cube::card_brief(f);
            let mut colors = ColorSet::empty();
            for f in deck.iter().filter(|f| !b(**f).is_land) {
                for c in b(*f).pip_colors.iter() {
                    colors.insert(c);
                }
            }
            let out: Vec<Factory> = pool
                .iter()
                .copied()
                .filter(|f| !b(*f).is_land && b(*f).fits(colors) && !deck.iter().any(|g| *g as usize == *f as usize))
                .collect();
            let spells: Vec<usize> = (0..deck.len()).filter(|&i| !b(deck[i]).is_land).collect();
            if let (Some(&add), Some(&at)) = (out.choose(&mut xrng), spells.choose(&mut xrng)) {
                deck[at] = add;
            }
        }
        let mut seen = vec![false; cards.len()];
        for f in &deck {
            if let Some(&i) = index.get(&(*f as usize))
                && !seen[i]
            {
                seen[i] = true;
                in_deck[i] += 1;
            }
        }
        decks.push(deck);
    }
    std::fs::create_dir_all(&out).unwrap();
    let mut w = std::fs::File::create(out.join("cards.tsv")).unwrap();
    writeln!(w, "name\trarity\tstatic_score\tquality\tin_pool\tin_deck").unwrap();
    for (i, c) in cards.iter().enumerate() {
        let b = crabomination::cube::card_brief(c.f);
        writeln!(
            w,
            "{}\t{}\t{}\t{}\t{}\t{}",
            c.name,
            c.rarity,
            b.base_score + b.quality,
            b.quality,
            in_pool[i],
            in_deck[i]
        )
        .unwrap();
    }
    let mut w = std::fs::File::create(out.join("decks.tsv")).unwrap();
    for (d, deck) in decks.iter().enumerate() {
        let names: Vec<&str> = deck.iter().map(|f| crabomination::cube::card_brief(*f).def.name).collect();
        writeln!(w, "{d}\t{}", names.join("\t")).unwrap();
    }
    drop(w);
    eprintln!("built {n_decks} decks");

    // Matches: `rounds` random perfect pairings.
    let mut matches = Vec::new();
    for _ in 0..rounds {
        let mut order: Vec<usize> = (0..n_decks).collect();
        order.shuffle(&mut rng);
        matches.extend(order.as_chunks::<2>().0.iter().map(|p| (p[0], p[1], rng.random::<u64>())));
    }
    let next = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let results = std::sync::Mutex::new(Vec::with_capacity(matches.len()));
    let start = std::time::Instant::now();
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                loop {
                    let k = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&(a, b, sd)) = matches.get(k) else { break };
                    let t = simulate_match_pairs_piloted(&decks[a], &decks[b], pairs, [pilot, pilot], 50_000, sd);
                    results.lock().unwrap().push((a, b, t.wins_a, t.wins_b, t.undecided));
                    let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                    if n.is_multiple_of(500) {
                        eprintln!("{n}/{} matches, {:.0}s", matches.len(), start.elapsed().as_secs_f64());
                    }
                }
            });
        }
    });
    let mut w = std::fs::File::create(out.join("games.tsv")).unwrap();
    for (a, b, wa, wb, u) in results.into_inner().unwrap() {
        writeln!(w, "{a}\t{b}\t{wa}\t{wb}\t{u}").unwrap();
    }
    eprintln!("done: {} matches in {:.0}s", matches.len(), start.elapsed().as_secs_f64());
}
