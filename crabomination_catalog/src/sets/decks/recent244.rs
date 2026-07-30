//! MKM (Murders at Karlov Manor) gap batch — Detectives, Disguise, and
//! investigate value. Tests in `tests/recent_b/recent244.rs`.

use crate::card::{AdditionalCastCost, ArtifactSubtype, EventKind, EventScope, EventSpec};
use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Keyword, SelectionRequirement as R,
    Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{investigate, target_filtered};
use crate::effect::{Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, x};

/// Wispdrinker Vampire — {2}{W}{B} Creature — Vampire Rogue 2/4, flying. Whenever
/// another creature you control with power 2 or less enters, drain 1. {5}{W}{B}:
/// creatures you control with power 2 or less gain deathtouch and lifelink.
pub fn wispdrinker_vampire() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::Duration;
    CardDefinition {
        name: "Wispdrinker Vampire",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::PowerAtMost(2)),
                }),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), w(), b()]),
            effect: Effect::GrantKeywords {
                what: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::PowerAtMost(2)),
                ),
                keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Torch the Witness — {X}{R} Sorcery. Deal twice X damage to target creature.
/// If excess damage was dealt this way, investigate.
pub fn torch_the_witness() -> CardDefinition {
    CardDefinition {
        name: "Torch the Witness",
        cost: cost(&[x(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(2))),
            },
            Effect::If {
                cond: Predicate::ExcessDamageDealtThisResolution,
                then: Box::new(investigate(1)),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Extract a Confession — {1}{B} Sorcery. Optional additional cost: collect
/// evidence 6. Each opponent sacrifices a creature of their choice — the
/// greatest-power one if evidence was collected.
pub fn extract_a_confession() -> CardDefinition {
    CardDefinition {
        name: "Extract a Confession",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::CollectEvidence {
            amount: 6,
            optional: true,
        }],
        effect: Effect::If {
            cond: Predicate::SpellCollectedEvidence,
            then: Box::new(Effect::SacrificeGreatestMV {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Creature,
                by_power: true,
            }),
            else_: Box::new(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Creature,
            }),
        },
        ..Default::default()
    }
}

/// Vitu-Ghazi Inspector — {1}{G} Creature — Elf Detective 1/3, reach. Optional
/// additional cost: collect evidence 6. ETB: if evidence was collected, put a
/// +1/+1 counter on target creature and gain 2 life.
pub fn vitu_ghazi_inspector() -> CardDefinition {
    CardDefinition {
        name: "Vitu-Ghazi Inspector",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Detective],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        additional_cast_cost: vec![AdditionalCastCost::CollectEvidence {
            amount: 6,
            optional: true,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            // "if evidence was collected" is read at resolution off the source's
            // cast flag (CR 701.59), so gate the body rather than the event.
            effect: Effect::If {
                cond: Predicate::SpellCollectedEvidence,
                then: Box::new(Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(R::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::Const(2),
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Curious Cadaver — {2}{U}{B} Creature — Zombie Detective 3/1, flying. When you
/// sacrifice a Clue, return this card from your graveyard to your hand.
pub fn curious_cadaver() -> CardDefinition {
    CardDefinition {
        name: "Curious Cadaver",
        cost: cost(&[generic(2), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Detective],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PermanentSacrificed,
                EventScope::FromYourGraveyard,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasArtifactSubtype(ArtifactSubtype::Clue),
            }),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// They Went This Way — {2}{G} Sorcery. Search your library for a basic land,
/// put it onto the battlefield tapped, then shuffle. Investigate.
pub fn they_went_this_way() -> CardDefinition {
    CardDefinition {
        name: "They Went This Way",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
            investigate(1),
        ]),
        ..Default::default()
    }
}

/// Undercover Crocodelf — {4}{G}{U} Creature — Elf Crocodile Detective 5/5.
/// Whenever it deals combat damage to a player, investigate. Disguise {3}{G/U}{G/U}.
pub fn undercover_crocodelf() -> CardDefinition {
    use crate::mana::{Color, hybrid};
    CardDefinition {
        name: "Undercover Crocodelf",
        cost: cost(&[generic(4), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Elf,
                CreatureType::Crocodile,
                CreatureType::Detective,
            ],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Disguise(cost(&[
            generic(3),
            hybrid(Color::Green, Color::Blue),
            hybrid(Color::Green, Color::Blue),
        ]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: investigate(1),
        }],
        ..Default::default()
    }
}

/// Sharp-Eyed Rookie — {1}{G} Creature — Human Detective 2/2, vigilance.
/// Whenever a creature you control with greater power or toughness enters, put a
/// +1/+1 counter on this creature and investigate.
pub fn sharp_eyed_rookie() -> CardDefinition {
    // "power greater than this" OR "toughness greater than this" ==
    // NOT (power ≤ this AND toughness ≤ this).
    let bigger = Predicate::Not(Box::new(Predicate::All(vec![
        Predicate::ValueAtMost(
            Value::PowerOf(Box::new(Selector::TriggerSource)),
            Value::PowerOf(Box::new(Selector::This)),
        ),
        Predicate::ValueAtMost(
            Value::ToughnessOf(Box::new(Selector::TriggerSource)),
            Value::ToughnessOf(Box::new(Selector::This)),
        ),
    ])));
    CardDefinition {
        name: "Sharp-Eyed Rookie",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Detective],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::All(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    },
                    bigger,
                ])),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                investigate(1),
            ]),
        }],
        ..Default::default()
    }
}
