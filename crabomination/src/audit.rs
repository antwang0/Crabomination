//! Structural card audit: the one walker that decides whether a card's
//! effect tree does nothing.
//!
//! Two hand-written copies of this used to live in `bin/audit_incomplete.rs`
//! (over the serialized JSON) and the since-retired `bin/audit_stubs.rs`
//! (over the typed `Effect`), and they had already drifted — the typed one
//! didn't know about `Escalate`. One walker, over the JSON form, so a new combinator can't
//! silently make one of them wrong: externally-tagged enums serialize a unit
//! variant as the bare string `"Noop"` and everything else as
//! `{"Variant": payload}`, so an unrecognized tag reads as "does something",
//! which is the safe answer.
//!
//! [`dead_capabilities`] is also asserted over the whole catalog by
//! `crabomination_tests` (`core_rules::structural_audit`), so a card shipped
//! with a dead mode or a dead ability fails the suite instead of waiting for
//! someone to re-run the auditor. Dead abilities are gated outright; dead
//! modes are gated against [`REVIEWED_DEAD_MODES`], because a `Noop` arm is
//! also how a printed "you may … (or decline)" is spelled. [`stub_kind`]
//! (blank spells and blank permanents) is gated outright by the same file.

use crabomination_base::card::{CardDefinition, CardType};
use serde_json::Value;

/// The dead *modes* that are correct, and why each one is.
///
/// A `Noop` arm is genuinely ambiguous: it is both how a missing primitive
/// looks and how a printed "you may … (or decline)" is spelled. That is why
/// the suite gated dead *abilities* and left modes to whoever next ran the
/// auditor by hand — and it meant the auditor reported one card every run,
/// forever, so the only signal a new one gave was a count going from 1 to 2.
/// Nobody diffs a count they have learned to expect.
///
/// An allowlist converts the triage into a gate. A new dead mode fails the
/// suite with the card's name; a reviewer either implements the arm or adds
/// it here with the printed text that makes it correct. The list is asserted
/// in **both** directions, so an entry whose card stopped having a dead mode
/// fails too rather than quietly licensing a future one.
pub const REVIEWED_DEAD_MODES: &[(&str, &str)] = &[(
    "Hullbreaker Horror",
    "\"Choose up to one\" — choosing neither mode is the empty arm, which is \
     the printed card and not a gap",
), (
    "Elite Interceptor",
    "\"You may tap or untap target creature\" — declining is the empty arm, \
     which is the printed card and not a gap",
), (
    "Braids, Arisen Nightmare",
    "\"You may sacrifice an artifact, creature, enchantment, land, or \
     planeswalker\" — the sixth arm is declining the sacrifice, which is the \
     printed card and not a gap",
)];

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
/// you've also checked it has no cast trigger. See [`stub_kind`].
pub fn resolve_effect_is_empty(def: &CardDefinition) -> bool {
    serde_json::to_value(&def.effect).is_ok_and(|v| effect_is_empty(&v))
}

/// Does this definition give a permanent *any* text?
///
/// The four ability vectors plus keywords are the obvious carriers, but a
/// permanent's rules text can live in a dedicated field instead, and every
/// one of those is a card that reads as blank if it is not listed here. As
/// of 2026-08-14 that was **59 of the 59 cards the stub audit flagged** —
/// every Saga, Room, Siege, Case, enters-as-copy and state-triggered
/// enchantment in the catalog — which is a broken audit, not a catalog of
/// stubs.
///
/// **When a new mechanic adds a carrier field to `CardDefinition`, add it
/// here.** `core_rules::structural_audit` pins one representative per family
/// (`blank_permanent_check_knows_every_carrier_field`), and a new family with
/// no entry fails `no_shipped_card_is_a_blank_stub` by name.
pub fn def_has_any_ability(def: &CardDefinition) -> bool {
    !def.triggered_abilities.is_empty()
        || !def.activated_abilities.is_empty()
        || !def.static_abilities.is_empty()
        || !def.loyalty_abilities.is_empty()
        || !def.keywords.is_empty()
        // Chapter / door / mode / band carriers: the text is a list keyed by
        // something other than "ability kind".
        || !def.saga_chapters.is_empty()
        || def.room.is_some()
        || def.case.is_some()
        || def.enter_modes.is_some()
        || def.enters_as_choice.is_some()
        || !def.level_bands.is_empty()
        || !def.station.is_empty()
        || !def.attraction_lights.is_empty()
        // Replacement / as-enters text.
        || def.enters_as_copy.is_some()
        || def.as_enters_effect.is_some()
        || def.as_transforms_effect.is_some()
        || def.enters_with_counters.is_some()
        || def.opening_hand.is_some()
        // State-triggered ability (CR 603.8) — Veiled Crocodile, Hidden
        // Predators wake into creatures without a `TriggeredAbility`.
        || def.state_trigger.is_some()
        // Self-sacrifice / countdown clocks.
        || def.sacrifice_when.is_some()
        || def.exile_countdown.is_some()
        || def.sacrifice_when_you_control_no_other.is_some()
        || def.sacrifice_and_burn_when_stolen.is_some()
        // Characteristic-defining and attachment text.
        || def.dynamic_pt.is_some()
        || def.equipped_bonus.is_some()
        || def.soulbond_bonus.is_some()
        || def.copies_top_graveyard_creature
        || def.max_counters_of_kind.is_some()
        // A permanent whose other face carries the text.
        || def.back_face.is_some()
        || def.flip_face.is_some()
}

/// Why a card "does nothing", or `None` for a card with real text: an
/// instant / sorcery that resolves to nothing and has no cast trigger, a
/// non-creature permanent with no text at all, or a planeswalker with no
/// loyalty abilities. Vanilla creatures are fine — the body is the card.
///
/// This was the `audit_stubs` binary, which read 0 flagged from 2026-08-14
/// on; `core_rules::structural_audit` now asserts that zero instead.
pub fn stub_kind(def: &CardDefinition) -> Option<&'static str> {
    if def.is_instant() || def.is_sorcery() {
        if resolve_effect_is_empty(def) && def.triggered_abilities.is_empty() {
            return Some("BLANK SPELL (resolves to nothing)");
        }
        return None;
    }
    let non_creature_perm = (def.card_types.contains(&CardType::Artifact)
        || def.card_types.contains(&CardType::Enchantment)
        || def.card_types.contains(&CardType::Planeswalker))
        && !def.is_creature();
    if non_creature_perm && resolve_effect_is_empty(def) && !def_has_any_ability(def) {
        return Some("BLANK PERMANENT (no abilities)");
    }
    if def.is_planeswalker() && def.loyalty_abilities.is_empty() {
        return Some("PLANESWALKER without loyalty abilities");
    }
    None
}
