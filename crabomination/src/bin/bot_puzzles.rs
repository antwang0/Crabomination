//! Puzzle benchmark runner.
//!
//! Two modes:
//!
//! * `--certify` — score every *candidate* against the certifier: is
//!   there a solution, and does the default heuristic already find it?
//!   This is the corpus-authoring tool. A candidate that certifies
//!   trivial is not a puzzle, and one with no solution is a broken
//!   position; both are reported rather than quietly dropped.
//! * default — run each profile against the kept corpus and report
//!   pass rates overall, by mechanic, and by difficulty tier.
//!
//! Unlike `bot_ladder`, this is deterministic: the same binary on the
//! same corpus gives the same number every time, so a one-point move is
//! a real change rather than a sampling artifact.

use crabomination::server::puzzle::{self, Goal};
use crabomination::server::puzzle_corpus::{Certified, Mechanic, Puzzle, certify};
use crabomination::server::{Bot, EvalWeights, HeuristicBot, MctsBot, MctsConfig};

const CHAMPION: &str = "nets/champion.safetensors";
/// `--depth N` (default 3): the solver's search depth, in substantive choices.
fn max_depth() -> usize {
    let a: Vec<String> = std::env::args().collect();
    a.iter().position(|x| x == "--depth").and_then(|i| a.get(i + 1)).map_or(3, |v| v.parse().expect("--depth N"))
}

/// `--set fra` picks the Reality Fracture corpus (`server::puzzle_fra`);
/// the default is the original eleven.
fn candidates() -> Vec<Puzzle> {
    let a: Vec<String> = std::env::args().collect();
    match a.iter().position(|x| x == "--set").and_then(|i| a.get(i + 1)).map(String::as_str) {
        Some("fra") => crabomination::server::puzzle_fra::candidates(),
        None | Some("core") => crabomination::server::puzzle_corpus::candidates(),
        Some(other) => panic!("--set {other}: expected core or fra"),
    }
}

/// The null control: never chooses anything (decisions still go to the
/// default heuristic, which owns the answer to a prompt). Every puzzle must
/// fail under it — one that passes is not testing a choice at all, e.g. a
/// survival goal the opponent never threatens.
struct PassBot(HeuristicBot);

impl Bot for PassBot {
    fn next_action(&mut self, state: &crabomination::game::GameState, seat: usize) -> Option<crabomination::game::GameAction> {
        if state.pending_decision.as_ref().is_some_and(|d| d.acting_player() == seat) {
            return self.0.next_action(state, seat);
        }
        None
    }
}

fn bot_for(profile: &str) -> Option<Box<dyn Bot>> {
    if profile == "pass" {
        return Some(Box::new(PassBot(HeuristicBot::new())));
    }
    // The policy pilot on the loaded net's policy head (`CRAB_NET`).
    if profile == "policy" {
        return Some(Box::new(crabomination::server::policy_pilot::PolicyBot::new(EvalWeights::net_on_default(), 0.0, 0)));
    }
    let w = match profile {
        "gang" => EvalWeights::default(),
        "atk-sim" => EvalWeights::attack_search_sim(),
        "net" => EvalWeights::net_eval_det1(),
        // Today's pilots (the ladder's names): the adopted default, the
        // default on the net leaf, and both under 256-iteration search.
        "dflt" => EvalWeights::default(),
        "net67" => EvalWeights::net_on_default(),
        // The FRA fixes bundle, which the client plays in FRA, Sealed and
        // Cube games, under the menu search (`mcts_cfg`).
        "fixes" => EvalWeights::default().with_fra_fixes(),
        "net67-fixes" => EvalWeights::net_on_default().with_fra_fixes(),
        p if mcts_cfg(p).is_some() => {
            return Some(Box::new(MctsBot::new(mcts_cfg(profile).expect("an MCTS profile"))));
        }
        "mcts-net-deep" => {
            return Some(Box::new(MctsBot::new(MctsConfig {
                iterations: 64,
                horizon_turns: 3,
                weights: EvalWeights::net_eval_det1(),
                ..MctsConfig::default()
            })));
        }
        _ => return None,
    };
    Some(Box::new(HeuristicBot::with_weights(w)))
}

/// The 256-iteration search profiles' configurations. A
/// `mcts-net67-menu-<windows>[-suffix..]` name is the ladder's menu search
/// (`server::mcts::menu_profile`; priors from `CRAB_POLICY_NET`):
/// `mcts-net67-menu-mc-df-anm10` is the client's limited pilot since
/// 2026-11-10, `mcts-net67-menu-mc` the one before it (2026-11-02), and
/// `mcts-net67-menu-mc-fixes` this tool's first name for that one.
fn mcts_cfg(profile: &str) -> Option<MctsConfig> {
    if profile.starts_with("mcts-net67-menu-") {
        assert!(
            std::env::var_os("CRAB_POLICY_NET").is_some(),
            "a `-menu-` profile needs CRAB_POLICY_NET=<policy-headed weights>"
        );
        let name = if profile == "mcts-net67-menu-mc-fixes" { "mcts-net67-menu-mc" } else { profile };
        return crabomination::server::mcts::menu_profile(name);
    }
    let weights = match profile {
        "mcts-dflt-256" => EvalWeights::default(),
        "mcts-net67-256" => EvalWeights::net_on_default(),
        "mcts-net67-256-fixes" => EvalWeights::net_on_default().with_fra_fixes(),
        _ => return None,
    };
    Some(MctsConfig { iterations: 256, horizon_turns: 3, weights, ..MctsConfig::default() })
}

/// `--leaf-pairs`: the value net (`CRAB_NET`, default the champion) and the
/// material heuristic on `blunder::leaf_pairs` — two boards that differ in
/// one thing, the first better for seat 0. A pair the net orders the wrong
/// way is a leaf the search is misled at.
fn run_leaf_pairs() {
    use crabomination::server::blunder::{leaf_pairs, material};
    use crabomination::server::net_eval::{SLOT_BEST, win_prob};
    let pairs = leaf_pairs();
    println!("leaf pairs: {} (net = win probability for seat 0; material = the heuristic's score)\n", pairs.len());
    println!("{:<38} {:>7} {:>7} {:>7}   {:>6} {:>6}", "pair", "better", "worse", "net Δ", "mat b", "mat w");
    let (mut net_ok, mut mat_ok) = (0, 0);
    for p in &pairs {
        let (b, w) = (p.build)();
        let (nb, nw) = (win_prob(&b, 0, SLOT_BEST).unwrap_or(f32::NAN), win_prob(&w, 0, SLOT_BEST).unwrap_or(f32::NAN));
        let (mb, mw) = (material(&b, 0), material(&w, 0));
        net_ok += usize::from(nb > nw);
        mat_ok += usize::from(mb > mw);
        println!(
            "{:<38} {nb:>7.3} {nw:>7.3} {:>+7.3} {} {mb:>6} {mw:>6} {}",
            p.id,
            nb - nw,
            if nb > nw { " " } else { "✗" },
            if mb > mw { " " } else { "✗" }
        );
    }
    println!("\nordered the right way: net {net_ok}/{}, material {mat_ok}/{}", pairs.len(), pairs.len());
    for p in &pairs {
        println!("  {}: {}", p.id, p.prompt);
    }
}

/// `--action-gap GAMES [--probe N] [--sample P] [--threads T] [--seed S]
/// [PROFILE]`: the legal-action gap (`server::action_gap`) over sealed
/// games between client-built decks (`CRAB_SEALED_SET=fra` for FRA pools;
/// `CRAB_NET` picks the leaf). PROFILE is a search profile, default the
/// client's limited pilot.
fn run_action_gap(games: usize) {
    use crabomination::server::action_gap::{Class, Context, GapConfig, Point, audit_game};
    let a: Vec<String> = std::env::args().collect();
    let flag = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
    let mut cfg = GapConfig::default();
    if let Some(v) = flag("--probe") {
        cfg.probe = v.parse().expect("--probe N");
    }
    if let Some(v) = flag("--sample") {
        cfg.sample = v.parse().expect("--sample P");
    }
    cfg.scored_rollouts = a.iter().any(|x| x == "--scored-rollouts");
    let threads: usize = flag("--threads").map_or(8, |v| v.parse().expect("--threads T"));
    let seed: u64 = flag("--seed").map_or(43, |v| v.parse().expect("--seed S"));
    let profile = a
        .iter()
        .skip(1)
        .find(|x| mcts_cfg(x).is_some())
        .cloned()
        .unwrap_or_else(|| "mcts-net67-256-fixes".to_string());
    let pilot = mcts_cfg(&profile).expect("checked");
    println!(
        "action gap: {games} games, {profile}, probe {} {} rollouts x 2 passes, sample {}, {threads} threads, seed {seed}\n",
        cfg.probe,
        if cfg.scored_rollouts { "scored" } else { "unscored" },
        cfg.sample
    );
    let next = std::sync::atomic::AtomicUsize::new(0);
    let points: Vec<Point> = std::thread::scope(|sc| {
        let hs: Vec<_> = (0..threads)
            .map(|_| {
                sc.spawn(|| {
                    let mut mine = Vec::new();
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if i >= games {
                            break;
                        }
                        let s = seed ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
                        let build = |k: u64| {
                            let pool = crabomination::selfplay::sealed_pool(s ^ k);
                            crabomination::selfplay::sealed_opponent_build(&pool, 16, s ^ (k << 8), Default::default())
                        };
                        let template = crabomination::selfplay::sealed_game_template(&build(1), &build(2));
                        mine.extend(audit_game(&template, pilot, cfg, s));
                        eprint!(".");
                    }
                    mine
                })
            })
            .collect();
        hs.into_iter().flat_map(|h| h.join().expect("worker")).collect()
    });
    eprintln!();
    let stats = |ps: &[&Point]| {
        let n = ps.len() as f64;
        let r: Vec<f64> = ps.iter().map(|p| p.regret()).collect();
        let mean = r.iter().sum::<f64>() / n.max(1.0);
        let var = r.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
        (mean, (var / n.max(1.0)).sqrt(), r.iter().filter(|&&x| x > 0.10).count())
    };
    println!(
        "{:<11} {:>6} {:>7} {:>9} {:>16} {:>9}  better move's class when regret > 0.10",
        "context", "points", "legal", "same", "mean regret", ">0.10"
    );
    for ctx in Context::ALL {
        let ps: Vec<&Point> = points.iter().filter(|p| p.context == ctx).collect();
        if ps.is_empty() {
            continue;
        }
        let (mean, se, big) = stats(&ps);
        let same = ps.iter().filter(|p| p.same).count();
        let legal = ps.iter().map(|p| p.legal).sum::<usize>() as f64 / ps.len() as f64;
        let classes: Vec<String> = Class::ALL
            .iter()
            .filter_map(|&c| {
                let k = ps.iter().filter(|p| p.regret() > 0.10 && Class::of(&p.best) == c).count();
                (k > 0).then(|| format!("{} {k}", c.name()))
            })
            .collect();
        println!(
            "{:<11} {:>6} {:>7.1} {:>8.0}% {:>+8.3} ±{:.3} {:>8.0}%  {}",
            ctx.name(),
            ps.len(),
            legal,
            100.0 * same as f64 / ps.len() as f64,
            mean,
            se,
            100.0 * big as f64 / ps.len() as f64,
            classes.join(", ")
        );
    }
    let main: Vec<&Point> = points.iter().filter(|p| p.best_on_menu.is_some()).collect();
    let off = main.iter().filter(|p| p.best_on_menu == Some(false)).count();
    let off_big: Vec<&&Point> = main.iter().filter(|p| p.best_on_menu == Some(false) && p.regret() > 0.10).collect();
    println!(
        "\nown main: the better move was off the search's menu at {off} of {} points ({:.0}%), {} of them with regret > 0.10",
        main.len(),
        100.0 * off as f64 / main.len().max(1) as f64,
        off_big.len()
    );
    let mut worst: Vec<&Point> = points.iter().collect();
    worst.sort_by(|x, y| y.regret().partial_cmp(&x.regret()).unwrap_or(std::cmp::Ordering::Equal));
    println!("\nthe 25 largest regrets:");
    for p in worst.iter().take(25) {
        println!("  {:+.3} [{}] {}", p.regret(), p.context.name(), p.note);
    }
}

/// `--menu-census GAMES [--threads T] [--seed S] [MCTS PROFILE]`: the move
/// list's coverage (`server::decision_menu`) — at every decision the pilot
/// makes in sealed games between client-built decks, is its move on the
/// list? (`CRAB_SEALED_SET=fra` for FRA pools; `CRAB_NET` the leaf.)
fn run_menu_census(games: usize) {
    use crabomination::server::decision_menu::{CensusRow, census_game};
    let a: Vec<String> = std::env::args().collect();
    let flag = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
    let threads: usize = flag("--threads").map_or(8, |v| v.parse().expect("--threads T"));
    let seed: u64 = flag("--seed").map_or(43, |v| v.parse().expect("--seed S"));
    let profile = a
        .iter()
        .skip(1)
        .find(|x| mcts_cfg(x).is_some())
        .cloned()
        .unwrap_or_else(|| "mcts-net67-256-fixes".to_string());
    let pilot = mcts_cfg(&profile).expect("checked");
    println!("menu census: {games} games, {profile}, {threads} threads, seed {seed}\n");
    let next = std::sync::atomic::AtomicUsize::new(0);
    let rows: Vec<CensusRow> = std::thread::scope(|sc| {
        let hs: Vec<_> = (0..threads)
            .map(|_| {
                sc.spawn(|| {
                    let mut mine = Vec::new();
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if i >= games {
                            break;
                        }
                        let s = seed ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
                        let build = |k: u64| {
                            let pool = crabomination::selfplay::sealed_pool(s ^ k);
                            crabomination::selfplay::sealed_opponent_build(&pool, 16, s ^ (k << 8), Default::default())
                        };
                        let template = crabomination::selfplay::sealed_game_template(&build(1), &build(2));
                        mine.extend(census_game(&template, pilot, s));
                        eprint!(".");
                    }
                    mine
                })
            })
            .collect();
        hs.into_iter().flat_map(|h| h.join().expect("worker")).collect()
    });
    eprintln!();
    let mut kinds: Vec<_> = rows.iter().map(|r| r.kind).collect();
    kinds.sort();
    kinds.dedup();
    let covered = rows.iter().filter(|r| r.covered).count();
    let tot = |f: fn(&CensusRow) -> f64| rows.iter().map(f).sum::<f64>();
    println!(
        "time per decision (release build, under the pilot's load): list {:.0} µs, successors {:.0} µs",
        tot(|r| r.menu_us) / rows.len().max(1) as f64,
        tot(|r| r.succ_us) / rows.len().max(1) as f64
    );
    println!(
        "all decisions: {} — the pilot's move on the list at {covered} ({:.2} %)\n",
        rows.len(),
        100.0 * covered as f64 / rows.len().max(1) as f64
    );
    println!(
        "{:<24} {:>8} {:>9} {:>10} {:>8} {:>6} {:>9} {:>9} {:>9}",
        "kind", "count", "covered", "truncated", "mean", "max", "rejected", "list µs", "succ µs"
    );
    for k in kinds {
        let rs: Vec<&CensusRow> = rows.iter().filter(|r| r.kind == k).collect();
        let n = rs.len() as f64;
        println!(
            "{:<24} {:>8} {:>8.2}% {:>9.2}% {:>8.1} {:>6} {:>9} {:>9.0} {:>9.0}",
            k.name(),
            rs.len(),
            100.0 * rs.iter().filter(|r| r.covered).count() as f64 / n,
            100.0 * rs.iter().filter(|r| r.truncated).count() as f64 / n,
            rs.iter().map(|r| r.moves).sum::<usize>() as f64 / n,
            rs.iter().map(|r| r.moves).max().unwrap_or(0),
            rs.iter().map(|r| r.rejected).sum::<usize>(),
            rs.iter().map(|r| r.menu_us).sum::<f64>() / n,
            rs.iter().map(|r| r.succ_us).sum::<f64>() / n,
        );
    }
    let mut misses: Vec<(String, usize, String)> = Vec::new();
    for r in rows.iter().filter(|r| !r.covered) {
        let v = format!("{} in {}", r.variant, r.kind.name());
        match misses.iter_mut().find(|m| m.0 == v) {
            Some(m) => m.1 += 1,
            None => misses.push((v, 1, r.chosen.clone())),
        }
    }
    misses.sort_by_key(|m| std::cmp::Reverse(m.1));
    println!("\nmisses by the pilot's move:");
    for (v, n, ex) in &misses {
        println!("  {n:>6}  {v:<36} e.g. {}", ex.chars().take(160).collect::<String>());
    }
}

/// `--policy-agree GAMES [--threads T] [--seed S] [TEACHER]`: the policy
/// pilot (`CRAB_NET`'s policy head) against a teacher's moves at every
/// decision of the teacher's own sealed games — by kind, and by whether the
/// teacher acted or passed. TEACHER is any profile (default the client's
/// limited pilot).
fn run_policy_agree(games: usize) {
    use crabomination::server::policy_pilot::{AgreeRow, PolicyBot, agreement_game};
    let a: Vec<String> = std::env::args().collect();
    let flag = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
    let threads: usize = flag("--threads").map_or(8, |v| v.parse().expect("--threads T"));
    let seed: u64 = flag("--seed").map_or(43, |v| v.parse().expect("--seed S"));
    let teacher = a
        .iter()
        .skip(1)
        .find(|x| !x.starts_with("--") && x.parse::<u64>().is_err() && bot_for(x).is_some())
        .cloned()
        .unwrap_or_else(|| "mcts-net67-256-fixes".to_string());
    let w = EvalWeights::net_on_default().with_fra_fixes();
    println!("policy agreement: {games} games, teacher {teacher}, {threads} threads, seed {seed}\n");
    let next = std::sync::atomic::AtomicUsize::new(0);
    let rows: Vec<AgreeRow> = std::thread::scope(|sc| {
        let hs: Vec<_> = (0..threads)
            .map(|t| {
                let teacher = teacher.clone();
                let next = &next;
                sc.spawn(move || {
                    let mut mine = Vec::new();
                    let mut policy = PolicyBot::new(w, 0.0, t as u64);
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if i >= games {
                            break;
                        }
                        let s = seed ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
                        let build = |k: u64| {
                            let pool = crabomination::selfplay::sealed_pool(s ^ k);
                            crabomination::selfplay::sealed_opponent_build(&pool, 16, s ^ (k << 8), Default::default())
                        };
                        let template = crabomination::selfplay::sealed_game_template(&build(1), &build(2));
                        let mut teachers = [bot_for(&teacher).expect("teacher"), bot_for(&teacher).expect("teacher")];
                        mine.extend(agreement_game(&template, &mut teachers, &mut policy, &w, s));
                        eprint!(".");
                    }
                    mine
                })
            })
            .collect();
        hs.into_iter().flat_map(|h| h.join().expect("worker")).collect()
    });
    eprintln!();
    let pct = |n: usize, d: usize| 100.0 * n as f64 / d.max(1) as f64;
    let agree = rows.iter().filter(|r| r.agree).count();
    println!("decisions with 2+ moves: {} — agreement {:.1} %\n", rows.len(), pct(agree, rows.len()));
    println!("{:<20} {:>7} {:>7} {:>9} {:>11} {:>11} {:>13}", "kind", "count", "moves", "agree", "t acted", "agree|act", "pol passed|act");
    let mut kinds: Vec<_> = rows.iter().map(|r| r.kind).collect();
    kinds.sort();
    kinds.dedup();
    for k in kinds {
        let rs: Vec<&AgreeRow> = rows.iter().filter(|r| r.kind == k).collect();
        let acted: Vec<&&AgreeRow> = rs.iter().filter(|r| !r.teacher_passed).collect();
        println!(
            "{:<20} {:>7} {:>7.1} {:>8.1}% {:>10.1}% {:>10.1}% {:>12.1}%",
            k.name(),
            rs.len(),
            rs.iter().map(|r| r.moves).sum::<usize>() as f64 / rs.len() as f64,
            pct(rs.iter().filter(|r| r.agree).count(), rs.len()),
            pct(acted.len(), rs.len()),
            pct(acted.iter().filter(|r| r.agree).count(), acted.len()),
            pct(acted.iter().filter(|r| r.policy_passed).count(), acted.len()),
        );
    }
}

fn goal_name(g: Goal) -> String {
    match g {
        Goal::WinThisTurn => "win".into(),
        Goal::SurviveTurn => "survive".into(),
        Goal::ClearOpposingCreatures => "clear".into(),
        Goal::TakeNoDamage => "no-damage".into(),
        Goal::WinWithin(n) => format!("win+{n}"),
        Goal::SurviveWithin(n) => format!("live+{n}"),
    }
}

fn run_certify() {
    let cands = candidates();
    println!("certifying {} candidates (max depth {})\n", cands.len(), max_depth());
    println!(
        "{:<32} {:<9} {:<8} {:>6}  {:<8}  prompt",
        "id", "mechanic", "goal", "depth", "verdict"
    );
    let mut kept = 0;
    for p in &cands {
        let c: Certified = certify(p, max_depth());
        let depth = c.depth.map(|d| d.to_string()).unwrap_or_else(|| "-".into());
        // A candidate is a puzzle only if it is solvable AND the default
        // policy does not already find it. Anything else is reported with
        // the reason, because "dropped silently" is how a benchmark ends
        // up measuring nothing.
        let verdict = if c.keep() {
            kept += 1;
            "KEEP"
        } else if c.depth.is_none() {
            "unsolvable"
        } else if c.depth == Some(0) {
            "trivial"
        } else {
            "pass-only"
        };
        let trunc = if c.truncated { " (truncated)" } else { "" };
        println!(
            "{:<32} {:<9} {:<8} {:>6}  {:<8}  {}{}",
            p.id,
            p.mechanic.name(),
            goal_name(p.goal),
            depth,
            verdict,
            p.prompt,
            trunc
        );
        if std::env::args().any(|a| a == "--show-lines")
            && let Some(cert) = puzzle::solve(&(p.build)(), p.seat, p.goal, max_depth())
        {
            for (i, a) in cert.line.iter().enumerate() {
                println!("      {i}: {a:?}");
            }
        }
    }
    println!("\n{kept}/{} candidates certify as real puzzles", cands.len());
}

fn run_scoring(profiles: &[String]) {
    let cands = candidates();
    let kept: Vec<(&Puzzle, Certified)> = cands
        .iter()
        .map(|p| (p, certify(p, max_depth())))
        // `--all` scores every candidate, kept or not — the sanity read: the
        // `pass` control should fail all of them, and a trivial one should
        // pass under `dflt`.
        .filter(|(_, c)| c.keep() || std::env::args().any(|a| a == "--all"))
        .collect();
    if kept.is_empty() {
        println!("no candidate certifies as a real puzzle — nothing to score");
        return;
    }
    println!("corpus: {} puzzles (of {} candidates)\n", kept.len(), cands.len());

    for profile in profiles {
        let Some(_probe) = bot_for(profile) else {
            eprintln!("unknown profile {profile}");
            continue;
        };
        let mut passed = 0usize;
        let mut by_mech: Vec<(Mechanic, usize, usize)> = Vec::new();
        let mut failures: Vec<&str> = Vec::new();
        for (p, c) in &kept {
            // A fresh bot per puzzle: these carry per-combat latch state,
            // and reusing one across positions would leak a declaration
            // from the previous puzzle into the next.
            let mut bot = bot_for(profile).expect("checked above");
            let g = (p.build)();
            let ok = puzzle::passes(&g, p.seat, p.goal, bot.as_mut());
            if ok {
                passed += 1;
            } else {
                failures.push(p.id);
            }
            match by_mech.iter_mut().find(|(m, _, _)| *m == p.mechanic) {
                Some(e) => {
                    e.1 += usize::from(ok);
                    e.2 += 1;
                }
                None => by_mech.push((p.mechanic, usize::from(ok), 1)),
            }
            let _ = c;
        }
        let pct = 100.0 * passed as f64 / kept.len() as f64;
        println!("{profile}: {passed}/{} ({pct:.1} %)", kept.len());
        for (m, ok, n) in &by_mech {
            println!("    {:<9} {ok}/{n}", m.name());
        }
        if !failures.is_empty() {
            println!("    failed: {}", failures.join(", "));
        }
        println!();
    }
}

/// `--blunders`: score each profile on the blunder set (`server::blunder`),
/// `--reps N` times an item (the search draws its own random stream, so a
/// searched pilot's verdict is a rate; the heuristic's repeats).
fn run_blunders(profiles: &[String]) {
    use crabomination::server::blunder::{Area, Cards, items, judge};
    let a: Vec<String> = std::env::args().collect();
    let reps: usize = a.iter().position(|x| x == "--reps").and_then(|i| a.get(i + 1)).map_or(1, |v| v.parse().expect("--reps N"));
    let verbose = a.iter().any(|x| x == "--show-lines");
    // `--only ID[,ID..]`: just these items (for reading one failure's trace).
    let only: Option<Vec<String>> =
        a.iter().position(|x| x == "--only").and_then(|i| a.get(i + 1)).map(|v| v.split(',').map(str::to_string).collect());
    // `--cards plain|sos|fra`: the items built from that pool's cards.
    let cards_only: Option<String> = a.iter().position(|x| x == "--cards").and_then(|i| a.get(i + 1)).cloned();
    let mut all = items();
    if let Some(only) = &only {
        all.retain(|i| only.iter().any(|o| o == i.id));
    }
    if let Some(c) = &cards_only {
        all.retain(|i| i.cards.name() == c);
    }
    let pairs = all.iter().filter(|i| i.twin.is_some_and(|t| i.id < t)).count();
    println!("blunder set: {} items ({pairs} twin pairs), {reps} rep(s) each\n", all.len());
    for profile in profiles {
        if bot_for(profile).is_none() {
            eprintln!("unknown profile {profile}");
            continue;
        }
        // Per item: passes out of `reps`.
        let mut score: Vec<usize> = Vec::with_capacity(all.len());
        let mut notes: Vec<String> = Vec::new();
        for it in &all {
            let mut ok = 0;
            for _ in 0..reps {
                let mut bot = bot_for(profile).expect("checked above");
                let v = judge(it, bot.as_mut());
                if v.ok() {
                    ok += 1;
                } else if notes.last().is_none_or(|n| !n.starts_with(it.id)) {
                    notes.push(format!("{}: failed to {}", it.id, v.failed.join(", ")));
                    if verbose {
                        for act in &v.actions {
                            notes.push(format!("{}      {act:?}", it.id));
                        }
                    }
                }
            }
            score.push(ok);
        }
        let rate = |i: usize| score[i] as f64 / reps as f64;
        let items_ok: f64 = (0..all.len()).map(rate).sum();
        // A pair scores the chance both halves pass — the product of the
        // two rates, which is 0/1 for a deterministic pilot.
        let mut pair_ok = 0.0;
        for (i, it) in all.iter().enumerate() {
            if let Some(t) = it.twin.filter(|t| it.id < *t) {
                let j = all.iter().position(|o| o.id == t).expect("twin");
                pair_ok += rate(i) * rate(j);
            }
        }
        println!("{profile}: items {items_ok:.1}/{}, pairs {pair_ok:.1}/{pairs}", all.len());
        // Each earlier version's totals, the numbers notes written then
        // quote (22 items before 2026-11-10, 35 before 2026-11-12), and
        // the real-card items by pool.
        let newest = all.iter().map(|i| i.since).max().unwrap_or(1);
        for v in 1..newest {
            let vi: Vec<usize> = (0..all.len()).filter(|&i| all[i].since <= v).collect();
            if vi.is_empty() {
                continue;
            }
            let s: f64 = vi.iter().map(|&i| rate(i)).sum();
            let (mut pv, mut nv) = (0.0, 0);
            for &i in &vi {
                if let Some(t) = all[i].twin.filter(|t| all[i].id < *t)
                    && let Some(j) = all.iter().position(|o| o.id == t)
                {
                    pv += rate(i) * rate(j);
                    nv += 1;
                }
            }
            println!("    through version {v}: items {s:.1}/{}, pairs {pv:.1}/{nv}", vi.len());
        }
        for c in [Cards::Plain, Cards::Sos, Cards::Fra] {
            let idx: Vec<usize> = (0..all.len()).filter(|&i| all[i].cards == c).collect();
            if !idx.is_empty() && idx.len() != all.len() {
                let s: f64 = idx.iter().map(|&i| rate(i)).sum();
                println!("    cards {:<5} {s:.1}/{}", c.name(), idx.len());
            }
        }
        let areas = [Area::Attack, Area::HoldBack, Area::Block, Area::Removal, Area::Instant, Area::Mana, Area::Polarity, Area::Lethal];
        for area in areas {
            let idx: Vec<usize> = (0..all.len()).filter(|&i| all[i].area == area).collect();
            if !idx.is_empty() {
                let s: f64 = idx.iter().map(|&i| rate(i)).sum();
                println!("    {:<9} {s:.1}/{}", area.name(), idx.len());
            }
        }
        // Items a stochastic pilot passes only some of the time.
        for (i, it) in all.iter().enumerate() {
            if score[i] != 0 && score[i] != reps {
                println!("    {}: {}/{reps}", it.id, score[i]);
            }
        }
        for n in &notes {
            println!("    {n}");
        }
        println!();
    }
}

/// `--crn-census GAMES [--probe N] [--arms A] [--sample P] [--threads T]
/// [--seed S] [PROFILE]`: how much common random numbers would sharpen the
/// search's arm comparisons (`server::action_gap::crn_game`). Prints, per
/// window, the pooled variance ratio independent / paired for an arm
/// difference (paired rollouts are worth that many independent ones) and
/// how often the first-cut leader (probe / arms rollouts per arm) is the
/// full-probe leader, independent vs paired.
fn run_crn_census(games: usize) {
    use crabomination::server::action_gap::{Context, CrnPoint, crn_game};
    let a: Vec<String> = std::env::args().collect();
    let flag = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
    let probe: u32 = flag("--probe").map_or(80, |v| v.parse().expect("--probe N"));
    let arms: usize = flag("--arms").map_or(8, |v| v.parse().expect("--arms A"));
    let sample: f64 = flag("--sample").map_or(0.15, |v| v.parse().expect("--sample P"));
    let threads: usize = flag("--threads").map_or(8, |v| v.parse().expect("--threads T"));
    let seed: u64 = flag("--seed").map_or(43, |v| v.parse().expect("--seed S"));
    let profile = a
        .iter()
        .skip(1)
        .find(|x| mcts_cfg(x).is_some())
        .cloned()
        .unwrap_or_else(|| "mcts-net67-256-fixes".to_string());
    let pilot = mcts_cfg(&profile).expect("checked");
    println!("crn census: {games} games, {profile}, {arms} arms x {probe} rollouts, sample {sample}, {threads} threads, seed {seed}\n");
    let next = std::sync::atomic::AtomicUsize::new(0);
    let points: Vec<CrnPoint> = std::thread::scope(|sc| {
        let hs: Vec<_> = (0..threads)
            .map(|_| {
                sc.spawn(|| {
                    let mut mine = Vec::new();
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if i >= games {
                            break;
                        }
                        let s = seed ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
                        let build = |k: u64| {
                            let pool = crabomination::selfplay::sealed_pool(s ^ k);
                            crabomination::selfplay::sealed_opponent_build(&pool, 16, s ^ (k << 8), Default::default())
                        };
                        let template = crabomination::selfplay::sealed_game_template(&build(1), &build(2));
                        mine.extend(crn_game(&template, pilot, probe, arms, sample, s));
                        eprint!(".");
                    }
                    mine
                })
            })
            .collect();
        hs.into_iter().flat_map(|h| h.join().expect("worker")).collect()
    });
    eprintln!();
    println!(
        "{:<11} {:>6} {:>5} {:>8} {:>14} {:>14} {:>9} {:>9}",
        "window", "points", "arms", "pairs", "pooled ratio", "median ratio", "top ind", "top crn"
    );
    let row = |name: &str, ps: &[&CrnPoint]| {
        if ps.is_empty() {
            return;
        }
        let pairs: Vec<(f64, f64)> = ps.iter().flat_map(|p| p.pairs.iter().copied()).collect();
        let (ind, crn) = pairs.iter().fold((0.0, 0.0), |(a, b), &(i, c)| (a + i, b + c));
        let mut ratios: Vec<f64> = ps
            .iter()
            .filter_map(|p| {
                let (i, c) = p.pairs.iter().fold((0.0, 0.0), |(a, b), &(i, c)| (a + i, b + c));
                (c > 0.0).then(|| i / c)
            })
            .collect();
        ratios.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
        let med = ratios.get(ratios.len() / 2).copied().unwrap_or(f64::NAN);
        let n = ps.len() as f64;
        println!(
            "{:<11} {:>6} {:>5.1} {:>8} {:>14.2} {:>14.2} {:>8.0}% {:>8.0}%",
            name,
            ps.len(),
            ps.iter().map(|p| p.arms).sum::<usize>() as f64 / n,
            pairs.len(),
            if crn > 0.0 { ind / crn } else { f64::NAN },
            med,
            100.0 * ps.iter().filter(|p| p.top_match_ind).count() as f64 / n,
            100.0 * ps.iter().filter(|p| p.top_match_crn).count() as f64 / n,
        );
    };
    for ctx in Context::ALL {
        let ps: Vec<&CrnPoint> = points.iter().filter(|p| p.context == ctx).collect();
        row(ctx.name(), &ps);
    }
    let all: Vec<&CrnPoint> = points.iter().collect();
    row("all", &all);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!(
            "bot_puzzles [--set core|fra] [--depth N] [--certify] [PROFILE ...]\n\
             bot_puzzles --blunders [--reps N] [--show-lines] [--only ID,ID] [--cards plain|sos|fra] [PROFILE ...]\n\
             bot_puzzles --leaf-pairs   (CRAB_NET's value head and the material heuristic on pairs of boards)\n\
             bot_puzzles --action-gap GAMES [--probe N] [--sample P] [--threads T] [--seed S] [--scored-rollouts] [MCTS PROFILE]\n\
             bot_puzzles --crn-census GAMES [--probe N] [--arms A] [--sample P] [--threads T] [--seed S] [MCTS PROFILE]\n\
             bot_puzzles --menu-census GAMES [--threads T] [--seed S] [MCTS PROFILE]\n\
             bot_puzzles --policy-agree GAMES [--threads T] [--seed S] [TEACHER]  (CRAB_NET's policy head)\n\n\
             PROFILE: pass (null control) | policy (CRAB_NET's policy head) | dflt | fixes | net67 | net67-fixes | mcts-dflt-256 | mcts-net67-256 |\n\
             \x20        mcts-net67-256-fixes (the client's limited pilot before 2026-11-02) |\n\
             \x20        mcts-net67-menu-mc[-df][-an<N>|-anm<N>].. (the menu search and the ladder's suffixes; needs CRAB_POLICY_NET;\n\
             \x20        the client's limited pilot is mcts-net67-menu-mc-df-anm10, before 2026-11-10 mcts-net67-menu-mc) |\n\
             \x20        gang | atk-sim | net | mcts-net-deep\n\
             --certify reports which candidates are real puzzles. CRAB_NET overrides the champion."
        );
        return;
    }
    // The net profiles read the champion out of a slot; without it they
    // silently degrade to the heuristic, which would make a net column
    // that is really a second gang column.
    let net = std::env::var("CRAB_NET").unwrap_or_else(|_| CHAMPION.to_string());
    if std::path::Path::new(&net).exists() {
        crabomination::server::net_eval::load_slot(
            crabomination::server::net_eval::SLOT_BEST,
            std::path::Path::new(&net),
        )
        .expect("load the value net");
    }
    // The menu search's priors (`mcts-net67-menu-mc-fixes`).
    if let Ok(pp) = std::env::var("CRAB_POLICY_NET") {
        crabomination::server::net_eval::load_slot(
            crabomination::server::net_eval::SLOT_CANDIDATE,
            std::path::Path::new(&pp),
        )
        .expect("load the policy net");
    }
    if args.iter().any(|a| a == "--leaf-pairs") {
        run_leaf_pairs();
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--policy-agree") {
        let games = args.get(i + 1).and_then(|v| v.parse().ok()).unwrap_or(20);
        run_policy_agree(games);
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--menu-census") {
        let games = args.get(i + 1).and_then(|v| v.parse().ok()).unwrap_or(40);
        run_menu_census(games);
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--crn-census") {
        let games = args.get(i + 1).and_then(|v| v.parse().ok()).unwrap_or(40);
        run_crn_census(games);
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--action-gap") {
        let games = args.get(i + 1).and_then(|v| v.parse().ok()).unwrap_or(40);
        run_action_gap(games);
        return;
    }
    if args.iter().any(|a| a == "--blunders") {
        let only = args.iter().position(|a| a == "--only").map(|i| i + 1);
        let cards = args.iter().position(|a| a == "--cards").map(|i| i + 1);
        let profiles: Vec<String> = args
            .iter()
            .enumerate()
            .filter(|(i, a)| Some(*i) != only && Some(*i) != cards && !a.starts_with("--") && a.parse::<usize>().is_err())
            .map(|(_, a)| a.clone())
            .collect();
        let profiles = if profiles.is_empty() { vec!["pass".to_string(), "dflt".to_string()] } else { profiles };
        run_blunders(&profiles);
        return;
    }
    if args.iter().any(|a| a == "--certify") {
        run_certify();
        return;
    }
    let mut profiles: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if matches!(args[i].as_str(), "--set" | "--depth") {
            i += 2;
            continue;
        }
        if !args[i].starts_with("--") {
            profiles.push(args[i].clone());
        }
        i += 1;
    }
    let profiles = if profiles.is_empty() {
        vec!["dflt".to_string(), "net67".to_string()]
    } else {
        profiles
    };
    run_scoring(&profiles);
}
