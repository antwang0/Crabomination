//! Commander: 2026-10-19 (`01DJHkWk`) — Trystan, Callous Cultivator and its
//! Elf graveyard trio: Dawnhand Eulogist, Moon-Vigil Adherents, Unbury.
//! Tests in `tests/recent_b/cmdr_01dj_b.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::etb;
use crate::effect::{
    Effect, PlayerRef, Predicate, Selector, StaticAbility, StaticEffect, Value, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, b, cost, g, generic};

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

fn mill_three() -> Effect {
    Effect::Mill {
        who: Selector::You,
        amount: Value::Const(3),
    }
}

/// "Then if there is an Elf card in your graveyard."
fn elf_in_graveyard() -> Predicate {
    Predicate::ValueAtLeast(
        Value::CardsInGraveyardMatching {
            who: PlayerRef::You,
            filter: R::HasCreatureType(CreatureType::Elf),
        },
        Value::ONE,
    )
}

/// "At the beginning of your first main phase, you may pay {cost}. If you do,
/// transform [this]."
fn pay_to_transform(mana: ManaCost) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::ActivePlayer),
        effect: Effect::MayPay {
            description: "Pay to transform?".into(),
            mana_cost: mana,
            body: Box::new(Effect::Transform { what: Selector::This }),
            else_: None,
        },
    }
}

/// Trystan, Callous Cultivator // Trystan, Penitent Culler — {2}{G}
/// legendary 3/4 deathtouch Elf Druid. Entering or transforming into this
/// face: mill three, then gain 2 if an Elf card is in your graveyard. First
/// main phase: may pay {B} to transform. Back (Elf Warlock): transforming into
/// it mills three, then you may exile an Elf card from your graveyard to have
/// each opponent lose 2; first main phase, may pay {G} to transform.
pub fn trystan_callous_cultivator() -> CardDefinition {
    let front_body = || {
        Effect::Seq(vec![
            mill_three(),
            Effect::If {
                cond: elf_in_graveyard(),
                then: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        ])
    };
    let back = CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    mill_three(),
                    Effect::MayExileFromYourGraveyard {
                        filter: R::HasCreatureType(CreatureType::Elf),
                        max: Some(1),
                        then: Box::new(Effect::LoseLife {
                            who: Selector::Player(PlayerRef::EachOpponent),
                            amount: Value::Const(2),
                        }),
                    },
                ]),
            },
            pay_to_transform(cost(&[g()])),
        ],
        ..creature(
            "Trystan, Penitent Culler",
            ManaCost::default(),
            vec![CreatureType::Elf, CreatureType::Warlock],
            3,
            4,
        )
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            etb(front_body()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource),
                effect: front_body(),
            },
            pay_to_transform(cost(&[b()])),
        ],
        back_face: Some(Box::new(back)),
        ..creature(
            "Trystan, Callous Cultivator",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            3,
            4,
        )
    }
}

/// Dawnhand Eulogist — {3}{B} 3/3 menace Elf Warlock. Enters: mill three;
/// then if an Elf card is in your graveyard, each opponent loses 2 and you
/// gain 2.
pub fn dawnhand_eulogist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            mill_three(),
            Effect::If {
                cond: elf_in_graveyard(),
                then: Box::new(Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(2),
                    },
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::Const(2),
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature(
            "Dawnhand Eulogist",
            cost(&[generic(3), b()]),
            vec![CreatureType::Elf, CreatureType::Warlock],
            3,
            3,
        )
    }
}

/// Moon-Vigil Adherents — {2}{G}{G} 0/0 trample Elf Druid; +1/+1 for each
/// creature you control and each creature card in your graveyard.
pub fn moon_vigil_adherents() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each creature you control and each creature card in your graveyard.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::Sum(vec![
                    Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByYou)))),
                    Value::CardsInGraveyardMatching {
                        who: PlayerRef::You,
                        filter: R::Creature,
                    },
                ]),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..creature(
            "Moon-Vigil Adherents",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            0,
            0,
        )
    }
}

/// Unbury — {1}{B} instant. Choose one — return target creature card from
/// your graveyard to your hand; or return two target creature cards that
/// share a creature type from your graveyard to your hand.
pub fn unbury() -> CardDefinition {
    let back = |slot: u8, filter: R| Effect::Move {
        what: Selector::TargetFiltered { slot, filter },
        to: ZoneDest::Hand(PlayerRef::You),
    };
    let card = || R::Creature.and(R::InYourGraveyard);
    CardDefinition {
        name: "Unbury",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseModesCast {
            modes: vec![
                back(0, card()),
                Effect::Seq(vec![
                    back(0, card()),
                    back(
                        1,
                        card()
                            .and(R::OtherThanTargetSlot(0))
                            .and(R::SharesCreatureTypeWithTargetSlot(0)),
                    ),
                ]),
            ],
            min: 1,
            max: 1,
            allow_repeats: false,
        },
        ..Default::default()
    }
}
