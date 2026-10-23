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

use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

use crabomination::card::CardDefinition;
use crabomination::catalog::sets::all_factories::all_catalog_card_factories;
use crabomination::recommend::{Pilot, simulate_match_pairs_piloted};
use crabomination::server::bot::EvalWeights;
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

fn main() {
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
            let path = std::env::var("CRAB_NET").unwrap_or_else(|_| "nets/champion.safetensors".into());
            crabomination::server::net_eval::load_slot(
                crabomination::server::net_eval::SLOT_BEST,
                std::path::Path::new(&path),
            )
            .expect("load net");
            eprintln!("loaded value net from {path}");
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
