//! BLB/FDN gap batch riding the mana-value-vs-trigger-event primitives:
//! Jackdaw Savior (flyer-dies reanimate lesser-MV), Clement, the Worrywort
//! (creature-enters bounce lesser-MV), Soul-Shackled Zombie (single-graveyard
//! exile → creature-exiled drain). Tests in `tests/recent193.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, Subtypes,
    Supertype, Value,
};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, TriggeredAbility,
    ZoneDest,
};
use crate::mana::{b, cost, g, generic, u, w};

/// Jackdaw Savior — {2}{W} 3/1 Bird Cleric, Flying. When this or another flying
/// creature you control dies, return another target creature card with lesser
/// mana value from your graveyard to the battlefield.
pub fn jackdaw_savior() -> CardDefinition {
    CardDefinition {
        name: "Jackdaw Savior",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasKeyword(Keyword::Flying)),
                },
            ),
            effect: Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature
                        .and(R::InYourGraveyard)
                        .and(R::ManaValueLessThanEventAmount),
                },
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        }],
        ..Default::default()
    }
}

/// Clement, the Worrywort — {1}{G}{U} 3/3 Legendary Frog Druid, Vigilance. When
/// Clement or another creature you control enters, return up to one target
/// creature you control with lesser mana value to its owner's hand. (The
/// Frog-mana-granting static is omitted — no granted-mana-ability primitive.)
pub fn clement_the_worrywort() -> CardDefinition {
    CardDefinition {
        name: "Clement, the Worrywort",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::ControlledByYou),
                }),
            effect: Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature
                        .and(R::ControlledByYou)
                        .and(R::ManaValueLessThanEventAmount),
                },
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..Default::default()
    }
}

/// Soul-Shackled Zombie — {3}{B} 4/2 Zombie. When this enters, exile up to two
/// target cards from a single graveyard. If at least one creature card was
/// exiled this way, each opponent loses 2 life and you gain 2 life.
pub fn soul_shackled_zombie() -> CardDefinition {
    CardDefinition {
        name: "Soul-Shackled Zombie",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ExileUpToNFromGraveyards {
                    count: Value::Const(2),
                    of: None,
                    single: true,
                },
                Effect::If {
                    cond: Predicate::EntityMatchesAny {
                        what: Selector::LastMoved,
                        filter: R::Creature,
                    },
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
            ]),
        }],
        ..Default::default()
    }
}
