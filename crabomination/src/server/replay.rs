//! Optional per-match replay log (FEATURE_ROADMAP Tier 14). When
//! `CRAB_REPLAY_DIR` is set, every event batch a match broadcasts is
//! appended as one JSON line to `<dir>/replay-<unix-ts>-<seq>.jsonl`,
//! bracketed by a header (players, format label) and a footer (winner).
//! The event stream is the replay: a viewer can re-narrate the whole game
//! from it. Unset = zero overhead.

use std::cell::RefCell;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::net::GameEventWire;

thread_local! {
    static SINK: RefCell<Option<BufWriter<File>>> = const { RefCell::new(None) };
}

fn replay_dir() -> Option<PathBuf> {
    std::env::var_os("CRAB_REPLAY_DIR").map(PathBuf::from)
}

/// RAII guard for one match's replay file. [`begin`] arms the thread-local
/// sink (each match runs its loop on one thread); dropping the guard writes
/// the footer and disarms it, so every `run_match_inner` exit path is
/// covered.
///
/// [`begin`]: MatchReplay::begin
pub(crate) struct MatchReplay {
    active: bool,
}

impl MatchReplay {
    pub(crate) fn begin(player_names: &[String]) -> Self {
        let Some(dir) = replay_dir() else {
            return Self { active: false };
        };
        if std::fs::create_dir_all(&dir).is_err() {
            return Self { active: false };
        }
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let path = dir.join(format!("replay-{ts}-{seq}.jsonl"));
        let Ok(file) = File::create(&path) else {
            return Self { active: false };
        };
        let mut w = BufWriter::new(file);
        let header = serde_json::json!({ "replay": 1, "players": player_names });
        let _ = writeln!(w, "{header}");
        SINK.with(|s| *s.borrow_mut() = Some(w));
        Self { active: true }
    }
}

impl Drop for MatchReplay {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        SINK.with(|s| {
            if let Some(mut w) = s.borrow_mut().take() {
                let _ = writeln!(w, "{}", serde_json::json!({ "end": true }));
                let _ = w.flush();
            }
        });
    }
}

/// Append one broadcast batch to the active match's replay, if any.
pub(crate) fn log_events(events: &[GameEventWire]) {
    if events.is_empty() {
        return;
    }
    SINK.with(|s| {
        if let Some(w) = s.borrow_mut().as_mut()
            && let Ok(line) = serde_json::to_string(events)
        {
            let _ = writeln!(w, "{line}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// begin → log → drop writes header, one event line, and the footer.
    #[test]
    fn replay_file_brackets_the_event_stream() {
        let dir = std::env::temp_dir().join(format!("crab-replay-test-{}", std::process::id()));
        // SAFETY: single-threaded env mutation scoped to this test; the
        // unique dir keeps concurrent matches from asserting on our file.
        unsafe { std::env::set_var("CRAB_REPLAY_DIR", &dir) };
        {
            let _guard = MatchReplay::begin(&["A".into(), "B".into()]);
            log_events(&[GameEventWire::TurnStarted { player: 0, turn: 1 }]);
            log_events(&[]); // empty batches are skipped
        }
        unsafe { std::env::remove_var("CRAB_REPLAY_DIR") };
        let entry = std::fs::read_dir(&dir).unwrap().next().unwrap().unwrap();
        let text = std::fs::read_to_string(entry.path()).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3, "header + one batch + footer");
        assert!(lines[0].contains("\"players\":[\"A\",\"B\"]"));
        assert!(lines[1].contains("TurnStarted"));
        assert!(lines[2].contains("\"end\":true"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
