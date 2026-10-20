//! Commander: the 2026-10-20 (`01DDeUtc`) closers — Leori, Sparktouched
//! Hunter's Chandras. Tests in `tests/recent_b/cmdr_01dd_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, EventKind, EventScope, EventSpec, LandType, LoyaltyAbility,
    PlaneswalkerSubtype, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, Zone,
};
use crate::effect::shortcut::{etb, target_any, target_filtered};
use crate::effect::{ActivationTrait, Effect, PlayerRef, Predicate, Selector, Value};
use crate::mana::{Color, cost, generic, r, u};

fn chandra(name: &'static str, loyalty: u32, abilities: Vec<LoyaltyAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Planeswalker],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Chandra], ..Default::default() },
        base_loyalty: loyalty,
        loyalty_abilities: abilities,
        ..Default::default()
    }
}

fn loyalty(cost: i32, effect: Effect) -> LoyaltyAbility {
    LoyaltyAbility { loyalty_cost: cost, effect, ..Default::default() }
}

/// Chandra's Regulator — {1}{R} legendary artifact. Whenever you activate a
/// loyalty ability of a Chandra planeswalker, you may pay {1} to copy it. {1},
/// {T}, discard a Mountain card or a red card: draw a card.
pub fn chandras_regulator() -> CardDefinition {
    CardDefinition {
        name: "Chandra's Regulator",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivatedWith(ActivationTrait::Loyalty), EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasPlaneswalkerType(PlaneswalkerSubtype::Chandra),
                }),
            effect: Effect::MayPay {
                description: "Pay {1} to copy that loyalty ability?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::CopyActivatedAbilityMayChooseTargets),
                else_: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            discard_cost: Some((R::HasLandType(LandType::Mountain).or(R::HasColor(Color::Red)), 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Way of the Cryomancer — {2}{U} legendary enchantment. ETB: empower Jace 5.
/// Planeswalkers you control have "[−3]: When you next cast an instant or
/// sorcery spell this turn, copy that spell; new targets allowed."
pub fn way_of_the_cryomancer() -> CardDefinition {
    CardDefinition {
        name: "Way of the Cryomancer",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::EmpowerJace { who: PlayerRef::You, count: Value::Const(5) })],
        static_abilities: vec![StaticAbility {
            description: "Planeswalkers you control have \"[−3]: When you next cast an instant or sorcery spell this turn, copy that spell.\"",
            effect: StaticEffect::PlaneswalkersHaveLoyaltyAbilities {
                abilities: vec![loyalty(-3, Effect::CopyNextInstantSorceryThisTurn { count: Value::ONE })],
            },
        }],
        ..Default::default()
    }
}

/// Chandra, Flame's Catalyst — {4}{R}{R}, loyalty 5. +1: 3 damage to each
/// opponent. −2: you may cast target red instant or sorcery card from your
/// graveyard, exiled if it would go back. −8: discard your hand, draw seven;
/// until end of turn, cast spells from your hand without paying their mana
/// costs.
pub fn chandra_flames_catalyst() -> CardDefinition {
    chandra(
        "Chandra, Flame's Catalyst",
        5,
        vec![
            loyalty(1, Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(3) }),
            loyalty(
                -2,
                Effect::CastWithoutPayingImmediate {
                    what: target_filtered(
                        R::HasColor(Color::Red)
                            .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))
                            .from_your_graveyard(),
                    ),
                    source_zone: Zone::Graveyard,
                    exile_after: true,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: true,
                },
            ),
            loyalty(
                -8,
                Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false },
                    Effect::Draw { who: Selector::You, amount: Value::Const(7) },
                    Effect::CastHandSpellsFreeThisTurn,
                ]),
            ),
        ],
    )
}

/// Chandra, Flame's Fury — {4}{R}{R}, loyalty 4. +1: 2 damage to any target.
/// −2: 4 damage to target creature and 2 to its controller. −8: 10 damage to
/// target player and each creature they control.
pub fn chandra_flames_fury() -> CardDefinition {
    chandra(
        "Chandra, Flame's Fury",
        4,
        vec![
            loyalty(1, Effect::DealDamage { to: target_any(), amount: Value::Const(2) }),
            loyalty(
                -2,
                Effect::Seq(vec![
                    Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) },
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                        amount: Value::Const(2),
                    },
                ]),
            ),
            loyalty(
                -8,
                Effect::Seq(vec![
                    Effect::DealDamage { to: target_filtered(R::Player), amount: Value::Const(10) },
                    Effect::DealDamage {
                        to: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                        amount: Value::Const(10),
                    },
                ]),
            ),
        ],
    )
}
