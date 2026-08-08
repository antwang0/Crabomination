//! Whole-catalog invariant for the parallel target walkers.
//!
//! `Effect` carries three hand-written walks over its ~1000 variants:
//! `requires_target`, `primary_target_filter` and `target_filter_for_slot`.
//! Adding a variant that holds a `Selector::TargetFiltered` without also
//! adding an arm to the slot walker compiles fine and then silently does
//! nothing at resolution — the cast/activate path never surfaces the slot, so
//! the effect resolves against an empty target list. That has bitten
//! `Effect::MoveCounters` (Afiya Grove) and
//! `Effect::AttachAuraFromGraveyardTo` (Hakim, Iridescent Drake).
//!
//! This test closes the class: every `TargetFiltered { slot }` reachable in a
//! catalog card's effect tree must be answerable by `target_filter_for_slot`.

use crabomination::card::CardDefinition;
use crabomination::catalog;
use serde_json::Value;

/// A serialized nested ability definition (a granted trigger, a token's
/// ability, a Room half). Its body gets its own slot numbering when *it*
/// resolves, so the enclosing effect never surfaces those slots.
fn is_nested_ability(map: &serde_json::Map<String, Value>) -> bool {
    map.contains_key("effect")
        && (map.contains_key("event") || map.contains_key("mana_cost") || map.contains_key("cost"))
}

/// Every `slot` mentioned by a `Selector::TargetFiltered` in `v`, not
/// descending into nested ability definitions.
fn declared_slots(v: &Value, owner: &str, out: &mut Vec<(u8, String)>) {
    match v {
        Value::Object(map) => {
            if is_nested_ability(map) {
                return;
            }
            if let Some(Value::Object(tf)) = map.get("TargetFiltered")
                && let Some(slot) = tf.get("slot").and_then(|s| s.as_u64())
            {
                out.push((slot as u8, owner.to_string()));
            }
            for (k, inner) in map {
                // An externally-tagged enum object names its variant; keep the
                // nearest one so a finding points at the walker arm to add.
                let next = if k.chars().next().is_some_and(char::is_uppercase) { k } else { owner };
                declared_slots(inner, next, out);
            }
        }
        Value::Array(items) => {
            for inner in items {
                declared_slots(inner, owner, out);
            }
        }
        _ => {}
    }
}

#[test]
fn every_declared_target_slot_is_answerable() {
    let mut bad: Vec<String> = Vec::new();
    for factory in catalog::all_known_factories() {
        let def: CardDefinition = factory();
        // The spell body plus every ability body: each is walked with its own
        // slot numbering by the cast / activate / trigger paths.
        let mut bodies: Vec<(&'static str, &crabomination::effect::Effect)> =
            vec![("spell", &def.effect)];
        for a in &def.activated_abilities {
            bodies.push(("activated", &a.effect));
        }
        for t in &def.triggered_abilities {
            bodies.push(("triggered", &t.effect));
        }
        for l in &def.loyalty_abilities {
            bodies.push(("loyalty", &l.effect));
        }
        for (kind, body) in bodies {
            let json = serde_json::to_value(body).expect("Effect serializes");
            let mut slots = Vec::new();
            declared_slots(&json, "?", &mut slots);
            slots.sort();
            slots.dedup();
            for (slot, owner) in slots {
                // The cast path consults the walker per mode, and the
                // kicker-aware variant when the spell was kicked; a slot only
                // counts as lost when *no* branch can answer it.
                let modes = (0..8).map(Some).chain(std::iter::once(None));
                let answered = modes.clone().any(|m| {
                    body.target_filter_for_slot_in_mode_kicked(slot, m, false).is_some()
                        || body.target_filter_for_slot_in_mode_kicked(slot, m, true).is_some()
                }) || (slot == 0 && body.primary_target_filter().is_some());
                if !answered {
                    let root = match &json {
                        Value::Object(m) => m.keys().next().cloned().unwrap_or_default(),
                        _ => json.to_string(),
                    };
                    bad.push(format!(
                        "root Effect::{root} / inner Effect::{owner} slot {slot} — e.g. {} ({kind})",
                        def.name
                    ));
                }
            }
        }
    }
    bad.sort();
    bad.dedup();
    // A ratchet, not a clean bill of health: the walker still can't answer
    // this many slots (see TODO.md — "The parallel target-walker class").
    // Lower it as arms are added; it must never rise.
    const BASELINE: usize = 39;
    assert!(
        bad.len() <= BASELINE,
        "{} effect bodies (baseline {BASELINE}) declare a TargetFiltered slot \
         that `Effect::target_filter_for_slot` can't answer — the effect \
         resolves against an empty target list:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
}
