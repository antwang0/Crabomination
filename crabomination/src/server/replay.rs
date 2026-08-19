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

/// The active file plus the card ids whose names are already on record —
/// each id's name is written once, the first time the id appears.
struct Sink {
    w: BufWriter<File>,
    named: crate::fxhash::HashSet<u32>,
}

thread_local! {
    static SINK: RefCell<Option<Sink>> = const { RefCell::new(None) };
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
        // v2: event lines are `{"e": [events], "n": {id: name}}` — wire
        // events carry card ids, and a replay file has no live state to
        // resolve them against, so the recorder writes each id's name the
        // first time it appears. v1 lines were the bare event array.
        let header = serde_json::json!({ "replay": 2, "players": player_names });
        let _ = writeln!(w, "{header}");
        SINK.with(|s| {
            *s.borrow_mut() = Some(Sink { w, named: crate::fxhash::HashSet::default() })
        });
        Self { active: true }
    }
}

impl Drop for MatchReplay {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        SINK.with(|s| {
            if let Some(mut sink) = s.borrow_mut().take() {
                let _ = writeln!(sink.w, "{}", serde_json::json!({ "end": true }));
                let _ = sink.w.flush();
            }
        });
    }
}

/// Append one broadcast batch to the active match's replay, if any,
/// with first-appearance names for the card ids the batch mentions.
pub(crate) fn log_events(state: &crate::game::GameState, events: &[GameEventWire]) {
    if events.is_empty() {
        return;
    }
    SINK.with(|s| {
        if let Some(sink) = s.borrow_mut().as_mut() {
            let mut names = serde_json::Map::new();
            for ev in events {
                let Ok(v) = serde_json::to_value(ev) else { continue };
                collect_card_ids(&v, &mut |id| {
                    if sink.named.insert(id)
                        && let Some(c) = state.find_card_anywhere(crate::card::CardId(id))
                    {
                        names.insert(
                            id.to_string(),
                            serde_json::Value::String(c.definition.name.to_string()),
                        );
                    }
                });
            }
            let line = if names.is_empty() {
                serde_json::json!({ "e": events })
            } else {
                serde_json::json!({ "e": events, "n": names })
            };
            let _ = writeln!(sink.w, "{line}");
        }
    });
}

/// Card-id fields by name, wherever they sit in a wire event's shape —
/// including tuple variants like `AttackerDeclared(CardId)`, whose id is
/// the whole payload under the variant-name key. A field name missing
/// here simply narrates as a bare id until added.
fn collect_card_ids(v: &serde_json::Value, f: &mut impl FnMut(u32)) {
    match v {
        serde_json::Value::Object(m) => {
            for (k, val) in m {
                if matches!(
                    k.as_str(),
                    "card_id" | "source" | "to_card" | "attacker" | "blocker" | "object"
                ) && let Some(id) = val.as_u64()
                {
                    f(id as u32);
                }
                if k == "AttackerDeclared"
                    && let Some(id) = val.as_u64()
                {
                    f(id as u32);
                }
                collect_card_ids(val, f);
            }
        }
        serde_json::Value::Array(a) => a.iter().for_each(|x| collect_card_ids(x, f)),
        _ => {}
    }
}

/// Serializes every test that mutates `CRAB_REPLAY_DIR` — env vars are
/// process-global, and the harness runs tests in parallel, so an unlock
/// between one test's `set_var` and its `begin` lets another test's
/// `remove_var` land in the gap.
#[cfg(test)]
pub(crate) fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// begin → log → drop writes header, one event line, and the footer.
    #[test]
    fn replay_file_brackets_the_event_stream() {
        let _env = env_lock();
        let dir = std::env::temp_dir().join(format!("crab-replay-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir); // clear any stale file first
        // SAFETY: single-threaded env mutation scoped to this test.
        unsafe { std::env::set_var("CRAB_REPLAY_DIR", &dir) };
        let mut state = crate::game::GameState::new(vec![
            crate::player::Player::new(0, "A"),
            crate::player::Player::new(1, "B"),
        ]);
        let bear = state.add_card_to_battlefield(
            0,
            crate::card::CardDefinition {
                name: "Replay Bear",
                card_types: vec![crate::card::CardType::Creature],
                power: 2,
                toughness: 2,
                ..Default::default()
            },
        );
        {
            let _guard = MatchReplay::begin(&["A".into(), "B".into()]);
            log_events(&state, &[GameEventWire::TurnStarted { player: 0, turn: 1 }]);
            // The bear's name rides the first line that mentions its id,
            // and only that line.
            log_events(&state, &[GameEventWire::PermanentEntered { card_id: bear }]);
            log_events(&state, &[GameEventWire::PermanentTapped { card_id: bear }]);
            log_events(&state, &[]); // empty batches are skipped
        }
        unsafe { std::env::remove_var("CRAB_REPLAY_DIR") };
        // Pick our own file (matching the players header) in case a concurrent
        // writer landed here while our env var was set.
        let text = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
            .find(|t| t.contains("\"players\":[\"A\",\"B\"]"))
            .expect("our replay file");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 5, "header + three batches + footer");
        assert!(lines[0].contains("\"players\":[\"A\",\"B\"]"));
        assert!(lines[0].contains("\"replay\":2"));
        assert!(lines[1].contains("TurnStarted"));
        assert!(lines[2].contains("Replay Bear"), "first mention carries the name");
        assert!(!lines[3].contains("Replay Bear"), "the name is written once");
        assert!(lines[3].contains("PermanentTapped"));
        assert!(lines[4].contains("\"end\":true"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
