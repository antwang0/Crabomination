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
//! [--decks 2000] [--rounds 20] [--pairs 2] [--seed 1] [--threads N]`.
//! `net67` loads `CRAB_NET` (default `nets/champion.safetensors`).
//!
//! `fra_card_eval --build POOL.txt [--candidates 64]` is the sealed helper:
//! the pool (a decklist — one card per line, `N Name`, Arena or MTGO export)
//! built by the rated builder (best of N rated builds), then every pool card
//! ranked by its measured value, marked `*` where the build plays it.
//!
//! `fra_card_eval --ab [--pools 2000] [--pairs 8] [--seed 43] [--pilot P]`
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
fn race_builders(pilot: Pilot, pools: usize, pairs: usize, seed: u64, threads: usize) {
    let packs = crabomination::fra_mode::FraPacks::get();
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
                let pool: Vec<Factory> = (0..6).flat_map(|_| packs.roll(&mut rng)).collect();
                let old = crabomination::selfplay::heuristic_sealed_build_v3(&pool, salt ^ 1);
                let new = crabomination::selfplay::heuristic_sealed_build_rated(&pool, salt ^ 1);
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
        "rated vs v3: {mean:.2}% [{:.2}, {:.2}] pool-level 95% (n={} pools, {identical} identical decks); games {wa}-{wb} ({:.2}%)",
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

/// `--build`: the rated build of a player's pool, and the pool ranked.
fn build_pool(path: &str, candidates: usize) {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let parsed = crabomination::decklist::parse_decklist(&text);
    for u in &parsed.unknown {
        eprintln!("not in the catalog: {} {:?} (did you mean {:?})", u.count, u.name, u.suggestions);
    }
    let pool: Vec<Factory> = parsed.main.iter().chain(&parsed.sideboard).copied().collect();
    let deck = crabomination::selfplay::rated_sealed_build(&pool, candidates, 0x5EA1);
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
        println!("{mark} {v}  {}", brief(f).def.name);
    }
}

fn main() {
    if let Some(path) = arg("--build") {
        build_pool(&path, num("--candidates", 64));
        return;
    }
    if std::env::args().any(|a| a == "--ab") {
        let pilot = match arg("--pilot").as_deref().unwrap_or("dflt") {
            "dflt" => Pilot::Scored(EvalWeights::default()),
            "net67" => {
                load_net();
                Pilot::Scored(EvalWeights::net_on_default())
            }
            "mcts-dflt-256" => Pilot::Mcts(crabomination::server::MctsConfig {
                iterations: 256,
                horizon_turns: 3,
                weights: EvalWeights::default(),
                ..crabomination::server::MctsConfig::default()
            }),
            other => panic!("--ab pilot {other}: dflt, net67 or mcts-dflt-256"),
        };
        let threads = num("--threads", std::thread::available_parallelism().map_or(8, |n| n.get()));
        race_builders(pilot, num("--pools", 2000), num("--pairs", 8), num("--seed", 43) as u64, threads);
        return;
    }
    let ratings = arg("--ratings").expect("--ratings FRA.tsv");
    let out = std::path::PathBuf::from(arg("--out").expect("--out DIR"));
    let pilot_name = arg("--pilot").unwrap_or_else(|| "dflt".into());
    let n_decks = num("--decks", 2000);
    let rounds = num("--rounds", 20);
    let pairs = num("--pairs", 2);
    let seed = num("--seed", 1) as u64;
    let threads = num("--threads", std::thread::available_parallelism().map_or(8, |n| n.get()));

    let pilot = match pilot_name.as_str() {
        "dflt" => Pilot::Scored(EvalWeights::default()),
        "net67" => {
            load_net();
            Pilot::Scored(EvalWeights::net_on_default())
        }
        other => panic!("unknown pilot {other}; expected dflt or net67"),
    };

    let cards = load_cards(&ratings);
    eprintln!("{} FRA cards resolved", cards.len());
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
        let pool_idx: Vec<usize> = (0..6).flat_map(|_| open_pack(&by_rarity, &mut rng)).collect();
        let mut seen = vec![false; cards.len()];
        for &i in &pool_idx {
            if !seen[i] {
                seen[i] = true;
                in_pool[i] += 1;
            }
        }
        let pool: Vec<Factory> = pool_idx.iter().map(|&i| cards[i].f).collect();
        let deck = crabomination::selfplay::heuristic_sealed_build_v3(&pool, seed ^ (d as u64) << 20);
        let mut seen = vec![false; cards.len()];
        for f in &deck {
            if let Some(&i) = index.get(&(*f as usize)) {
                if !seen[i] {
                    seen[i] = true;
                    in_deck[i] += 1;
                }
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
        matches.extend(order.chunks_exact(2).map(|p| (p[0], p[1], rng.random::<u64>())));
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
                    if n % 500 == 0 {
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
