//! Audit: find cards that are implemented but **incomplete** — a printed
//! capability is missing or approximated.
//!
//! Companion to `audit_stubs.rs` (which finds *blank* cards). This binary
//! attacks the harder "looks done, isn't" class with two independent passes:
//!
//!   1. STRUCTURAL (comment-free, authoritative). Walks every catalog card's
//!      serialized effect tree and flags:
//!        - **Dead modes**: a `ChooseMode` / `ChooseN` / `Escalate` arm that
//!          resolves to nothing (`Noop`, empty `Seq`, …) — e.g. Sublime
//!          Epiphany's unimplemented "counter target ability" mode. NOTE these
//!          need human triage: a `Noop` mode is *also* the idiom for a
//!          deliberate "you may … (or decline)" option (Elite Interceptor),
//!          which is correct, not a gap.
//!        - **Dead abilities**: a triggered / activated / loyalty ability
//!          whose `effect` is entirely empty. The ability exists but does
//!          nothing when it fires.
//!
//!      These need no knowledge of the printed card — an empty arm/ability is
//!      a bug by construction.
//!
//!   2. COMMENT SCAN (the automated version of the manual partial-impl audit).
//!      Greps the catalog source for `pub fn … -> CardDefinition` factories
//!      whose doc comment flags an approximation ("approximation", "modeled
//!      as", "omitted", "stub", "body only", "collapsed", …). This regenerates
//!      the incomplete-card inventory on every run, so it can be diffed over
//!      time — and cross-referenced against pass 1 to surface **stale
//!      comments** (a comment says "omitted" but the code now wires it, or
//!      vice-versa).
//!
//! Run: `cargo run -p crabomination --bin audit_incomplete`
//!      `cargo run -p crabomination --bin audit_incomplete -- --comments-only`
//!      `cargo run -p crabomination --bin audit_incomplete -- --structural-only`

use crabomination::fxhash::HashSet;
use std::path::{Path, PathBuf};

use crabomination::audit::{DeadCapability, dead_capabilities};
use crabomination::catalog::all_known_factories;

// ── Pass 1: structural (serde-walk the effect tree) ──────────────────────────
//
// The walker itself lives in `crabomination::audit` so this binary,
// `audit_stubs`, and the `core_rules::structural_audit` regression test all
// share one definition of "does nothing".

fn run_structural() {
    let mut seen: HashSet<String> = HashSet::default();
    let mut flagged: Vec<(String, Vec<DeadCapability>)> = Vec::new();
    let mut total = 0usize;

    for factory in all_known_factories() {
        let def = factory();
        if !seen.insert(def.name.to_string()) {
            continue;
        }
        total += 1;
        let findings = dead_capabilities(&def);
        if !findings.is_empty() {
            flagged.push((def.name.to_string(), findings));
        }
    }

    flagged.sort_by(|a, b| a.0.cmp(&b.0));
    eprintln!("── STRUCTURAL ──────────────────────────────────────────────");
    eprintln!("Scanned {total} unique cards; {} have dead modes/abilities.\n", flagged.len());
    for (name, findings) in &flagged {
        for f in findings {
            eprintln!("  {name}  — {f}");
        }
    }
    eprintln!();
}

// ── Pass 2: comment scan over the catalog source ─────────────────────────────

/// Substrings that flag a documented approximation / missing capability.
/// Lower-cased before matching.
const MARKERS: &[&str] = &[
    "approximat",
    "modeled as",
    "modelled as",
    "simplif",
    "stub",
    "todo",
    "fixme",
    "omit",
    "placeholder",
    "collapsed",
    "folded into",
    "body only",
    "not yet implement",
    "not implemented",
    "not modeled",
    "not wired",
    "can't model",
    "cannot model",
    "dropped",
];

fn catalog_src_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR = .../crabomination; the catalog lives beside it.
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    here.parent()
        .expect("crate has a parent dir")
        .join("crabomination_catalog/src")
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn marker_hit(line_lc: &str) -> bool {
    MARKERS.iter().any(|m| line_lc.contains(m))
}

fn run_comment_scan() {
    let root = catalog_src_dir();
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    files.sort();

    let mut findings: Vec<(String, u32, String, String)> = Vec::new(); // (rel_path, line, fn, snippet)
    let mut factory_count = 0usize;

    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else { continue };
        let lines: Vec<&str> = text.lines().collect();
        let rel = file.strip_prefix(&root).unwrap_or(file).display().to_string();

        for (i, line) in lines.iter().enumerate() {
            if !line.contains("-> CardDefinition") {
                continue;
            }
            factory_count += 1;
            // Find the fn name on this line (or just above).
            let fn_name = (i.saturating_sub(2)..=i)
                .rev()
                .find_map(|j| extract_fn_name(lines[j]))
                .unwrap_or_else(|| "<unknown>".to_string());

            // Walk the contiguous doc-comment block immediately above the fn.
            let fn_line = (i.saturating_sub(2)..=i)
                .rev()
                .find(|&j| lines[j].trim_start().starts_with("pub fn"))
                .unwrap_or(i);
            let mut snippet: Option<String> = None;
            let mut j = fn_line;
            while j > 0 {
                j -= 1;
                let t = lines[j].trim_start();
                if t.starts_with("///") || t.starts_with("//!") || t.starts_with("//") {
                    if marker_hit(&t.to_lowercase()) && snippet.is_none() {
                        snippet = Some(t.trim_start_matches('/').trim().to_string());
                    }
                } else if t.is_empty() {
                    continue;
                } else {
                    break;
                }
            }
            if let Some(s) = snippet {
                findings.push((rel.clone(), (fn_line + 1) as u32, fn_name, s));
            }
        }
    }

    findings.sort();
    eprintln!("── COMMENT SCAN ────────────────────────────────────────────");
    eprintln!(
        "Scanned {} catalog files, {factory_count} factory fns; {} carry an approximation note.\n",
        files.len(),
        findings.len()
    );
    let mut last_file = String::new();
    for (file, line, fn_name, snippet) in &findings {
        if *file != last_file {
            eprintln!("\n=== {file} ===");
            last_file = file.clone();
        }
        let snip = if snippet.chars().count() > 100 {
            format!("{}…", snippet.chars().take(100).collect::<String>())
        } else {
            snippet.clone()
        };
        eprintln!("  {file}:{line}  {fn_name}()  — {snip}");
    }
    eprintln!();
}

/// Pull the function name out of a `pub fn name(...)` line, if present.
fn extract_fn_name(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix("pub fn ")?;
    let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
    (!name.is_empty()).then_some(name)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let comments_only = args.iter().any(|a| a == "--comments-only");
    let structural_only = args.iter().any(|a| a == "--structural-only");

    if !comments_only {
        run_structural();
    }
    if !structural_only {
        run_comment_scan();
    }
}
