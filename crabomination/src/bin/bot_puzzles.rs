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
        // Cube games; `mcts-net67-256-fixes` is the client's limited pilot.
        "fixes" => EvalWeights::default().with_fra_fixes(),
        "net67-fixes" => EvalWeights::net_on_default().with_fra_fixes(),
        "mcts-dflt-256" | "mcts-net67-256" | "mcts-net67-256-fixes" => {
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

/// The 256-iteration search profiles' configurations.
fn mcts_cfg(profile: &str) -> Option<MctsConfig> {
    let weights = match profile {
        "mcts-dflt-256" => EvalWeights::default(),
        "mcts-net67-256" => EvalWeights::net_on_default(),
        "mcts-net67-256-fixes" => EvalWeights::net_on_default().with_fra_fixes(),
        _ => return None,
    };
    Some(MctsConfig { iterations: 256, horizon_turns: 3, weights, ..MctsConfig::default() })
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
    use crabomination::server::blunder::{Area, items, judge};
    let a: Vec<String> = std::env::args().collect();
    let reps: usize = a.iter().position(|x| x == "--reps").and_then(|i| a.get(i + 1)).map_or(1, |v| v.parse().expect("--reps N"));
    let verbose = a.iter().any(|x| x == "--show-lines");
    let all = items();
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
        let areas = [Area::Attack, Area::HoldBack, Area::Block, Area::Removal, Area::Instant, Area::Mana, Area::Polarity];
        for area in areas {
            let idx: Vec<usize> = (0..all.len()).filter(|&i| all[i].area == area).collect();
            if !idx.is_empty() {
                let s: f64 = idx.iter().map(|&i| rate(i)).sum();
                println!("    {:<9} {s:.1}/{}", area.name(), idx.len());
            }
        }
        for n in &notes {
            println!("    {n}");
        }
        println!();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!(
            "bot_puzzles [--set core|fra] [--depth N] [--certify] [PROFILE ...]\n\
             bot_puzzles --blunders [--reps N] [--show-lines] [PROFILE ...]\n\
             bot_puzzles --action-gap GAMES [--probe N] [--sample P] [--threads T] [--seed S] [--scored-rollouts] [MCTS PROFILE]\n\
             bot_puzzles --menu-census GAMES [--threads T] [--seed S] [MCTS PROFILE]\n\n\
             PROFILE: pass (null control) | policy (CRAB_NET's policy head) | dflt | fixes | net67 | net67-fixes | mcts-dflt-256 | mcts-net67-256 |\n\
             \x20        mcts-net67-256-fixes (the client's limited pilot) | gang | atk-sim | net | mcts-net-deep\n\
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
    if let Some(i) = args.iter().position(|a| a == "--menu-census") {
        let games = args.get(i + 1).and_then(|v| v.parse().ok()).unwrap_or(40);
        run_menu_census(games);
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--action-gap") {
        let games = args.get(i + 1).and_then(|v| v.parse().ok()).unwrap_or(40);
        run_action_gap(games);
        return;
    }
    if args.iter().any(|a| a == "--blunders") {
        let profiles: Vec<String> = args.iter().filter(|a| !a.starts_with("--") && a.parse::<usize>().is_err()).cloned().collect();
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
