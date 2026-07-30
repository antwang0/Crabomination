//! Darksteel completion batch 3 — the artifact-hate commons, the Entwine
//! modal spells, and the white/green rares. Tests in `recent_b/dst`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, Selector,
    StaticAbility, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    mana: ManaCost,
    power: i32,
    toughness: i32,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power,
        toughness,
        keywords,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, sorcery: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if sorcery {
            CardType::Sorcery
        } else {
            CardType::Instant
        }],
        effect,
        ..Default::default()
    }
}

/// "Destroy target [filter]. It can't be regenerated."
fn hard_removal(name: &'static str, mana: ManaCost, filter: R) -> CardDefinition {
    spell(
        name,
        mana,
        false,
        Effect::Seq(vec![
            Effect::CantBeRegeneratedThisTurn {
                what: target_filtered(filter),
            },
            Effect::Destroy {
                what: Selector::Target(0),
            },
        ]),
    )
}

/// Oxidize — destroy target artifact; it can't be regenerated.
pub fn oxidize() -> CardDefinition {
    hard_removal("Oxidize", cost(&[g()]), R::Artifact)
}

/// Purge — destroy target artifact creature or black creature; it can't be
/// regenerated.
pub fn purge() -> CardDefinition {
    hard_removal(
        "Purge",
        cost(&[generic(1), w()]),
        R::Creature.and(R::Artifact.or(R::HasColor(Color::Black))),
    )
}

/// Ritual of Restoration — return target artifact card from your graveyard to
/// your hand.
pub fn ritual_of_restoration() -> CardDefinition {
    spell(
        "Ritual of Restoration",
        cost(&[w()]),
        true,
        Effect::Move {
            what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Rebuking Ceremony — put two target artifacts on top of their owners'
/// libraries.
pub fn rebuking_ceremony() -> CardDefinition {
    spell(
        "Rebuking Ceremony",
        cost(&[generic(3), g(), g()]),
        true,
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Artifact,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::Top,
                },
            }),
        },
    )
}

/// Second Sight — rearrange the top five of your library, an opponent's, or
/// (entwined) both.
pub fn second_sight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Entwine(cost(&[u()]))],
        ..spell(
            "Second Sight",
            cost(&[generic(2), u()]),
            false,
            Effect::ChooseMode(vec![
                Effect::RearrangeTop {
                    who: PlayerRef::Target(0),
                    amount: Value::Const(5),
                },
                Effect::RearrangeTop {
                    who: PlayerRef::You,
                    amount: Value::Const(5),
                },
            ]),
        )
    }
}

/// Reap and Sow — blow up a land, fetch one, or (entwined) both.
pub fn reap_and_sow() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Entwine(cost(&[generic(1), g()]))],
        ..spell(
            "Reap and Sow",
            cost(&[generic(3), g()]),
            true,
            Effect::ChooseMode(vec![
                Effect::Destroy {
                    what: target_filtered(R::Land),
                },
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Land,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            ]),
        )
    }
}

/// Pristine Angel — Flying. While untapped it has protection from artifacts
/// and from each color; your casts may untap it.
pub fn pristine_angel() -> CardDefinition {
    let untapped_protection = |keyword: Keyword| StaticAbility {
        description: "While untapped, protection from artifacts and each color.",
        effect: StaticEffect::SelfHasKeywordWhile {
            keyword,
            condition: R::Untapped,
        },
    };
    CardDefinition {
        static_abilities: vec![
            untapped_protection(Keyword::ProtectionFromCardType(CardType::Artifact)),
            untapped_protection(Keyword::Protection(Color::White)),
            untapped_protection(Keyword::Protection(Color::Blue)),
            untapped_protection(Keyword::Protection(Color::Black)),
            untapped_protection(Keyword::Protection(Color::Red)),
            untapped_protection(Keyword::Protection(Color::Green)),
        ],
        triggered_abilities: vec![crate::effect::shortcut::on_cast(Effect::MayDo {
            description: "Untap Pristine Angel".into(),
            body: Box::new(Effect::Untap {
                what: Selector::This,
                up_to: None,
            }),
        })],
        ..creature(
            "Pristine Angel",
            cost(&[generic(4), w(), w()]),
            4,
            4,
            vec![CreatureType::Angel],
            vec![Keyword::Flying],
        )
    }
}

/// Pteron Ghost — Flying. Sacrifice it: regenerate target artifact.
pub fn pteron_ghost() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Regenerate {
                what: target_filtered(R::Artifact),
            },
            ..Default::default()
        }],
        ..creature(
            "Pteron Ghost",
            cost(&[generic(1), w()]),
            1,
            1,
            vec![CreatureType::Dinosaur, CreatureType::Spirit],
            vec![Keyword::Flying],
        )
    }
}

/// Quicksilver Behemoth — Affinity for artifacts. Attacking or blocking sends
/// it home at end of combat.
pub fn quicksilver_behemoth() -> CardDefinition {
    let bounce = || Effect::DelayUntil {
        kind: crate::effect::DelayedTriggerKind::EndOfCombat,
        body: Box::new(Effect::Move {
            what: Selector::This,
            to: ZoneDest::Hand(PlayerRef::You),
        }),
    };
    CardDefinition {
        affinity_filter: Some(R::Artifact.and(R::ControlledByYou)),
        triggered_abilities: vec![
            on_attack(bounce()),
            crate::effect::shortcut::blocks(bounce()),
        ],
        ..creature(
            "Quicksilver Behemoth",
            cost(&[generic(6), u()]),
            4,
            5,
            vec![CreatureType::Beast],
            vec![],
        )
    }
}

/// Razor Golem — Affinity for Plains. Vigilance.
pub fn razor_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        affinity_filter: Some(R::HasLandType(LandType::Plains).and(R::ControlledByYou)),
        ..creature(
            "Razor Golem",
            cost(&[generic(6)]),
            3,
            4,
            vec![CreatureType::Golem],
            vec![Keyword::Vigilance],
        )
    }
}

/// Roaring Slagwurm — whenever it attacks, tap all artifacts.
pub fn roaring_slagwurm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Tap {
            what: Selector::EachPermanent(R::Artifact),
        })],
        ..creature(
            "Roaring Slagwurm",
            cost(&[generic(5), g(), g()]),
            6,
            4,
            vec![CreatureType::Wurm],
            vec![],
        )
    }
}

/// Scavenging Scarab — a 3/3 that can't block.
pub fn scavenging_scarab() -> CardDefinition {
    creature(
        "Scavenging Scarab",
        cost(&[generic(3), b()]),
        3,
        3,
        vec![CreatureType::Insect],
        vec![Keyword::CantBlock],
    )
}

/// Screams from Within — Aura: -1/-1, and it returns from your graveyard when
/// the host dies.
pub fn screams_from_within() -> CardDefinition {
    CardDefinition {
        name: "Screams from Within",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: -1,
            toughness: -1,
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        }],
        ..Default::default()
    }
}

/// Psychic Overload — Aura: taps the host on entry, locks its untap step, and
/// grants a two-artifact-discard escape hatch.
pub fn psychic_overload() -> CardDefinition {
    CardDefinition {
        name: "Psychic Overload",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Permanent),
        },
        triggered_abilities: vec![etb(Effect::Tap {
            what: Selector::AttachedTo(Box::new(Selector::This)),
        })],
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted permanent doesn't untap during its untap step.",
                effect: StaticEffect::PreventUntap {
                    applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                },
            },
            StaticAbility {
                description: "Enchanted permanent has \"Discard two artifact cards: Untap it.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                    ability: ActivatedAbility {
                        discard_cost: Some((R::Artifact, 2)),
                        effect: Effect::Untap {
                            what: Selector::This,
                            up_to: None,
                        },
                        ..Default::default()
                    },
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Shield of Kaldra — the equipped creature is indestructible, and so is the
/// whole Kaldra Equipment set.
pub fn shield_of_kaldra() -> CardDefinition {
    CardDefinition {
        name: "Shield of Kaldra",
        cost: cost(&[generic(4)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Indestructible],
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "The Kaldra Equipment have indestructible.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::HasName("Sword of Kaldra".into())
                        .or(R::HasName("Shield of Kaldra".into()))
                        .or(R::HasName("Helm of Kaldra".into())),
                ),
                keyword: Keyword::Indestructible,
            },
        }],
        ..Default::default()
    }
}

/// "Cast this spell only during combat on your turn."
fn during_your_combat() -> crate::effect::Predicate {
    use crate::effect::Predicate as P;
    use crate::game::types::TurnStep as S;
    P::All(vec![
        P::IsTurnOf(PlayerRef::You),
        P::Any(
            [
                S::BeginCombat,
                S::DeclareAttackers,
                S::DeclareBlockers,
                S::FirstStrikeDamage,
                S::CombatDamage,
                S::EndCombat,
            ]
            .into_iter()
            .map(P::CurrentStepIs)
            .collect(),
        ),
    ])
}

/// Shunt — change the target of target spell with a single target.
pub fn shunt() -> CardDefinition {
    spell(
        "Shunt",
        cost(&[generic(1), r(), r()]),
        false,
        Effect::ChangeSpellTarget {
            what: target_filtered(R::IsSpellOnStack),
        },
    )
}

/// Savage Beating — double strike for the team, or untap and take another
/// combat phase. Entwine {1}{R}.
pub fn savage_beating() -> CardDefinition {
    let mine = R::Creature.and(R::ControlledByYou);
    CardDefinition {
        keywords: vec![Keyword::Entwine(cost(&[generic(1), r()]))],
        cast_condition: Some(during_your_combat()),
        ..spell(
            "Savage Beating",
            cost(&[generic(3), r(), r()]),
            false,
            Effect::ChooseMode(vec![
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(mine.clone()),
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
                Effect::Seq(vec![
                    Effect::Untap {
                        what: Selector::EachPermanent(mine),
                        up_to: None,
                    },
                    Effect::AdditionalCombatPhase { count: Value::ONE },
                ]),
            ]),
        )
    }
}

/// Stir the Pride — +2/+2 for the team, or lifelink for the team. Entwine
/// {1}{W}.
pub fn stir_the_pride() -> CardDefinition {
    let mine = R::Creature.and(R::ControlledByYou);
    CardDefinition {
        keywords: vec![Keyword::Entwine(cost(&[generic(1), w()]))],
        ..spell(
            "Stir the Pride",
            cost(&[generic(4), w()]),
            false,
            Effect::ChooseMode(vec![
                Effect::PumpPT {
                    what: Selector::EachPermanent(mine.clone()),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(mine),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
            ]),
        )
    }
}

/// Scrounge — reanimate an artifact card out of an opponent's graveyard under
/// your control. (You pick the card rather than its owner.)
pub fn scrounge() -> CardDefinition {
    spell(
        "Scrounge",
        cost(&[generic(2), b()]),
        true,
        Effect::Move {
            what: target_filtered(R::Artifact.and(R::InOpponentGraveyard)),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
    )
}
