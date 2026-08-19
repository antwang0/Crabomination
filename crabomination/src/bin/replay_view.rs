//! Narrate a recorded match replay as readable text.
//!
//! ```text
//! replay_view <replay.jsonl> [--all]
//! replay_view                # newest replay-*.jsonl under $CRAB_REPLAY_DIR
//! ```
//!
//! Reads the JSONL files `CRAB_REPLAY_DIR` records (see
//! `server/replay.rs`): a header with the players, one line per event
//! batch — v2 lines carry a first-appearance card-name table, which is
//! what makes the narration readable — and an end footer. By default the
//! mana/tap noise is hidden; `--all` prints every event. Cross-reference
//! against a `CRAB_DECISION_LOG` file from the same match to see what
//! the bot thought of each of your plays as the game unfolds.

use std::collections::HashMap;

use serde_json::Value;

fn main() {
    let mut args = std::env::args().skip(1);
    let mut path: Option<String> = None;
    let mut all = false;
    for a in args.by_ref() {
        match a.as_str() {
            "--all" => all = true,
            other => path = Some(other.to_string()),
        }
    }
    let path = path.map(std::path::PathBuf::from).unwrap_or_else(|| {
        let dir = std::env::var_os("CRAB_REPLAY_DIR").unwrap_or_else(|| {
            eprintln!("usage: replay_view <replay.jsonl> [--all]  (or set CRAB_REPLAY_DIR)");
            std::process::exit(2);
        });
        let newest = std::fs::read_dir(&dir)
            .ok()
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with("replay-"))
            .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
        match newest {
            Some(e) => e.path(),
            None => {
                eprintln!("no replay-*.jsonl under {}", std::path::Path::new(&dir).display());
                std::process::exit(2);
            }
        }
    });
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("cannot read {}: {e}", path.display());
        std::process::exit(2);
    });

    let mut players: Vec<String> = Vec::new();
    let mut names: HashMap<u64, String> = HashMap::new();
    for line in text.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            eprintln!("(unparseable line skipped)");
            continue;
        };
        if v.get("replay").is_some() {
            players = v["players"]
                .as_array()
                .map(|a| a.iter().filter_map(|p| p.as_str().map(String::from)).collect())
                .unwrap_or_default();
            println!("replay: {} — {}", path.display(), players.join(" vs "));
            continue;
        }
        if v.get("end").is_some() {
            println!("\n── end of replay ──");
            continue;
        }
        // v2: {"e": [...], "n": {...}}; v1: a bare event array.
        let (events, new_names) = match &v {
            Value::Object(m) => (m.get("e").cloned().unwrap_or(Value::Null), m.get("n").cloned()),
            Value::Array(_) => (v.clone(), None),
            _ => continue,
        };
        if let Some(Value::Object(n)) = new_names {
            for (id, name) in n {
                if let (Ok(id), Some(name)) = (id.parse::<u64>(), name.as_str()) {
                    names.insert(id, name.to_string());
                }
            }
        }
        // The engine emits an origin-scoped twin for exile-from-play and
        // the wire collapses both onto `PermanentExiled` (net.rs documents
        // the collapse), so one exile arrives as two identical adjacent
        // events. Byte-identical neighbors narrate once; distinct copies
        // of a same-named card have different ids and still both print.
        let mut prev: Option<&Value> = None;
        for ev in events.as_array().into_iter().flatten() {
            if prev == Some(ev) {
                continue;
            }
            prev = Some(ev);
            if let Some(line) = narrate(ev, &players, &names, all) {
                println!("{line}");
            }
        }
    }
}

fn player(players: &[String], i: u64) -> String {
    players.get(i as usize).cloned().unwrap_or_else(|| format!("P{i}"))
}

fn card(names: &HashMap<u64, String>, v: &Value) -> String {
    v.as_u64()
        .map(|id| names.get(&id).cloned().unwrap_or_else(|| format!("card#{id}")))
        .unwrap_or_else(|| v.to_string())
}

/// One line per event; `None` hides it. Externally-tagged serde shapes:
/// `{"Variant": {fields}}`, `{"Variant": value}` (tuple), `"Variant"`
/// (unit). Common events get prose; the rest fall back to
/// `Variant field: value` with card ids resolved.
fn narrate(ev: &Value, players: &[String], names: &HashMap<u64, String>, all: bool) -> Option<String> {
    let (variant, f): (&str, Value) = match ev {
        Value::String(s) => (s.as_str(), Value::Null),
        Value::Object(m) if m.len() == 1 => {
            let (k, v) = m.iter().next().unwrap();
            (k.as_str(), v.clone())
        }
        _ => return Some(format!("  ? {ev}")),
    };
    // Internal signals that render blank in the client, plus (by
    // default) the mana-and-tapping noise between the real beats.
    const BLANK: &[&str] = &["FirstCardDrawnThisTurn", "PermanentDied"];
    const NOISE: &[&str] = &[
        "ManaAdded",
        "ColorlessManaAdded",
        "TappedForMana",
        "PermanentTapped",
        "PermanentUntapped",
        // Redundant beside the "dies" / "discards" / activation lines
        // they always accompany.
        "CardPutIntoGraveyard",
        "CardLeftGraveyard",
        "CombatResolved",
        "ChoseTargets",
    ];
    if BLANK.contains(&variant) || (!all && NOISE.contains(&variant)) {
        return None;
    }
    let p = |key: &str| player(players, f[key].as_u64().unwrap_or(99));
    let c = |key: &str| card(names, &f[key]);
    Some(match variant {
        "TurnStarted" => {
            format!("\n── turn {} — {} ──", f["turn"], p("player"))
        }
        "StepChanged" => format!("  · {}", f.as_str().unwrap_or("?")),
        "CardDrawn" => format!("  {} draws {}", p("player"), c("card_id")),
        "CardDiscarded" => format!("  {} discards {}", p("player"), c("card_id")),
        "LandPlayed" => format!("  {} plays {}", p("player"), c("card_id")),
        "SpellCast" => {
            let face = f["face"].as_str().filter(|s| *s != "Front");
            match face {
                Some(face) => format!("  {} casts {} ({face})", p("player"), c("card_id")),
                None => format!("  {} casts {}", p("player"), c("card_id")),
            }
        }
        "AbilityActivated" => format!("  {} activates", c("source")),
        // Tuple variant: the payload IS the attacker id.
        "AttackerDeclared" => format!("  {} attacks", card(names, &f)),
        "BlockerDeclared" => format!("  {} blocks {}", c("blocker"), c("attacker")),
        "AttackerWentUnblocked" => format!("  {} is unblocked", c("attacker")),
        "PermanentEntered" => format!("  {} enters", c("card_id")),
        "TokenCreated" => format!("  token: {}", c("card_id")),
        "PermanentExiled" => format!("  {} is exiled", c("card_id")),
        "CreatureDied" => format!("  {} dies", c("card_id")),
        "CreatureSacrificed" | "PermanentSacrificed" => {
            format!("  {} sacrifices {}", p("who"), c("card_id"))
        }
        "DamageDealt" | "DamagePrevented" => {
            let target = if f["to_player"].is_u64() {
                player(players, f["to_player"].as_u64().unwrap())
            } else {
                card(names, &f["to_card"])
            };
            let verb = if variant == "DamageDealt" { "damage to" } else { "damage prevented on" };
            format!("  {} {verb} {target}", f["amount"])
        }
        "LifeLost" => format!("  {} loses {} life", p("player"), f["amount"]),
        "LifeGained" => format!("  {} gains {} life", p("player"), f["amount"]),
        "PaidLife" => format!("  {} pays {} life", p("player"), f["amount"]),
        "PumpApplied" => {
            format!("  {} gets {:+}/{:+}", c("card_id"), f["power"], f["toughness"])
        }
        "CounterAdded" => format!(
            "  {} +{} {} counter(s)",
            c("card_id"),
            f["count"],
            f["counter_type"].as_str().unwrap_or("?")
        ),
        _ => {
            // Generic fallback: variant plus its fields, ids resolved.
            let fields = match &f {
                Value::Object(m) => m
                    .iter()
                    .map(|(k, v)| {
                        if matches!(
                            k.as_str(),
                            "card_id" | "source" | "to_card" | "attacker" | "blocker" | "object"
                        ) {
                            format!("{k}: {}", card(names, v))
                        } else {
                            format!("{k}: {v}")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                Value::Null => String::new(),
                other => other.to_string(),
            };
            format!("  {variant} {fields}").trim_end().to_string()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Vec<String>, HashMap<u64, String>) {
        (vec!["You".into(), "Bot".into()], HashMap::from([(7, "Grizzly Bears".into())]))
    }

    #[test]
    fn common_events_read_as_prose() {
        let (players, names) = setup();
        let ev = serde_json::json!({"SpellCast": {"player": 1, "card_id": 7, "face": "Front"}});
        assert_eq!(narrate(&ev, &players, &names, false).unwrap(), "  Bot casts Grizzly Bears");
        let ev = serde_json::json!({"DamageDealt": {"amount": 3, "to_player": 0, "to_card": null}});
        assert_eq!(narrate(&ev, &players, &names, false).unwrap(), "  3 damage to You");
        let ev = serde_json::json!({"TurnStarted": {"player": 0, "turn": 4}});
        assert_eq!(narrate(&ev, &players, &names, false).unwrap(), "\n── turn 4 — You ──");
    }

    #[test]
    fn noise_hides_by_default_and_unknown_ids_fall_back() {
        let (players, names) = setup();
        let tap = serde_json::json!({"PermanentTapped": {"card_id": 7}});
        assert!(narrate(&tap, &players, &names, false).is_none(), "tap noise hidden");
        assert!(narrate(&tap, &players, &names, true).is_some(), "--all shows it");
        let ev = serde_json::json!({"CreatureDied": {"card_id": 99}});
        assert_eq!(narrate(&ev, &players, &names, false).unwrap(), "  card#99 dies");
        // A future variant this viewer has never heard of still prints.
        let ev = serde_json::json!({"BrandNewThing": {"card_id": 7, "n": 2}});
        assert_eq!(
            narrate(&ev, &players, &names, false).unwrap(),
            "  BrandNewThing card_id: Grizzly Bears, n: 2"
        );
    }
}
