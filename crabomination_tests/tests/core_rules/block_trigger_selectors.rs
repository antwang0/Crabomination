//! Whole-catalog invariant: a *self-scoped* block trigger never reads
//! `Selector::TriggerSource`.
//!
//! `event_subject` binds `ctx.trigger_source` for a `BlockerDeclared` event to
//! the trigger's **own side of the pair** — the attacker under
//! `BecomesBlocked` / `BecomesBlockedByNOrMore`, the blocker under `Blocks` /
//! `BlocksNOrMore`. Under `EventScope::SelfSource` that side *is* the ability's
//! source, so `TriggerSource` can only ever mean "me" — while the card text
//! that mints these triggers ("whenever this creature blocks *a creature*, …")
//! means the partner. Writing it there is a silent self-reference: it bit
//! Dream Fighter, Crimson Roc and Catacomb Dragon, then Infernal Medusa,
//! Frostweb Spider, Tolarian Entrancer and Hedron Blade, and
//! `combat_partner_punisher` carries a hand-rolled workaround for the same
//! reason.
//!
//! The partner has its own selectors and they are the fix: `BlockedAttacker`
//! and `BlockingCreatures` while `block_map` is live (a trigger filter, which
//! is evaluated during declare-blockers), and
//! `CreaturesBlockedBySourceThisTurn` / `SelectionRequirement::
//! BlockedSourceThisTurn` in an `AtEndOfCombat` body, where `resolve_combat`
//! has already dropped `block_map`. A card that genuinely means itself writes
//! `This`.
//!
//! **Wider scopes are exempt and must stay so.** On `AnyPlayer` / `YourControl`
//! the watcher is a third object (Heat of Battle, Righteous Indignation), so
//! `TriggerSource` is the pair-side creature the text calls "that creature" —
//! correct and unreplaceable. Same for an `equipped_bonus` ability with
//! `triggers_on_equipment` set (Godsend): the trigger fires from the Equipment,
//! so the blocking bearer is the subject and `This` is the Equipment.
//!
//! This is an invariant, not a ratchet — rewrite the body onto the partner
//! selector, do not add a threshold.

use crabomination::card::{CardDefinition, TriggeredAbility};
use crabomination::catalog;
use crabomination::effect::{EventKind, EventScope};
use serde_json::Value;

/// The four `BlockerDeclared`-fed event kinds. `event_subject` binds all four
/// to the trigger's own side of the pair.
fn is_block_pair_event(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Blocks
            | EventKind::BecomesBlocked
            | EventKind::BlocksNOrMore(_)
            | EventKind::BecomesBlockedByNOrMore(_)
    )
}

/// A serialized nested ability definition (a granted trigger, a token's
/// ability, a Room half). It fires on *its* own event, so the enclosing
/// trigger's binding does not reach it.
fn is_nested_ability(map: &serde_json::Map<String, Value>) -> bool {
    map.contains_key("effect")
        && (map.contains_key("event") || map.contains_key("mana_cost") || map.contains_key("cost"))
}

/// Does `v` mention `Selector::TriggerSource` (a unit variant, so a bare
/// string) outside a nested ability body?
fn reads_trigger_source(v: &Value) -> bool {
    match v {
        Value::String(s) => s == "TriggerSource",
        Value::Object(map) => {
            if is_nested_ability(map) {
                return false;
            }
            map.values().any(reads_trigger_source)
        }
        Value::Array(items) => items.iter().any(reads_trigger_source),
        _ => false,
    }
}

/// The trigger's filter and its body — both are evaluated with the same
/// `trigger_source` binding, so both are in scope for this invariant.
fn offends(t: &TriggeredAbility) -> bool {
    if !is_block_pair_event(&t.event.kind) || t.event.scope != EventScope::SelfSource {
        return false;
    }
    let body = serde_json::to_value(&t.effect).expect("effect serializes");
    let filter = serde_json::to_value(&t.event.filter).expect("filter serializes");
    reads_trigger_source(&body) || reads_trigger_source(&filter)
}

#[test]
fn no_self_scoped_block_trigger_reads_trigger_source() {
    let mut bad: Vec<String> = Vec::new();
    for factory in catalog::all_known_factories() {
        let def: CardDefinition = factory();
        let mut abilities: Vec<&TriggeredAbility> = def.triggered_abilities.iter().collect();
        // An `equipped_bonus` ability is granted to the *host* unless it
        // triggers on the Equipment itself, in which case the bearer — not the
        // source — is the subject and `TriggerSource` is the only way to name
        // it.
        if let Some(bonus) = def.equipped_bonus.as_ref()
            && !bonus.triggers_on_equipment
        {
            abilities.extend(bonus.triggered_abilities.iter());
        }
        for t in abilities {
            if offends(t) {
                bad.push(format!("{} ({:?})", def.name, t.event.kind));
            }
        }
    }
    bad.sort();
    bad.dedup();
    assert!(
        bad.is_empty(),
        "{} self-scoped block trigger(s) read Selector::TriggerSource, which binds the \
         trigger's own source, not its combat partner — rewrite onto BlockedAttacker / \
         BlockingCreatures / CreaturesBlockedBySourceThisTurn / BlockedSourceThisTurn, or say \
         This:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
}
