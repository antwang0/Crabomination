//! Throwaway audit: scan every catalog card for "does nothing" stubs.
//!
//! Two failure classes, both fully automatable:
//!   1. Instant/Sorcery whose resolve `effect` is Noop (and the whole
//!      effect tree is empty) — the spell resolves and does literally
//!      nothing.
//!   2. Non-creature, non-land permanent (Artifact/Enchantment/
//!      Planeswalker) with no abilities of any kind and no keywords —
//!      a blank permanent.
//!
//! Run: `cargo run -p crabomination --bin audit_stubs`

use std::collections::HashSet;

use crabomination::card::{CardDefinition, CardType};
use crabomination::catalog::all_known_factories;
use crabomination::effect::Effect;

/// True if the effect tree does nothing at all (Noop, or nested
/// combinators that bottom out in Noop). Conservative: any leaf that
/// isn't a combinator counts as "does something".
fn effect_is_empty(e: &Effect) -> bool {
    match e {
        Effect::Noop => true,
        Effect::Seq(v) => v.iter().all(effect_is_empty),
        Effect::If { then, else_, .. } => effect_is_empty(then) && effect_is_empty(else_),
        Effect::ForEach { body, .. } => effect_is_empty(body),
        Effect::Repeat { body, .. } => effect_is_empty(body),
        Effect::MayDo { body, .. } => effect_is_empty(body),
        Effect::ChooseMode(v) => v.iter().all(effect_is_empty),
        Effect::ChooseN { modes, .. } => modes.iter().all(effect_is_empty),
        _ => false,
    }
}

fn def_has_any_ability(def: &CardDefinition) -> bool {
    !def.triggered_abilities.is_empty()
        || !def.activated_abilities.is_empty()
        || !def.static_abilities.is_empty()
        || !def.loyalty_abilities.is_empty()
        || !def.keywords.is_empty()
}

fn classify(def: &CardDefinition) -> Option<&'static str> {
    let is_is = def.is_instant() || def.is_sorcery();
    if is_is {
        // A spell whose resolve does nothing and has no cast-trigger.
        if effect_is_empty(&def.effect) && def.triggered_abilities.is_empty() {
            return Some("BLANK SPELL (resolves to nothing)");
        }
        return None;
    }
    // Permanents (non-land). Creatures with a body are fine even with
    // no abilities (vanilla), so only flag non-creature permanents.
    let non_creature_perm = (def.card_types.contains(&CardType::Artifact)
        || def.card_types.contains(&CardType::Enchantment)
        || def.card_types.contains(&CardType::Planeswalker))
        && !def.is_creature();
    if non_creature_perm && effect_is_empty(&def.effect) && !def_has_any_ability(def) {
        return Some("BLANK PERMANENT (no abilities)");
    }
    // Planeswalker with no loyalty abilities is unusable.
    if def.is_planeswalker() && def.loyalty_abilities.is_empty() {
        return Some("PLANESWALKER without loyalty abilities");
    }
    None
}

fn main() {
    let mut seen: HashSet<String> = HashSet::new();
    let mut flagged: Vec<(String, &'static str, String)> = Vec::new();
    let mut total = 0usize;

    for factory in all_known_factories() {
        let def = factory();
        if !seen.insert(def.name.to_string()) {
            continue;
        }
        total += 1;
        if let Some(reason) = classify(&def) {
            let types: Vec<&str> = def
                .card_types
                .iter()
                .map(|t| match t {
                    CardType::Land => "Land",
                    CardType::Creature => "Creature",
                    CardType::Artifact => "Artifact",
                    CardType::Enchantment => "Enchantment",
                    CardType::Planeswalker => "Planeswalker",
                    CardType::Battle => "Battle",
                    CardType::Instant => "Instant",
                    CardType::Sorcery => "Sorcery",
                    CardType::Kindred => "Kindred",
                })
                .collect();
            flagged.push((def.name.to_string(), reason, types.join(" ")));
        }
    }

    flagged.sort();
    eprintln!("Scanned {total} unique cards; {} flagged.\n", flagged.len());
    let mut last = "";
    for (name, reason, types) in &flagged {
        if *reason != last {
            eprintln!("\n=== {reason} ===");
            last = reason;
        }
        eprintln!("  {name}  [{types}]");
    }
}
