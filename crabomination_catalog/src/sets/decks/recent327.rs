//! Duskmourn / Bloomburrow gap batch — the cards previously blocked on one
//! engine primitive each. Tests in `tests/recent_b/recent327.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, OpeningHandEffect, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::on_attack;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{ManaCost, b, cost, g, generic, r, u};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn leyline(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        opening_hand: Some(OpeningHandEffect::StartInPlay { tapped: false, extra: Effect::Noop }),
        ..Default::default()
    }
}

/// Cursed Recording — {2}{R}{R} Artifact. Your instants and sorceries load it
/// with time counters (seven kills you for 20); {T} copies your next one.
pub fn cursed_recording() -> CardDefinition {
    CardDefinition {
        name: "Cursed Recording",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Time,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::SourceHasCountersAtLeast { counter: CounterType::Time, n: 7 },
                    then: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter {
                            what: Selector::This,
                            kind: CounterType::Time,
                            amount: Value::CountersOn {
                                what: Box::new(Selector::This),
                                kind: CounterType::Time,
                            },
                        },
                        Effect::DealDamage {
                            to: Selector::Player(PlayerRef::You),
                            amount: Value::Const(20),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::OnYourNextInstantSorceryThisTurn {
                body: Box::new(Effect::CopySpellMayChooseTargets {
                    what: Selector::TriggerSource,
                    count: Value::ONE,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Leyline of Resonance — {2}{R}{R}. Copies each of your instants and
/// sorceries that targets only a single creature you control.
pub fn leyline_of_resonance() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::All(vec![
                    Predicate::CastSpellMatches(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                    Predicate::CastSpellTargetsOnlyOneMatching(
                        R::Creature.and(R::ControlledByYou),
                    ),
                ]),
            ),
            effect: Effect::CopySpellMayChooseTargets {
                what: Selector::TriggerSource,
                count: Value::ONE,
            },
        }],
        ..leyline("Leyline of Resonance", cost(&[generic(2), r(), r()]))
    }
}

/// Leyline of Transformation — {2}{U}{U}. The chosen creature type applies to
/// your creatures on the battlefield and to every creature card you own
/// elsewhere.
pub fn leyline_of_transformation() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control are the chosen type in addition to their \
                              other types.",
                effect: StaticEffect::MatchingAreChosenTypeToo {
                    filter: R::Creature.and(R::ControlledByYou),
                },
            },
            StaticAbility {
                description: "The same is true for creature spells you control and creature \
                              cards you own that aren't on the battlefield.",
                effect: StaticEffect::OwnedCardsOffBattlefieldAreChosenTypeToo {
                    filter: R::Creature,
                },
            },
        ],
        ..leyline("Leyline of Transformation", cost(&[generic(2), u(), u()]))
    }
}

/// Hedge Shredder — {2}{G}{G} 5/5 Vehicle, crew 1. Attacking mills two, and
/// any land you mill goes straight onto the battlefield tapped.
pub fn hedge_shredder() -> CardDefinition {
    CardDefinition {
        name: "Hedge Shredder",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Crew(1)],
        triggered_abilities: vec![
            on_attack(Effect::MayDo {
                description: "Mill two cards?".into(),
                body: Box::new(Effect::Mill {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::Const(2),
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardMilled, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land },
                ),
                effect: Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
            },
        ],
        ..Default::default()
    }
}

/// Undead Sprinter — {B}{R} 2/2 trample haste. Castable from your graveyard
/// once a non-Zombie creature has died this turn, entering bigger for it.
pub fn undead_sprinter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste, Keyword::GraveyardCast],
        flashback_condition: Some(Predicate::CreatureDiedThisTurnMatching {
            filter: R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Zombie)))),
        }),
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::CastFromGraveyard),
                then: Box::new(Value::ONE),
                else_: Box::new(Value::Const(0)),
            },
        )),
        ..creature("Undead Sprinter", cost(&[b(), r()]), vec![CreatureType::Zombie], 2, 2)
    }
}
