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
//! `--polish-rounds 4` swaps, `--no-polish` skips it; every swap by
//! successive halving, `--polish-narrow` for the old 6 x 6 screen). The
//! field draws its colour pairs by real FRA Sealed shares (`--field-mix`,
//! `best` for the old white-heavy field). `--start LIST` polishes a given
//! list instead of racing builds first. Earlier, with the narrow polish:
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
//! `fra_card_eval --draft-ab --arms A,B [--set sos|cube] [--pods 250]
//! [--pairs 8] [--seed 43] [--pilot P]` races two builders on DRAFT piles:
//! a seeded 8-seat pod of `draft::bot_pick` drafters (three packs of
//! `DraftPool::generate_pack`), every seat's pile built by both arms and the
//! two decks played in seat-swapped pairs. Arms: `suggest` (the client's
//! deckbuilding screen: `suggest_main_deck` 23 + `suggest_basic_split` 17),
//! `opponent` (`selfplay::sealed_opponent_build`, 16), `rated`
//! (`rated_sealed_build`, 16), `v3` (`v3_opponent_build`, 16). The unit is
//! the pile; the share printed is B's.
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
    // `--trial deckvalue` (with `--best N` and `CRAB_DECK_VALUE=FILE.json`,
    // `deck_value_train --export`): the rated pick polished by the learned
    // deck value, three swaps at most, against the rated builder as it
    // stands (its own polish on) — step 1's play gate.
    // `--trial deckvalue-after`: the same swaps on top of the rated
    // builder's own polish (step 1b).
    let after_polish = arg("--trial").as_deref() == Some("deckvalue-after");
    let deck_value: Option<crabomination::deck_value::DeckValue> = (arg("--trial").as_deref() == Some("deckvalue") || after_polish).then(|| {
        assert!(best > 0, "--trial deckvalue races best-of-N builds: pass --best 16");
        let path = std::env::var("CRAB_DECK_VALUE").expect("--trial deckvalue needs CRAB_DECK_VALUE=<deck_value_train --export>");
        let dv = crabomination::deck_value::DeckValue::load(std::path::Path::new(&path)).unwrap_or_else(|e| panic!("{e}"));
        eprintln!("deck value {} ({} cards) from {path}", dv.arm, dv.cards.len());
        dv
    });
    // `--trial policy` / `--trial policy-value` (with `--best N` for the
    // control, `CRAB_DECK_POLICY=FILE.json` from `deck_policy_train`, and for
    // `policy-value` `CRAB_DECK_VALUE`): step 2's G2a — the add-one-card
    // policy's greedy build — and G2b — the greedy build and 16 sampled ones,
    // the value's pick — each against the rated builder as it stands.
    let policy: Option<(crabomination::deck_policy::DeckPolicy, bool)> =
        arg("--trial").filter(|t| t.starts_with("policy")).map(|t| {
            assert!(best > 0, "--trial {t} races the rated builder's best-of-N: pass --best 16");
            let path = std::env::var("CRAB_DECK_POLICY").expect("--trial policy needs CRAB_DECK_POLICY=<deck_policy_train --export>");
            let p = crabomination::deck_policy::DeckPolicy::load(std::path::Path::new(&path)).unwrap_or_else(|e| panic!("{e}"));
            eprintln!("deck policy ({} cards) from {path}", p.cards.len());
            (p, t.starts_with("policy-value"))
        });
    // `--trial policy-value+rated|lands|polishN` (2026-11-08 research
    // sweep): the adopted learned builder (`build_valued`, 16 samples)
    // against itself with one build-time lever on
    // (`DeckPolicy::build_valued_with`): the rated best-of-`--best` build as
    // a 28th candidate; more-land variants (to 18) of the value's top 3; at
    // most N value-guided swaps after the pick.
    let variant: Option<String> = arg("--trial").and_then(|t| t.strip_prefix("policy-value+").map(str::to_string));
    if let Some(v) = &variant {
        let polish_n = v.strip_prefix("polish").and_then(|n| n.parse::<usize>().ok());
        assert!(matches!(v.as_str(), "rated" | "lands") || polish_n.is_some_and(|n| n > 0), "--trial policy-value+{v}: rated, lands or polishN");
    }
    // `--trial policy-castable`: the splash rule's own gate — the policy's
    // greedy build under it against the same policy's build without it.
    let castable_ab = arg("--trial").as_deref() == Some("policy-castable");
    // `--trial value-vs-adopted` (`CRAB_DECK_VALUE=NEW.json`): the production
    // sealed builder (`sealed_opponent_build`: compiled policy and value, SOS
    // polish `SOS_SEALED_POLISH`) against the same builder with the new
    // value in the compiled one's place. Also prints the starvation test
    // for both values on the production decks: all but one of the second
    // colour's basics moved to the main colour — a value that sees the mana
    // base rates that lower.
    let value_ab: Option<crabomination::deck_value::DeckValue> = (arg("--trial").as_deref() == Some("value-vs-adopted")).then(|| {
        let path = std::env::var("CRAB_DECK_VALUE").expect("--trial value-vs-adopted needs CRAB_DECK_VALUE");
        let dv = crabomination::deck_value::DeckValue::load(std::path::Path::new(&path)).unwrap_or_else(|e| panic!("{e}"));
        eprintln!("new deck value {} (mana {}) from {path}", dv.arm, dv.mana);
        dv
    });
    let starve = std::sync::Mutex::new([0usize; 3]);
    let policy_value: Option<crabomination::deck_value::DeckValue> = policy.as_ref().filter(|p| p.1).map(|_| {
        let path = std::env::var("CRAB_DECK_VALUE").expect("--trial policy-value needs CRAB_DECK_VALUE");
        crabomination::deck_value::DeckValue::load(std::path::Path::new(&path)).unwrap_or_else(|e| panic!("{e}"))
    });
    let trial: Option<(bool, bool)> =
        arg("--trial").filter(|t| !t.starts_with("deckvalue") && !t.starts_with("policy") && t != "value-vs-adopted").map(|t| match t.as_str() {
        "polish" => (true, false),
        "floor" => (false, true),
        "both" => (true, true),
        other => panic!("--trial {other}: polish, floor, both, deckvalue, deckvalue-after, policy or policy-value"),
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
    // `CRAB_AIVAT=1` (`server::aivat`): every game's draws scored by the
    // set's value net (`CRAB_AIVAT_NET`, default the FRA / SOS champion) and
    // a luck-corrected pool share printed beside the raw one.
    let aivat = crabomination::recommend::aivat_on();
    if aivat {
        let path = std::env::var("CRAB_AIVAT_NET").unwrap_or_else(|_| {
            if arg("--set").as_deref() == Some("sos") { "nets/champion.safetensors" } else { "nets/fra-champion.safetensors" }.into()
        });
        crabomination::server::net_eval::load_slot(crabomination::server::net_eval::SLOT_BEST, std::path::Path::new(&path))
            .expect("CRAB_AIVAT: load the value net");
        eprintln!("aivat: draws scored by {path}");
    }
    let luck_rows = std::sync::Mutex::new(Vec::<(f64, f64)>::new());
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
                let (old, new) = if let Some(nv) = &value_ab {
                    use crabomination::selfplay as sp;
                    let sos_pool = crabomination::card_ratings::sos_pool(&pool);
                    let (adopted_v, polish) = if sos_pool { (sp::sos_deck_value(), sp::SOS_SEALED_POLISH) } else { (sp::fra_deck_value(), 0) };
                    let opts = crabomination::deck_policy::ValuedOpts { polish, ..Default::default() };
                    let build = |v: &crabomination::deck_value::DeckValue| {
                        sp::sealed_deck_policy().build_valued_with(&pool, v, 16, &mut StdRng::seed_from_u64(salt ^ 3), &opts)
                    };
                    let old = build(adopted_v);
                    debug_assert_eq!(old, sp::sealed_opponent_build(&pool, 16, salt, ColorSet::empty()));
                    let new = build(nv);
                    if let Some(starved) = starve_second_colour(&new) {
                        let mut z = starve.lock().unwrap();
                        z[0] += 1;
                        z[1] += usize::from(adopted_v.value(&starved) >= adopted_v.value(&new));
                        z[2] += usize::from(nv.value(&starved) >= nv.value(&new));
                    }
                    (old, new)
                } else if let Some((p, _)) = &policy
                    && let Some(v) = &variant
                {
                    let value = policy_value.as_ref().expect("policy-value+ loads CRAB_DECK_VALUE");
                    let adopted = p.build_valued(&pool, value, 16, &mut StdRng::seed_from_u64(salt ^ 3));
                    let mut opts = crabomination::deck_policy::ValuedOpts::default();
                    match v.as_str() {
                        "rated" => opts.extra.push(rated(&pool)),
                        "lands" => {
                            opts.land_variants = 3;
                            opts.max_lands = 18;
                        }
                        _ => opts.polish = v["polish".len()..].parse().expect("checked above"),
                    }
                    (adopted, p.build_valued_with(&pool, value, 16, &mut StdRng::seed_from_u64(salt ^ 3), &opts))
                } else if let Some((p, _)) = &policy
                    && castable_ab
                {
                    (p.build_unchecked(&pool, None), p.build(&pool, None))
                } else if let Some((p, _)) = &policy {
                    let built = match &policy_value {
                        Some(v) => p.build_valued(&pool, v, 16, &mut StdRng::seed_from_u64(salt ^ 3)),
                        None => p.build(&pool, None),
                    };
                    (rated(&pool), built)
                } else if let Some(dv) = &deck_value {
                    (rated(&pool), crabomination::selfplay::deck_value_build(&pool, best, salt ^ 1, dv, 3, after_polish))
                } else if let Some((polish, floor)) = trial {
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
                if aivat && !t.pairs.is_empty() {
                    let k = t.pairs.len() as f64;
                    let raw: f64 = t.pairs.iter().map(|&s| (f64::from(s) + 1.0) / 2.0).sum::<f64>() / k;
                    let luck: f64 = t.pair_luck.iter().sum::<f64>() / (2.0 * k);
                    luck_rows.lock().unwrap().push((100.0 * raw, 100.0 * (raw - luck)));
                }
                let n = done_count(&rows, (t.wins_a, t.wins_b, same));
                if n % 250 == 0 {
                    eprintln!("{n}/{pools} pools, {:.0}s", start.elapsed().as_secs_f64());
                }
            });
        }
    });
    let rows = rows.into_inner().unwrap();
    let luck_rows = luck_rows.into_inner().unwrap();
    if aivat && luck_rows.len() > 1 {
        let n = luck_rows.len() as f64;
        let stats = |f: &dyn Fn(&(f64, f64)) -> f64| {
            let m = luck_rows.iter().map(f).sum::<f64>() / n;
            let v = luck_rows.iter().map(|r| (f(r) - m).powi(2)).sum::<f64>() / (n - 1.0);
            (m, v)
        };
        let (rm, rv) = stats(&|r| r.0);
        let (cm, cv) = stats(&|r| r.1);
        let (dm, dv) = stats(&|r| r.0 - r.1);
        println!(
            "aivat: {cm:.2}% [{:.2}, {:.2}] luck-corrected over decided pairs, raw {rm:.2}% [{:.2}, {:.2}] (n={} pools); \
             variance x{:.2}; mean correction {dm:+.2} (±{:.2})",
            cm - 1.96 * (cv / n).sqrt(),
            cm + 1.96 * (cv / n).sqrt(),
            rm - 1.96 * (rv / n).sqrt(),
            rm + 1.96 * (rv / n).sqrt(),
            luck_rows.len(),
            cv / rv.max(1e-12),
            1.96 * (dv / n).sqrt(),
        );
    }
    let z = starve.into_inner().unwrap();
    if value_ab.is_some() {
        println!(
            "starvation test on {} decks: starved rated >= built by the adopted value {:.1}%, by the new value {:.1}%",
            z[0],
            100.0 * z[1] as f64 / z[0].max(1) as f64,
            100.0 * z[2] as f64 / z[0].max(1) as f64
        );
    }
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
        if value_ab.is_some() {
            "new value vs adopted value"
        } else if castable_ab {
            "castable vs unchecked"
        } else if variant.is_some() {
            "variant vs adopted builder"
        } else if policy_value.is_some() {
            "policy+value vs rated"
        } else if policy.is_some() {
            "policy vs rated"
        } else if deck_value.is_some() {
            "deckvalue vs rated"
        } else if table.is_some() {
            "table vs compiled"
        } else {
            "rated vs v3"
        },
        mean - half,
        mean + half,
        shares.len(),
        100.0 * wa as f64 / (wa + wb) as f64
    );
}

/// `deck` with all but one of its second colour's basics turned into basics
/// of its main colour (most basics), or `None` with fewer than two of the
/// second colour.
fn starve_second_colour(deck: &[Factory]) -> Option<Vec<Factory>> {
    use crabomination::draft::basic_land_factory;
    let name = |f: Factory| crabomination::cube::card_def(f).name;
    let count = |c: Color| deck.iter().filter(|f| name(**f) == name(basic_land_factory(c))).count();
    let mut by: Vec<Color> = Color::ALL.into_iter().filter(|&c| count(c) > 0).collect();
    by.sort_by_key(|&c| std::cmp::Reverse(count(c)));
    let (&main, &second) = (by.first()?, by.get(1)?);
    if count(second) < 2 {
        return None;
    }
    let mut keep = 1;
    Some(
        deck.iter()
            .map(|&f| {
                if name(f) == name(basic_land_factory(second)) {
                    if keep > 0 {
                        keep -= 1;
                        return f;
                    }
                    return basic_land_factory(main);
                }
                f
            })
            .collect(),
    )
}

/// One seeded 8-seat booster draft of `draft::bot_pick` drafters: every
/// seat's 3 x pack pile. Passing follows `DraftPod` (left, right, left);
/// the CR 905 draft-matters cards are taken as plain picks.
fn draft_piles(kind: crabomination::draft::DraftPool, pool: &[Factory], rng: &mut StdRng) -> Vec<Vec<Factory>> {
    use crabomination::draft::{POD_SIZE, PACKS_PER_SEAT, bot_pick};
    let mut picks: Vec<Vec<Factory>> = vec![Vec::new(); POD_SIZE];
    for round in 0..PACKS_PER_SEAT {
        let mut packs: Vec<Vec<Factory>> = (0..POD_SIZE).map(|_| kind.generate_pack(pool, rng)).collect();
        while packs.iter().any(|p| !p.is_empty()) {
            for (seat, pack) in packs.iter_mut().enumerate() {
                if let Some(i) = bot_pick(pack, &picks[seat]) {
                    picks[seat].push(pack.remove(i));
                }
            }
            // Seat i's pack goes to seat i - 1 in rounds 1 and 3.
            if round == 1 {
                packs.rotate_right(1);
            } else {
                packs.rotate_left(1);
            }
        }
    }
    picks
}

/// A draft pile built by the named arm (see `--draft-ab` in the header).
fn draft_arm(name: &str, pile: &[Factory], seed: u64) -> Vec<Factory> {
    use crabomination::draft;
    match name {
        "suggest" => {
            let (mut deck, _) = draft::suggest_main_deck(pile, 23);
            let colors = draft::top_two_colors(pile);
            let basics = draft::suggest_basic_split(&deck, colors, 17);
            for c in Color::ALL {
                for _ in 0..basics.get(&c).copied().unwrap_or(0) {
                    deck.push(draft::basic_land_factory(c));
                }
            }
            draft::enforce_copy_cap(deck)
        }
        "opponent" => crabomination::selfplay::sealed_opponent_build(pile, 16, seed, ColorSet::empty()),
        "rated" => crabomination::selfplay::rated_sealed_build(pile, 16, seed),
        "v3" => crabomination::selfplay::v3_opponent_build(pile, 16, seed, ColorSet::empty()),
        other => panic!("--arms: {other} is not suggest, opponent, rated or v3"),
    }
}

fn race_draft_builders(pilot: Pilot, pods: usize, pairs: usize, seed: u64, threads: usize) {
    use crabomination::draft::{DraftPool, POD_SIZE};
    let arms = arg("--arms").unwrap_or_else(|| "suggest,opponent".into());
    let (a, b) = arms.split_once(',').expect("--arms A,B");
    let kind = match arg("--set").as_deref() {
        Some("cube") => DraftPool::Cube,
        Some("sos") | None => DraftPool::Sos,
        Some(other) => panic!("--draft-ab --set {other}: sos or cube"),
    };
    let pool = kind.factories();
    let piles = pods * POD_SIZE;
    let next = AtomicUsize::new(0);
    let rows = std::sync::Mutex::new(Vec::with_capacity(piles));
    let sizes = std::sync::Mutex::new((0usize, 0usize, 0usize));
    let start = std::time::Instant::now();
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= piles {
                    break;
                }
                let pod = (i / POD_SIZE) as u64;
                let pod_salt = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ pod.wrapping_mul(0xD6E8_FEB8_6659_FD93);
                let pile = draft_piles(kind, &pool, &mut StdRng::seed_from_u64(pod_salt)).swap_remove(i % POD_SIZE);
                let salt = pod_salt ^ ((i % POD_SIZE) as u64 + 1).wrapping_mul(0xA24B_AED4_963E_E407);
                let old = draft_arm(a, &pile, salt ^ 1);
                let new = draft_arm(b, &pile, salt ^ 1);
                if std::env::var_os("CRAB_DRAFT_DEBUG").is_some() && i < 2 {
                    let show = |d: &[Factory]| {
                        let mut v: Vec<String> = d.iter().map(|f| crabomination::cube::card_def(*f).name.to_string()).collect();
                        v.sort();
                        v.join(" | ")
                    };
                    eprintln!("pile {i} ({} cards): {}\n  {a} ({}): {}\n  {b} ({}): {}", pile.len(), show(&pile), old.len(), show(&old), new.len(), show(&new));
                }
                {
                    let lands = |d: &[Factory]| d.iter().filter(|f| crabomination::cube::card_def(**f).is_land()).count();
                    let mut z = sizes.lock().unwrap();
                    z.0 += lands(&old);
                    z.1 += lands(&new);
                    z.2 += usize::from(old.len() < 40 || new.len() < 40);
                }
                let key = |d: &[Factory]| {
                    let mut v: Vec<usize> = d.iter().map(|f| *f as usize).collect();
                    v.sort_unstable();
                    v
                };
                let same = key(&old) == key(&new);
                let t = simulate_match_pairs_piloted(&new, &old, pairs, [pilot, pilot], 50_000, salt ^ 2);
                let n = done_count(&rows, (t.wins_a, t.wins_b, same));
                if n % 250 == 0 {
                    eprintln!("{n}/{piles} piles, {:.0}s", start.elapsed().as_secs_f64());
                }
            });
        }
    });
    let rows = rows.into_inner().unwrap();
    let (la, lb, short) = sizes.into_inner().unwrap();
    let identical = rows.iter().filter(|r| r.2).count();
    let shares: Vec<f64> =
        rows.iter().filter(|r| r.0 + r.1 > 0).map(|r| 100.0 * r.0 as f64 / (r.0 + r.1) as f64).collect();
    let n = shares.len() as f64;
    let mean = shares.iter().sum::<f64>() / n;
    let sd = (shares.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
    let half = 1.96 * sd / n.sqrt();
    let (wa, wb): (u32, u32) = rows.iter().fold((0, 0), |acc, r| (acc.0 + r.0, acc.1 + r.1));
    println!(
        "{b} vs {a} on {} draft piles: {mean:.2}% [{:.2}, {:.2}] pile-level 95% (n={}, {identical} identical decks, \
         {short} under 40 cards); games {wa}-{wb} ({:.2}%); lands/deck {a} {:.2}, {b} {:.2}; {:.0}s",
        kind.label(),
        mean - half,
        mean + half,
        shares.len(),
        100.0 * wa as f64 / (wa + wb) as f64,
        la as f64 / rows.len() as f64,
        lb as f64 / rows.len() as f64,
        start.elapsed().as_secs_f64(),
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
    // The sealed opponents' builder (learned policy + same-pool value), and
    // any `--lists A.txt,B.txt` the player built, race beside the rated ones.
    cands.push(("learned builder".to_string(), crabomination::selfplay::sealed_opponent_build(pool, 16, 0x5EA1, ColorSet::empty())));
    for l in arg("--lists").map(|s| s.split(',').map(str::to_string).collect::<Vec<_>>()).unwrap_or_default() {
        let stem = std::path::Path::new(&l).file_stem().unwrap().to_string_lossy().to_string();
        cands.push((format!("list {stem}"), read_list(&l).0));
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
    let labelled = field_rows_labelled(&lists, bp.pilot, bp.pilot, bp.field, bp.pairs, bp.seed, bp.threads);
    let rows: Vec<Vec<(u32, u32)>> = labelled.iter().map(|r| r.1.clone()).collect();
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
    // Each build against each opponent colour pair: a build whose edge lives
    // in one corner of the field shows it here.
    let mut labels: Vec<String> = labelled.iter().map(|r| r.0.clone()).collect();
    labels.sort();
    labels.dedup();
    labels.sort_by_key(|l| std::cmp::Reverse(labelled.iter().filter(|r| &r.0 == l).count()));
    println!("\n  by opponent pair (share of the field):");
    print!("  {:24}", "");
    for l in &labels {
        let n = labelled.iter().filter(|r| &r.0 == l).count();
        print!(" {:>5} {:>2.0}%", l, 100.0 * n as f64 / labelled.len() as f64);
    }
    println!();
    for &(_, _, d) in scored.iter().take(6) {
        print!("  {:24}", cands[d].0);
        for l in &labels {
            let (w, n) = labelled.iter().filter(|r| &r.0 == l).fold((0u32, 0u32), |a, r| (a.0 + r.1[d].0, a.1 + r.1[d].0 + r.1[d].1));
            print!(" {:>9.1}", 100.0 * w as f64 / n.max(1) as f64);
        }
        println!();
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
    // `--polish-wide`: every spell out x every unplayed on-colour spell in
    // (plus the land swaps), cut by successive halving, instead of the six
    // lowest-rated out x six highest-rated in on one noisy screen — the
    // ratings that pick those twelve are the measured values, and they are
    // what is in doubt (ML_NOTES "the sim under-rates blue").
    // The default since 2026-11-17; `--polish-narrow` is the old search.
    let wide = std::env::args().all(|a| a != "--polish-narrow");
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
        if !wide {
            ins.truncate(6);
        }
        let mut outs: Vec<usize> = Vec::new();
        let mut spells: Vec<usize> = (0..deck.len()).filter(|&i| !brief(deck[i]).is_land).collect();
        spells.sort_by(|&a, &b| value(deck[a]).total_cmp(&value(deck[b])));
        for i in spells {
            if (wide || outs.len() < 6) && !outs.iter().any(|&j| name(deck[j]) == name(deck[i])) {
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
                for &i in ins.iter().take(if wide { ins.len() } else { 3 }) {
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
        let order: Vec<usize> = if wide {
            // Successive halving: every swap on a small field, the best
            // quarter on a fresh field twice the size, until three remain —
            // each stage ranked by the paired gain over the current deck.
            let mut alive: Vec<usize> = (1..lists.len()).collect();
            let mut field = 100usize;
            let mut stage = 0u64;
            while alive.len() > 3 {
                let mut sub: Vec<Vec<Factory>> = vec![deck.clone()];
                sub.extend(alive.iter().map(|&d| lists[d].clone()));
                let rows = field_rows(&sub, bp.pilot, bp.pilot, field, 2, bp.seed ^ 0x9044_0000 ^ (round as u64) << 8 ^ stage, bp.threads);
                let mut ranked: Vec<(f64, usize)> = (1..sub.len()).map(|k| (paired(&rows, k).0, alive[k - 1])).collect();
                ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
                let keep = (alive.len() / 4).max(3);
                eprintln!("  wide polish round {} stage {stage}: {} swaps on {field} x 2, best {:+.2}", round + 1, alive.len(), ranked[0].0);
                alive = ranked.into_iter().take(keep).map(|r| r.1).collect();
                field *= 2;
                stage += 1;
            }
            alive
        } else {
            let scores = screen(&lists, bp.seed ^ 0x9011_0000 ^ round as u64);
            let mut order: Vec<usize> = (1..lists.len()).filter(|&d| scores[d] > scores[0]).collect();
            order.sort_by(|&a, &b| scores[b].total_cmp(&scores[a]));
            order.truncate(3);
            order
        };
        if order.is_empty() {
            println!("  round {}: no swap of {} screens above the deck", round + 1, cands.len());
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
    // `--start LIST`: polish this list (cards from the pool) instead of
    // racing builds for one.
    let mut deck = match (&by_play, arg("--start")) {
        (Some(_), Some(start)) => read_list(&start).0,
        (Some(bp), None) if colors.is_empty() => build_by_play(&pool, candidates, bp),
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

/// `--analyze POOL --out DIR` (with `CRAB_DECK_POLICY`, `CRAB_DECK_VALUE`,
/// optional `--lists A,B` of hand-built decks): every builder's deck for one
/// pool, written to `DIR/*.txt` for `--gauntlet`, with each deck's colours,
/// learned value and difference from the rated deck; the policy's greedy
/// trace; the colours 200 sampled builds land in; and the pool cards either
/// model cannot see.
fn analyze_pool(path: &str, out: &std::path::Path) {
    use crabomination::deck_policy::DeckPolicy;
    use crabomination::deck_value::DeckValue;
    let (main, side) = read_list(path);
    let pool: Vec<Factory> = main.iter().chain(&side).copied().collect();
    let name = |f: Factory| crabomination::cube::card_def(f).name;
    let is_land = |f: Factory| crabomination::cube::card_def(f).is_land();
    let pp = std::env::var("CRAB_DECK_POLICY").expect("CRAB_DECK_POLICY");
    let policy = DeckPolicy::load(std::path::Path::new(&pp)).unwrap_or_else(|e| panic!("{e}"));
    let vp = std::env::var("CRAB_DECK_VALUE").expect("CRAB_DECK_VALUE");
    let dv = DeckValue::load(std::path::Path::new(&vp)).unwrap_or_else(|e| panic!("{e}"));
    let spells_in_pool: Vec<Factory> = pool.iter().copied().filter(|f| !is_land(*f)).collect();
    println!("pool {path}: {} cards, {} non-land", pool.len(), spells_in_pool.len());
    let mut blind_p: Vec<&str> = spells_in_pool.iter().map(|f| name(*f)).filter(|n| !policy.cards.iter().any(|c| c == n)).collect();
    let mut blind_v: Vec<&str> = spells_in_pool.iter().map(|f| name(*f)).filter(|n| !dv.knows(n)).collect();
    blind_p.dedup();
    blind_v.dedup();
    println!("policy cannot see: {blind_p:?}");
    println!("value cannot see: {blind_v:?}");
    let colors = |d: &[Factory]| -> String {
        Color::ALL
            .into_iter()
            .filter(|&c| d.iter().filter(|f| !is_land(**f)).map(|f| crabomination::cube::card_brief(*f).pips.get(c)).sum::<u32>() >= 3)
            .map(|c| c.short_name())
            .collect()
    };
    let rated = crabomination::selfplay::rated_sealed_build(&pool, 16, 0x5EA1);
    let mut decks: Vec<(String, Vec<Factory>)> = vec![
        ("rated".into(), rated.clone()),
        ("rated_value".into(), crabomination::selfplay::deck_value_build(&pool, 16, 0x5EA1, &dv, 3, true)),
        ("policy_greedy".into(), policy.build(&pool, None)),
        ("policy_value".into(), policy.build_valued(&pool, &dv, 16, &mut StdRng::seed_from_u64(3))),
    ];
    for seed in 1..=4u64 {
        decks.push((format!("rated_seed{seed}"), crabomination::selfplay::rated_sealed_build(&pool, 16, seed)));
    }
    for l in arg("--lists").map(|s| s.split(',').map(str::to_string).collect::<Vec<_>>()).unwrap_or_default() {
        let stem = std::path::Path::new(&l).file_stem().unwrap().to_string_lossy().to_string();
        decks.push((format!("list_{stem}"), read_list(&l).0));
    }
    std::fs::create_dir_all(out).unwrap();
    let rated_spells: Vec<&str> = rated.iter().filter(|f| !is_land(**f)).map(|f| name(*f)).collect();
    println!("\n{:<22} {:>6} {:>6} {:>5} {:>5}  colours  vs rated (+ in, - out)", "deck", "value", "spells", "lands", "cards");
    for (label, d) in &decks {
        let sp: Vec<&str> = d.iter().filter(|f| !is_land(**f)).map(|f| name(*f)).collect();
        let mut plus: Vec<&str> = sp.clone();
        let mut minus: Vec<&str> = Vec::new();
        for r in &rated_spells {
            match plus.iter().position(|x| x == r) {
                Some(i) => {
                    plus.swap_remove(i);
                }
                None => minus.push(r),
            }
        }
        println!("{label:<22} {:>6.3} {:>6} {:>5} {:>5}  {:<7}  +{plus:?} -{minus:?}", dv.value(d), sp.len(), d.len() - sp.len(), d.len(), colors(d));
        let mut text = String::new();
        let mut seen: Vec<(&str, usize)> = Vec::new();
        for f in d {
            match seen.iter_mut().find(|e| e.0 == name(*f)) {
                Some(e) => e.1 += 1,
                None => seen.push((name(*f), 1)),
            }
        }
        for (n, k) in seen {
            text += &format!("{k} {n}\n");
        }
        std::fs::write(out.join(format!("{label}.txt")), text).unwrap();
    }
    println!("\npolicy greedy trace (pick, its probability, the next three):");
    let mut deck: Vec<Factory> = Vec::new();
    loop {
        let st = policy.step(&pool, &deck);
        let mut opts: Vec<(Option<Factory>, f32)> = st.cards.iter().map(|&(f, v)| (Some(f), v)).collect();
        if let Some(v) = st.stop {
            opts.push((None, v));
        }
        let m = opts.iter().map(|o| o.1).fold(f32::MIN, f32::max);
        let z: f32 = opts.iter().map(|o| (o.1 - m).exp()).sum();
        opts.sort_by(|a, b| b.1.total_cmp(&a.1));
        let show = |o: &(Option<Factory>, f32)| format!("{} {:.2}", o.0.map_or("STOP", name), (o.1 - m).exp() / z);
        println!("{:>2}. {}   | {}", deck.len() + 1, show(&opts[0]), opts[1..opts.len().min(4)].iter().map(show).collect::<Vec<_>>().join(", "));
        match opts[0].0 {
            Some(f) if deck.len() < crabomination::deck_policy::MAX_SPELLS => deck.push(f),
            _ => break,
        }
    }
    println!("\n200 sampled builds (temperature 1): colours, share, mean value");
    let mut rng = StdRng::seed_from_u64(7);
    let mut by: Vec<(String, usize, f64)> = Vec::new();
    for _ in 0..200 {
        let d = policy.build(&pool, Some((&mut rng, 1.0)));
        let c = colors(&d);
        let v = dv.value(&d);
        match by.iter_mut().find(|e| e.0 == c) {
            Some(e) => {
                e.1 += 1;
                e.2 += v;
            }
            None => by.push((c, 1, v)),
        }
    }
    by.sort_by_key(|e| std::cmp::Reverse(e.1));
    for (c, n, v) in by {
        println!("  {c:<6} {:>5.1} %  {:.3}", n as f64 / 2.0, v / n as f64);
    }
}

/// `job(0..n)` on `threads` threads, results in index order.
fn par_map<T: Send>(n: usize, threads: usize, job: &(dyn Fn(usize) -> T + Sync)) -> Vec<T> {
    let next = AtomicUsize::new(0);
    let rows = std::sync::Mutex::new(Vec::with_capacity(n));
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::Relaxed);
                if j >= n {
                    break;
                }
                let row = job(j);
                rows.lock().unwrap().push((j, row));
            });
        }
    });
    let mut rows = rows.into_inner().unwrap();
    rows.sort_by_key(|r| r.0);
    rows.into_iter().map(|r| r.1).collect()
}

/// A cost as `{2}{W}{G/U}` text.
fn cost_text(def: &CardDefinition) -> String {
    use crabomination::mana::ManaSymbol as S;
    def.cost
        .symbols
        .iter()
        .map(|s| match s {
            S::Colored(c) => format!("{{{}}}", c.short_name()),
            S::Generic(n) => format!("{{{n}}}"),
            S::Colorless(n) => "{C}".repeat(*n as usize),
            S::Hybrid(a, b) => format!("{{{}/{}}}", a.short_name(), b.short_name()),
            S::Phyrexian(c) => format!("{{{}/P}}", c.short_name()),
            S::PhyrexianHybrid(a, b) => format!("{{{}/{}/P}}", a.short_name(), b.short_name()),
            S::MonoHybrid(n, c) => format!("{{{n}/{}}}", c.short_name()),
            S::ColorlessHybrid(c) => format!("{{C/{}}}", c.short_name()),
            S::Snow => "{S}".into(),
            S::X => "{X}".into(),
        })
        .collect()
}

/// Whether a deck making `colors` can pay every coloured pip of `def`
/// (a hybrid pip needs either half; mono/colourless hybrid and Phyrexian
/// pips need nothing). A land fits a deck it makes a colour for, or any
/// deck when it makes none.
fn castable_in(f: Factory, colors: ColorSet) -> bool {
    use crabomination::mana::ManaSymbol as S;
    let def = crabomination::cube::card_def(f);
    if def.is_land() {
        let made = crabomination::cube::card_brief(f).produces;
        return made.is_empty() || made.iter().any(|c| colors.contains(c));
    }
    def.cost.symbols.iter().all(|s| match s {
        S::Colored(c) => colors.contains(*c),
        S::Hybrid(a, b) => colors.contains(*a) || colors.contains(*b),
        _ => true,
    })
}

/// The colours a deck's lands make.
fn land_colors(deck: &[Factory]) -> ColorSet {
    let mut set = ColorSet::empty();
    for f in deck {
        let b = crabomination::cube::card_brief(*f);
        if b.is_land {
            for c in b.produces.iter() {
                set.insert(c);
            }
        }
    }
    set
}

/// `deck` with `card` in place of whichever slot the value likes losing
/// most — a spell for a spell, a basic for a land: (new value, the card it
/// replaced, the deck).
fn best_swap(dv: &crabomination::deck_value::DeckValue, deck: &[Factory], card: Factory) -> (f64, &'static str, Vec<Factory>) {
    let def = |f: Factory| crabomination::cube::card_def(f);
    let land = def(card).is_land();
    let mut best: Option<(f64, usize)> = None;
    let mut tried: Vec<usize> = Vec::new();
    for (i, f) in deck.iter().enumerate() {
        let d = def(*f);
        let fits = if land { matches!(d.name, "Plains" | "Island" | "Swamp" | "Mountain" | "Forest") } else { !d.is_land() };
        if !fits || tried.contains(&(*f as usize)) {
            continue;
        }
        tried.push(*f as usize);
        let mut t = deck.to_vec();
        t[i] = card;
        let v = dv.value(&t);
        if best.is_none_or(|b| v > b.0) {
            best = Some((v, i));
        }
    }
    let (v, i) = best.expect("a slot to swap");
    let mut t = deck.to_vec();
    let out = def(t[i]).name;
    t[i] = card;
    (v, out, t)
}

/// `--pool-adds POOL --deck LIST --out DIR [--sample-decks 400]
/// [--confirm 40] [--only "A|B"] [--field 600] [--pairs 4] [--seed 43] [--pilot fra-fixes]`:
/// the learned same-pool deck value (`selfplay::fra_deck_value`, the
/// production sealed selector) asked two questions.
///
/// * **The set** (`DIR/set.tsv`): every FRA card's value-net gain over the
///   slot it would replace, averaged over `--sample-decks` learned-builder
///   decks of random FRA pools whose colours can cast it — "how much better
///   is a typical deck that can play this card for playing it" — beside its
///   standalone weight `a_c` and its `FRA_SIM_VALUE`.
/// * **Your pool** (`DIR/adds.tsv`): per FRA card, the deck you play
///   (`--deck`) with the card swapped in at its best slot, and the learned
///   builder's deck of the pool plus the card; the value gain of the better
///   of the two over your deck. The `--confirm` best are then played against
///   one fixed field (`DIR/confirm.tsv`), each read as a paired difference
///   against your deck.
fn pool_adds(pool_path: &str, deck_path: &str, out: &std::path::Path, sample_decks: usize, confirm: usize, bp: &ByPlay) {
    use crabomination::selfplay::{fra_deck_value, sealed_opponent_build};
    let def = |f: Factory| crabomination::cube::card_def(f);
    let name = |f: Factory| def(f).name;
    let dv = fra_deck_value();
    let (main, side) = read_list(pool_path);
    let pool: Vec<Factory> = main.iter().chain(&side).copied().collect();
    let (mine, _) = read_list(deck_path);
    assert_eq!(mine.len(), 40, "{deck_path}: {} cards, not 40", mine.len());
    std::fs::create_dir_all(out).unwrap();
    let packs = crabomination::fra_mode::FraPacks::get();
    let rarity: Vec<(Factory, char)> = [(&packs.common, 'C'), (&packs.uncommon, 'U'), (&packs.rare, 'R'), (&packs.mythic, 'M')]
        .into_iter()
        .flat_map(|(s, r)| s.iter().map(move |f| (*f, r)))
        .collect();
    let set: Vec<Factory> = rarity.iter().map(|r| r.0).collect();
    let t0 = std::time::Instant::now();

    // 1. The set, in context.
    let sample: Vec<Vec<Factory>> = par_map(sample_decks, bp.threads, &|j| {
        let mut rng = StdRng::seed_from_u64(0xADD5 ^ (j as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let p: Vec<Factory> = (0..6).flat_map(|_| packs.roll(&mut rng)).collect();
        sealed_opponent_build(&p, 16, j as u64, ColorSet::empty())
    });
    eprintln!("{} sample decks built, {:.0}s", sample.len(), t0.elapsed().as_secs_f64());
    let sample_colors: Vec<ColorSet> = sample.iter().map(|d| land_colors(d)).collect();
    let sample_value: Vec<f64> = sample.iter().map(|d| dv.value(d)).collect();
    let a_of = |n: &str| dv.cards.iter().position(|c| c == n).map(|i| dv.a[i]);
    let set_rows = par_map(set.len(), bp.threads, &|k| {
        let c = set[k];
        let mut gains = Vec::new();
        let mut played = 0usize;
        for (d, deck) in sample.iter().enumerate() {
            if !castable_in(c, sample_colors[d]) {
                continue;
            }
            played += usize::from(deck.iter().any(|f| *f as usize == c as usize));
            gains.push(best_swap(dv, deck, c).0 - sample_value[d]);
        }
        let (m, h) = if gains.len() > 1 { mean_ci(&gains) } else { (f64::NAN, f64::NAN) };
        let b = crabomination::cube::card_brief(c);
        let colors: String = b.pip_colors.iter().map(|c| c.short_name()).collect();
        format!(
            "{}\t{}\t{}\t{}\t{}\t{:?}\t{}\t{}\t{:.5}\t{:.5}\t{}\t{}\t{}\t{}",
            name(c),
            rarity[k].1,
            cost_text(def(c)),
            colors,
            b.cmc,
            def(c).card_types,
            sim_value(c).map_or("".into(), |v| format!("{v:.4}")),
            a_of(name(c)).map_or("".into(), |v| format!("{v:.5}")),
            m,
            h,
            gains.len(),
            played,
            pool.iter().filter(|f| **f as usize == c as usize).count(),
            u8::from(dv.knows(name(c))),
        )
    });
    let mut text = "name\trarity\tcost\tcolors\tcmc\ttypes\tsim_value\tdv_a\tdv_gain\tdv_gain_ci\tfit_decks\tplayed_in\tin_pool\tdv_knows\n".to_string();
    for r in &set_rows {
        text += r;
        text.push('\n');
    }
    std::fs::write(out.join("set.tsv"), text).unwrap();
    eprintln!("set ranked, {:.0}s", t0.elapsed().as_secs_f64());

    // 2. Your pool, plus one card.
    let v_mine = dv.value(&mine);
    let base_build = sealed_opponent_build(&pool, 16, 0x5EA1, ColorSet::empty());
    let v_base = dv.value(&base_build);
    let list_text = |d: &[Factory]| d.iter().map(|f| name(*f)).collect::<Vec<_>>().join(";");
    let colors_of = |d: &[Factory]| -> String { land_colors(d).iter().map(|c| c.short_name()).collect() };
    std::fs::write(
        out.join("base.tsv"),
        format!(
            "kind\tvalue\tcolors\tdeck\nyour_deck\t{v_mine:.5}\t{}\t{}\nbuilder_on_pool\t{v_base:.5}\t{}\t{}\n",
            colors_of(&mine),
            list_text(&mine),
            colors_of(&base_build),
            list_text(&base_build)
        ),
    )
    .unwrap();
    let mut adds: Vec<(f64, String, Vec<Factory>)> = Vec::new();
    let rows = par_map(set.len(), bp.threads, &|k| {
        let c = set[k];
        let (v_swap, out_card, swap_deck) = best_swap(dv, &mine, c);
        let mut p = pool.clone();
        p.push(c);
        let rebuild = sealed_opponent_build(&p, 16, 0x5EA1, ColorSet::empty());
        let v_re = dv.value(&rebuild);
        let rebuild_plays = rebuild.iter().any(|f| *f as usize == c as usize);
        let (kind, best, deck) = if v_re > v_swap && rebuild_plays { ("rebuild", v_re, rebuild.clone()) } else { ("swap", v_swap, swap_deck) };
        let row = format!(
            "{}\t{}\t{:.5}\t{}\t{:.5}\t{}\t{}\t{kind}\t{:.5}\t{}\t{}",
            name(c),
            pool.iter().filter(|f| **f as usize == c as usize).count(),
            v_swap - v_mine,
            out_card,
            v_re - v_mine,
            colors_of(&rebuild),
            u8::from(rebuild_plays),
            best - v_mine,
            list_text(&deck),
            list_text(&rebuild),
        );
        (row, best - v_mine, deck)
    });
    let mut text = "name\tin_pool\tswap_gain\tswap_out\trebuild_gain\trebuild_colors\trebuild_plays\tbest_kind\tbest_gain\tbest_deck\trebuild_deck\n".to_string();
    for (k, (r, gain, deck)) in rows.into_iter().enumerate() {
        text += &r;
        text.push('\n');
        adds.push((gain, name(set[k]).to_string(), deck));
    }
    std::fs::write(out.join("adds.tsv"), text).unwrap();
    eprintln!("pool additions screened, {:.0}s", t0.elapsed().as_secs_f64());

    // 3. The best by value, played.
    adds.sort_by(|a, b| b.0.total_cmp(&a.0));
    // `--only "A|B"`: confirm just these cards (a re-read on a fresh seed).
    if let Some(only) = arg("--only") {
        let names: Vec<&str> = only.split('|').collect();
        adds.retain(|a| names.contains(&a.1.as_str()));
    }
    adds.truncate(confirm);
    // The control a new card has to beat: the best one-card swap from the
    // pool's own sideboard (what re-tuning buys without opening anything).
    let spare: Vec<Factory> = {
        let mut left = pool.clone();
        for f in &mine {
            if let Some(i) = left.iter().position(|g| *g as usize == *f as usize) {
                left.swap_remove(i);
            }
        }
        left.sort_by_key(|f| name(*f));
        left.dedup_by_key(|f| *f as usize);
        left
    };
    let (v_free, free_in, free_out, free_deck) = spare
        .iter()
        .map(|&c| {
            let (v, out_card, d) = best_swap(dv, &mine, c);
            (v, name(c), out_card, d)
        })
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .expect("a sideboard card");
    std::fs::write(
        out.join("free_swap.tsv"),
        format!("in\tout\tgain\tdeck\n{free_in}\t{free_out}\t{:.5}\t{}\n", v_free - v_mine, list_text(&free_deck)),
    )
    .unwrap();
    eprintln!("best sideboard swap: {free_in} for {free_out}, value {:+.3}", v_free - v_mine);
    let mut lists: Vec<Vec<Factory>> = vec![mine.clone(), base_build.clone(), free_deck];
    lists.extend(adds.iter().map(|a| a.2.clone()));
    let mut labels: Vec<String> = vec!["(your deck)".into(), "(builder on your pool)".into(), "(best sideboard swap)".into()];
    labels.extend(adds.iter().map(|a| a.1.clone()));
    let rows = field_rows(&lists, bp.pilot, bp.pilot, bp.field, bp.pairs, bp.seed, bp.threads);
    let mut text = format!("# {} opponents x {} seat-swapped pairs, seed {}\nname\tshare\tshare_ci\tgain\tgain_ci\twins\tlosses\n", bp.field, bp.pairs, bp.seed);
    for (d, l) in labels.iter().enumerate() {
        let (m, h) = field_share(&rows, d);
        let diffs: Vec<f64> = rows.iter().filter_map(|r| Some(win_share(r[d])? - win_share(r[0])?)).collect();
        let (dm, dh) = if d == 0 { (0.0, 0.0) } else { mean_ci(&diffs) };
        let (w, lo) = rows.iter().fold((0, 0), |a, r| (a.0 + r[d].0, a.1 + r[d].1));
        text += &format!("{l}\t{m:.3}\t{h:.3}\t{dm:.3}\t{dh:.3}\t{w}\t{lo}\n");
        println!("{m:6.2} ±{h:4.2}  vs your deck {dm:+6.2} ±{dh:4.2}  {l}");
    }
    std::fs::write(out.join("confirm.tsv"), text).unwrap();
    eprintln!("done, {:.0}s", t0.elapsed().as_secs_f64());
}

/// `--set-played --out DIR [--sample-decks 1000] [--per-card 400]
/// [--pairs 4] [--seed 43] [--pilot fra-fixes]`: the `--pool-adds` set
/// rating measured in play instead of read off the deck value. The same
/// sample decks (learned builder on seeded FRA pools); per card, up to
/// `--per-card` of them whose lands can cast it, each with the card swapped
/// into the slot the value would give it ([`best_swap`]). Swapped and
/// unswapped deck play the same field opponent with the same seeds, so a
/// card's rating is the mean paired difference in win-rate points, with
/// the value's prediction for the very same swaps beside it
/// (`DIR/set_played.tsv`).
fn set_played(out: &std::path::Path, sample_decks: usize, per_card: usize, bp: &ByPlay) {
    use crabomination::selfplay::{fra_deck_value, sealed_opponent_build};
    let dv = fra_deck_value();
    let packs = crabomination::fra_mode::FraPacks::get();
    std::fs::create_dir_all(out).unwrap();
    let t0 = std::time::Instant::now();
    // The same sample `--pool-adds` rates the set on.
    let sample: Vec<Vec<Factory>> = par_map(sample_decks, bp.threads, &|j| {
        let mut rng = StdRng::seed_from_u64(0xADD5 ^ (j as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let p: Vec<Factory> = (0..6).flat_map(|_| packs.roll(&mut rng)).collect();
        sealed_opponent_build(&p, 16, j as u64, ColorSet::empty())
    });
    let colors: Vec<ColorSet> = sample.iter().map(|d| land_colors(d)).collect();
    let opps: Vec<Vec<Factory>> = par_map(sample.len(), bp.threads, &|j| field_opponent(packs, bp.seed, j).1);
    let salt = |j: usize| bp.seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (j as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93) ^ 2;
    let share = |deck: &[Factory], j: usize| -> Option<f64> {
        let t = simulate_match_pairs_piloted(deck, &opps[j], bp.pairs, [bp.pilot, bp.pilot], 50_000, salt(j));
        win_share((t.wins_a, t.wins_b))
    };
    let base: Vec<Option<f64>> = par_map(sample.len(), bp.threads, &|j| share(&sample[j], j));
    eprintln!("{} sample decks built and played, {:.0}s", sample.len(), t0.elapsed().as_secs_f64());
    let set: Vec<Factory> = packs.all().filter(|f| !crabomination::cube::card_def(*f).is_land()).collect();
    // One job per (card, deck) so every thread stays busy.
    let jobs: Vec<(usize, usize)> = set
        .iter()
        .enumerate()
        .flat_map(|(k, &c)| {
            let colors = &colors;
            (0..sample.len()).filter(move |&j| castable_in(c, colors[j])).take(per_card).map(move |j| (k, j))
        })
        .collect();
    eprintln!("{} swaps to play", jobs.len());
    let done = AtomicUsize::new(0);
    let res: Vec<Option<(f64, f64)>> = par_map(jobs.len(), bp.threads, &|i| {
        let (k, j) = jobs[i];
        let (v, _, deck) = best_swap(dv, &sample[j], set[k]);
        let n = done.fetch_add(1, Ordering::Relaxed) + 1;
        if n.is_multiple_of(10_000) {
            eprintln!("{n}/{} swaps, {:.0}s", jobs.len(), t0.elapsed().as_secs_f64());
        }
        Some((share(&deck, j)? - base[j]?, 25.0 * (v - dv.value(&sample[j]))))
    });
    let mut text = "name\tn\tplayed_gain\tplayed_ci\tpred_gain\tsim_value\n".to_string();
    for (k, &c) in set.iter().enumerate() {
        let (d, p): (Vec<f64>, Vec<f64>) = jobs.iter().zip(&res).filter(|(jb, _)| jb.0 == k).filter_map(|(_, r)| *r).unzip();
        if d.len() < 2 {
            continue;
        }
        let (m, h) = mean_ci(&d);
        let pm = p.iter().sum::<f64>() / p.len() as f64;
        text += &format!(
            "{}\t{}\t{m:.3}\t{h:.3}\t{pm:.3}\t{}\n",
            crabomination::cube::card_def(c).name,
            d.len(),
            sim_value(c).map_or(String::new(), |v| format!("{v:.3}"))
        );
    }
    std::fs::write(out.join("set_played.tsv"), text).unwrap();
    eprintln!("done, {:.0}s", t0.elapsed().as_secs_f64());
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
    field_rows_labelled(lists, pilot, field_pilot, field, pairs, seed, threads).into_iter().map(|r| r.1).collect()
}

/// The ten two-colour pairs, WUBRG order (`"WU"`, `"WB"`, ...).
fn color_pairs() -> Vec<(String, ColorSet)> {
    let wubrg = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
    let mut out = Vec::new();
    for i in 0..5 {
        for j in i + 1..5 {
            let mut set = ColorSet::empty();
            set.insert(wubrg[i]);
            set.insert(wubrg[j]);
            out.push((format!("{}{}", wubrg[i].short_name(), wubrg[j].short_name()), set));
        }
    }
    out
}

/// Two-colour decks per pair in FRA Sealed on 17lands (`color_ratings/data`,
/// `.ladder/l17/color_Sealed.json`, fetched 2026-11-16).
const FRA_SEALED_PAIR_GAMES: [(&str, f64); 10] = [
    ("WU", 6476.0),
    ("WB", 2728.0),
    ("WR", 2652.0),
    ("WG", 5323.0),
    ("UB", 6815.0),
    ("UR", 4329.0),
    ("UG", 2753.0),
    ("BR", 4295.0),
    ("BG", 4127.0),
    ("RG", 6042.0),
];

/// `--field-mix`: how the field's opponents choose colours. Unset: every
/// opponent builds its pool's best deck (`rated_sealed_build`) — which on
/// FRA puts white in 76 % of decks against 38 % in real Sealed (17lands,
/// ML_NOTES "the recommender's field is white-heavy"). `uniform`: each
/// opponent draws one of the ten pairs evenly and builds its best deck in
/// it. A path to a 17lands `color_ratings/data` JSON (`.ladder/l17/
/// color_Sealed.json`): pairs drawn by their share of real two-colour decks.
/// Weights per pair, in `color_pairs` order.
///
/// `--build` defaults to [`FRA_SEALED_PAIR_GAMES`] (the white-heavy field
/// main-decked Flourishing Grapple on the user's pool, ML_NOTES); `--field-mix
/// best` there is the old every-pool's-best field. Every other mode keeps
/// the old default, so the card ratings stay comparable until a re-rate.
fn field_mix() -> Option<&'static [f64]> {
    static MIX: std::sync::OnceLock<Option<Vec<f64>>> = std::sync::OnceLock::new();
    MIX.get_or_init(|| {
        let building = arg("--build").is_some();
        let spec = arg("--field-mix").or_else(|| building.then(|| "sealed17".to_string()))?;
        let pairs = color_pairs();
        if spec == "best" {
            return None;
        }
        if spec == "uniform" {
            return Some(vec![1.0; pairs.len()]);
        }
        if spec == "sealed17" {
            let w: Vec<f64> = pairs
                .iter()
                .map(|(name, _)| {
                    FRA_SEALED_PAIR_GAMES.iter().find(|(n, _)| n == name).map(|(_, g)| *g).unwrap_or_else(|| panic!("no {name} share"))
                })
                .collect();
            return Some(w);
        }
        let text = std::fs::read_to_string(&spec).unwrap_or_else(|e| panic!("--field-mix {spec}: {e}"));
        let rows: Vec<serde_json::Value> = serde_json::from_str(&text).expect("--field-mix: a 17lands color_ratings JSON");
        let w: Vec<f64> = pairs
            .iter()
            .map(|(name, _)| {
                rows.iter()
                    .filter(|r| r["is_summary"] == false)
                    .find(|r| {
                        let mut a: Vec<char> = r["short_name"].as_str().unwrap_or("").chars().collect();
                        let mut b: Vec<char> = name.chars().collect();
                        a.sort_unstable();
                        b.sort_unstable();
                        a == b
                    })
                    .and_then(|r| r["games"].as_f64())
                    .unwrap_or_else(|| panic!("--field-mix {spec}: no {name} row"))
            })
            .collect();
        let total: f64 = w.iter().sum();
        eprintln!(
            "field mix from {spec}: {}",
            pairs.iter().zip(&w).map(|((n, _), x)| format!("{n} {:.1}%", 100.0 * x / total)).collect::<Vec<_>>().join(", ")
        );
        Some(w)
    })
    .as_deref()
}

/// Field opponent `j` of `seed`: its main colour pair (the two colours with
/// the most pips) and its deck, under `--field-mix`.
fn field_opponent(packs: &crabomination::fra_mode::FraPacks, seed: u64, j: usize) -> (String, Vec<Factory>) {
    let salt = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (j as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93);
    let mut rng = StdRng::seed_from_u64(salt);
    let pool: Vec<Factory> = (0..6).flat_map(|_| packs.roll(&mut rng)).collect();
    let mut opp = Vec::new();
    if let Some(w) = field_mix() {
        let mut pick = StdRng::seed_from_u64(salt ^ 0xC010_0B5E).random_range(0.0..w.iter().sum::<f64>());
        let pairs = color_pairs();
        let k = w.iter().position(|&x| {
            pick -= x;
            pick < 0.0
        });
        let set = pairs[k.unwrap_or(pairs.len() - 1)].1;
        opp = crabomination::selfplay::rated_sealed_build_in_colors(&pool, 16, salt ^ 1, set);
    }
    if opp.len() < 40 {
        opp = crabomination::selfplay::rated_sealed_build(&pool, 16, salt ^ 1);
    }
    (main_pair(&opp), opp)
}

/// A deck's two colours with the most pips, WUBRG order (`"W"` for a mono deck).
fn main_pair(deck: &[Factory]) -> String {
    let mut pips = [0usize; 5];
    for f in deck {
        for c in crabomination::cube::card_def(*f).cost.colored_symbols() {
            pips[c as usize] += 1;
        }
    }
    let mut order: Vec<usize> = (0..5).filter(|&i| pips[i] > 0).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(pips[i]));
    order.truncate(2);
    order.sort_unstable();
    order.iter().map(|&i| "WUBRG".as_bytes()[i] as char).collect()
}

/// [`field_rows`] with each opponent's main pair beside its row.
fn field_rows_labelled(
    lists: &[Vec<Factory>],
    pilot: Pilot,
    field_pilot: Pilot,
    field: usize,
    pairs: usize,
    seed: u64,
    threads: usize,
) -> Vec<(String, Vec<(u32, u32)>)> {
    let packs = crabomination::fra_mode::FraPacks::get();
    let next = AtomicUsize::new(0);
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
                let (label, opp) = field_opponent(packs, seed, j);
                let row: Vec<(u32, u32)> = lists
                    .iter()
                    .map(|deck| {
                        let t = simulate_match_pairs_piloted(deck, &opp, pairs, [pilot, field_pilot], 50_000, salt ^ 2);
                        (t.wins_a, t.wins_b)
                    })
                    .collect();
                let n = done_count(&rows, (j, label, row));
                if n % 50 == 0 {
                    eprintln!("{n}/{field} opponents, {:.0}s", start.elapsed().as_secs_f64());
                }
            });
        }
    });
    let mut rows = rows.into_inner().unwrap();
    rows.sort_by_key(|r| r.0);
    rows.into_iter().map(|(_, l, r)| (l, r)).collect()
}

/// `CRAB_FORCE_CARDS=A;B;..` (pair-strength census runs only): each named
/// FRA card whose colours fit `set` replaces the deck's lowest-sim-value
/// spell, pool or not — so a trace census sees cards the rated builder
/// leaves out. Not a measurement of the decks.
fn force_cards(deck: &mut Vec<Factory>, set: ColorSet) {
    static FORCED: std::sync::OnceLock<Vec<Factory>> = std::sync::OnceLock::new();
    let forced = FORCED.get_or_init(|| {
        let Ok(spec) = std::env::var("CRAB_FORCE_CARDS") else { return Vec::new() };
        spec.split(';')
            .filter(|n| !n.is_empty())
            .map(|n| {
                crabomination::fra_mode::FraPacks::get()
                    .all()
                    .find(|f| crabomination::cube::card_def(*f).name == n)
                    .unwrap_or_else(|| panic!("CRAB_FORCE_CARDS: no FRA card {n}"))
            })
            .collect()
    });
    if deck.len() != 40 {
        return;
    }
    for &f in forced {
        let d = crabomination::cube::card_def(f);
        if !d.cost.colored_symbols().all(|c| set.contains(c)) || deck.iter().any(|g| *g as usize == f as usize) {
            continue;
        }
        let worst = (0..deck.len())
            .filter(|&i| !crabomination::cube::card_def(deck[i]).is_land() && !forced.iter().any(|g| *g as usize == deck[i] as usize))
            .min_by(|&a, &b| sim_value(deck[a]).unwrap_or(0.0).total_cmp(&sim_value(deck[b]).unwrap_or(0.0)));
        if let Some(i) = worst {
            deck[i] = f;
        }
    }
}

/// `--pair-strength`: how strong each colour pair is in the sim. Per seeded
/// FRA pool, the best rated deck in each of the ten pairs; all ten play the
/// same opponent (field opponent `i` of `--field-mix`, default the
/// builder's own pick), `pairs` seat-swapped pairs each. Prints each pair's
/// win rate beside 17lands' real Sealed one. `--deck-pilot P` pilots the ten
/// decks with P against the field's `--pilot` (same pools and seeds, so two
/// runs differing only in P pair up per pool and pair:
/// `.ladder/react/scripts/paired_ps.py`); `--rows FILE` writes the per-pool
/// shares for that.
/// (`.ladder/l17/color_Sealed.json`,
/// when present), for the colour-bias question (ML_NOTES "the recommender's
/// field is white-heavy").
fn pair_strength(pilot: Pilot, field_pilot: Pilot, pools: usize, pairs: usize, seed: u64, threads: usize) {
    let packs = crabomination::fra_mode::FraPacks::get();
    let cps = color_pairs();
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
                let salt = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (i as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93) ^ 0x9A1E;
                let mut rng = StdRng::seed_from_u64(salt);
                let pool: Vec<Factory> = (0..6).flat_map(|_| packs.roll(&mut rng)).collect();
                let (_, opp) = field_opponent(packs, seed ^ 0x0FF0, i);
                let row: Vec<Option<(u32, u32)>> = cps
                    .iter()
                    .map(|(_, set)| {
                        let mut deck = crabomination::selfplay::rated_sealed_build_in_colors(&pool, 16, salt ^ 1, *set);
                        force_cards(&mut deck, *set);
                        (deck.len() == 40).then(|| {
                            let t = simulate_match_pairs_piloted(&deck, &opp, pairs, [pilot, field_pilot], 50_000, salt ^ 2);
                            (t.wins_a, t.wins_b)
                        })
                    })
                    .collect();
                let n = done_count(&rows, (i, row));
                if n % 50 == 0 {
                    eprintln!("{n}/{pools} pools, {:.0}s", start.elapsed().as_secs_f64());
                }
            });
        }
    });
    let mut rows = rows.into_inner().unwrap();
    rows.sort_by_key(|r| r.0);
    if let Some(path) = arg("--rows") {
        let text: String = rows
            .iter()
            .map(|(i, r)| {
                let cells: Vec<String> = r.iter().map(|c| c.and_then(win_share).map_or("NA".to_string(), |x| format!("{x}"))).collect();
                format!("{i}\t{}\n", cells.join("\t"))
            })
            .collect();
        std::fs::write(&path, text).unwrap_or_else(|e| panic!("--rows {path}: {e}"));
    }
    let rows: Vec<Vec<Option<(u32, u32)>>> = rows.into_iter().map(|(_, r)| r).collect();
    let real: Option<Vec<serde_json::Value>> =
        std::fs::read_to_string(".ladder/l17/color_Sealed.json").ok().and_then(|t| serde_json::from_str(&t).ok());
    println!("{:4} {:>7} {:>7} {:>9} {:>9}", "pair", "sim WR", "±", "17l WR", "17l share");
    for (k, (name, _)) in cps.iter().enumerate() {
        let shares: Vec<f64> = rows.iter().filter_map(|r| r[k].and_then(win_share)).collect();
        let (m, h) = mean_ci(&shares);
        let l17 = real.as_ref().and_then(|rs| {
            rs.iter().find(|r| {
                let mut a: Vec<char> = r["short_name"].as_str().unwrap_or("").chars().collect();
                let mut b: Vec<char> = name.chars().collect();
                a.sort_unstable();
                b.sort_unstable();
                r["is_summary"] == false && a == b
            })
        });
        let (wr, games) = l17.map_or((f64::NAN, 0.0), |r| (100.0 * r["wins"].as_f64().unwrap_or(0.0) / r["games"].as_f64().unwrap_or(1.0), r["games"].as_f64().unwrap_or(0.0)));
        println!("{name:4} {m:7.2} {h:7.2} {wr:9.1} {games:9.0}");
    }
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
    // `<pilot>+cfix`: the pilot with `EvalWeights::color_fixing` on.
    if let Some(base) = name.strip_suffix("+cfix") {
        return match pilot_named(base) {
            Pilot::Scored(w) => Pilot::Scored(EvalWeights { color_fixing: true, ..w }),
            Pilot::Mcts(c) => Pilot::Mcts(crabomination::server::MctsConfig { weights: EvalWeights { color_fixing: true, ..c.weights }, ..c }),
            other => other,
        };
    }
    match name {
        "dflt" => Pilot::Scored(EvalWeights::default()),
        "net67" => {
            load_net();
            Pilot::Scored(EvalWeights::net_on_default())
        }
        "burn-arms" => Pilot::Scored(EvalWeights::burn_creature_arms_on()),
        "fra-fixes" => Pilot::Scored(EvalWeights::fra_fixes_on()),
        // The 2026-11-22 Phase 3 policy flags together (bot_ladder `fixes+p3`).
        "fixes+p3" => Pilot::Scored(EvalWeights {
            hand_abilities_wide: true,
            skip_noop_casts: true,
            loyalty_fixes: true,
            ..EvalWeights::fra_fixes_on()
        }),
        "free-attacks" => Pilot::Scored(EvalWeights { free_attacks: true, ..EvalWeights::fra_fixes_on() }),
        "trick-riders" => Pilot::Scored(EvalWeights { free_attacks: true, trick_riders: true, ..EvalWeights::fra_fixes_on() }),
        "fra-fixes-noburn" => Pilot::Scored(EvalWeights { burn_for_lethal: false, ..EvalWeights::fra_fixes_on() }),
        // 2026-11-17 reactive-play leads (bot_ladder has the same names).
        "fixes+bar" => Pilot::Scored(EvalWeights { counter_bar_low: true, ..EvalWeights::fra_fixes_on() }),
        "fixes+holdinst" => Pilot::Scored(EvalWeights { hold_instants: true, combat_aware: true, ..EvalWeights::fra_fixes_on() }),
        "fixes+bar+holdinst" => Pilot::Scored(EvalWeights {
            counter_bar_low: true,
            hold_instants: true,
            combat_aware: true,
            ..EvalWeights::fra_fixes_on()
        }),
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

/// `fra_card_eval --dump-cards DIR`: for a finished rating run's
/// `DIR/decks.tsv`, the deck value trainer's inputs (ML_NOTES "A deck net
/// that builds decks", step 1) — `DIR/card_feats.tsv`, each card name the
/// decks use with its play-net vocabulary index and its 59 hand-object
/// features (`encode_deck_card`), and `DIR/deck_feats.tsv`, each deck's 16
/// `encode_deck` features.
fn dump_cards(dir: &std::path::Path) {
    let mut by_name: HashMap<String, Factory> = HashMap::new();
    for f in all_catalog_card_factories() {
        let d = f();
        let front = d.name.split(" // ").next().unwrap_or(d.name).to_string();
        by_name.entry(d.name.to_string()).or_insert(f);
        by_name.entry(front).or_insert(f);
    }
    let vocab = crabomination::server::net_eval::vocab();
    let text = std::fs::read_to_string(dir.join("decks.tsv")).expect("decks.tsv");
    let mut names: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut decks = String::new();
    let mut mana = String::new();
    for line in text.lines() {
        let mut col = line.split('\t');
        let id = col.next().expect("deck id");
        let mut deck: Vec<Factory> = Vec::new();
        for name in col {
            let f = *by_name.get(name).unwrap_or_else(|| panic!("unknown card {name}"));
            if seen.insert(name.to_string()) {
                names.push(name.to_string());
            }
            deck.push(f);
        }
        let (_, feats) = crabomination::server::encode::encode_deck(&deck, vocab);
        decks.push_str(id);
        for v in feats {
            decks.push_str(&format!("\t{v}"));
        }
        decks.push('\n');
        mana.push_str(id);
        for v in crabomination::deck_value::mana_feats(&deck) {
            mana.push_str(&format!("\t{v}"));
        }
        mana.push('\n');
    }
    let mut cards = String::new();
    for name in &names {
        let o = crabomination::server::encode::encode_deck_card(by_name[name](), vocab);
        cards.push_str(&format!("{name}\t{}", o.card));
        for v in o.feats {
            cards.push_str(&format!("\t{v}"));
        }
        cards.push('\n');
    }
    std::fs::write(dir.join("card_feats.tsv"), cards).expect("write card_feats.tsv");
    std::fs::write(dir.join("deck_feats.tsv"), decks).expect("write deck_feats.tsv");
    std::fs::write(dir.join("mana_feats.tsv"), mana).expect("write mana_feats.tsv");
    eprintln!("{} cards, {} decks -> {}", names.len(), text.lines().count(), dir.display());
}

fn main() {
    let threads = num("--threads", std::thread::available_parallelism().map_or(8, |n| n.get()));
    // `--card-types`: one TSV line per FRA card (name, types, flash, pip
    // colours, mana value, sim value), for trace censuses.
    if std::env::args().any(|a| a == "--card-types") {
        for f in crabomination::fra_mode::FraPacks::get().all() {
            let d = crabomination::cube::card_def(f);
            let flash = d.keywords.iter().any(|k| matches!(k, crabomination::card::Keyword::Flash));
            let colors: String = d.cost.colored_symbols().map(|c| c.short_name()).collect();
            println!("{}\t{:?}\t{flash}\t{colors}\t{}\t{:?}", d.name, d.card_types, d.cost.cmc(), sim_value(f));
        }
        return;
    }
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
    if std::env::args().any(|a| a == "--pair-strength") {
        let field_pilot = pilot_named(arg("--pilot").as_deref().unwrap_or("fra-fixes"));
        pair_strength(arg("--deck-pilot").map_or(field_pilot, |n| pilot_named(&n)), field_pilot, num("--pools", 500), num("--pairs", 4), num("--seed", 43) as u64, threads);
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
    if let Some(path) = arg("--analyze") {
        analyze_pool(&path, std::path::Path::new(&arg("--out").expect("--analyze needs --out DIR")));
        return;
    }
    if let Some(dir) = arg("--dump-cards") {
        dump_cards(std::path::Path::new(&dir));
        return;
    }
    if std::env::args().any(|a| a == "--set-played") {
        let out = std::path::PathBuf::from(arg("--out").expect("--set-played needs --out DIR"));
        let bp = ByPlay {
            pilot: pilot_named(arg("--pilot").as_deref().unwrap_or("fra-fixes")),
            field: 0,
            pairs: num("--pairs", 4),
            seed: num("--seed", 43) as u64,
            threads,
        };
        set_played(&out, num("--sample-decks", 1000), num("--per-card", 400), &bp);
        return;
    }
    if let Some(path) = arg("--pool-adds") {
        let deck = arg("--deck").expect("--pool-adds needs --deck LIST (the deck you play)");
        let out = std::path::PathBuf::from(arg("--out").expect("--pool-adds needs --out DIR"));
        let bp = ByPlay {
            pilot: pilot_named(arg("--pilot").as_deref().unwrap_or("fra-fixes")),
            field: num("--field", 600),
            pairs: num("--pairs", 4),
            seed: num("--seed", 43) as u64,
            threads,
        };
        pool_adds(&path, &deck, &out, num("--sample-decks", 400), num("--confirm", 40), &bp);
        return;
    }
    if std::env::args().any(|a| a == "--draft-ab") {
        let pilot = pilot_named(arg("--pilot").as_deref().unwrap_or("dflt"));
        race_draft_builders(pilot, num("--pods", 250), num("--pairs", 8), num("--seed", 43) as u64, threads);
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
