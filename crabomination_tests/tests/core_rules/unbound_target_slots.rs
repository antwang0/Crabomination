//! Whole-catalog invariant: a declared target slot is actually *bound*.
//!
//! `core_rules::target_walkers` asserts every `Selector::TargetFiltered` slot
//! is **answerable** by `target_filter_for_slot`. That is necessary and not
//! sufficient: Silent Hallcreeper's copy mode was answerable and still resolved
//! against an empty list, because `Effect::ChooseUnchosenMode` reports
//! `requires_target() == false` and so the cast / activate / trigger path never
//! asked for the slot at all. Answerable is where the filter comes from; bound
//! is whether anything supplies a target.
//!
//! So: for every catalog body, if `requires_target()` is `false`, no slot in it
//! may survive this walk. A slot is allowed to disappear from the walk only by
//! sitting under a variant that supplies targets some other way, and there are
//! exactly three such families:
//!
//! * **Resolution-time targeting** — the variant auto-targets its body when it
//!   runs (`auto_targets_for_effect_all_slots*`). `Reflexive` and
//!   `ReflexiveTrigger` are CR 603.7 "when you do" payoffs; `ChooseUnchosenMode`
//!   picks its mode at resolution and auto-targets it there.
//! * **Cast-time modal** — the *action* carries the picks and their targets
//!   (`GameAction::CastSpellSpree` and friends stamp them onto the
//!   `CardInstance`), and resolution consumes one slot per chosen mode.
//! * **Deferred-fire** — the variant *queues* its body and something else
//!   fires it later; that fire site auto-targets. Haunt registers a
//!   `WhenHauntedCreatureDies` delayed trigger (CR 702.55) and
//!   `ReplaceYourNextDrawThisTurn` queues onto `next_draw_replacements`. Both
//!   are correct only because their fire sites call `auto_target*` — a new
//!   entry here has to be checked at *its* fire site, not at its resolution.
//!
//! A new variant that runs a sub-body and answers `requires_target() == false`
//! belongs in one of those three lists **or** it is a dead mode. Add it here
//! with the reason, do not add a threshold.
//!
//! The first run of this test found **eleven** bodies over **nine** cards, and
//! eight of them were one missing arm each in `requires_target` — the same
//! hand-written-walker class `target_walkers` closed on the filter side.

use crabomination::card::CardDefinition;
use crabomination::catalog;
use crabomination::effect::Effect;
use serde_json::Value;

/// Variants that auto-target their own body when they resolve.
const RESOLUTION_TIME_TARGETING: &[&str] =
    &["Reflexive", "ReflexiveTrigger", "ChooseUnchosenMode"];

/// Modal variants whose targets ride on the cast action, one slot per chosen
/// mode, rather than on a fixed cast-time slot.
const CAST_TIME_MODAL: &[&str] =
    &["Spree", "Tiered", "ChooseModesCast", "ChooseModesByPoints"];

/// Variants that queue their body for a later fire, where the *fire site*
/// auto-targets it. `HauntCreature` -> the `WhenHauntedCreatureDies` delayed
/// trigger in `fire_delayed_event_watchers`; `ReplaceYourNextDrawThisTurn` ->
/// `next_draw_replacements` in the draw path. Both were verified by reading
/// the fire site, which is the only place the check can be made.
const DEFERRED_FIRE: &[&str] = &["HauntCreature", "ReplaceYourNextDrawThisTurn"];

/// A serialized nested ability definition (a granted trigger, a token's
/// ability, a Room half). It is cast or triggered in its own right, so its
/// slots are numbered and bound by *that* resolution, not this one.
fn is_nested_ability(map: &serde_json::Map<String, Value>) -> bool {
    map.contains_key("effect")
        && (map.contains_key("event") || map.contains_key("mana_cost") || map.contains_key("cost"))
}

/// Slots reachable without passing through a variant that binds them some
/// other way. `owner` is the nearest enclosing variant name, so a finding
/// points at the arm to fix.
fn unbound_slots(v: &Value, owner: &str, out: &mut Vec<(u64, String)>) {
    match v {
        Value::Object(map) => {
            if is_nested_ability(map) {
                return;
            }
            if let Some(Value::Object(tf)) = map.get("TargetFiltered")
                && let Some(slot) = tf.get("slot").and_then(|s| s.as_u64())
            {
                out.push((slot, owner.to_string()));
            }
            for (k, inner) in map {
                if RESOLUTION_TIME_TARGETING.contains(&k.as_str())
                    || CAST_TIME_MODAL.contains(&k.as_str())
                    || DEFERRED_FIRE.contains(&k.as_str())
                {
                    continue;
                }
                let next = if k.chars().next().is_some_and(char::is_uppercase) { k } else { owner };
                unbound_slots(inner, next, out);
            }
        }
        Value::Array(items) => {
            for inner in items {
                unbound_slots(inner, owner, out);
            }
        }
        _ => {}
    }
}

#[test]
fn no_body_declares_a_slot_it_never_binds() {
    let mut bad: Vec<String> = Vec::new();
    // How many bodies of each kind the walk actually reached. A green run is
    // only worth something if it looked at something: this run found four
    // shipped tests that passed without the ability under test ever firing,
    // and a catalog walk that silently stops visiting a holder is the same
    // failure one level up.
    let mut seen: std::collections::BTreeMap<&'static str, usize> = Default::default();
    for factory in catalog::all_known_factories() {
        let def: CardDefinition = factory();
        let mut bodies: Vec<(&'static str, &Effect)> = vec![("spell", &def.effect)];
        for a in &def.activated_abilities {
            bodies.push(("activated", &a.effect));
        }
        for t in &def.triggered_abilities {
            bodies.push(("triggered", &t.effect));
        }
        for l in &def.loyalty_abilities {
            bodies.push(("loyalty", &l.effect));
        }
        // An `equipped_bonus` ability is granted to the host and pushed like
        // any other trigger, so its slots are bound (or not) the same way.
        if let Some(bonus) = def.equipped_bonus.as_ref() {
            for t in &bonus.triggered_abilities {
                bodies.push(("equip-bonus", &t.effect));
            }
            for a in &bonus.activated_abilities {
                bodies.push(("equip-bonus", &a.effect));
            }
        }
        // A back face is cast and resolves in its own right.
        if let Some(back) = def.back_face.as_ref() {
            bodies.push(("back-face", &back.effect));
            for a in &back.activated_abilities {
                bodies.push(("back-face", &a.effect));
            }
            for t in &back.triggered_abilities {
                bodies.push(("back-face", &t.effect));
            }
            for l in &back.loyalty_abilities {
                bodies.push(("back-face", &l.effect));
            }
        }
        // CR 715 — an Adventure's instant/sorcery half is cast on its own.
        if let Some(adv) = def.adventure.as_ref() {
            bodies.push(("adventure", &adv.effect));
        }
        for (kind, body) in bodies {
            // A body that demands a slot gets one; this invariant is only
            // about the bodies that say they need nothing.
            if body.requires_target() {
                continue;
            }
            *seen.entry(kind).or_default() += 1;
            let mut slots = Vec::new();
            unbound_slots(&serde_json::to_value(body).expect("effect serializes"), kind, &mut slots);
            for (slot, owner) in slots {
                bad.push(format!("{} [{kind}] slot {slot} under {owner}", def.name));
            }
        }
    }
    // Floors, not exact counts: the catalog grows and these want to be
    // maintenance-free. Each is well under the current figure.
    for (kind, floor) in
        [("spell", 5000), ("activated", 2000), ("triggered", 5000), ("loyalty", 100),
         ("equip-bonus", 20), ("back-face", 100), ("adventure", 20)]
    {
        let n = seen.get(kind).copied().unwrap_or(0);
        assert!(n >= floor, "the walk reached only {n} `{kind}` bodies, expected >= {floor} — a \
                             holder stopped being visited and this invariant went vacuous");
    }
    bad.sort();
    bad.dedup();
    assert!(
        bad.is_empty(),
        "{} body/bodies declare a target slot that nothing binds — `requires_target()` is \
         false, so the cast/activate/trigger path never asks for it and the effect resolves \
         against an empty target list. Either the enclosing variant must auto-target at \
         resolution (add it to RESOLUTION_TIME_TARGETING and make it do so, as \
         ChooseUnchosenMode does) or `requires_target` must report the slot:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
}
