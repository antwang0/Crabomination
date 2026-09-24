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
//!
//! ## The third side: a slot a body *reads*
//!
//! `no_body_reads_a_slot_the_fill_loop_never_reaches` closes the direction the
//! other two leave open. `target_walkers` proves every declared slot is
//! *answerable* and the test above proves a declared slot is *asked for*;
//! neither proves that a slot a body **reads** was ever declared. It is a
//! one-directional invariant, and the gap is silent rather than loud:
//! `resolve_selector` and `resolve_player` both read `ctx.targets.get(n)`, so
//! an undeclared slot is `None` — an empty selector and no player, no panic,
//! nothing in a log. Half a card does nothing and self-play never notices.
//!
//! It came back **0 over the catalog**, which is the result: the slot walker
//! and the effect bodies agree today. What it is worth is the ratchet, because
//! the walker it checks against (`eff_find` in `target_filter_for_slot_in_
//! mode_kicked`) is a hand-written match ending in `_ => None`, exactly the
//! shape whose missing arms this file was written about. A new effect that
//! holds a `PlayerRef` or `Selector` its arm does not walk goes quiet, and the
//! card that reads that slot goes quiet with it.

use crabomination::card::CardDefinition;
use crabomination::catalog;
use crabomination::effect::Effect;
use serde_json::Value;

/// `DEFERRED_FIRE` is deliberately **not** excused here: those bodies do get
/// their targets from a fire site rather than the cast loop, but none of them
/// reads a bare slot today, and leaving them in keeps the check strict. A
/// future finding under one of them points at its fire site, not at the cast.
///
/// Variants that stamp their own `EffectContext::targets` before running a
/// sub-body, so a `Target(n)` inside one is bound by *them* and says nothing
/// about the cast-time slots. Read out of `run_effect`'s arms: every arm that
/// assigns `targets` on a derived context or builds one with `targets: vec![…]`.
/// A new arm that does either belongs here, or `no_body_reads_a_slot_the_fill_
/// loop_never_reaches` will read its sub-body's slots as the outer body's.
const REBINDS_TARGETS: &[&str] = &[
    "AddCardTypeIndefinitely",
    "AddCounterOfPresentKind",
    "AddCountersOfChosenKind",
    "ApplyToTargets",
    "Blight",
    "CantAttackPlayerThisTurn",
    "ChooseUnchosenMode",
    "CollectEvidence",
    "CollectEvidenceX",
    "CopySpellForEachOtherTarget",
    "CreateToken",
    "DestroyTargetsPolymorph",
    "Earthbend",
    "ExileRandomGraveyardCopyTapped",
    "EyeOfTheStorm",
    "ForEachOpponentTarget",
    "Forage",
    "ImprintFromGraveyard",
    "LookAtHandCastFree",
    "MayCastExiledWithSource",
    "MayCastPermanentFromHandFree",
    "OathCatchUp",
    "OpponentChoosesTargetForDamage",
    "PlayersMayAccept",
    "PutIntoLibraryBeneathTop",
    "Reflexive",
    "ReflexiveTrigger",
    "RevealTopExileOnePerCardType",
    "SearchAndCastFree",
    "TurnFaceUpFree",
];

/// Variants that auto-target their own body when they resolve.
/// `ChooseUnchosenModeThisTurn` shares `ChooseUnchosenMode`'s resolution arm
/// (Parapet Thrasher).
const RESOLUTION_TIME_TARGETING: &[&str] =
    &["Reflexive", "ReflexiveTrigger", "ChooseUnchosenMode", "ChooseUnchosenModeThisTurn"];

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

/// Slot indices one body *reads* (`Target(n)`, `CastSpellTarget(n)`), not
/// descending through a variant that rebinds `targets` or through a nested
/// ability. Which slots are *declared* is the engine walker's question, not
/// this one's.
fn slots_read(v: &Value, read: &mut Vec<u64>) {
    match v {
        Value::Object(map) => {
            if is_nested_ability(map) {
                return;
            }
            for (k, inner) in map {
                if REBINDS_TARGETS.contains(&k.as_str()) || CAST_TIME_MODAL.contains(&k.as_str()) {
                    continue;
                }
                // `Selector::Target(n)` and `PlayerRef::Target(n)` are the only
                // two `Target(u8)` variants in the effect model, and both are a
                // slot index. `Target::Permanent`/`Target::Player` is a
                // different enum and serializes under those names, not this one.
                if (k == "Target" || k == "CastSpellTarget")
                    && let Some(n) = inner.as_u64()
                {
                    read.push(n);
                    continue;
                }
                slots_read(inner, read);
            }
        }
        Value::Array(items) => {
            for inner in items {
                slots_read(inner, read);
            }
        }
        _ => {}
    }
}

/// CR 601.2c — a spell's targets are chosen as it is cast, one per declared
/// slot. `auto_target_for_effect_all_slots` fills slot 0 (by its filter, or by
/// the source-aware heuristic when it has none) and then walks slots 1, 2, …
/// **breaking at the first slot with no `TargetFiltered` filter**. So a body
/// that reads `Target(n)` for an `n` past that break gets `ctx.targets.get(n)`
/// == `None`: `resolve_selector` yields an empty vec and `resolve_player`
/// yields nothing, and the clause silently does nothing. No panic — both read
/// sites use `.get()` — which is exactly why it survives self-play.
///
/// `core_rules::target_walkers` proves the other direction (every declared
/// slot is *answerable*) and `no_body_declares_a_slot_it_never_binds` proves
/// the `requires_target() == false` case. This is the third side: every slot a
/// body *reads* is one the fill loop actually reaches.
#[test]
fn no_body_reads_a_slot_the_fill_loop_never_reaches() {
    let mut bad: Vec<String> = Vec::new();
    let mut bodies_seen = 0usize;
    let mut reads_seen = 0usize;
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
        if let Some(back) = def.back_face.as_ref() {
            bodies.push(("back-face", &back.effect));
        }
        if let Some(adv) = def.adventure.as_ref() {
            bodies.push(("adventure", &adv.effect));
        }
        for (kind, body) in bodies {
            let mut read = Vec::new();
            slots_read(&serde_json::to_value(body).expect("effect serializes"), &mut read);
            if read.is_empty() {
                continue;
            }
            bodies_seen += 1;
            reads_seen += read.len();
            // Ask the engine's own walker, never a re-derivation of it: a slot
            // is declared by far more than a literal `TargetFiltered`, and the
            // first draft of this test read only those and called five correct
            // cards broken. `ControlledBy { who: Target(n) }`,
            // `Value::CreatureCountControlledBy(Target(n))` and every other
            // player-reading selector surface slot `n` as a `Player` filter
            // (`implicit_player_for_ref_slot`), which is exactly how How to
            // Start a Riot and Biomantic Mastery are already answerable.
            // Both kicker branches: a kicked-only slot (Rushing River's
            // "return **another** target nonland permanent", CR 702.33) is
            // declared when kicked and correctly absent when not, so asking
            // only the unkicked walk reads a legal card as broken.
            let declares = |n: u64| {
                body.target_filter_for_slot_in_mode_kicked(n as u8, None, false).is_some()
                    || body.target_filter_for_slot_in_mode_kicked(n as u8, None, true).is_some()
            };
            // Slot 0 is always supplied. Above it the loop stops at the first
            // gap, so only a contiguous run of filters from 1 upward is filled
            // — and it stops at 16 regardless, which this has to mirror or it
            // hangs: a handful of arms (the divided-damage and support shapes)
            // answer the same filter for *every* slot index, so an uncapped
            // walk up never terminates. The engine caps there for the same
            // reason; no real card declares more than four.
            const MAX_SLOTS: u64 = 16;
            let mut reachable = 0u64;
            while reachable + 1 < MAX_SLOTS && declares(reachable + 1) {
                reachable += 1;
            }
            read.sort_unstable();
            read.dedup();
            for n in read {
                if n > reachable {
                    bad.push(format!(
                        "{} [{kind}] reads Target({n}) but the fill loop stops after slot \
                         {reachable}",
                        def.name
                    ));
                }
            }
        }
    }
    assert!(
        bodies_seen >= 1500 && reads_seen >= 2000,
        "the walk reached only {bodies_seen} bodies / {reads_seen} slot reads — a holder \
         stopped being visited and this invariant went vacuous"
    );
    bad.sort();
    bad.dedup();
    assert!(
        bad.is_empty(),
        "{} body/bodies read a target slot the cast-time fill loop never reaches, so the \
         clause resolves against `None` and silently does nothing:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
}
