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
use crabomination::server::puzzle_corpus::{Certified, Mechanic, Puzzle, candidates, certify};
use crabomination::server::{Bot, EvalWeights, HeuristicBot, MctsBot, MctsConfig};

const CHAMPION: &str = "nets/champion.safetensors";
const MAX_DEPTH: usize = 3;

fn bot_for(profile: &str) -> Option<Box<dyn Bot>> {
    let w = match profile {
        "gang" => EvalWeights::default(),
        "atk-sim" => EvalWeights::attack_search_sim(),
        "net" => EvalWeights::net_eval_det1(),
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
    println!("certifying {} candidates (max depth {MAX_DEPTH})\n", cands.len());
    println!(
        "{:<32} {:<9} {:<8} {:>6}  {:<8}  {}",
        "id", "mechanic", "goal", "depth", "verdict", "prompt"
    );
    let mut kept = 0;
    for p in &cands {
        let c: Certified = certify(p, MAX_DEPTH);
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
            && let Some(cert) = puzzle::solve(&(p.build)(), p.seat, p.goal, MAX_DEPTH)
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
        .map(|p| (p, certify(p, MAX_DEPTH)))
        .filter(|(_, c)| c.keep())
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

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!(
            "bot_puzzles [--certify] [PROFILE ...]\n\n\
             PROFILE: gang | atk-sim | net | mcts-net-deep\n\
             --certify reports which candidates are real puzzles."
        );
        return;
    }
    // The net profiles read the champion out of a slot; without it they
    // silently degrade to the heuristic, which would make a net column
    // that is really a second gang column.
    if std::path::Path::new(CHAMPION).exists() {
        let _ = crabomination::server::net_eval::load_slot(
            crabomination::server::net_eval::SLOT_BEST,
            std::path::Path::new(CHAMPION),
        );
    }
    if args.iter().any(|a| a == "--certify") {
        run_certify();
        return;
    }
    let profiles: Vec<String> = args.iter().filter(|a| !a.starts_with("--")).cloned().collect();
    let profiles = if profiles.is_empty() {
        vec!["gang".to_string(), "atk-sim".to_string()]
    } else {
        profiles
    };
    run_scoring(&profiles);
}
