//! Commander: the cards the **Keen Engineering** precon (FDC, Sai, Master
//! Thopterist) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EquipScale, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::catalog::sets::{tap_add, tap_add_colorless};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    ActivatedAbility, Duration, Effect, ExtraManaKind, LookPick, PlayerRef, Predicate,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, cost, generic, u, x};
use std::sync::Arc;

fn thopters(n: i32) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: Arc::new(TokenDefinition {
            name: "Thopter".into(),
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Artifact, CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
    }
}

fn your_artifacts() -> Value {
    Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou))))
}

fn artifact_creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Adaptive Omnitool — +1/+1 per artifact, and an artifact dig on attack.
pub fn adaptive_omnitool() -> CardDefinition {
    CardDefinition {
        name: "Adaptive Omnitool",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                filter: R::Artifact,
                per_power: 1,
                per_toughness: 1,
                ..Default::default()
            }),
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::LookPickToHand(Box::new(LookPick {
                    who: PlayerRef::You,
                    count: Value::Const(6),
                    pick_filter: Some(R::Artifact),
                    optional: true,
                    ..Default::default()
                })),
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Darksteel Juggernaut — indestructible, as big as your artifact count, and
/// attacks each combat.
pub fn darksteel_juggernaut() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Indestructible, Keyword::MustAttack],
        static_abilities: vec![StaticAbility {
            description: "Darksteel Juggernaut's power and toughness are each equal to the number of artifacts you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: your_artifacts(), toughness: your_artifacts() },
        }],
        ..artifact_creature(
            "Darksteel Juggernaut",
            cost(&[generic(5)]),
            vec![CreatureType::Juggernaut],
            0,
            0,
        )
    }
}

/// Fall from Favor — tap it, take the crown, and it stays tapped unless its
/// controller wears the crown.
pub fn fall_from_favor() -> CardDefinition {
    let enchanted = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Fall from Favor",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: enchanted() },
            Effect::BecomeMonarch { who: PlayerRef::You },
        ]))],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step unless that player is the monarch.",
            effect: StaticEffect::PreventUntapGlobal {
                applies_to: enchanted(),
                condition: Some(Predicate::Not(Box::new(Predicate::IsMonarch {
                    who: PlayerRef::ControllerOf(Box::new(enchanted())),
                }))),
            },
        }],
        ..Default::default()
    }
}

/// Forsaken Monument — +2/+2 for colorless creatures, an extra {C} per
/// colorless tap, and 2 life per colorless spell.
pub fn forsaken_monument() -> CardDefinition {
    CardDefinition {
        name: "Forsaken Monument",
        cost: cost(&[generic(5)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "Colorless creatures you control get +2/+2.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        R::Creature.and(R::Colorless).and(R::ControlledByYou),
                    ),
                    power: 2,
                    toughness: 2,
                },
            },
            StaticAbility {
                description: "Whenever you tap a permanent for {C}, add an additional {C}.",
                effect: StaticEffect::ExtraManaOnLandTap {
                    enchanted_only: false,
                    filter: R::ControlledByYou,
                    extra: ExtraManaKind::MirrorColorless,
                    while_monarch: false,
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Colorless },
            ),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        }],
        ..Default::default()
    }
}

/// Foundry of the Consuls — {C}, or cash in for two Thopters.
pub fn foundry_of_the_consuls() -> CardDefinition {
    CardDefinition {
        name: "Foundry of the Consuls",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                sac_cost: true,
                effect: thopters(2),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Launch Mishap — counter a creature or planeswalker spell, and a Thopter.
pub fn launch_mishap() -> CardDefinition {
    CardDefinition {
        name: "Launch Mishap",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::Creature.or(R::Planeswalker))),
            },
            thopters(1),
        ]),
        ..Default::default()
    }
}

/// Master Transmuter — return an artifact to put one from hand into play.
pub fn master_transmuter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            return_permanent_cost: Some((R::Artifact, 1)),
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Artifact,
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Master Transmuter",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            2,
        )
    }
}

/// Misleading Signpost — flash it in during declare attackers to point an
/// attacker somewhere else; otherwise a blue mana rock.
pub fn misleading_signpost() -> CardDefinition {
    CardDefinition {
        name: "Misleading Signpost",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Flash],
        activated_abilities: vec![tap_add(Color::Blue)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::CurrentStepIs(TurnStep::DeclareAttackers)),
            effect: Effect::ReselectAttackTarget { what: target_filtered(R::IsAttacking) },
        }],
        ..Default::default()
    }
}

/// Research Thief — a card per artifact creature of yours that connects.
pub fn research_thief() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact.and(R::Creature),
                }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..artifact_creature(
            "Research Thief",
            cost(&[generic(4), u()]),
            vec![CreatureType::Moonfolk, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Shimmer Dragon — hexproof with four artifacts; tap two artifacts to draw.
pub fn shimmer_dragon() -> CardDefinition {
    CardDefinition {
        name: "Shimmer Dragon",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "As long as you control four or more artifacts, this creature has hexproof.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Hexproof,
                condition: Predicate::ValueAtLeast(your_artifacts(), Value::Const(4)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((R::Artifact, 2)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Skysovereign, Consul Flagship — 3 damage to an opposing creature or
/// planeswalker whenever it enters or attacks.
pub fn skysovereign_consul_flagship() -> CardDefinition {
    let shot = || Effect::DealDamage {
        to: target_filtered(R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent)),
        amount: Value::Const(3),
    };
    CardDefinition {
        name: "Skysovereign, Consul Flagship",
        cost: cost(&[generic(5)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Crew(3)],
        triggered_abilities: vec![
            etb(shot()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: shot(),
            },
        ],
        ..Default::default()
    }
}

/// Steel Hellkite — firebreathing for {2}, and {X} to sweep mana value X from
/// every player it hit this turn, once a turn.
pub fn steel_hellkite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[x()]),
                once_per_turn: true,
                effect: Effect::Destroy {
                    what: Selector::EachPermanent(
                        R::Nonland
                            .and(R::ManaValueExactlyXFromCost)
                            .and(R::ControllerDamagedBySourceThisTurn),
                    ),
                },
                ..Default::default()
            },
        ],
        ..artifact_creature("Steel Hellkite", cost(&[generic(6)]), vec![CreatureType::Dragon], 5, 5)
    }
}
