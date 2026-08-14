//! Throwaway audit: scan every catalog card for "does nothing" stubs.
//!
//! Two failure classes, both fully automatable:
//!   1. Instant/Sorcery whose resolve `effect` is Noop (and the whole
//!      effect tree is empty) — the spell resolves and does literally
//!      nothing.
//!   2. Non-creature, non-land permanent (Artifact/Enchantment/
//!      Planeswalker) with no abilities of any kind and no keywords —
//!      a blank permanent. "No abilities" means no *carrier field* set,
//!      not just four empty ability vectors — see `def_has_any_ability`.
//!
//! **Reads 0 flagged as of 2026-08-14** over 21,795 unique cards. It read
//! 59 before that, and all 59 were false positives from a stale
//! `def_has_any_ability`. A non-empty run is now worth reading; before,
//! the bucket was noise that hid whatever else landed in it.
//!
//! Run: `cargo run -p crabomination --bin audit_stubs`

use crabomination::fxhash::HashSet;

use crabomination::audit::resolve_effect_is_empty;
use crabomination::card::{CardDefinition, CardType};
use crabomination::catalog::all_known_factories;

/// Does this definition give a permanent *any* text?
///
/// The four ability vectors plus keywords are the obvious carriers, but a
/// permanent's rules text can live in a dedicated field instead, and every
/// one of those is a card that reads as blank if it is not listed here. As
/// of 2026-08-14 that was **59 of the 59 cards this audit flagged** — every
/// Saga, Room, Siege, Case, enters-as-copy and state-triggered enchantment
/// in the catalog — which is a broken audit, not a catalog of stubs.
///
/// **When a new mechanic adds a carrier field to `CardDefinition`, add it
/// here.** `blank_permanent_check_knows_every_carrier_field` in this file
/// pins one representative per family; a new family with no entry shows up
/// as noise in the BLANK PERMANENT bucket rather than as a test failure, so
/// the bucket being non-empty is the signal to re-read this list.
fn def_has_any_ability(def: &CardDefinition) -> bool {
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

fn classify(def: &CardDefinition) -> Option<&'static str> {
    let is_is = def.is_instant() || def.is_sorcery();
    if is_is {
        // A spell whose resolve does nothing and has no cast-trigger.
        if resolve_effect_is_empty(def) && def.triggered_abilities.is_empty() {
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
    if non_creature_perm && resolve_effect_is_empty(def) && !def_has_any_ability(def) {
        return Some("BLANK PERMANENT (no abilities)");
    }
    // Planeswalker with no loyalty abilities is unusable.
    if def.is_planeswalker() && def.loyalty_abilities.is_empty() {
        return Some("PLANESWALKER without loyalty abilities");
    }
    None
}

fn main() {
    let mut seen: HashSet<String> = HashSet::default();
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
                    CardType::Vanguard => "Vanguard",
                    CardType::Conspiracy => "Conspiracy",
                    CardType::Scheme => "Scheme",
                    CardType::Plane => "Plane",
                    CardType::Phenomenon => "Phenomenon",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crabomination::catalog;

    /// One representative per carrier family. Each of these was in the
    /// BLANK PERMANENT bucket before `def_has_any_ability` learned its
    /// field, and each is a shipped card with real text — so a regression
    /// here means the audit has started lying about the catalog again.
    #[test]
    fn blank_permanent_check_knows_every_carrier_field() {
        let cases: &[(&str, fn() -> CardDefinition)] = &[
            ("saga_chapters", catalog::history_of_benalia),
            ("room", catalog::bottomless_pool_locker_room),
            ("enter_modes", catalog::barrensteppe_siege),
            ("enters_as_copy", catalog::copy_enchantment),
            ("state_trigger", catalog::veiled_crocodile),
        ];
        for (field, factory) in cases {
            let def = factory();
            assert!(
                def_has_any_ability(&def),
                "{} carries its text in `{field}` and reads as blank",
                def.name
            );
            assert_eq!(
                classify(&def),
                None,
                "{} is flagged as a stub but is fully implemented",
                def.name
            );
        }
    }

    /// The other half of the contract: a genuinely blank permanent is still
    /// caught. Built by hand rather than taken from the catalog, so the test
    /// keeps meaning if every real stub is fixed.
    #[test]
    fn a_permanent_with_no_text_at_all_is_still_flagged() {
        let blank = CardDefinition {
            name: "Test Blank",
            card_types: vec![CardType::Enchantment],
            ..Default::default()
        };
        assert_eq!(classify(&blank), Some("BLANK PERMANENT (no abilities)"));
    }
}
