//! Darksteel gap batch: the indestructible artifacts, the colour-watch
//! Horns/Feathers, and the charge-counter engine. Tests in `recent_b/dst`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Selector, SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Predicate, StaticAbility, StaticEffect, ZoneDest,
};
use crate::mana::{b, cost, generic, r, u, w, Color, ManaCost};

/// A vanilla artifact shell.
fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

/// The DST "Horn/Feather" cycle: {2} artifact, "whenever a player casts a
/// [color] spell, you may gain 1 life."
fn color_watch_horn(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(color),
                },
            ),
            effect: Effect::MayDo {
                description: "gain 1 life".into(),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..artifact(name, cost(&[generic(2)]))
    }
}

/// Angel's Feather — {2} Artifact. Whenever a player casts a white spell, you
/// may gain 1 life.
pub fn angels_feather() -> CardDefinition {
    color_watch_horn("Angel's Feather", Color::White)
}

/// Demon's Horn — {2} Artifact. Whenever a player casts a black spell, you may
/// gain 1 life.
pub fn demons_horn() -> CardDefinition {
    color_watch_horn("Demon's Horn", Color::Black)
}

/// Darksteel Pendant — {2} indestructible Artifact. {1}, {T}: Scry 1.
pub fn darksteel_pendant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Indestructible],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..artifact("Darksteel Pendant", cost(&[generic(2)]))
    }
}

/// Darksteel Brute — {2} indestructible Artifact. {3}: it becomes a 2/2 Beast
/// artifact creature until end of turn.
pub fn darksteel_brute() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Indestructible],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                creature_types: vec![CreatureType::Beast],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact("Darksteel Brute", cost(&[generic(2)]))
    }
}

/// Darksteel Forge — {9} Artifact. Artifacts you control have indestructible.
pub fn darksteel_forge() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Artifacts you control have indestructible.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                keyword: Keyword::Indestructible,
            },
        }],
        ..artifact("Darksteel Forge", cost(&[generic(9)]))
    }
}

/// Darksteel Gargoyle — {7} 3/3 indestructible artifact Gargoyle with flying.
pub fn darksteel_gargoyle() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Gargoyle],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Indestructible],
        ..artifact("Darksteel Gargoyle", cost(&[generic(7)]))
    }
}

/// Arcane Spyglass — {4} Artifact. {2}, {T}, Sacrifice a land: draw a card and
/// charge it. Remove three charge counters: draw a card.
pub fn arcane_spyglass() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_other_filter: Some((R::Land, 1)),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Charge,
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                condition: Some(Predicate::ValueAtLeast(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Charge,
                    },
                    Value::Const(3),
                )),
                effect: Effect::Seq(vec![
                    Effect::RemoveCounter {
                        what: Selector::This,
                        kind: CounterType::Charge,
                        amount: Value::Const(3),
                    },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
                ..Default::default()
            },
        ],
        ..artifact("Arcane Spyglass", cost(&[generic(4)]))
    }
}

/// Coretapper — {2} 1/1 artifact Myr. {T}: charge an artifact; sacrifice it to
/// charge twice.
pub fn coretapper() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Myr], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: target_filtered(R::Artifact),
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                sac_cost: true,
                effect: Effect::AddCounter {
                    what: target_filtered(R::Artifact),
                    kind: CounterType::Charge,
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..artifact("Coretapper", cost(&[generic(2)]))
    }
}

/// Drill-Skimmer — {4} 2/1 artifact Thopter with flying; it has shroud while
/// you control another artifact creature.
pub fn drill_skimmer() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "This creature has shroud as long as you control another artifact creature.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::Artifact
                            .and(R::Creature)
                            .and(R::ControlledByYou)
                            .and(R::OtherThanSource),
                    ),
                    n: Value::ONE,
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Shroud],
            },
        }],
        ..artifact("Drill-Skimmer", cost(&[generic(4)]))
    }
}

/// Dross Golem — {5} 3/2 artifact Golem with fear and affinity for Swamps.
pub fn dross_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Fear],
        affinity_filter: Some(R::HasLandType(crate::card::LandType::Swamp)),
        ..artifact("Dross Golem", cost(&[generic(5)]))
    }
}

/// Auriok Glaivemaster — {W} 1/1 Human Soldier that gets +1/+1 and first strike
/// while equipped.
pub fn auriok_glaivemaster() -> CardDefinition {
    CardDefinition {
        name: "Auriok Glaivemaster",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is equipped, it gets +1/+1 and has first strike.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SourceIsEquipped,
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::FirstStrike],
            },
        }],
        ..Default::default()
    }
}

/// Crazed Goblin — {R} 1/1 Goblin Warrior that attacks each combat if able.
pub fn crazed_goblin() -> CardDefinition {
    CardDefinition {
        name: "Crazed Goblin",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::MustAttack],
        ..Default::default()
    }
}

/// Chittering Rats — {1}{B}{B} 2/2 Rat. ETB: target opponent puts a card from
/// their hand on top of their library.
pub fn chittering_rats() -> CardDefinition {
    CardDefinition {
        name: "Chittering Rats",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MoveChosen {
                from: Selector::CardsInZone {
                    who: PlayerRef::Target(0),
                    zone: crate::card::Zone::Hand,
                    filter: R::Any,
                },
                filter: None,
                count: Value::ONE,
                up_to: false,
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: LibraryPosition::Top,
                },
            },
        }],
        ..Default::default()
    }
}

/// Burden of Greed — {3}{B} Instant. Target player loses 1 life for each tapped
/// artifact they control.
pub fn burden_of_greed() -> CardDefinition {
    CardDefinition {
        name: "Burden of Greed",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::LoseLife {
            who: target_filtered(R::Player),
            amount: Value::count(Selector::ControlledBy {
                who: PlayerRef::Target(0),
                filter: R::Artifact.and(R::Tapped),
            }),
        },
        ..Default::default()
    }
}

/// Carry Away — {U}{U} Aura. Enchant Equipment; it unattaches on arrival and
/// you control it.
pub fn carry_away() -> CardDefinition {
    CardDefinition {
        name: "Carry Away",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment)),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Unattach { what: Selector::AttachedTo(Box::new(Selector::This)) },
                Effect::GainControl {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    to: Some(PlayerRef::You),
                    duration: Duration::Permanent,
                },
            ]),
        }],
        ..Default::default()
    }
}
