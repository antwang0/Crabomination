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

/// The auto-picker's off-board fallback is gated the same way the
/// `ChooseCards` modal path is.
///
/// The filter language has no zone predicate, so `legal_targets_for_filter`
/// applies a board-shaped requirement to every graveyard and to exile as well
/// as to the battlefield. The modal path separates the two with
/// `SelectionRequirement::mentions_offboard_zone`; the picker's last-resort
/// walk had no gate at all, so a "destroy target creature" with no legal
/// battlefield creature aimed at a creature *card* in a graveyard or in exile
/// and then fizzled at resolution — in the training path, where no modal is
/// posed and nothing watched it. See ENGINE_BACKLOG, "the target enumerator
/// is zone-blind".
mod offboard_gate {
    use crabomination::card::{CardDefinition, CardType, SelectionRequirement as R};
    use crabomination::effect::{Effect, Selector};
    use crabomination::game::*;

    fn bear(name: &'static str) -> CardDefinition {
        CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            ..Default::default()
        }
    }

    /// A board-shaped filter never reaches a graveyard or exile, even when
    /// the board offers nothing and those zones do.
    #[test]
    fn board_shaped_filter_resolves_targetless_over_an_offboard_card() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, bear("Graveyard Bear"));
        g.add_card_to_exile(1, bear("Exiled Bear"));
        let destroy = Effect::Destroy {
            what: Selector::TargetFiltered { slot: 0, filter: R::Creature },
        };
        assert!(
            g.auto_target_for_effect(&destroy, 0).is_none(),
            "no battlefield creature means no target, not a graveyard cursor"
        );
        // The board is authoritative once it does offer one.
        let live = g.add_card_to_battlefield(0, bear("Live Bear"));
        assert_eq!(
            g.auto_target_for_effect(&destroy, 0),
            Some(Target::Permanent(live)),
            "the battlefield creature is picked"
        );
    }

    /// A filter that names the zone still reaches it.
    #[test]
    fn a_zone_naming_filter_still_reaches_the_graveyard() {
        let mut g = two_player_game();
        let gy = g.add_card_to_graveyard(0, bear("Graveyard Bear"));
        let exile_it = Effect::Exile {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::And(Box::new(R::Creature), Box::new(R::InGraveyard)),
            },
        };
        assert_eq!(
            g.auto_target_for_effect(&exile_it, 0),
            Some(Target::Permanent(gy)),
            "`InGraveyard` is the gate the modal path uses, and it opens this one"
        );
    }

    /// Reanimation keeps the walk without naming a zone: the classifier
    /// (`prefers_graveyard_target`) is the other half of the gate.
    #[test]
    fn reanimation_still_reaches_the_graveyard_without_a_zone_predicate() {
        let mut g = two_player_game();
        let gy = g.add_card_to_graveyard(1, bear("Their Bear"));
        let raise = Effect::Move {
            what: Selector::TargetFiltered { slot: 0, filter: R::Creature },
            to: crabomination::effect::ZoneDest::Hand(crabomination::effect::PlayerRef::You),
        };
        assert_eq!(
            g.auto_target_for_effect(&raise, 0),
            Some(Target::Permanent(gy)),
            "a Move-to-your-hand is reanimation-shaped and keeps the walk"
        );
    }

    /// The **enumerator** is scoped the same way the picker is.
    ///
    /// `legal_targets_for_filter` walked every graveyard and exile for any
    /// filter, so `enumerate_legal_targets` — the client's clickable list and
    /// the engine's own `ChooseTarget` sets — offered candidates the resolver
    /// rejects. The picker was gated at the eighty-sixth pass and the
    /// enumerator was not, so the UI path and the training path targeted
    /// different sets for the same effect. See ENGINE_BACKLOG, "the target
    /// enumerator is zone-blind".
    #[test]
    fn the_enumerator_and_the_picker_agree_on_which_zones_an_effect_reaches() {
        let mut g = two_player_game();
        let gy = g.add_card_to_graveyard(0, bear("Graveyard Bear"));
        let ex = g.add_card_to_exile(1, bear("Exiled Bear"));
        let live = g.add_card_to_battlefield(0, bear("Live Bear"));

        let destroy = Effect::Destroy {
            what: Selector::TargetFiltered { slot: 0, filter: R::Creature },
        };
        let legal = g.enumerate_legal_targets(&destroy, 0);
        assert!(
            legal.contains(&Target::Permanent(live)),
            "the battlefield creature is still offered: {legal:?}"
        );
        assert!(
            !legal.contains(&Target::Permanent(gy)) && !legal.contains(&Target::Permanent(ex)),
            "a board-shaped destroy must not offer off-board cards: {legal:?}"
        );

        // The reanimation shape keeps both halves, exactly as the picker does.
        let raise = Effect::Move {
            what: Selector::TargetFiltered { slot: 0, filter: R::Creature },
            to: crabomination::effect::ZoneDest::Hand(crabomination::effect::PlayerRef::You),
        };
        let legal = g.enumerate_legal_targets(&raise, 0);
        assert!(
            legal.contains(&Target::Permanent(gy)),
            "a Move-to-your-hand reaches the graveyard: {legal:?}"
        );
    }

    /// `SelectionRequirement::Any` is the widest board-shaped filter there is,
    /// and it used to match every card in every graveyard and in exile.
    /// Cuombajj Witches ("1 damage to any target, chosen by an opponent") is
    /// the shipped card that poses it.
    #[test]
    fn an_any_filter_offers_the_board_and_the_players_only() {
        let mut g = two_player_game();
        let gy = g.add_card_to_graveyard(0, bear("Graveyard Bear"));
        let ex = g.add_card_to_exile(1, bear("Exiled Bear"));
        let live = g.add_card_to_battlefield(1, bear("Live Bear"));
        let legal = g.legal_targets_for_filter(&R::Any, true, 0, None);
        assert!(
            legal.contains(&Target::Player(0)) && legal.contains(&Target::Player(1)),
            "both players are any-targets: {legal:?}"
        );
        assert!(legal.contains(&Target::Permanent(live)), "so is the board: {legal:?}");
        assert!(
            !legal.contains(&Target::Permanent(gy)) && !legal.contains(&Target::Permanent(ex)),
            "a card in a graveyard or in exile is not an `any target`: {legal:?}"
        );
    }
}

/// The whole-catalog twin of [`every_declared_target_slot_is_answerable`], for
/// the off-board gate rather than the slot walker.
///
/// `Effect::prefers_graveyard_target` and `Effect::may_target_offboard_card`
/// are two more hand-written walks over the same ~2 000 variants, and unlike
/// `requires_target` they end in `_ => false`. So a wrapper neither of them
/// names answers `false` for its whole subtree, and the auto-picker's
/// last-resort graveyard/exile walk is then closed for an effect that really
/// does reanimate — which resolves targetless in the training path, silently,
/// which is the defect the gate was written for seen from the other side.
/// `scripts/audit_target_walkers.py` reports the whole (walker, wrapper)
/// matrix statically; this asserts the half a catalog card can reach.
///
/// **The shape has to be the reanimation one, not "holds a `Move`".** A
/// blanket version of this test reports 29 bodies and every one of them is
/// correct to answer `false`: Feign Death, Malakir Rebirth and Saffi
/// Eriksdotter target a *live* creature and move it after it dies; Crystal
/// Shard and Erratic Portal bounce a battlefield permanent. Opening the gate
/// for those would aim them at a graveyard card that then fizzles — the
/// original defect, re-created. What the gate is for is a `Move` **of the
/// target itself** to *your* hand or battlefield, which only a card already
/// off the board can satisfy.
///
/// Two exclusions, both load-bearing:
///
/// * **The moved thing has to be the target.** Intrude on the Mind moves a
///   `SeparatedPile` to your hand and targets the *player* who splits it.
/// * **`ZoneDest::Exile` is not a tell.** 112 catalog sites move a target to
///   exile and most of them are battlefield removal (Excise's "exile target
///   attacking creature"); `Effect::Exile` is the canonical form for that.
///   `prefers_graveyard_target` reads a Move-to-Exile as reanimation anyway,
///   as a walk-*order* heuristic — which is why Ugin, Eye of the Storms had
///   to become an `Effect::Exile` when the `OptionalTargets` arm below
///   stopped hiding it.
#[test]
fn every_reachable_reanimation_is_visible_to_the_offboard_gate() {
    /// A destination that makes the *target* an off-board card: your hand or
    /// your battlefield. See the exclusions above for why exile is not one.
    fn is_reanimating_dest(dest: &Value) -> bool {
        match dest {
            Value::Object(m) => match (m.get("Hand"), m.get("Battlefield")) {
                (Some(Value::String(who)), _) => who == "You",
                (_, Some(Value::Object(b))) => {
                    b.get("controller").and_then(|c| c.as_str()) == Some("You")
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Does this selector name a target slot at its own root?
    fn is_target_selector(what: &Value) -> bool {
        matches!(what, Value::Object(m) if m.contains_key("Target")
            || m.contains_key("TargetFiltered"))
    }

    /// Is there such a `Move` under `v`, not descending into a nested ability
    /// definition or a resolution-time body?
    fn holds_reanimation(v: &Value) -> bool {
        match v {
            Value::Object(map) => {
                if is_nested_ability(map) {
                    return false;
                }
                map.iter().any(|(k, inner)| {
                    if RESOLUTION_TIME_TARGETING.contains(&k.as_str()) {
                        return false;
                    }
                    if k == "Move"
                        && inner.get("to").is_some_and(is_reanimating_dest)
                        && inner.get("what").is_some_and(is_target_selector)
                    {
                        return true;
                    }
                    holds_reanimation(inner)
                })
            }
            Value::Array(items) => items.iter().any(holds_reanimation),
            _ => false,
        }
    }

    let mut bad: Vec<String> = Vec::new();
    for factory in catalog::all_known_factories() {
        let def: CardDefinition = factory();
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
            if !body.requires_target() || body.may_target_offboard_card() {
                continue;
            }
            // A filter that names the zone opens the walk on its own
            // (`mentions_offboard_zone`), so the gate is not what decides it.
            if body
                .primary_target_filter()
                .is_some_and(|f| f.mentions_offboard_zone())
            {
                continue;
            }
            let json = serde_json::to_value(body).expect("Effect serializes");
            if !holds_reanimation(&json) {
                continue;
            }
            let root = match &json {
                Value::Object(m) => m.keys().next().cloned().unwrap_or_default(),
                _ => json.to_string(),
            };
            bad.push(format!("Effect::{root} — {} ({kind})", def.name));
        }
    }
    bad.sort();
    bad.dedup();
    assert!(
        bad.is_empty(),
        "{} targeting bodies reanimate through a wrapper the off-board gate \
         cannot see, so the auto-picker never reaches the graveyard card they \
         are about:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
}

/// The two cards the invariant above named, at the site the arm changed.
///
/// Reap and Rise from the Wreck both read "return … from your graveyard to
/// your hand", and both express it the way the catalog always has — a
/// `Move { to: Hand(You) }`, whose zone is carried by the effect shape rather
/// than by a filter predicate (Raise Dead and Regrowth are the same). Wrapped
/// in `OptionalTargets` for the "up to X" clause, the shape was invisible to
/// `prefers_graveyard_target`, so the picker walked the battlefield first,
/// found a permanent that satisfies a zone-blind filter, and bounced it.
#[test]
fn up_to_x_graveyard_returns_aim_at_the_graveyard() {
    use crabomination::game::two_player_game;
    use crabomination::game::types::Target;

    for (name, def) in
        [("Reap", catalog::reap()), ("Rise from the Wreck", catalog::rise_from_the_wreck())]
    {
        let mut g = two_player_game();
        // A battlefield permanent the zone-blind filter also matches, so the
        // walk *order* is what decides — not the absence of a board target.
        let onboard = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let buried = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        // Reap's slot 0 is "target opponent"; its graveyard slots are 1-4, so
        // read every slot rather than just the primary one.
        let (primary, extra) = g.auto_targets_for_effect_all_slots(&def.effect, 0, None);
        let picked: Vec<Target> = primary.into_iter().chain(extra).collect();
        assert!(
            picked.contains(&Target::Permanent(buried)),
            "{name} returns from your graveyard and aimed at {picked:?} instead \
             (board card {onboard:?})",
        );
        assert!(
            !picked.contains(&Target::Permanent(onboard)),
            "{name} bounced a battlefield permanent: {picked:?}",
        );
    }
}

/// No catalog body that names a target **player** may land on one of
/// `accepts_player_target`'s explicit `=> false` arms.
///
/// `legal_targets_for_filter` offers Player candidates only when
/// `accepts_player_target()` is true (`targeting.rs`, "Skip Player candidates
/// entirely when the effect operates on permanents/stack"), so a body that
/// names a target player and is refused never gets that slot filled.
///
/// **This walker's fallthrough is `_ => true`, not `_ => false`, and that
/// makes it the one exception in the family.** The other three restrict on
/// the fallback (`prefers_graveyard_target` and `may_target_offboard_card`
/// answer `false`, `primary_target_filter` answers `None`), so for them an
/// unnamed wrapper silently closes the gate for its whole subtree — the drift
/// `ENGINE_BACKLOG`'s "the gate's own wrappers" is about. Here the unnamed
/// case is *permitted*: `accepts_player_target`'s own comment calls it a
/// conservative default and points out the legality gate still rejects a
/// mismatch. So this test is not about the 101 unnamed wrappers at all — it
/// cannot be, and a version of it that claimed to be would be vacuous. It is
/// about the ~30 arms that say `false` on purpose (the `CounterSpell` family,
/// `SupportCounters`, `DistributeCounters`, `Fight`): those are the ones that
/// can refuse a player, and a card routing a player target through one is the
/// bug this catches.
///
/// **Narrow on purpose.** Only `Selector::Player(PlayerRef::Target(n))` is
/// checked, because it is the one player-target form that cannot be confused
/// with a permanent one in the serialized tree: both `Selector::Target(n)` and
/// `PlayerRef::Target(n)` are a bare `{"Target": n}`, and `PlayerRef` sits in
/// 65 distinct JSON positions. Reading `{"Player": {"Target": n}}` needs no
/// list of those and cannot go stale as variants are added. It therefore
/// **under**-reports — a `who: PlayerRef::Target` field under an unnamed
/// wrapper is not caught — and never false-reports, which is the direction an
/// invariant has to err in. (The effects that carry a bare `who:
/// PlayerRef::Target` — `Search`, `SearchPickedBy` — are already named.)
///
/// A blanket version of this test does not work, which is why it is this
/// shape: see the note in `ENGINE_BACKLOG.md` about the "holds a `Move`"
/// version reporting 29 bodies that are all right to answer `false`.
///
/// **Population 295 and 0 findings** — no shipped card routes a target player
/// through a refusing arm, so this is an invariant rather than a ratchet from
/// the day it lands. The floor below is what keeps it from going quietly
/// vacuous.
#[test]
fn every_reachable_target_player_is_visible_to_the_player_gate() {
    /// Is there a `Selector::Player(PlayerRef::Target(_))` under `v`, not
    /// descending into a nested ability definition or a resolution-time body?
    fn holds_target_player(v: &Value) -> bool {
        match v {
            Value::Object(map) => {
                if is_nested_ability(map) {
                    return false;
                }
                map.iter().any(|(k, inner)| {
                    if RESOLUTION_TIME_TARGETING.contains(&k.as_str()) {
                        return false;
                    }
                    if k == "Player"
                        && matches!(inner, Value::Object(p) if p.contains_key("Target"))
                    {
                        return true;
                    }
                    holds_target_player(inner)
                })
            }
            Value::Array(items) => items.iter().any(holds_target_player),
            _ => false,
        }
    }

    let mut bad: Vec<String> = Vec::new();
    // The population the invariant is over. A test that reports nothing
    // because it *looks at* nothing is the failure mode an empty ratchet
    // hides, so the floor is asserted with the finding list.
    let mut covered = 0usize;
    for factory in catalog::all_known_factories() {
        let def: CardDefinition = factory();
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
            if !body.requires_target() {
                continue;
            }
            let json = serde_json::to_value(body).expect("Effect serializes");
            if !holds_target_player(&json) {
                continue;
            }
            covered += 1;
            if body.accepts_player_target() {
                continue;
            }
            let root = match &json {
                Value::Object(m) => m.keys().next().cloned().unwrap_or_default(),
                _ => json.to_string(),
            };
            bad.push(format!("Effect::{root} — {} ({kind})", def.name));
        }
    }
    bad.sort();
    bad.dedup();
    assert!(
        bad.is_empty(),
        "{} targeting bodies name a target PLAYER through a wrapper \
         `accepts_player_target` cannot see, so the auto-picker never offers a \
         player for the slot they are about:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
    assert!(
        covered >= 250,
        "the player-gate invariant is looking at only {covered} bodies — it \
         has gone vacuous (a serde rename, or the shape moved). It was 295 \
         when written; re-derive the shape before lowering this."
    );
}

/// The third walker, and this one needs no tree walk: **`primary_target_filter`
/// and `target_filter_for_slot(0)` are two answers to the same question and
/// have to agree.**
///
/// `auto_targets_for_effect` picks slot 0 with `primary_target_filter()` and
/// falls back to `SelectionRequirement::Any` when it is `None`
/// (`legal_targets_for_effect`, `targeting.rs`). `target_filter_for_slot(0)`
/// is the other walk over the same tree and is the one
/// `every_declared_target_slot_is_answerable` above holds to zero. So a body
/// where the slot walker finds a filter and the primary walker does not picks
/// slot 0 against `Any` — the auto-picker offers a target the card's own
/// restriction forbids, and the engine rejects it at resolution.
///
/// This is the narrow shape `ENGINE_BACKLOG` asks for on this walker, and it
/// is sharper than a wrapper census: it compares the walker against another
/// walk of the same tree rather than against a guess at what the tree means,
/// so it cannot false-report the way a blanket "holds a `Move`" test does.
///
/// **Population 7,728 and 0 findings**, which is what makes it an invariant
/// on the day it lands rather than a ratchet.
#[test]
fn the_primary_target_filter_agrees_with_the_slot_walker_on_slot_zero() {
    let mut bad: Vec<String> = Vec::new();
    let mut covered = 0usize;
    for factory in catalog::all_known_factories() {
        let def: CardDefinition = factory();
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
            let Some(slot0) = body.target_filter_for_slot(0) else { continue };
            covered += 1;
            if body.primary_target_filter().is_some() {
                continue;
            }
            bad.push(format!("{} ({kind}) — slot 0 is {slot0:?}", def.name));
        }
    }
    bad.sort();
    bad.dedup();
    assert!(
        bad.is_empty(),
        "{} bodies declare a slot-0 target filter that `primary_target_filter` \
         cannot see, so the auto-picker chooses slot 0 against `Any` and offers \
         targets the card forbids:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
    assert!(
        covered >= 7_000,
        "the primary/slot agreement invariant is looking at only {covered} \
         bodies — it has gone vacuous. It was 7,728 when written."
    );
}

/// `Effect::for_each_inner` names every `Effect` wrapper — 130 of 130.
///
/// The five walkers' whole failure mode is that four of them end in `_ => …`,
/// so a wrapper they do not name answers the fallback for its entire subtree.
/// `for_each_inner` is the shared recursion they are meant to fall through
/// to, which only works if *it* is complete. This reads the enum and holds it
/// there.
///
/// The extraction is the one `scripts/audit_target_walkers.py` uses — a
/// wrapper is an `Effect` variant with an `Effect` somewhere in its fields —
/// so the test and the audit cannot disagree about what a wrapper is. The
/// population is asserted with the finding list: a parse that silently stops
/// finding variants would otherwise pass by looking at nothing.
#[test]
fn the_shared_recursion_names_every_effect_wrapper() {
    let effect_rs = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../crabomination_base/src/effect.rs"
    );
    let query_rs = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../crabomination_base/src/effect/query.rs"
    );
    let enum_src = std::fs::read_to_string(effect_rs).expect("effect.rs");
    let query_src = std::fs::read_to_string(query_rs).expect("query.rs");

    // The `{ … }` of `pub enum Effect`, brace-matched.
    let start = enum_src.find("\npub enum Effect {").expect("enum Effect");
    let open = enum_src[start..].find('{').expect("brace") + start;
    let mut depth = 0usize;
    let mut end = open;
    for (i, ch) in enum_src[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = open + i;
                    break;
                }
            }
            _ => {}
        }
    }
    // Strip doc comments and attributes BEFORE splitting: they carry
    // unbalanced brackets of their own, and counting those is what makes a
    // depth-based split find 10 variants instead of 130. Angle brackets are
    // deliberately not counted either — `scripts/audit_target_walkers.py`
    // splits on `{([` / `})]` only, and the two have to agree.
    let stripped: String = enum_src[open + 1..end]
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            !(t.starts_with("///") || t.starts_with("//!") || t.starts_with("//")
                || t.starts_with("#["))
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut variants: Vec<String> = Vec::new();
    let (mut d, mut cur) = (0i32, String::new());
    for ch in stripped.chars() {
        match ch {
            '{' | '(' | '[' => d += 1,
            '}' | ')' | ']' => d -= 1,
            _ => {}
        }
        cur.push(ch);
        if ch == ',' && d == 0 {
            variants.push(std::mem::take(&mut cur));
        }
    }
    if !cur.trim().is_empty() {
        variants.push(cur);
    }

    // The function's own `{ … }`, brace-matched. Taking the rest of the file
    // instead makes the test vacuous: the other four walkers name most of
    // these variants too, so every lookup below would succeed no matter what
    // `for_each_inner` contains. (Found by deleting an arm and watching this
    // test pass.)
    let fn_at = query_src.find("pub fn for_each_inner").expect("for_each_inner");
    let fn_open = query_src[fn_at..].find('{').expect("brace") + fn_at;
    let mut fd = 0usize;
    let mut fn_end = fn_open;
    for (i, ch) in query_src[fn_open..].char_indices() {
        match ch {
            '{' => fd += 1,
            '}' => {
                fd -= 1;
                if fd == 0 {
                    fn_end = fn_open + i;
                    break;
                }
            }
            _ => {}
        }
    }
    let body = &query_src[fn_open..=fn_end];
    assert!(
        body.len() > 2_000,
        "`for_each_inner`'s body brace-matched to {} bytes — the extraction \
         is wrong, not the function",
        body.len()
    );

    let mut wrappers = 0usize;
    let mut missing: Vec<String> = Vec::new();
    for v in &variants {
        let clean = v.as_str();
        if !clean.contains("Effect") {
            continue;
        }
        let Some(name) = clean
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .find(|w| w.chars().next().is_some_and(char::is_uppercase))
        else {
            continue;
        };
        wrappers += 1;
        if !body.contains(&format!("Effect::{name} ")) && !body.contains(&format!("Effect::{name}("))
        {
            missing.push(name.to_string());
        }
    }
    missing.sort();
    missing.dedup();
    assert!(
        missing.is_empty(),
        "{} of {wrappers} `Effect` wrappers are missing from `for_each_inner`, \
         so every walker that falls through to it answers the fallback for \
         their whole subtree:\n  {}",
        missing.len(),
        missing.join("\n  ")
    );
    assert!(
        wrappers >= 120,
        "the wrapper extraction found only {wrappers} variants — it has gone \
         vacuous (the enum moved, or a `///` shape changed). It was 130 when \
         written."
    );
}
