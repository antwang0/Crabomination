//! Whole-catalog structural audit as a test: no shipped card may carry a
//! triggered / activated / loyalty ability whose effect resolves to nothing.
//!
//! Dead abilities are real bugs — the engine puts an empty object on the
//! stack and burns a priority round for an ability real Magic doesn't have.
//! Three shipped cards had one (Magosi and Oran-Rief via a `tapped_etb_land`
//! helper that emitted `etb(Noop)`; Annie Joins Up via a filler
//! `legend_enters(Noop)`), and nothing caught them until someone re-ran
//! `audit_incomplete` by hand. This is that auditor's pass 1, wired into the
//! suite so the class can't come back silently.
//!
//! Dead *modes* are deliberately not asserted: a `Noop` arm is also the
//! idiom for "you may … (or decline)", so they need triage rather than a
//! gate. `cargo run -p crabomination --bin audit_incomplete --
//! --structural-only` still lists them.

use crabomination::audit::{DeadCapability, dead_capabilities};
use crabomination::catalog::all_known_factories;
use std::collections::HashSet;

#[test]
fn no_shipped_card_has_a_dead_ability() {
    let mut seen: HashSet<&'static str> = HashSet::new();
    let mut bad: Vec<String> = Vec::new();
    for factory in all_known_factories() {
        let def = factory();
        if !seen.insert(def.name) {
            continue;
        }
        for f in dead_capabilities(&def) {
            if matches!(f, DeadCapability::Ability { .. }) {
                bad.push(format!("{}: {f}", def.name));
            }
        }
    }
    bad.sort();
    assert!(
        bad.is_empty(),
        "{} card(s) ship an ability that resolves to nothing — either give it \
         its printed effect or stop emitting the ability:\n  {}",
        bad.len(),
        bad.join("\n  "),
    );
}

/// The carve-out the audit relies on: an activated ability whose cost moves
/// the source is complete with an empty resolution effect. Circling Vultures
/// ("You may discard this any time you could cast an instant") is the live
/// example — pin it so a future tightening of `ability_is_cost_only` fails
/// here rather than turning the test above into a false alarm.
#[test]
fn cost_only_abilities_are_not_dead() {
    let vultures = crabomination::catalog::circling_vultures();
    assert!(
        vultures.activated_abilities.iter().any(|a| a.discard_self_cost),
        "Circling Vultures still has its discard-self ability",
    );
    assert_eq!(dead_capabilities(&vultures), Vec::new());
}

/// The two helpers that used to emit a filler trigger now emit none.
#[test]
fn tapped_etb_lands_without_an_etb_effect_have_no_trigger() {
    for def in [
        crabomination::catalog::magosi_the_waterveil(),
        crabomination::catalog::oran_rief_the_vastwood(),
        crabomination::catalog::annie_joins_up(),
    ] {
        assert!(
            def.triggered_abilities.iter().all(|t| !matches!(
                serde_json::to_value(&t.effect).as_ref(),
                Ok(serde_json::Value::String(s)) if s == "Noop"
            )),
            "{} still carries an empty triggered ability",
            def.name,
        );
    }
    // Annie keeps her real ETB (5 damage) and loses only the filler.
    assert_eq!(crabomination::catalog::annie_joins_up().triggered_abilities.len(), 1);
    assert!(crabomination::catalog::magosi_the_waterveil().triggered_abilities.is_empty());
    assert!(crabomination::catalog::oran_rief_the_vastwood().triggered_abilities.is_empty());
}
