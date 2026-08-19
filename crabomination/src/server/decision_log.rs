//! Optional shadow log of a human's decisions. When `CRAB_DECISION_LOG`
//! is set (a directory), every action a human seat submits is logged as
//! one JSON line next to what the heuristic bot would have done from the
//! *same position with the same information* — the tool for evaluating
//! the bot's judgment against a human's game, and for debugging the
//! specific decisions where they part ways (the converge repair started
//! as exactly this kind of suspicion). Unset = zero overhead.
//!
//! File shape (`<dir>/decisions-<unix-ts>-<seq>.jsonl`): a header line
//! (players, which seats are human), one line per human action with
//! `you`/`bot` human-readable summaries, an `agree` flag and the raw
//! `GameAction` JSON for tooling, and a footer with the disagreement
//! tally. The shadow consult is read-only: `Bot::next_action` takes
//! `&GameState` and simulates on clones, and a *fresh* `HeuristicBot`
//! is built per query because the bot struct keeps per-step combat
//! latches — a persistent shadow that answered once would go quiet for
//! the rest of the step.

use std::cell::RefCell;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::card::CardId;
use crate::game::{GameAction, GameState, Target};
use crate::server::bot::{Bot, HeuristicBot};

thread_local! {
    static SINK: RefCell<Option<Sink>> = const { RefCell::new(None) };
}

struct Sink {
    w: BufWriter<File>,
    /// Seats occupied by humans — only their actions are logged.
    human_mask: u8,
    decisions: u32,
    disagreements: u32,
    /// The last committed dedupe key — see [`commit`].
    last_key: Option<String>,
}

fn log_dir() -> Option<PathBuf> {
    std::env::var_os("CRAB_DECISION_LOG").map(PathBuf::from)
}

/// RAII guard for one match's decision log, mirroring
/// [`super::replay::MatchReplay`]: [`begin`] arms the thread-local sink
/// (each match runs its loop on one thread); dropping the guard writes
/// the footer and disarms it on every `run_match_inner` exit path.
///
/// [`begin`]: DecisionShadowLog::begin
pub(crate) struct DecisionShadowLog {
    active: bool,
}

impl DecisionShadowLog {
    pub(crate) fn begin(player_names: &[String], human_seats: &[usize]) -> Self {
        let Some(dir) = log_dir() else {
            return Self { active: false };
        };
        if human_seats.is_empty() || std::fs::create_dir_all(&dir).is_err() {
            return Self { active: false };
        }
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let path = dir.join(format!("decisions-{ts}-{seq}.jsonl"));
        let Ok(file) = File::create(&path) else {
            return Self { active: false };
        };
        let mut w = BufWriter::new(file);
        let header = serde_json::json!({
            "decision_log": 1,
            "players": player_names,
            "human_seats": human_seats,
        });
        let _ = writeln!(w, "{header}");
        let mut human_mask = 0u8;
        for &s in human_seats {
            if s < 8 {
                human_mask |= 1 << s;
            }
        }
        SINK.with(|s| {
            *s.borrow_mut() =
                Some(Sink { w, human_mask, decisions: 0, disagreements: 0, last_key: None })
        });
        Self { active: true }
    }
}

impl Drop for DecisionShadowLog {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        SINK.with(|s| {
            if let Some(mut sink) = s.borrow_mut().take() {
                let footer = serde_json::json!({
                    "end": true,
                    "decisions": sink.decisions,
                    "disagreements": sink.disagreements,
                });
                let _ = writeln!(sink.w, "{footer}");
                let _ = sink.w.flush();
            }
        });
    }
}

/// A line prepared against the pre-action state, committed only if the
/// action actually applied — an action the engine rejects was never a
/// decision, and describing it needs the state *before* `perform_action`
/// mutates it.
pub(crate) struct PendingLine {
    line: String,
    agree: bool,
    /// Dedupe key: a manual mana payer resubmits the same cast after
    /// every tap (`ManualTapRequired` flow), and each resubmission that
    /// applies would otherwise log as a fresh decision.
    key: String,
}

/// "Do nothing" comes in four spellings — passing priority, an empty
/// attack declaration (banded or not), and an empty block declaration —
/// and the human's client and the bot pick different ones for the same
/// intent. One equivalence class, or half the log is fake disagreement.
fn is_noop(action: &GameAction) -> bool {
    match action {
        GameAction::PassPriority => true,
        GameAction::DeclareAttackers(a) => a.is_empty(),
        GameAction::DeclareAttackersBanded { attacks, .. } => attacks.is_empty(),
        GameAction::DeclareBlockers(b) => b.is_empty(),
        _ => false,
    }
}

/// Ask the shadow bot what it would do in the human's shoes, and prepare
/// the comparison line. Cheap no-op (one thread-local read) when the log
/// is unarmed or the seat isn't human.
pub(crate) fn prepare(state: &GameState, seat: usize, action: &GameAction) -> Option<PendingLine> {
    let armed = SINK.with(|s| {
        s.borrow()
            .as_ref()
            .is_some_and(|sink| seat < 8 && sink.human_mask & (1 << seat) != 0)
    });
    if !armed {
        return None;
    }
    // A bare mana-ability activation is payment mechanics, not a
    // decision — the human taps lands by hand while the bot pays inside
    // its cast action, so comparing the two mid-payment is noise (the
    // first recorded game logged fourteen of these).
    if let GameAction::ActivateAbility { card_id, ability_index, .. } = action
        && state
            .battlefield_find(*card_id)
            .and_then(|c| c.definition.activated_abilities.get(*ability_index))
            .is_some_and(|a| crate::game::actions::is_mana_ability_public(&a.effect))
    {
        return None;
    }
    let mut shadow = HeuristicBot::new();
    let bot = shadow.next_action(state, seat);
    // `GameAction` doesn't derive `PartialEq`; the serde rendering is a
    // faithful identity. Every spelling of "do nothing" agrees with
    // every other.
    let human_json = serde_json::to_string(action).unwrap_or_default();
    let agree = match &bot {
        Some(b) => {
            serde_json::to_string(b).ok().as_deref() == Some(human_json.as_str())
                || (is_noop(b) && is_noop(action))
        }
        None => is_noop(action),
    };
    let line = serde_json::json!({
        "turn": state.turn_number,
        "step": format!("{:?}", state.step),
        "seat": seat,
        "agree": agree,
        "you": describe(state, action),
        "bot": bot
            .as_ref()
            .map(|a| describe(state, a))
            .unwrap_or_else(|| "(no action — pass)".to_string()),
        "you_raw": serde_json::to_value(action).unwrap_or(serde_json::Value::Null),
        "bot_raw": serde_json::to_value(&bot).unwrap_or(serde_json::Value::Null),
    });
    let key = format!("{}|{:?}|{human_json}", state.turn_number, state.step);
    Some(PendingLine { line: line.to_string(), agree, key })
}

/// Write a prepared line once the action applied. Consecutive identical
/// submissions (same turn, step and action — the manual-mana retry
/// loop) count once.
pub(crate) fn commit(pending: Option<PendingLine>) {
    let Some(p) = pending else { return };
    SINK.with(|s| {
        if let Some(sink) = s.borrow_mut().as_mut() {
            if sink.last_key.as_deref() == Some(p.key.as_str()) {
                return;
            }
            sink.last_key = Some(p.key);
            sink.decisions += 1;
            if !p.agree {
                sink.disagreements += 1;
            }
            let _ = writeln!(sink.w, "{}", p.line);
        }
    });
}

fn name_of(state: &GameState, id: CardId) -> String {
    state
        .find_card_anywhere(id)
        .map(|c| c.definition.name.to_string())
        .unwrap_or_else(|| format!("card#{}", id.0))
}

fn target_text(state: &GameState, t: &Target) -> String {
    match t {
        Target::Player(p) => format!("player {p}"),
        Target::Permanent(id) => name_of(state, *id),
    }
}

/// Human-readable one-liner for the common action shapes; anything
/// exotic falls back to a trimmed Debug rendering (the raw JSON is on
/// the same line for tooling anyway).
fn describe(state: &GameState, action: &GameAction) -> String {
    use GameAction::*;
    let with_target = |verb: &str, id: CardId, target: &Option<Target>| match target {
        Some(t) => format!("{verb} {} @ {}", name_of(state, id), target_text(state, t)),
        None => format!("{verb} {}", name_of(state, id)),
    };
    match action {
        PassPriority => "pass".to_string(),
        Concede => "concede".to_string(),
        PlayLand(id) | PlayLandBack(id) | PlayLandFromGraveyard(id) => {
            format!("play land {}", name_of(state, *id))
        }
        CastSpell { card_id, target, .. } => with_target("cast", *card_id, target),
        CastSpellKicked { card_id, target, .. } => with_target("cast (kicked)", *card_id, target),
        ActivateAbility { card_id, ability_index, target, .. } => match target {
            Some(t) => format!(
                "activate {}[{ability_index}] @ {}",
                name_of(state, *card_id),
                target_text(state, t)
            ),
            None => format!("activate {}[{ability_index}]", name_of(state, *card_id)),
        },
        DeclareAttackers(attacks) => {
            if attacks.is_empty() {
                "attack with nobody".to_string()
            } else {
                let names: Vec<String> =
                    attacks.iter().map(|a| name_of(state, a.attacker)).collect();
                format!("attack with {}", names.join(", "))
            }
        }
        DeclareBlockers(pairs) => {
            if pairs.is_empty() {
                "no blocks".to_string()
            } else {
                let blocks: Vec<String> = pairs
                    .iter()
                    .map(|(blocker, attacker)| {
                        format!("{} blocks {}", name_of(state, *blocker), name_of(state, *attacker))
                    })
                    .collect();
                blocks.join("; ")
            }
        }
        SubmitDecision(answer) => {
            let mut s = format!("decision: {answer:?}");
            s.truncate(160);
            s
        }
        other => {
            let mut s = format!("{other:?}");
            s.truncate(160);
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Without `CRAB_DECISION_LOG` armed, `prepare` is a no-op (the
    /// shadow bot must not run during normal matches), and the
    /// describe formatter renders the common shapes without panicking —
    /// including the unknown-card fallback.
    #[test]
    fn unarmed_prepare_is_a_noop_and_describe_renders() {
        let pool = crate::selfplay::sealed_pool(1);
        let deck = crate::selfplay::heuristic_sealed_build(&pool, 2);
        let state = crate::selfplay::sealed_game_template(&deck, &deck);
        assert!(
            prepare(&state, 0, &GameAction::PassPriority).is_none(),
            "no env var, no shadow consult"
        );
        assert_eq!(describe(&state, &GameAction::PassPriority), "pass");
        assert_eq!(describe(&state, &GameAction::DeclareAttackers(vec![])), "attack with nobody");
        // Every spelling of "do nothing" is one equivalence class.
        assert!(is_noop(&GameAction::PassPriority));
        assert!(is_noop(&GameAction::DeclareAttackers(vec![])));
        assert!(is_noop(&GameAction::DeclareBlockers(vec![])));
        assert!(!is_noop(&GameAction::Concede));
        // An id no zone knows falls back to card#N rather than panicking.
        let ghost = describe(&state, &GameAction::PlayLand(CardId(9_999_999)));
        assert!(ghost.contains("card#9999999"), "unknown id falls back: {ghost}");
    }
}
