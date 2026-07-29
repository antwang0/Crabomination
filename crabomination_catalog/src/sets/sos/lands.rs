//! Secrets of Strixhaven (SOS) — Lands.
//!
//! The SOS school lands all share the same template: enters tapped, taps
//! for one of two colors, and has a `{2}{C1}{C2}, {T}: Surveil 1`
//! activated ability gated by tap. The engine has the surveil primitive
//! already (see `Effect::Surveil`), so wiring these is straightforward.

use crate::card::{CardDefinition, CardType, Effect, StaticAbility};
use crate::effect::{ActivatedAbility, PlayerRef, Selector, StaticEffect, Value};
use crate::mana::{Color, ManaCost, ManaSymbol, b, cost, g, generic, r, u, w};

/// Build a Strixhaven school land — enters tapped (a true CR 614.13
/// replacement, not an ETB trigger, so it can never tap for mana in the
/// window before a trigger would resolve), two color-pip mana abilities,
/// and a `{2}{c1}{c2}, {T}: Surveil 1` ability. No basic land types —
/// the template is a plain dual, not a typed one.
fn school_land(
    name: &'static str,
    color_a: Color,
    color_b: Color,
    surveil_pips: [ManaSymbol; 2],
) -> CardDefinition {
    use super::super::tap_add;
    let surveil = ActivatedAbility {
        energy_cost: 0,
        discard_cost: None,
        tap_cost: true,
        mana_cost: cost(&[generic(2), surveil_pips[0], surveil_pips[1]]),
        effect: Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(1),
        },
        once_per_turn: false,
        sorcery_speed: false,
        sac_cost: false,
        condition: None,
        life_cost: 0,
        from_graveyard: false,
        exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        ..Default::default()
    };
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add(color_a), tap_add(color_b), surveil],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        ..Default::default()
    }
}

/// Forum of Amity — Silverquill (W/B) school land.
pub fn forum_of_amity() -> CardDefinition {
    school_land("Forum of Amity", Color::White, Color::Black, [w(), b()])
}

/// Fields of Strife — Lorehold (R/W) school land.
pub fn fields_of_strife() -> CardDefinition {
    school_land("Fields of Strife", Color::Red, Color::White, [r(), w()])
}

/// Paradox Gardens — Quandrix (G/U) school land.
pub fn paradox_gardens() -> CardDefinition {
    school_land("Paradox Gardens", Color::Green, Color::Blue, [g(), u()])
}

/// Titan's Grave — Witherbloom (B/G) school land.
pub fn titans_grave() -> CardDefinition {
    school_land("Titan's Grave", Color::Black, Color::Green, [b(), g()])
}

/// Spectacle Summit — Prismari (U/R) school land.
pub fn spectacle_summit() -> CardDefinition {
    school_land("Spectacle Summit", Color::Blue, Color::Red, [u(), r()])
}

/// Great Hall of the Biblioplex — colorless legendary utility land.
///
/// Real Oracle: "{T}: Add {C}. / {T}, Pay 1 life: Add one mana of any
/// color. Spend this mana only to cast an instant or sorcery spell. /
/// {5}: If this land isn't a creature, it becomes a 2/4 Wizard creature
/// with 'Whenever you cast an instant or sorcery spell, this creature
/// gets +1/+0 until end of turn.' It's still a land."
///
/// Wired (push XV):
/// - `{T}: Add {C}` via the shared `tap_add_colorless` helper.
/// - `{T}, Pay 1 life: Add one mana of any color. Spend this mana only to
///   cast an instant or sorcery spell.` The any-color mana is produced via
///   `ManaPayload::Restricted(AnyOneColor, InstantSorceryOnly)`, so it can
///   only fund I/S spells (enforced by `ManaPool::pay_for_spell`). The
///   life cost uses the `ActivatedAbility::life_cost` slot.
/// - `{5}: If this land isn't a creature, it becomes a 2/4 Wizard
///   creature with "Whenever you cast an instant or sorcery spell, this
///   creature gets +1/+0 until end of turn." It's still a land.` — a
///   PERMANENT animation via `Effect::BecomeCreature { duration:
///   Permanent }` (it keeps the Land type), the magecraft pump granted
///   permanently via `GrantTriggeredAbility`, and the "isn't a creature"
///   gate as the ability's activation condition.
pub fn great_hall_of_the_biblioplex() -> CardDefinition {
    use super::super::tap_add_colorless;
    use crate::effect::{ActivatedAbility, ManaPayload};
    use crate::mana::SpendRestriction;
    // Pure mana ability (`AddMana` only) → resolves immediately without
    // going on the stack. Life is paid up front; per CR 119.4 paying your
    // last life point is legal (pre-flight only rejects `life < cost`),
    // and the SBA loss follows.
    let pay_life_for_any = ActivatedAbility {
        energy_cost: 0,
        discard_cost: None,
        tap_cost: true,
        mana_cost: ManaCost::default(),
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Restricted(
                Box::new(ManaPayload::AnyOneColor(Value::Const(1))),
                SpendRestriction::InstantSorceryOnly,
            ),
        },
        once_per_turn: false,
        sorcery_speed: false,
        sac_cost: false,
        condition: None,
        life_cost: 1,
        from_graveyard: false,
        exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        ..Default::default()
    };
    // {5}: If this land isn't a creature, it becomes a 2/4 Wizard
    // creature with the magecraft pump. It's still a land. The printed
    // text has no "until end of turn" — the animation is permanent.
    let animate = ActivatedAbility {
        mana_cost: cost(&[crate::mana::generic(5)]),
        condition: Some(crate::effect::Predicate::EntityMatches {
            what: crate::effect::Selector::This,
            filter: crate::card::SelectionRequirement::Not(Box::new(
                crate::card::SelectionRequirement::Creature,
            )),
        }),
        effect: Effect::Seq(vec![
            Effect::BecomeCreature {
                what: crate::effect::Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(4),
                creature_types: vec![crate::card::CreatureType::Wizard],
                keywords: vec![],
                duration: crate::effect::Duration::Permanent,
            },
            Effect::GrantTriggeredAbility {
                what: crate::effect::Selector::This,
                trigger: Box::new(crate::effect::shortcut::magecraft(Effect::PumpPT {
                    what: crate::effect::Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: crate::effect::Duration::EndOfTurn,
                })),
                duration: crate::effect::Duration::Permanent,
            },
        ]),
        ..Default::default()
    };
    CardDefinition {
        // Printed type line is a plain "Land" — NOT legendary (audit fix).
        name: "Great Hall of the Biblioplex",
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add_colorless(), pay_life_for_any, animate],
        ..Default::default()
    }
}

/// Skycoach Waypoint — Land.
/// Real Oracle: "{T}: Add {C}. / {3}, {T}: Target creature becomes
/// prepared. (Only creatures with prepare spells can become prepared.)"
///
/// ✅ Both abilities wired. `{T}: Add {C}` via the shared
/// `tap_add_colorless` helper. `{3}, {T}: Target creature becomes
/// prepared` via `AddCounter` of `CounterType::Prepared`. The
/// "(only creatures with prepare spells can become prepared)"
/// reminder is enforced by the target filter:
/// `SelectionRequirement::HasPrepareSpell` only matches creatures
/// whose definition carries an inset prepare spell.
pub fn skycoach_waypoint() -> CardDefinition {
    use super::super::tap_add_colorless;
    use crate::card::{CounterType, SelectionRequirement};
    use crate::effect::shortcut::target_filtered;
    use crate::effect::ActivatedAbility;
    // Printed reminder: "(Only creatures with prepare spells can
    // become prepared.)" — restrict target to creatures whose
    // definition carries an inset prepare spell (`prepare_spell`).
    let prepare_target = ActivatedAbility {
        energy_cost: 0,
        discard_cost: None,
        tap_cost: true,
        mana_cost: cost(&[generic(3)]),
        effect: Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::HasPrepareSpell),
            ),
            kind: CounterType::Prepared,
            amount: Value::Const(1),
        },
        once_per_turn: false,
        sorcery_speed: false,
        sac_cost: false,
        condition: None,
        life_cost: 0,
        from_graveyard: false,
        exile_self_cost: false,
        exile_other_filter: None,
        self_counter_cost_reduction: None, sac_other_filter: None,
        tap_other_filter: None, from_hand: false,
        ..Default::default()
    };
    CardDefinition {
        name: "Skycoach Waypoint",
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add_colorless(), prepare_target],
        ..Default::default()
    }
}

/// Petrified Hamlet — Land.
/// Real Oracle: "When this land enters, choose a land card name. /
/// Activated abilities of sources with the chosen name can't be activated
/// unless they're mana abilities. / Lands with the chosen name have
/// '{T}: Add {C}.' / {T}: Add {C}."
///
/// All four printed abilities wired:
/// - ETB `Effect::NameCard` (the Pithing Needle prompt/heuristic) stamps
///   the chosen name.
/// - The lock-out ("activated abilities of sources with the chosen name
///   can't be activated unless they're mana abilities") is the engine's
///   global `named_card` suppression — the same CR 201.3 rail Pithing
///   Needle rides.
/// - "Lands with the chosen name have '{T}: Add {C}'" via
///   `GrantActivatedAbility { EachPermanent(Land ∧ NamedBySource) }`.
/// - The printed `{T}: Add {C}` via the shared `tap_add_colorless`.
/// Residual: the NameCard decision offers any card name (the printed
/// text restricts the choice to LAND names — the UI doesn't yet filter
/// the namespace, though naming a nonland simply makes the two
/// name-keyed abilities dead).
pub fn petrified_hamlet() -> CardDefinition {
    use super::super::tap_add_colorless;
    use crate::card::{SelectionRequirement, TriggeredAbility};
    use crate::effect::{
        EventKind, EventScope, EventSpec, Selector, StaticAbility, StaticEffect,
    };
    CardDefinition {
        name: "Petrified Hamlet",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::NameCard { what: Selector::This },
        }],
        static_abilities: vec![StaticAbility {
            description: "Lands with the chosen name have \"{T}: Add {C}.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Land.and(SelectionRequirement::NamedBySource),
                ),
                ability: tap_add_colorless(),
                condition: None,
            },
        }],
        activated_abilities: vec![tap_add_colorless()],
        ..Default::default()
    }
}
