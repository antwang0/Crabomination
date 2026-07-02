//! Izzet spell-copy artifacts/enchantments & payoffs (batch 3). All ride
//! existing primitives (`CopySpellMayChooseTargets`, `MayPay`,
//! `CastFromHandWithoutPaying`, `DiscardHandDrawThatMany`). Tests in
//! `tests/recent92.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, Effect,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{cast_is_instant_or_sorcery, draw, target_any, you};
use crate::effect::{ManaPayload, PlayerRef};
use crate::mana::{cost, generic, r, u, x};

/// "Whenever you cast an instant or sorcery spell, [effect]" trigger.
fn on_cast_is(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(cast_is_instant_or_sorcery()),
        effect,
    }
}

/// Copy the just-cast spell `count` times, each choosing new targets.
fn copy_cast_spell(count: Value) -> Effect {
    Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count }
}

/// Firemind Vessel — {4} Artifact. Enters tapped. {T}: add two mana of any
/// colors. (Printed "two mana of different colors" — the different-colors
/// constraint is not enforced.)
pub fn firemind_vessel() -> CardDefinition {
    CardDefinition {
        name: "Firemind Vessel",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "This artifact enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::Const(2)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Thousand-Year Storm — {4}{U}{R} Enchantment. Cast an I/S → copy it for each
/// other spell you've cast before it this turn. (Uses the all-spell storm
/// count, not I/S-only.)
pub fn thousand_year_storm() -> CardDefinition {
    CardDefinition {
        name: "Thousand-Year Storm",
        cost: cost(&[generic(4), u(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![on_cast_is(copy_cast_spell(Value::StormCount))],
        ..Default::default()
    }
}

/// Swarm Intelligence — {6}{U} Enchantment. Cast an I/S → copy that spell.
pub fn swarm_intelligence() -> CardDefinition {
    CardDefinition {
        name: "Swarm Intelligence",
        cost: cost(&[generic(6), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![on_cast_is(copy_cast_spell(Value::Const(1)))],
        ..Default::default()
    }
}

/// Mirari — {5} Legendary Artifact. Cast an I/S → you may pay {3}; if you do,
/// copy that spell.
pub fn mirari() -> CardDefinition {
    CardDefinition {
        name: "Mirari",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![on_cast_is(Effect::MayPay {
            description: "Pay {3}: copy that spell.".into(),
            mana_cost: cost(&[generic(3)]),
            body: Box::new(copy_cast_spell(Value::Const(1))),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Niv-Mizzet, Dracogenius — {2}{U}{U}{R}{R} 5/5 Dragon Wizard, flying. Deals
/// damage → you may draw. {U}{R}: deal 1 to any target. (The draw trigger fires
/// on any damage Niv deals, approximating "to a player".)
pub fn nivmizzet_dracogenius() -> CardDefinition {
    CardDefinition {
        name: "Niv-Mizzet, Dracogenius",
        cost: cost(&[generic(2), u(), u(), r(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Wizard],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Draw a card.".into(),
                body: Box::new(draw(1)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), r()]),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Jhoira, Weatherlight Captain — {2}{U}{R} 3/3 Human Artificer. Cast a historic
/// spell (artifact, legendary, or Saga) → draw a card.
pub fn jhoira_weatherlight_captain() -> CardDefinition {
    CardDefinition {
        name: "Jhoira, Weatherlight Captain",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Artifact)
                        .or(R::HasSupertype(Supertype::Legendary))
                        .or(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga)),
                },
            ),
            effect: draw(1),
        }],
        ..Default::default()
    }
}

/// Arjun, the Shifting Flame — {4}{U}{R} 5/5 Sphinx Wizard, flying. Cast a spell
/// → put your hand into your library and draw that many. (Modeled as discard
/// hand + draw that many — cards route to the graveyard, not the library.)
pub fn arjun_the_shifting_flame() -> CardDefinition {
    CardDefinition {
        name: "Arjun, the Shifting Flame",
        cost: cost(&[generic(4), u(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sphinx, CreatureType::Wizard],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::DiscardHandDrawThatMany { who: you() },
        }],
        ..Default::default()
    }
}

/// Electrodominance — {X}{R}{R} Instant. Deal X to any target, then you may cast
/// a spell with mana value X or less from your hand without paying its cost.
pub fn electrodominance() -> CardDefinition {
    CardDefinition {
        name: "Electrodominance",
        cost: cost(&[x(), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::XFromCost },
            Effect::CastFromHandWithoutPaying {
                filter: Some(R::ManaValueAtMostXFromCost),
            },
        ]),
        ..Default::default()
    }
}
