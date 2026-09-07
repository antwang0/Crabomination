//! CR 612.1 — creature-type word rewrites over a serialized definition.

use crate::card::{CardDefinition, CreatureType};

/// `def` with every printed instance of `from` rewritten as `to` — type
/// line, filters and ability bodies alike; `name` is skipped so a card called
/// "Goblin …" keeps its name (CR 612.2). The walk runs over the serialized
/// definition so the whole `Effect` / `SelectionRequirement` tree is covered
/// without a per-variant visitor. `None` if the round trip fails.
///
/// Lives here rather than beside its one engine caller because the round
/// trip is a `CardDefinition: Deserialize` instantiation — the engine crate's
/// only one, and 482 k of its 4.4 M LLVM-IR lines (PERF "Serde derives").
/// In this crate it is codegen'd on a base edit, not on every engine edit.
pub fn rewrite_creature_type(
    def: &CardDefinition,
    from: CreatureType,
    to: CreatureType,
) -> Option<CardDefinition> {
    let json = serde_json::to_value(def).ok()?;
    let (from_word, to_word) = (format!("{from:?}"), format!("{to:?}"));
    serde_json::from_value(rewrite_json_words(json, &from_word, &to_word)).ok()
}

fn rewrite_json_words(v: serde_json::Value, from: &str, to: &str) -> serde_json::Value {
    use serde_json::Value;
    match v {
        Value::String(s) if s == from => Value::String(to.to_string()),
        Value::Array(a) => {
            Value::Array(a.into_iter().map(|x| rewrite_json_words(x, from, to)).collect())
        }
        Value::Object(o) => Value::Object(
            o.into_iter()
                .map(|(k, x)| {
                    let x = if k == "name" { x } else { rewrite_json_words(x, from, to) };
                    (k, x)
                })
                .collect(),
        ),
        other => other,
    }
}
