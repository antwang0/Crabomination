//! Structural card audit: the one walker that decides whether a card's
//! effect tree does nothing.
//!
//! Two hand-written copies of this used to live in `bin/audit_incomplete.rs`
//! (over the serialized JSON) and `bin/audit_stubs.rs` (over the typed
//! `Effect`), and they had already drifted — the typed one didn't know about
//! `Escalate`. One walker, over the JSON form, so a new combinator can't
//! silently make one of them wrong: externally-tagged enums serialize a unit
//! variant as the bare string `"Noop"` and everything else as
//! `{"Variant": payload}`, so an unrecognized tag reads as "does something",
//! which is the safe answer.
//!
//! [`dead_capabilities`] is also asserted over the whole catalog by
//! `crabomination_tests` (`core_rules::structural_audit`), so a card shipped
//! with a dead mode or a dead ability fails the suite instead of waiting for
//! someone to re-run the auditor.

use crabomination_base::card::CardDefinition;
use serde_json::Value;

/// A selectable capability that resolves to nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeadCapability {
    /// A `ChooseMode` / `ChooseN` / `Escalate` arm that resolves to nothing.
    /// Ambiguous on its own: a `Noop` arm is also the idiom for a deliberate
    /// "you may … (or decline)" option, so these need human triage.
    Mode { modal: &'static str, index: usize },
    /// A triggered / activated / loyalty ability with an empty effect. Always
    /// a bug unless the activation cost *is* the whole ability (see
    /// [`ability_is_cost_only`]).
    Ability { kind: &'static str, index: usize },
}

impl std::fmt::Display for DeadCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mode { modal, index } => write!(
                f,
                "{modal} arm #{index} resolves to nothing (REVIEW: a \
                 placeholder for a missing primitive, OR a deliberate \
                 \"you may … / decline\" mode)"
            ),
            Self::Ability { kind, index } => {
                write!(f, "{kind} ability #{index} has an empty effect")
            }
        }
    }
}

/// True if a serialized `Effect` node does nothing at all.
pub fn effect_is_empty(v: &Value) -> bool {
    match v {
        Value::String(s) => s == "Noop",
        Value::Object(m) if m.len() == 1 => {
            let (tag, p) = m.iter().next().expect("len 1");
            match tag.as_str() {
                "Seq" | "ChooseMode" => arr_all_empty(p),
                "ChooseN" | "Escalate" => p.get("modes").map(arr_all_empty).unwrap_or(false),
                "If" => matches!(
                    (p.get("then"), p.get("else_")),
                    (Some(t), Some(e)) if effect_is_empty(t) && effect_is_empty(e)
                ),
                "ForEach" | "Repeat" | "MayDo" => p.get("body").map(effect_is_empty).unwrap_or(false),
                _ => false,
            }
        }
        _ => false,
    }
}

fn arr_all_empty(v: &Value) -> bool {
    v.as_array().map(|a| a.iter().all(effect_is_empty)).unwrap_or(false)
}

/// The activation costs that *are* the ability: "{cost}: Sacrifice this."
/// (Hopeful Vigil), "You may discard this any time you could cast an instant."
/// (Circling Vultures), and the bounce/exile/return equivalents. An empty
/// resolution effect is correct for these — paying the cost is the whole
/// printed text — so they are not dead abilities.
fn ability_is_cost_only(ab: &Value) -> bool {
    /// Every one of these moves the source itself, so paying it is an
    /// observable game action on its own. A cost that merely *pays*
    /// (mana, tap, life) is not on the list — an ability with only those
    /// and no effect really does nothing.
    const COST_IS_THE_EFFECT: &[&str] = &[
        "sac_cost",
        "discard_self_cost",
        "bounce_self_cost",
        "return_self_cost",
        "exile_self_cost",
    ];
    COST_IS_THE_EFFECT.iter().any(|k| ab.get(k).and_then(Value::as_bool).unwrap_or(false))
}

/// Recursively hunt for modal nodes anywhere in the tree and report any arm
/// that resolves to nothing.
fn find_dead_modes(v: &Value, out: &mut Vec<DeadCapability>) {
    match v {
        Value::Object(m) => {
            for (tag, p) in m {
                match tag.as_str() {
                    "ChooseMode" => {
                        if let Some(arr) = p.as_array() {
                            for (i, arm) in arr.iter().enumerate() {
                                if effect_is_empty(arm) {
                                    out.push(DeadCapability::Mode { modal: "ChooseMode", index: i });
                                }
                            }
                        }
                    }
                    "ChooseN" | "Escalate" => {
                        if let Some(arr) = p.get("modes").and_then(Value::as_array) {
                            for (i, arm) in arr.iter().enumerate() {
                                if effect_is_empty(arm) {
                                    out.push(DeadCapability::Mode {
                                        modal: "ChooseN/Escalate",
                                        index: i,
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }
                find_dead_modes(p, out);
            }
        }
        Value::Array(a) => a.iter().for_each(|e| find_dead_modes(e, out)),
        _ => {}
    }
}

/// Every dead mode and dead ability in one card. Empty for a healthy card.
pub fn dead_capabilities(def: &CardDefinition) -> Vec<DeadCapability> {
    let mut out = Vec::new();
    let v = serde_json::to_value(def).expect("CardDefinition serializes");
    find_dead_modes(&v, &mut out);
    // Static abilities carry a `StaticEffect`, not an `Effect`, so they're
    // out of scope here.
    for (key, kind) in [
        ("triggered_abilities", "triggered"),
        ("activated_abilities", "activated"),
        ("loyalty_abilities", "loyalty"),
    ] {
        let Some(arr) = v.get(key).and_then(Value::as_array) else { continue };
        for (i, ab) in arr.iter().enumerate() {
            if ab.get("effect").is_some_and(effect_is_empty) && !ability_is_cost_only(ab) {
                out.push(DeadCapability::Ability { kind, index: i });
            }
        }
    }
    out
}

/// True if the card's *resolve* effect does nothing — a blank spell, once
/// you've also checked it has no cast trigger. Used by `audit_stubs`.
pub fn resolve_effect_is_empty(def: &CardDefinition) -> bool {
    serde_json::to_value(&def.effect).is_ok_and(|v| effect_is_empty(&v))
}
