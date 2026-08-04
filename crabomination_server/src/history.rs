//! Optional per-match result persistence: one JSON line appended to the file
//! named by `CRAB_MATCH_LOG` for every finished match. Unset = disabled.
//! `CRAB_MATCH_LOG_MAX_BYTES` caps the live file, rotating it to `<path>.1`.
//! Gives operators a durable match-results record (FEATURE_ROADMAP Tier 14)
//! beyond the in-memory rolling stats that die with the process.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crabomination::server::{LossReason, MatchOutcome};

/// Match-log line schema version. Bump when the field set or their meaning
/// changes so downstream consumers can branch on `"v"` instead of guessing.
const SCHEMA_VERSION: u32 = 1;

fn log_path() -> Option<&'static PathBuf> {
    static PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
    PATH.get_or_init(|| std::env::var_os("CRAB_MATCH_LOG").map(PathBuf::from))
        .as_ref()
}

fn loss_reason_label(r: &LossReason) -> &'static str {
    match r {
        LossReason::LifeDepleted => "life",
        LossReason::Poison => "poison",
        LossReason::Decked => "decked",
        LossReason::CommanderDamage => "commander",
        LossReason::Conceded => "concede",
        LossReason::Other => "other",
    }
}

/// Escape a string for embedding inside a JSON double-quoted value (RFC 8259
/// §7). `format_label` is a fixed label set today, but escaping keeps the log
/// valid JSON should a custom/deck-derived label ever carry a quote, backslash,
/// or control character.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Render one match result as a JSON line. Numeric fields and fixed label sets
/// are inlined directly; the free-form `format_label` is JSON-escaped.
fn render_line(format_label: &str, duration: Duration, outcome: &MatchOutcome) -> String {
    let format_label = json_escape(format_label);
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let winner = match outcome.winner {
        None => "\"aborted\"".to_string(),
        Some(None) => "\"draw\"".to_string(),
        Some(Some(seat)) => seat.to_string(),
    };
    let life = outcome
        .final_life_totals
        .iter()
        .map(|l| l.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let losses = outcome
        .loss_reasons
        .iter()
        .map(|r| match r {
            None => "null".to_string(),
            Some(r) => format!("\"{}\"", loss_reason_label(r)),
        })
        .collect::<Vec<_>>()
        .join(",");
    let libs = outcome
        .final_library_sizes
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",");
    // Explicit table size so a log reader never has to infer it from arrays
    // that may be empty (e.g. an aborted 1-seat match with no final snapshot).
    let players = outcome
        .final_life_totals
        .len()
        .max(outcome.loss_reasons.len())
        .max(outcome.final_library_sizes.len());
    format!(
        "{{\"v\":{SCHEMA_VERSION},\"ts\":{ts},\"format\":\"{format_label}\",\
         \"duration_ms\":{},\"turns\":{},\
         \"players\":{players},\"winner\":{winner},\"life\":[{life}],\
         \"loss_reasons\":[{losses}],\"libraries\":[{libs}]}}\n",
        duration.as_millis(),
        outcome.final_turn,
    )
}

/// Byte ceiling for the live match log, from `CRAB_MATCH_LOG_MAX_BYTES`.
/// `None` (unset, unparseable, or 0) disables rotation — the historical
/// unbounded-append behaviour.
fn max_bytes() -> Option<u64> {
    static MAX: OnceLock<Option<u64>> = OnceLock::new();
    *MAX.get_or_init(|| {
        std::env::var("CRAB_MATCH_LOG_MAX_BYTES").ok()?.parse::<u64>().ok().filter(|n| *n > 0)
    })
}

/// Rotate `path` to `path.1` once it would exceed `limit`, so an operator
/// leaving the log on for months keeps one bounded live file plus one
/// bounded archive instead of an unbounded one. A previous `.1` is replaced.
fn rotate_if_needed(path: &Path, incoming: usize, limit: u64) {
    let Ok(meta) = std::fs::metadata(path) else { return };
    if meta.len() + incoming as u64 <= limit {
        return;
    }
    let mut archive = path.as_os_str().to_owned();
    archive.push(".1");
    if let Err(e) = std::fs::rename(path, PathBuf::from(archive)) {
        eprintln!("warning: CRAB_MATCH_LOG rotate of {} failed: {e}", path.display());
    }
}

fn append(path: &Path, line: &str) {
    if let Some(limit) = max_bytes() {
        rotate_if_needed(path, line.len(), limit);
    }
    let write = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| f.write_all(line.as_bytes()));
    if let Err(e) = write {
        eprintln!("warning: CRAB_MATCH_LOG append to {} failed: {e}", path.display());
    }
}

/// Append `outcome` to the match log, if one is configured. Called from every
/// match-end path (lobby, bot, pair) alongside the rolling-stats fold.
pub(crate) fn record(format_label: &str, duration: Duration, outcome: &MatchOutcome) {
    if let Some(path) = log_path() {
        append(path, &render_line(format_label, duration, outcome));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_line_is_one_json_object_per_match() {
        let outcome = MatchOutcome {
            final_turn: 12,
            winner: Some(Some(1)),
            final_life_totals: vec![-3, 14],
            loss_reasons: vec![Some(LossReason::LifeDepleted), None],
            final_library_sizes: vec![20, 31],
            ..Default::default()
        };
        let line = render_line("cube", Duration::from_millis(1500), &outcome);
        assert!(line.ends_with('\n'));
        assert!(line.contains("\"v\":1"), "carries a schema version: {line}");
        assert!(line.contains("\"format\":\"cube\""), "{line}");
        assert!(line.contains("\"duration_ms\":1500"), "{line}");
        assert!(line.contains("\"turns\":12"), "{line}");
        assert!(line.contains("\"players\":2"), "{line}");
        assert!(line.contains("\"winner\":1"), "{line}");
        assert!(line.contains("\"life\":[-3,14]"), "{line}");
        assert!(line.contains("\"loss_reasons\":[\"life\",null]"), "{line}");
        assert!(line.contains("\"libraries\":[20,31]"), "{line}");
    }

    #[test]
    fn render_line_escapes_format_label() {
        let outcome = MatchOutcome::default();
        let line = render_line("weird\"label\\\twith\nctl", Duration::ZERO, &outcome);
        assert!(
            line.contains("\"format\":\"weird\\\"label\\\\\\twith\\nctl\""),
            "special chars escaped: {line}"
        );
    }

    #[test]
    fn render_line_draw_and_abort_winners() {
        let draw = MatchOutcome { winner: Some(None), ..Default::default() };
        assert!(render_line("demo", Duration::ZERO, &draw).contains("\"winner\":\"draw\""));
        let abort = MatchOutcome { winner: None, ..Default::default() };
        assert!(render_line("demo", Duration::ZERO, &abort).contains("\"winner\":\"aborted\""));
    }

    #[test]
    fn rotation_moves_the_live_log_aside_once_it_would_overflow() {
        let dir = std::env::temp_dir().join(format!("crab_rot_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("matches.jsonl");
        let _ = std::fs::remove_file(&path);
        std::fs::write(&path, "x".repeat(40)).unwrap();
        // Under the limit: nothing moves.
        rotate_if_needed(&path, 5, 100);
        assert!(!path.with_extension("jsonl.1").exists(), "no premature rotation");
        // Over it: the live file becomes the archive and the next append
        // starts a fresh one.
        rotate_if_needed(&path, 80, 100);
        assert!(!path.exists(), "live log rotated away");
        assert_eq!(std::fs::read_to_string(path.with_extension("jsonl.1")).unwrap().len(), 40);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn append_creates_and_appends() {
        let path = std::env::temp_dir().join(format!(
            "crab_match_log_test_{}.jsonl",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        append(&path, "{\"a\":1}\n");
        append(&path, "{\"a\":2}\n");
        let body = std::fs::read_to_string(&path).unwrap();
        assert_eq!(body.lines().count(), 2);
        let _ = std::fs::remove_file(&path);
    }
}
