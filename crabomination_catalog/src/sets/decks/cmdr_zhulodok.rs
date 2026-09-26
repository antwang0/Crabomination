//! Commander: the cards the **Eldrazi Unbound** precon (CMM, Zhulodok, Void
//! Gorger) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_zhulodok.rs`.
//!
//! Residuals (each also on its card):
//! - **Abstruse Archaic** — the target is the ability's source permanent (a
//!   colorless permanent you control with an ability on the stack), as
//!   Strionic Resonator.
//! - **Ugin's Mastery** — the face-down creature turned up is the first one
//!   you control.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_cast, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::mana::{colorless, cost, generic, x, Color, ManaCost};
use std::sync::Arc;

fn colorless_obj() -> R {
    R::Colorless
}

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

/// Zhulodok, Void Gorger — colorless spells of mana value 7 or more cast from
/// your hand cascade twice.
pub fn zhulodok_void_gorger() -> CardDefinition {
    let cascade = || Effect::Cascade { max_mv: Value::ManaValueOf(Box::new(Selector::TriggerSource)), filter: None };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastFromHand,
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: colorless_obj().and(R::ManaValueAtLeast(7)),
                },
            ])),
            effect: Effect::Seq(vec![cascade(), cascade()]),
        }],
        ..creature("Zhulodok, Void Gorger", cost(&[generic(5), colorless(1)]), vec![CreatureType::Eldrazi], 7, 4)
    }
}

/// Abstruse Archaic — vigilance; {1}, {T}: copy an ability of a colorless
/// source you control. Residual: targets the source permanent.
pub fn abstruse_archaic() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::CopyAbility {
                what: target_filtered(R::HasAbilityOnStack.and(R::ControlledByYou).and(colorless_obj())),
                times: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature("Abstruse Archaic", cost(&[generic(4)]), vec![CreatureType::Avatar], 3, 4)
    }
}

/// Calamity of the Titans — reveal a colorless creature; exile each creature
/// and planeswalker of lesser mana value.
pub fn calamity_of_the_titans() -> CardDefinition {
    CardDefinition {
        name: "Calamity of the Titans",
        cost: cost(&[generic(4), colorless(1), colorless(1)]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::RevealFromHand {
            filter: R::Creature.and(colorless_obj()),
        }],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(R::Creature.or(R::Planeswalker)),
            body: Box::new(Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::RevealedForCostManaValue,
                    Value::Sum(vec![Value::ManaValueOf(Box::new(Selector::TriggerSource)), Value::ONE]),
                ),
                then: Box::new(Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile }),
                else_: Box::new(Effect::Noop),
            }),
        },
        ..Default::default()
    }
}

/// Darksteel Monolith — indestructible; once each turn a colorless spell from
/// your hand may be cast for {0}.
pub fn darksteel_monolith() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Indestructible],
        static_abilities: vec![StaticAbility {
            description: "Once each turn, you may pay {0} rather than pay the mana cost for a colorless spell you cast from your hand.",
            effect: StaticEffect::ZeroAlternativeCostOncePerTurn { filter: colorless_obj() },
        }],
        ..artifact("Darksteel Monolith", cost(&[generic(8)]))
    }
}

/// Desecrate Reality — an even-mana-value permanent per opponent exiled;
/// adamant (three colorless) returns an odd one from your graveyard.
pub fn desecrate_reality() -> CardDefinition {
    CardDefinition {
        name: "Desecrate Reality",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ForEachOpponentTarget {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 5,
                    min_targets: 0,
                    filter: R::Permanent.and(R::ControlledByOpponent).and(R::ManaValueParity { odd: false }),
                    effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
                }),
            },
            Effect::If {
                cond: Predicate::ColorlessManaSpentAtLeast(3),
                then: Box::new(Effect::Move {
                    what: Selector::Take {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: crate::card::Zone::Graveyard,
                            filter: R::Permanent.and(R::ManaValueParity { odd: true }),
                        }),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Flayer of Loyalties — on cast, borrow a creature as a 10/10 trampling,
/// hasty annihilator.
pub fn flayer_of_loyalties() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Annihilator(2), Keyword::Trample],
        triggered_abilities: vec![on_cast(Effect::Seq(vec![
            Effect::GainControl { what: target_filtered(R::Creature), to: None, duration: Duration::EndOfTurn },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::SetBasePT {
                what: Selector::Target(0),
                power: Value::Const(10),
                toughness: Value::Const(10),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeywords {
                what: Selector::Target(0),
                keywords: vec![Keyword::Trample, Keyword::Annihilator(2), Keyword::Haste],
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature(
            "Flayer of Loyalties",
            cost(&[generic(8), colorless(1), colorless(1)]),
            vec![CreatureType::Eldrazi],
            10,
            10,
        )
    }
}

/// Guildless Commons — enters tapped, returns a land; {T}: {C}{C}.
pub fn guildless_commons() -> CardDefinition {
    CardDefinition {
        name: "Guildless Commons",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Land.and(R::ControlledByYou)),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(2)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Investigator's Journal — suspect counters for the biggest board; {2}, {T},
/// a counter: draw; {2}, sacrifice: draw.
pub fn investigators_journal() -> CardDefinition {
    let most = Value::CountOf(Box::new(Selector::ControlledBy { who: PlayerRef::MostCreatures, filter: R::Creature }));
    let draw = || Effect::Draw { who: Selector::You, amount: Value::ONE };
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Book, ArtifactSubtype::Clue],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::Suspect, most)),
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                remove_counter_cost: Some((CounterType::Suspect, 1)),
                effect: draw(),
                ..Default::default()
            },
            ActivatedAbility { mana_cost: cost(&[generic(2)]), sac_cost: true, effect: draw(), ..Default::default() },
        ],
        ..artifact("Investigator's Journal", cost(&[generic(2)]))
    }
}

/// It That Betrays — annihilator 2; an opponent's sacrificed nontoken
/// permanent comes to you.
pub fn it_that_betrays() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Annihilator(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken },
            ),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..creature("It That Betrays", cost(&[generic(12)]), vec![CreatureType::Eldrazi], 11, 11)
    }
}

/// Mage-Ring Network — {T}: {C}; {1}, {T}: a storage counter; {T}, remove X:
/// {C} for each.
pub fn mage_ring_network() -> CardDefinition {
    CardDefinition {
        name: "Mage-Ring Network",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Storage, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_x: Some(CounterType::Storage),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::XFromCost) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Mirage Mirror — {2}: a copy of a target artifact, creature, enchantment or
/// land until end of turn.
pub fn mirage_mirror() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::BecomeCopyOfFor {
                what: Selector::This,
                source: target_filtered(
                    R::Artifact.or(R::Creature).or(R::Enchantment).or(R::Land).and(R::OtherThanSource),
                ),
                duration: Duration::EndOfTurn,
                non_legendary: false,
            },
            ..Default::default()
        }],
        ..artifact("Mirage Mirror", cost(&[generic(3)]))
    }
}

/// Myriad Construct — kicked, a counter per opposing nonbasic land; targeted
/// by a spell, it scatters into 1/1 Constructs.
pub fn myriad_construct() -> CardDefinition {
    let construct = TokenDefinition {
        name: "Construct".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Kicker(cost(&[generic(3)]))],
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::CountOf(Box::new(Selector::EachPermanent(
                        R::Land.and(R::Not(Box::new(R::HasSupertype(Supertype::Basic)))).and(R::ControlledByOpponent),
                    ))),
                }),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec {
                    causer_filter: Some(R::IsSpellOnStack),
                    ..EventSpec::new(EventKind::BecameTarget, EventScope::AnyPlayer)
                },
                effect: Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::PowerOf(Box::new(Selector::This)),
                        definition: Arc::new(construct),
                    },
                    Effect::SacrificeSource,
                ]),
            },
        ],
        ..creature("Myriad Construct", cost(&[generic(4)]), vec![CreatureType::Construct], 4, 4)
    }
}

/// Not of This World — counter a spell or ability aimed at your permanent;
/// free against one aimed at your power-7 creature.
pub fn not_of_this_world() -> CardDefinition {
    let aimed_at = |inner: R| R::TargetsAPermanentYouControlMatching(Box::new(inner));
    CardDefinition {
        name: "Not of This World",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Kindred, CardType::Instant],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        effect: Effect::CounterSpellOrAbility {
            what: target_filtered(R::IsSpellOnStack.or(R::HasAbilityOnStack).and(aimed_at(R::Permanent))),
        },
        self_cost_reduction_cost_if_target: Some((
            aimed_at(R::Creature.and(R::PowerAtLeast(7))),
            cost(&[generic(7)]),
        )),
        ..Default::default()
    }
}

/// Omarthis, Ghostfire Initiate — X counters; grows with your other colorless
/// creatures'; dies into that many manifests.
pub fn omarthis_ghostfire_initiate() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(colorless_obj()).and(R::OtherThanSource),
                    }),
                effect: Effect::MayDo {
                    description: "Put a +1/+1 counter on Omarthis?".into(),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Manifest {
                    who: PlayerRef::You,
                    amount: Value::TotalCountersOn { what: Box::new(Selector::This) },
                },
            },
        ],
        ..creature(
            "Omarthis, Ghostfire Initiate",
            cost(&[x(), x()]),
            vec![CreatureType::Spirit, CreatureType::Snake],
            0,
            0,
        )
    }
}

/// Rise of the Eldrazi — uncounterable: destroy a permanent, a player draws
/// four, take an extra turn; exiled.
pub fn rise_of_the_eldrazi() -> CardDefinition {
    CardDefinition {
        name: "Rise of the Eldrazi",
        cost: cost(&[generic(9), colorless(1), colorless(1), colorless(1)]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::CantBeCountered],
        exile_on_resolve: true,
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Permanent) },
            Effect::Draw {
                who: Selector::TargetFiltered { slot: 1, filter: R::Player },
                amount: Value::Const(4),
            },
            Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Skittering Cicada — flash; colorless spells have flash; each colorless
/// spell pumps it by its mana value with trample.
pub fn skittering_cicada() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "You may cast colorless spells as though they had flash.",
            effect: StaticEffect::ControllerSpellsHaveFlash { filter: colorless_obj() },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: colorless_obj() },
            ),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                    toughness: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            ]),
        }],
        ..creature("Skittering Cicada", cost(&[generic(3)]), vec![CreatureType::Insect], 2, 2)
    }
}

/// Transmogrifying Wand — three charges; {1}, {T}, a charge: destroy a
/// creature, its controller gets a 2/4 Ox.
pub fn transmogrifying_wand() -> CardDefinition {
    let ox = TokenDefinition {
        name: "Ox".into(),
        power: 2,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ox], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        enters_with_counters: Some((CounterType::Charge, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            remove_counter_cost: Some((CounterType::Charge, 1)),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    count: Value::ONE,
                    definition: Arc::new(ox),
                },
                Effect::Destroy { what: target_filtered(R::Creature) },
            ]),
            ..Default::default()
        }],
        ..artifact("Transmogrifying Wand", cost(&[generic(3)]))
    }
}

/// Ugin's Mastery — each colorless creature spell manifests; a six-power
/// attack may turn a face-down creature up. Residual: the first face-down one.
pub fn ugins_mastery() -> CardDefinition {
    CardDefinition {
        name: "Ugin's Mastery",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(colorless_obj()) },
                ),
                effect: Effect::Manifest { who: PlayerRef::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                    .with_filter(Predicate::AttackedWithTotalPowerAtLeast { who: PlayerRef::You, at_least: 6 }),
                effect: Effect::MayDo {
                    description: "Turn a face-down creature you control face up?".into(),
                    body: Box::new(Effect::TurnFaceUpFree {
                        what: Selector::Take {
                            inner: Box::new(Selector::EachPermanent(R::FaceDown.and(R::ControlledByYou))),
                            count: Box::new(Value::ONE),
                        },
                        if_cant: None,
                    }),
                },
            },
        ],
        ..Default::default()
    }
}
