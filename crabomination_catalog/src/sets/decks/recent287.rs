//! OTJ/DFT gap batch — Miriam (turn-gated Mount/Vehicle hexproof via the new
//! `StaticEffect::WhileYourTurn`), Vadmir (crime counters + counter-gated
//! keywords), Skyserpent Seeker (`Effect::RevealUntilLandsToBattlefield` ramp).
//! Tests in `recent_b/recent287`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    Keyword, SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TriggeredAbility,
    Value,
};
use crate::effect::{Effect, EventKind, EventScope, EventSpec, Predicate, Selector, StaticEffect};
use crate::mana::{b, cost, g, generic, u, w};

fn legendary_creature(types: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: types,
        ..Default::default()
    }
}

/// Miriam, Herd Whisperer — {G}{W} Legendary Creature — Human Druid 3/2.
/// During your turn, Mounts and Vehicles you control have hexproof. Whenever a
/// Mount or Vehicle you control attacks, put a +1/+1 counter on it.
pub fn miriam_herd_whisperer() -> CardDefinition {
    let mount_or_vehicle = R::HasCreatureType(CreatureType::Mount)
        .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))
        .and(R::ControlledByYou);
    CardDefinition {
        name: "Miriam, Herd Whisperer",
        cost: cost(&[g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: legendary_creature(vec![CreatureType::Human, CreatureType::Druid]),
        power: 3,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "During your turn, Mounts and Vehicles you control have hexproof.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(mount_or_vehicle.clone()),
                    keyword: Keyword::Hexproof,
                }),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: mount_or_vehicle,
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Vadmir, New Blood — {1}{B} Legendary Creature — Vampire Rogue 2/2.
/// Whenever you commit a crime, put a +1/+1 counter on Vadmir (once each turn).
/// As long as Vadmir has four or more +1/+1 counters, it has menace and lifelink.
pub fn vadmir_new_blood() -> CardDefinition {
    let counter_gated = |kw: Keyword| StaticAbility {
        description: "As long as Vadmir has four or more +1/+1 counters, it has menace and lifelink.",
        effect: StaticEffect::SelfHasKeywordWhileCountersAtLeast {
            kind: CounterType::PlusOnePlusOne,
            n: 4,
            keyword: kw,
        },
    };
    CardDefinition {
        name: "Vadmir, New Blood",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: legendary_creature(vec![CreatureType::Vampire, CreatureType::Rogue]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        static_abilities: vec![
            counter_gated(Keyword::Menace),
            counter_gated(Keyword::Lifelink),
        ],
        ..Default::default()
    }
}

/// Skyserpent Seeker — {G}{U} Creature — Snake 1/1 (DFT).
/// Flying, deathtouch. Exhaust — {4}: reveal cards from the top of your library
/// until you reveal two lands, put them onto the battlefield tapped and the rest
/// on the bottom in a random order, then put a +1/+1 counter on this creature.
pub fn skyserpent_seeker() -> CardDefinition {
    CardDefinition {
        name: "Skyserpent Seeker",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::RevealUntilLandsToBattlefield {
                    count: Value::Const(2),
                    tapped: true,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
