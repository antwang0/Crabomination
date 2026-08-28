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
//! Dead *modes* are gated too, but against an allowlist
//! ([`crabomination::audit::REVIEWED_DEAD_MODES`]) rather than outright: a
//! `Noop` arm is also the idiom for "you may … (or decline)", so it needs a
//! reviewer's judgement once — and then it needs to stop costing that
//! judgement every run. Before the allowlist the auditor reported exactly one
//! card forever, so the only signal a *new* dead mode gave was a count going
//! from 1 to 2 in a report nobody diffs.

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

/// The mode half, gated against the reviewed list — in both directions.
///
/// Forwards: a card that grows a dead mode fails here by name, so the
/// reviewer decides once instead of the auditor asking forever. Backwards: an
/// entry whose card no longer *has* a dead mode fails too. Without that, a
/// list entry outlives the thing it excused and silently licenses the next
/// dead mode on the same card — an allowlist that cannot go stale is the
/// only kind worth having.
#[test]
fn every_dead_mode_is_one_a_reviewer_signed_off() {
    use crabomination::audit::REVIEWED_DEAD_MODES;

    let mut seen: HashSet<&'static str> = HashSet::new();
    let mut found: Vec<(&'static str, String)> = Vec::new();
    for factory in all_known_factories() {
        let def = factory();
        if !seen.insert(def.name) {
            continue;
        }
        for f in dead_capabilities(&def) {
            if matches!(f, DeadCapability::Mode { .. }) {
                found.push((def.name, f.to_string()));
            }
        }
    }
    found.sort();

    let reviewed: HashSet<&str> = REVIEWED_DEAD_MODES.iter().map(|(n, _)| *n).collect();
    let unreviewed: Vec<String> = found
        .iter()
        .filter(|(name, _)| !reviewed.contains(name))
        .map(|(name, f)| format!("{name}: {f}"))
        .collect();
    assert!(
        unreviewed.is_empty(),
        "{} card(s) have a mode that resolves to nothing and nobody has said \
         which kind it is. If the arm is a missing primitive, implement it. If \
         it is the printed \"you may … (or decline)\", add the card to \
         `crabomination::audit::REVIEWED_DEAD_MODES` with the printed text \
         that makes the empty arm correct:\n  {}",
        unreviewed.len(),
        unreviewed.join("\n  "),
    );

    let live: HashSet<&str> = found.iter().map(|(name, _)| *name).collect();
    let stale: Vec<&str> =
        REVIEWED_DEAD_MODES.iter().map(|(n, _)| *n).filter(|n| !live.contains(n)).collect();
    assert!(
        stale.is_empty(),
        "{} entr(ies) in `REVIEWED_DEAD_MODES` name a card with no dead mode \
         any more — the arm was implemented, or the card was renamed or \
         dropped. Remove them, or they will excuse the next dead mode on that \
         card:\n  {}",
        stale.len(),
        stale.join("\n  "),
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

/// `grant_bits::ANY_GRANT` is `CardDefinition::can_grant_keyword` at
/// `|_| true`, and `card_can_grant_keyword` returns `false` outright when it
/// is clear. That is sound only because the function is **monotone in its
/// predicate** — every leg is an `any` / `||` / a recursion, with no negation
/// — so this walks the whole catalog against a battery of the keywords real
/// callers ask about and pins the implication.
///
/// The battery is a sample; the argument is the proof. What the test catches
/// is a later edit that puts a negation in one of the legs, which would make
/// the bit stop being an over-approximation of the predicate form and start
/// hiding grants.
#[test]
fn a_clear_any_grant_bit_is_authoritative_for_every_predicate() {
    use crabomination::card::{Keyword, grant_bits};
    use crabomination::mana::Color;

    let battery: Vec<(&str, Box<dyn Fn(&Keyword) -> bool>)> = vec![
        ("flying", Box::new(|k: &Keyword| matches!(k, Keyword::Flying))),
        ("haste", Box::new(|k: &Keyword| matches!(k, Keyword::Haste))),
        ("hexproof", Box::new(|k: &Keyword| matches!(k, Keyword::Hexproof))),
        ("menace", Box::new(|k: &Keyword| matches!(k, Keyword::Menace))),
        ("cant_block", Box::new(|k: &Keyword| matches!(k, Keyword::CantBlock))),
        ("trample", Box::new(|k: &Keyword| matches!(k, Keyword::Trample))),
        ("absorb", Box::new(|k: &Keyword| matches!(k, Keyword::Absorb(_)))),
        ("protection", Box::new(|k: &Keyword| matches!(k, Keyword::Protection(_)))),
        ("protection_white", Box::new(|k: &Keyword| matches!(k, Keyword::Protection(Color::White)))),
        ("cumulative_upkeep", Box::new(|k: &Keyword| matches!(k, Keyword::CumulativeUpkeep(_)))),
        ("must_attack", Box::new(|k: &Keyword| matches!(k, Keyword::MustAttack))),
        ("phasing", Box::new(|k: &Keyword| matches!(k, Keyword::Phasing))),
        ("any", Box::new(|_: &Keyword| true)),
    ];

    let mut seen: HashSet<&'static str> = HashSet::new();
    let mut bad: Vec<String> = Vec::new();
    let (mut with_bit, mut total) = (0usize, 0usize);
    for factory in all_known_factories() {
        let def = factory();
        if !seen.insert(def.name) {
            continue;
        }
        total += 1;
        let set = def.grant_scan_bits() & grant_bits::ANY_GRANT != 0;
        if set {
            with_bit += 1;
            continue;
        }
        for (name, pred) in &battery {
            if def.can_grant_keyword(pred) {
                bad.push(format!("{}: ANY_GRANT clear but grants {name}", def.name));
            }
        }
    }
    assert!(bad.is_empty(), "the gate is not an over-approximation:\n{}", bad.join("\n"));
    // Not vacuous in either direction: the bit is set on a real population and
    // clear on a much larger one, which is the whole reason it is a gate.
    assert!(with_bit > 100, "only {with_bit} of {total} definitions can grant a keyword");
    assert!(total - with_bit > 1_000, "the gate excludes almost nothing");
}
