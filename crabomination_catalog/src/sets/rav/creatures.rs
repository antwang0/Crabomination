use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::mana::{cost, g, generic, r, w};

/// Boros Swiftblade — {R}{W} 1/2 Human Soldier with double strike.
pub fn boros_swiftblade() -> CardDefinition {
    CardDefinition {
        name: "Boros Swiftblade",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::DoubleStrike],
        ..Default::default()
    }
}

/// Courier Hawk — {1}{W} 1/2 Bird with flying and vigilance.
pub fn courier_hawk() -> CardDefinition {
    CardDefinition {
        name: "Courier Hawk",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..Default::default()
    }
}

/// Barbarian Riftcutter — {4}{R} 3/3 Human Barbarian. `{R}, Sacrifice this
/// creature: Destroy target land.`
pub fn barbarian_riftcutter() -> CardDefinition {
    CardDefinition {
        name: "Barbarian Riftcutter",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Barbarian],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Land) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dromad Purebred — {4}{W} 1/5 Camel Beast. Whenever it's dealt damage, you
/// gain 1 life.
pub fn dromad_purebred() -> CardDefinition {
    CardDefinition {
        name: "Dromad Purebred",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Camel, CreatureType::Beast],
            ..Default::default()
        },
        power: 1,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Conclave Equenaut — {4}{W}{W} 3/3 Human Soldier with convoke and flying.
pub fn conclave_equenaut() -> CardDefinition {
    CardDefinition {
        name: "Conclave Equenaut",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Convoke, Keyword::Flying],
        ..Default::default()
    }
}

/// Gate Hound — {2}{W} 1/1 Dog. Creatures you control have vigilance as long as
/// this creature is enchanted.
pub fn gate_hound() -> CardDefinition {
    CardDefinition {
        name: "Gate Hound",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have vigilance as long as this creature is enchanted.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::IsEnchanted },
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Vigilance],
            },
        }],
        ..Default::default()
    }
}

/// Watchwolf — {G}{W} 3/3
pub fn watchwolf() -> CardDefinition {
    CardDefinition {
        name: "Watchwolf",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wolf],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        ..Default::default()
    }
}
