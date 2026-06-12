//! Champions of Kamigawa (CHK) — Splice onto Arcane (CR 702.47), plus the
//! Legends-era World permanents exercising the world rule (CR 704.5k).

use crate::card::{
    CardDefinition, CardType, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement,
    Selector, SpellSubtype, StaticAbility, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, StaticEffect};
use crate::mana::{cost, g, generic, r, u};

fn arcane() -> Subtypes {
    Subtypes { spell_subtypes: vec![SpellSubtype::Arcane], ..Default::default() }
}

/// Glacial Ray — {1}{R} Instant — Arcane. Deals 2 damage to any target.
/// Splice onto Arcane {1}{R}.
pub fn glacial_ray() -> CardDefinition {
    CardDefinition {
        name: "Glacial Ray",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        keywords: vec![Keyword::Splice(cost(&[generic(1), r()]), SpellSubtype::Arcane)],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Any),
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Reach Through Mists — {U} Instant — Arcane. Draw a card.
pub fn reach_through_mists() -> CardDefinition {
    CardDefinition {
        name: "Reach Through Mists",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        ..Default::default()
    }
}

/// Kodama's Might — {G} Instant — Arcane. Target creature gets +2/+2 until
/// end of turn. Splice onto Arcane {G}.
pub fn kodamas_might() -> CardDefinition {
    CardDefinition {
        name: "Kodama's Might",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        keywords: vec![Keyword::Splice(cost(&[g()]), SpellSubtype::Arcane)],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Concordant Crossroads — {G} World Enchantment. All creatures have haste.
pub fn concordant_crossroads() -> CardDefinition {
    CardDefinition {
        name: "Concordant Crossroads",
        cost: cost(&[g()]),
        supertypes: vec![Supertype::World],
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "All creatures have haste",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(SelectionRequirement::Creature),
                keyword: Keyword::Haste,
            },
        }],
        ..Default::default()
    }
}

/// Nether Void — {3}{B} World Enchantment. Whenever a player casts a spell,
/// counter it unless that player pays {3}.
pub fn nether_void() -> CardDefinition {
    use crate::mana::b;
    CardDefinition {
        name: "Nether Void",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![Supertype::World],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: Effect::CounterUnlessPaid {
                what: Selector::TriggerSource,
                mana_cost: cost(&[generic(3)]),
                exile: false,
                extra_generic: None,
            },
        }],
        ..Default::default()
    }
}

/// Eiganjo Castle — {T}: Add {W}. {2}{W}: Prevent the next 2 damage to a
/// legendary creature this turn. (Mana half only — the prevention targets a
/// legendary creature.)
pub fn eiganjo_castle() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::mana::w;
    CardDefinition {
        name: "Eiganjo Castle",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add(crate::mana::Color::White),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w()]),
                effect: Effect::PreventNextDamage {
                    target: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::HasSupertype(Supertype::Legendary)),
                    ),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
