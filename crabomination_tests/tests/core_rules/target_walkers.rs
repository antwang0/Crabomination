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
//! It ran as a ratchet (164, then 39) until the sixty-third pass took it to
//! **0**; it is an invariant now — add the arm, do not add a threshold.

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

/// Effect variants whose body is **deliberately** invisible to the cast /
/// trigger-time walk, because it is auto-targeted fresh with its own slot
/// numbering when it resolves. Both are CR 603.7 "when you do" payoffs and
/// both say so in their own docs: `Reflexive` runs its body inline after the
/// gating cost is paid (`run_effect` calls
/// `auto_targets_for_effect_all_slots_sourced` on it), and `ReflexiveTrigger`
/// pushes its body onto the stack with targets picked at push time (CR
/// 603.7d, `auto_targets_for_effect_all_slots`). A slot under either is
/// answered, not lost — counting it inflated the ratchet below from 19 to 39
/// and hid how many real gaps are left.
const RESOLUTION_TIME_TARGETING: &[&str] = &["Reflexive", "ReflexiveTrigger"];

/// Every `slot` mentioned by a `Selector::TargetFiltered` in `v`, not
/// descending into nested ability definitions or resolution-time bodies.
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
                if RESOLUTION_TIME_TARGETING.contains(&k.as_str()) {
                    continue;
                }
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
    // **Closed.** This was a ratchet — 164, then 39 — for as long as it
    // existed; 20 of the last 39 were the walker being asked about
    // `Reflexive` / `ReflexiveTrigger` bodies it is deliberately blind to
    // (see `RESOLUTION_TIME_TARGETING`) and the other 19 were real. It is an
    // invariant now, not a budget: a new `Effect` variant that holds a
    // `TargetFiltered` without an arm in `target_filter_for_slot` fails here.
    // Do not reintroduce a threshold — add the arm.
    assert!(
        bad.is_empty(),
        "{} effect bodies declare a TargetFiltered slot that \
         `Effect::target_filter_for_slot` can't answer — the effect resolves \
         against an empty target list:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
}

/// The picker and the CR 608.2b checker aim at one slot, so they must not
/// disagree about it.
///
/// `primary_target_filter` (what the auto-picker aims with) and
/// `target_filter_for_slot(0)` (what CR 608.2b re-checks against at
/// resolution) were independent hand-written walks, and the seventy-fifth
/// pass's census found 65 definitions where both answered and the answers
/// differed. Most were honest — the fight family describes slot 1 with one
/// walker and slot 0 with the other, modal and kicker-branched bodies have a
/// slot 0 that depends on the branch — but two were not: `Feedback Bolt` and
/// `Reins of Power` target a **player** in slot 0, `primary_target_filter`
/// had no arm for that shape, and it fell through to a non-target subject
/// filter and aimed at a permanent the checker would then reject.
///
/// `primary_target_filter` defers to the checker now, so the two agree by
/// construction wherever the checker speaks. This asserts that, which is what
/// keeps the deferral from being "simplified" back out: a `sel_filter` arm
/// added above the deferral would reopen the class.
///
/// **A blanket ratchet is the thing this replaces.** The sixty-fifth pass
/// wrote one, watched it need 587 -> 83 -> 27 exceptions, and deleted it; the
/// invariant that holds with no exceptions is this one.
#[test]
fn primary_target_filter_defers_to_the_608_2b_checker() {
    let mut bad: Vec<String> = Vec::new();
    for factory in catalog::all_known_factories() {
        let def: CardDefinition = factory();
        let mut bodies: Vec<&crabomination::effect::Effect> = vec![&def.effect];
        for a in &def.activated_abilities {
            bodies.push(&a.effect);
        }
        for t in &def.triggered_abilities {
            bodies.push(&t.effect);
        }
        for l in &def.loyalty_abilities {
            bodies.push(&l.effect);
        }
        for body in bodies {
            let (Some(aim), Some(check)) =
                (body.primary_target_filter(), body.target_filter_for_slot(0))
            else {
                continue;
            };
            if aim != check {
                bad.push(format!("{}: aim {aim:?} vs check {check:?}", def.name));
            }
        }
    }
    bad.sort();
    bad.dedup();
    assert!(
        bad.is_empty(),
        "{} definitions aim at slot 0 with a filter CR 608.2b will not \
         re-check against:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
}

/// The two cards the census named, at the site the deferral actually changed.
///
/// `Feedback Bolt` deals damage equal to your artifact count **to a player**;
/// `primary_target_filter` returned the *artifact count's* filter, so
/// `enumerate_legal_targets` — the client's clickable-target list and the two
/// `bot.rs` fallback pickers — offered your own artifacts for a slot CR 608.2b
/// only accepts a player in. The cast path was never affected (it reads the
/// slot walker directly), which is why nothing failed and the bug sat there.
///
/// `Reins of Power` is the same shape one level in: its slot 0 is
/// `ControlledBy { who: Target(0) }`, and the picker returned the `Creature`
/// filter of the `Untap` clause that happens to be `Seq`'s first element.
#[test]
fn feedback_bolt_and_reins_of_power_offer_players_not_permanents() {
    use crabomination::game::types::Target;
    use crabomination::game::two_player_game;

    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let mine = g.add_card_to_battlefield(0, catalog::ornithopter());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    for (name, def) in
        [("Feedback Bolt", catalog::feedback_bolt()), ("Reins of Power", catalog::reins_of_power())]
    {
        let legal = g.enumerate_legal_targets(&def.effect, 0);
        assert!(
            legal.iter().any(|t| matches!(t, Target::Player(_))),
            "{name} targets a player in slot 0 and offered none: {legal:?}",
        );
        assert!(
            !legal.contains(&Target::Permanent(mine))
                && !legal.contains(&Target::Permanent(theirs)),
            "{name} offered a permanent for a player slot: {legal:?}",
        );
    }
}
