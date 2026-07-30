//! Darksteel completion batch 4 — the eleven cards that were each blocked on
//! one engine primitive. Tests in `recent_b/dst`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    Selector, StaticAbility, Subtypes, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, PlayerRef, RevealMissDest, StaticEffect, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r, u, w};

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
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

/// "You may copy a card exiled with this permanent; if you do, cast the copy
/// without paying its mana cost." Panoptic Mirror / Spellbinder's payoff.
fn cast_free_copy_of_imprint() -> Effect {
    Effect::MayDo {
        description: "Copy the exiled card and cast the copy without paying its mana cost".into(),
        body: Box::new(Effect::CastWithoutPayingImmediate {
            what: Selector::CardExiledWithSource,
            source_zone: Zone::Exile,
            copy: true,
            exile_after: false,
        }),
    }
}

// ── Artifacts ──

/// Death-Mask Duplicant — {1}: imprint a creature card from your graveyard; the
/// Duplicant gains that card's evasion keywords.
pub fn death_mask_duplicant() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::ExileTaggedWithSource {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
            },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Gains the evasion keywords of cards exiled with this creature.",
            effect: StaticEffect::GainKeywordsFromExiledWith {
                keywords: vec![
                    Keyword::Flying,
                    Keyword::Fear,
                    Keyword::FirstStrike,
                    Keyword::DoubleStrike,
                    Keyword::Haste,
                    Keyword::Landwalk(crate::card::LandType::Island),
                    Keyword::Protection(Color::White),
                    Keyword::Trample,
                ],
            },
        }],
        ..artifact("Death-Mask Duplicant", cost(&[generic(7)]))
    }
}

/// Mycosynth Lattice — everything is an artifact, everything is colorless, and
/// mana is spendable as any color.
pub fn mycosynth_lattice() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "All permanents are artifacts in addition to their other types.",
                effect: StaticEffect::AddCardTypeToMatching {
                    applies_to: Selector::EachPermanent(R::Permanent),
                    card_type: CardType::Artifact,
                },
            },
            StaticAbility {
                description: "All cards, spells, and permanents are colorless.",
                effect: StaticEffect::GrantColorless {
                    applies_to: Selector::EachPermanent(R::Permanent),
                },
            },
            StaticAbility {
                description: "Players may spend mana as though it were mana of any color.",
                effect: StaticEffect::PlayersMaySpendManaAsAnyColor,
            },
        ],
        ..artifact("Mycosynth Lattice", cost(&[generic(6)]))
    }
}

/// Panoptic Mirror — {X}, {T}: imprint an instant or sorcery with mana value X;
/// each upkeep you may cast a free copy of it.
pub fn panoptic_mirror() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::x()]),
            tap_cost: true,
            effect: Effect::ExileChosenFromHand {
                from: Selector::You,
                count: Value::ONE,
                filter: R::HasCardType(CardType::Instant)
                    .or(R::HasCardType(CardType::Sorcery))
                    .and(R::ManaValueExactlyXFromCost),
                link_to_source: true,
                face_down: false,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: cast_free_copy_of_imprint(),
        }],
        ..artifact("Panoptic Mirror", cost(&[generic(5)]))
    }
}

/// Spellbinder — imprint an instant on entry; equipped creature's combat damage
/// to a player lets you cast a free copy of it. Equip {4}.
pub fn spellbinder() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        triggered_abilities: vec![etb(Effect::ExileChosenFromHand {
            from: Selector::You,
            count: Value::ONE,
            filter: R::HasCardType(CardType::Instant),
            link_to_source: true,
            face_down: false,
        })],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: cast_free_copy_of_imprint(),
            }],
            triggers_on_equipment: true,
            ..Default::default()
        }),
        ..artifact("Spellbinder", cost(&[generic(3)]))
    }
}

/// Thought Dissector — {X}, {T}: dig X deep into an opponent's library for an
/// artifact; it enters under your control and the Dissector is sacrificed.
pub fn thought_dissector() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::x()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::RevealUntilFind {
                    who: PlayerRef::Target(0),
                    find: R::Artifact,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                    cap: Value::XFromCost,
                    life_per_revealed: 0,
                    miss_dest: RevealMissDest::Graveyard,
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountOf(Box::new(Selector::LastMoved)),
                        Value::ONE,
                    ),
                    then: Box::new(Effect::SacrificeSource),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..artifact("Thought Dissector", cost(&[generic(4)]))
    }
}

// ── Creatures ──

/// Synod Artificer — {X}, {T}: tap or untap X target noncreature artifacts.
pub fn synod_artificer() -> CardDefinition {
    let x_artifacts = |effect: Effect| ActivatedAbility {
        mana_cost: cost(&[crate::mana::x()]),
        tap_cost: true,
        effect: Effect::TargetsExactlyX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Artifact.and(R::Not(Box::new(R::Creature))),
                effect: Box::new(effect),
            }),
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Synod Artificer",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![
            x_artifacts(Effect::Tap {
                what: Selector::Target(0),
            }),
            x_artifacts(Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            }),
        ],
        ..Default::default()
    }
}

// ── Spells ──

/// Dismantle — destroy target artifact; it hands its counter total on as +1/+1
/// or charge counters on an artifact you control.
pub fn dismantle() -> CardDefinition {
    spell(
        "Dismantle",
        cost(&[generic(2), r()]),
        true,
        Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Artifact),
            },
            Effect::AddCountersOfChosenKind {
                onto: R::Artifact,
                kinds: vec![CounterType::PlusOnePlusOne, CounterType::Charge],
                amount: Value::TotalCountersOn {
                    what: Box::new(Selector::Target(0)),
                },
            },
        ]),
    )
}

/// Hallow — prevent all damage target spell would deal this turn and gain that
/// much life.
pub fn hallow() -> CardDefinition {
    spell(
        "Hallow",
        cost(&[w()]),
        false,
        Effect::PreventAllDamageFromTargetThisTurn {
            what: target_filtered(R::IsSpellOnStack),
            gain_life: true,
            next_instance_only: false,
        },
    )
}

/// Pulse of the Dross — a target player reveals three cards and you pick the
/// discard; it returns to hand while they're still holding more than you.
pub fn pulse_of_the_dross() -> CardDefinition {
    spell(
        "Pulse of the Dross",
        cost(&[generic(1), b(), b()]),
        true,
        Effect::Seq(vec![
            Effect::DiscardChosenFromRevealed {
                from: Selector::Player(PlayerRef::Target(0)),
                reveal: Value::Const(3),
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::HandSizeOf(PlayerRef::Target(0)),
                    Value::Sum(vec![Value::HandSizeOf(PlayerRef::You), Value::ONE]),
                ),
                then: Box::new(Effect::ReturnResolvingSpellToHand),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Shriveling Rot — until end of turn, damaged creatures are destroyed and/or
/// dying creatures drain their controller for their toughness. Entwine {2}{B}.
pub fn shriveling_rot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Entwine(cost(&[generic(2), b()]))],
        ..spell(
            "Shriveling Rot",
            cost(&[generic(2), b(), b()]),
            false,
            Effect::ChooseMode(vec![
                Effect::DamagedCreaturesDieThisTurn,
                Effect::CreatureDeathsDrainToughnessThisTurn,
            ]),
        )
    }
}

/// Turn the Tables — all combat damage that would be dealt to you this turn
/// hits a target attacking creature instead.
pub fn turn_the_tables() -> CardDefinition {
    spell(
        "Turn the Tables",
        cost(&[generic(3), w(), w()]),
        false,
        Effect::RedirectYourCombatDamageToTarget {
            what: target_filtered(R::Creature.and(R::IsAttacking)),
        },
    )
}
