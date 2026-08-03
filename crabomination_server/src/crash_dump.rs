//! Optional crash dumps: set `CRAB_CRASH_DUMP_DIR` to have a match that
//! panics write the `GameState` it started from, plus the panic message, to a
//! JSON file. Unset = disabled (and the state is never cloned).
//!
//! Before this, `run_match_caught` logged one line and dropped the state, so a
//! catalog bug that panicked mid-match left nothing to reproduce from
//! (FEATURE_ROADMAP Tier 16 — crash recovery).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Dump-file schema version. Bump when the field set changes.
const SCHEMA_VERSION: u32 = 1;
/// Default number of dumps retained; the oldest are pruned past this.
const DEFAULT_KEEP: usize = 20;

fn dump_dir() -> Option<&'static PathBuf> {
    static DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    DIR.get_or_init(|| std::env::var_os("CRAB_CRASH_DUMP_DIR").map(PathBuf::from))
        .as_ref()
}

fn keep() -> usize {
    static KEEP: OnceLock<usize> = OnceLock::new();
    *KEEP.get_or_init(|| {
        std::env::var("CRAB_CRASH_DUMP_KEEP")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_KEEP)
    })
}

/// True when dumping is configured. Callers gate the (non-trivial) state
/// serialization on this so the disabled path costs nothing.
pub(crate) fn enabled() -> bool {
    dump_dir().is_some()
}

/// Serialize the state a match is about to run, ready to hand to [`write`] if
/// that match panics. `None` when dumping is off or the state won't serialize.
pub(crate) fn capture(state: &crabomination::game::GameState) -> Option<String> {
    enabled().then(|| serde_json::to_string(state).ok()).flatten()
}

/// The latest mid-match checkpoint from a running match's snapshot sink,
/// serialized. `None` when dumping is off, the match never published, or the
/// state won't serialize — callers fall back to the pre-match capture.
pub(crate) fn capture_checkpoint(
    sink: Option<&crabomination::server::SnapshotSink>,
) -> Option<String> {
    let state = sink?.lock().ok()?.full_state()?;
    serde_json::to_string(&state).ok()
}

/// Escape a string for a JSON double-quoted value (RFC 8259 §7).
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

/// The dump body: metadata plus the verbatim pre-match state JSON.
fn render(ctx: &str, panic_msg: &str, state_json: &str, ts: u64) -> String {
    format!(
        "{{\"v\":{SCHEMA_VERSION},\"ts\":{ts},\"ctx\":\"{}\",\"panic\":\"{}\",\"state\":{}}}\n",
        json_escape(ctx),
        json_escape(panic_msg),
        state_json,
    )
}

/// Keep only the newest `keep` `*.json` files in `dir`, by filename — the
/// names are timestamp-then-counter ordered, so lexical order is chronological.
fn prune(dir: &Path, keep: usize) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut names: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    if names.len() <= keep {
        return;
    }
    names.sort();
    let drop = names.len() - keep;
    for p in names.into_iter().take(drop) {
        let _ = std::fs::remove_file(p);
    }
}

/// Write a dump into `dir`. Split from [`write`] so tests can target a temp
/// directory without touching the process environment.
fn write_to(dir: &Path, ctx: &str, panic_msg: &str, state_json: &str, ts: u64) -> Option<PathBuf> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    std::fs::create_dir_all(dir).ok()?;
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let path = dir.join(format!("crash-{ts:010}-{seq:04}.json"));
    // Write to a sibling temp file and rename, so a reader never sees a
    // half-written dump.
    let tmp = path.with_extension("json.tmp");
    let body = render(ctx, panic_msg, state_json, ts);
    {
        let mut f = std::fs::File::create(&tmp).ok()?;
        f.write_all(body.as_bytes()).ok()?;
        f.sync_all().ok()?;
    }
    std::fs::rename(&tmp, &path).ok()?;
    prune(dir, keep());
    Some(path)
}

/// Write a crash dump for a panicking match. Returns the path written, or
/// `None` when dumping is off or the write failed (never fatal — a failed
/// dump must not take the server down on top of the match it already lost).
pub(crate) fn write(ctx: &str, panic_msg: &str, state_json: &str) -> Option<PathBuf> {
    let dir = dump_dir()?;
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    write_to(dir, ctx, panic_msg, state_json, ts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("crab-crash-test-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    /// The dump is one JSON object carrying the context, the panic message,
    /// and the state verbatim.
    #[test]
    fn render_wraps_the_state_in_one_json_object() {
        let body = render("bot match 1.2.3.4", "index out of bounds", r#"{"turn":4}"#, 1_700_000_000);
        assert!(body.ends_with('\n'));
        assert!(body.contains("\"v\":1"), "{body}");
        assert!(body.contains("\"ts\":1700000000"), "{body}");
        assert!(body.contains("\"ctx\":\"bot match 1.2.3.4\""), "{body}");
        assert!(body.contains("\"panic\":\"index out of bounds\""), "{body}");
        assert!(body.contains("\"state\":{\"turn\":4}"), "{body}");
    }

    /// Quotes and newlines in a panic message stay inside the JSON string.
    #[test]
    fn panic_messages_are_escaped() {
        let body = render("ctx", "said \"boom\"\nthen died", "null", 0);
        assert!(body.contains(r#""panic":"said \"boom\"\nthen died""#), "{body}");
        assert_eq!(body.matches('\n').count(), 1, "only the trailing newline is literal");
    }

    /// A dump lands as a complete `.json` file with no `.tmp` left behind.
    #[test]
    fn write_to_renames_into_place() {
        let dir = temp_dir("write");
        let path = write_to(&dir, "ctx", "boom", "null", 7).expect("dump written");
        assert_eq!(path.extension().unwrap(), "json");
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("\"panic\":\"boom\""));
        assert!(
            std::fs::read_dir(&dir).unwrap().flatten().all(|e| e.path().extension().unwrap() == "json"),
            "no temp file survives"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Retention prunes the oldest dumps, keeping the newest `keep`.
    #[test]
    fn prune_keeps_the_newest() {
        let dir = temp_dir("prune");
        std::fs::create_dir_all(&dir).unwrap();
        for i in 0..5u32 {
            std::fs::write(dir.join(format!("crash-{i:010}-0000.json")), "{}").unwrap();
        }
        prune(&dir, 2);
        let mut left: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        assert_eq!(left, vec!["crash-0000000003-0000.json", "crash-0000000004-0000.json"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A sink that has published a state yields it as the repro; an empty or
    /// absent sink falls back to `None` so the caller uses the pre-match copy.
    #[test]
    fn capture_checkpoint_prefers_the_live_state() {
        use crabomination::server::SnapshotSinkState;
        use std::sync::{Arc, Mutex};
        assert!(capture_checkpoint(None).is_none());
        let sink = Arc::new(Mutex::new(SnapshotSinkState::default()));
        assert!(capture_checkpoint(Some(&sink)).is_none(), "nothing published yet");
        let mut state = crabomination::game::two_player_game();
        state.turn_number = 15;
        sink.lock().unwrap().state = Some(Arc::new(state));
        let json = capture_checkpoint(Some(&sink)).expect("checkpoint");
        assert!(json.contains("\"turn_number\":15"), "the live turn, not the opening one");
    }

    /// With no dump directory configured nothing is captured or written.
    #[test]
    fn disabled_by_default() {
        if std::env::var_os("CRAB_CRASH_DUMP_DIR").is_none() {
            assert!(!enabled());
            assert!(write("ctx", "boom", "null").is_none());
        }
    }
}
