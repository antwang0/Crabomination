//! Commander: two-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), sixteenth file. Tests in `tests/recent_b/cmdr_rank3k_p.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector};
use crate::mana::{Color, ManaCost, b, cost, generic, r, u};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

fn land(name: &'static str) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        ..Default::default()
    }
}

fn tap_for(pool: ManaPayload) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool,
        },
        ..Default::default()
    }
}

// ── Ellie, Brick Master // Ellie, Vengeful Hunter ───────────────────────────

/// Ellie, Brick Master — {1}{R} legendary 2/1 Human Survivor; Partner—Survivors.
/// Distract the Horde — whenever a player attacks one of your opponents, that
/// attacking player creates a tapped 1/1 black Fungus Zombie named Cordyceps
/// Infected that's attacking that opponent.
pub fn ellie_brick_master() -> CardDefinition {
    let infected = TokenDefinition {
        name: "Cordyceps Infected".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fungus, CreatureType::Zombie],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::PartnerLabel("Survivors".into())],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::OpponentOfYoursAttacked),
            effect: Effect::CreateTokenAttacking {
                who: PlayerRef::Target(0),
                count: Value::ONE,
                definition: Arc::new(infected),
                cleanup: Default::default(),
                defender: Some(PlayerRef::Triggerer),
            },
        }],
        ..legend(
            "Ellie, Brick Master",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Survivor],
            2,
            1,
        )
    }
}

/// Ellie, Vengeful Hunter — {1}{B}{R} legendary 3/1 Human Survivor;
/// Partner—Survivors. Pay 2 life, sacrifice another creature: 2 damage to
/// target player and she gains indestructible until end of turn.
pub fn ellie_vengeful_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerLabel("Survivors".into())],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 2,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(R::Player),
                    amount: Value::Const(2),
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Ellie, Vengeful Hunter",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Human, CreatureType::Survivor],
            3,
            1,
        )
    }
}

// ── The Lunar Whale ──────────────────────────────────────────────────────────

/// Theorist's Sanctum — Land — Island. As it enters you may behold a Jace
/// (choose one you control or reveal one from hand); if you don't, it enters
/// tapped. {2}{U}, {T}: empower Jace 2.
pub fn theorists_sanctum() -> CardDefinition {
    let jace = || R::HasPlaneswalkerType(crate::card::PlaneswalkerSubtype::Jace);
    CardDefinition {
        subtypes: Subtypes {
            land_types: vec![LandType::Island],
            ..Default::default()
        },
        static_abilities: vec![StaticAbility {
            description: "As this land enters, you may behold a Jace. If you don't, this land enters tapped.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::Any(vec![
                    Predicate::SelectorExists(Selector::EachPermanent(
                        jace().and(R::ControlledByYou),
                    )),
                    Predicate::SelectorExists(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Hand,
                        filter: jace(),
                    }),
                ]),
            },
        }],
        activated_abilities: vec![
            tap_for(ManaPayload::Colors(vec![Color::Blue])),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u()]),
                tap_cost: true,
                effect: Effect::EmpowerJace {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..land("Theorist's Sanctum")
    }
}

// ── Bilbo, Fellow Conspirator ────────────────────────────────────────────────

/// Hot Dog Cart — {3} artifact. ETB: a Food. {T}: add one mana of any color.
pub fn hot_dog_cart() -> CardDefinition {
    CardDefinition {
        name: "Hot Dog Cart",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(crabomination_base::tokens::food_token()),
        })],
        activated_abilities: vec![tap_for(ManaPayload::AnyOneColor(Value::ONE))],
        ..Default::default()
    }
}

/// Stardew Valley — land. {T}: add {C}. {2}, {T}, tap an untapped creature
/// you control: a Food. {3}, {T}: choose target permanent you control; draw a
/// card, then another player of your choice may gain control of that
/// permanent. Sorcery speed. "Another player" is chosen among opponents: the
/// engine plays no team format (2HG is tracked, not played), so the sets agree.
pub fn stardew_valley() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_for(ManaPayload::Colorless(Value::ONE)),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                tap_others_cost: Some((R::Creature, 1)),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(crabomination_base::tokens::food_token()),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                    Effect::ChooseOpponentThen {
                        then: Box::new(Effect::MayDoBy {
                            who: PlayerRef::ChosenPlayerOfSource,
                            description: "Gain control of the offered permanent?".into(),
                            body: Box::new(Effect::GainControl {
                                what: target_filtered(R::Permanent.and(R::ControlledByYou)),
                                to: Some(PlayerRef::ChosenPlayerOfSource),
                                duration: Duration::Permanent,
                            }),
                        }),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..land("Stardew Valley")
    }
}
