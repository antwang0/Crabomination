//! Oath of the Gatewatch (OGW) gap wave 4 — the planeswalker and the last
//! rares. Tests in `classic_sets/ogw`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CreatureType, EquipBonus, EquipScale, Keyword,
    LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R, Subtypes, Supertype, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::card::TokenDefinition;
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::mana::{b, cost, generic, r, x};

/// Chandra, Flamecaller — {4}{R}{R} loyalty 4. Transient Elementals, a full
/// rummage, and a scaling sweeper.
pub fn chandra_flamecaller() -> CardDefinition {
    let elemental = TokenDefinition {
        name: "Elemental".into(),
        power: 3,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::Red],
        keywords: vec![Keyword::Haste],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Chandra, Flamecaller",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Chandra],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(2),
                        definition: elemental,
                    },
                    Effect::ExileLastCreatedTokensAtNextEndStep,
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Seq(vec![
                    Effect::DiscardHandDrawThatMany { who: Selector::You },
                    crate::effect::shortcut::draw(1),
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                x_cost: true,
                effect: Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature),
                    amount: Value::XFromCost,
                },
            },
        ],
        ..Default::default()
    }
}

/// Fall of the Titans — {X}{X}{R} Instant. Surge {X}{R}. X damage to each of
/// up to two targets.
pub fn fall_of_the_titans() -> CardDefinition {
    CardDefinition {
        name: "Fall of the Titans",
        cost: cost(&[x(), x(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature.or(R::Player).or(R::Planeswalker),
            effect: Box::new(Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::XFromCost,
            }),
        },
        alternative_cost: Some(crate::effect::shortcut::surge(cost(&[x(), r()]), false)),
        ..Default::default()
    }
}

/// Immobilizer Eldrazi — {1}{R} 2/1 Eldrazi Drone. Devoid. {2}{C}: creatures
/// whose toughness beats their power can't block this turn.
pub fn immobilizer_eldrazi() -> CardDefinition {
    CardDefinition {
        name: "Immobilizer Eldrazi",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Drone],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Devoid],
        activated_abilities: vec![crate::card::ActivatedAbility {
            mana_cost: cost(&[generic(2), crate::mana::colorless(1)]),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ToughnessGreaterThanPower)),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Remorseless Punishment — {3}{B}{B} Sorcery. Twice: target opponent discards
/// two, sacrifices a creature or planeswalker, or loses 5 life.
pub fn remorseless_punishment() -> CardDefinition {
    let round = || Effect::Punisher {
        chooser: target_filtered(R::OpponentPlayer),
        options: vec![
            Effect::Discard {
                who: Selector::Target(0),
                amount: Value::Const(2),
                random: false,
            },
            Effect::Sacrifice {
                who: Selector::Target(0),
                count: Value::Const(1),
                filter: R::Creature.or(R::Planeswalker),
            },
        ],
        otherwise: Box::new(Effect::LoseLife {
            who: Selector::Target(0),
            amount: Value::Const(5),
        }),
    };
    CardDefinition {
        name: "Remorseless Punishment",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![round(), round()]),
        ..Default::default()
    }
}

/// Stoneforge Masterwork — {1} Equipment. Equipped creature gets +1/+1 for
/// each other creature you control sharing a type with it. Equip {2}.
pub fn stoneforge_masterwork() -> CardDefinition {
    CardDefinition {
        name: "Stoneforge Masterwork",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                per_power: 1,
                per_toughness: 1,
                count_sharing_type_with_host: true,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Endbringer — {5}{C} 5/5 Eldrazi. Untaps on every other player's untap step
/// and carries three colorless-fed tap abilities.
pub fn endbringer() -> CardDefinition {
    use crate::card::{ActivatedAbility, StaticAbility};
    use crate::effect::{shortcut::target_any, StaticEffect};
    let c1 = || cost(&[crate::mana::colorless(1)]);
    CardDefinition {
        name: "Endbringer",
        cost: cost(&[generic(5), crate::mana::colorless(1)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        static_abilities: vec![StaticAbility {
            description: "Untap this creature during each other player's untap step.",
            effect: StaticEffect::UntapSelfEachOtherUntapStep,
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: crate::effect::shortcut::deal(1, target_any()),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: c1(),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_filtered(R::Creature),
                        keyword: Keyword::CantAttack,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::CantBlock,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[crate::mana::colorless(2)]),
                tap_cost: true,
                effect: crate::effect::shortcut::draw(1),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Dazzling Reflection — {1}{W} Instant. Gain life equal to target creature's
/// power and blank the next damage it would deal this turn.
pub fn dazzling_reflection() -> CardDefinition {
    CardDefinition {
        name: "Dazzling Reflection",
        cost: cost(&[generic(1), crate::mana::w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::PowerOf(Box::new(target_filtered(R::Creature))),
            },
            Effect::PreventAllDamageFromTargetThisTurn {
                what: Selector::Target(0),
                gain_life: false,
                next_instance_only: true,
            },
        ]),
        ..Default::default()
    }
}

/// Dimensional Infiltrator — {1}{U} 2/1 Eldrazi. Devoid, flash, flying;
/// {1}{C} mills an opponent's top card into exile and bounces itself off a land.
pub fn dimensional_infiltrator() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Dimensional Infiltrator",
        cost: cost(&[generic(1), crate::mana::u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Devoid, Keyword::Flash, Keyword::Flying],
        activated_abilities: vec![crate::card::ActivatedAbility {
            mana_cost: cost(&[generic(1), crate::mana::colorless(1)]),
            effect: Effect::Seq(vec![
                Effect::ExileTopOfLibrary {
                    who: target_filtered(R::OpponentPlayer),
                    amount: Value::Const(1),
                    link_to_source: false,
                    face_down: false,
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::LastMoved,
                        filter: R::Land,
                    },
                    then: Box::new(Effect::MayDo {
                        description: "Return this creature to your hand?".into(),
                        body: Box::new(Effect::Move {
                            what: Selector::This,
                            to: crate::effect::ZoneDest::Hand(PlayerRef::You),
                        }),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hedron Alignment — {2}{U} Enchantment with hexproof. Reveal your hand at
/// upkeep to win if you own a Hedron Alignment in all four zones. {1}{U}: Scry 1.
pub fn hedron_alignment() -> CardDefinition {
    use crate::card::{ActivatedAbility, Predicate};
    use crate::effect::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Hedron Alignment",
        cost: cost(&[generic(2), crate::mana::u()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Hexproof],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Reveal your hand to check the Hedron Alignment?".into(),
                body: Box::new(Effect::If {
                    cond: Predicate::OwnsSourceNamedCardInEveryZone { who: PlayerRef::You },
                    then: Box::new(Effect::WinGame { who: PlayerRef::You }),
                    else_: Box::new(Effect::Noop),
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), crate::mana::u()]),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
