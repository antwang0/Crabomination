//! Murders at Karlov Manor / Bloomburrow batch — a token-tapping legend
//! (Baylen), two small-creature +1/+1 payoffs (Haazda Vigilante, Neighborhood
//! Guardian), a graveyard-hate flyer (Griffnaut Tracker), a self-suspecting
//! attacker (Rubblebelt Braggart), and a modal artifact-hater (Gearbane
//! Orangutan). Tests in `tests/recent218.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Predicate,
    Selector, Value,
};

/// Baylen, the Haymaker — {R}{G}{W} 4/3 Rabbit Warrior. Tap two/three/four
/// untapped tokens you control to add any-color mana, draw a card, or pump.
pub fn baylen_the_haymaker() -> CardDefinition {
    let tap_tokens = |n: u32, effect: Effect| ActivatedAbility {
        tap_n_filter: Some((R::IsToken.and(R::ControlledByYou), n)),
        effect,
        ..Default::default()
    };
    CardDefinition {
        name: "Baylen, the Haymaker",
        cost: crate::mana::cost(&[crate::mana::r(), crate::mana::g(), crate::mana::w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        activated_abilities: vec![
            tap_tokens(2, Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) }),
            tap_tokens(3, Effect::Draw { who: Selector::You, amount: Value::ONE }),
            tap_tokens(4, Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(3) },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            ])),
        ],
        ..Default::default()
    }
}

/// Haazda Vigilante — {4}{W} 4/4 Giant Soldier. Enters or attacks → a +1/+1
/// counter on a target creature you control with power 2 or less.
pub fn haazda_vigilante() -> CardDefinition {
    let boost = || Effect::AddCounter {
        what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::PowerAtMost(2))),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    };
    CardDefinition {
        name: "Haazda Vigilante",
        cost: crate::mana::cost(&[crate::mana::generic(4), crate::mana::w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(boost()), on_attack(boost())],
        ..Default::default()
    }
}

/// Neighborhood Guardian — {1}{W} 2/2 Unicorn. Whenever another creature you
/// control with power 2 or less enters, a target creature you control gets
/// +1/+1 until end of turn.
pub fn neighborhood_guardian() -> CardDefinition {
    CardDefinition {
        name: "Neighborhood Guardian",
        cost: crate::mana::cost(&[crate::mana::generic(1), crate::mana::w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Unicorn], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::PowerAtMost(2)),
                }),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Griffnaut Tracker — {3}{W} 3/2 Human Detective. Flying. When it enters, exile
/// up to two target cards from a single graveyard.
pub fn griffnaut_tracker() -> CardDefinition {
    CardDefinition {
        name: "Griffnaut Tracker",
        cost: crate::mana::cost(&[crate::mana::generic(3), crate::mana::w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Detective],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ExileUpToNFromGraveyards {
            count: Value::Const(2),
            of: None,
            single: true,
        })],
        ..Default::default()
    }
}

/// Rubblebelt Braggart — {4}{R} 5/5 Lizard Warrior. Whenever it attacks, if it's
/// not suspected, you may suspect it (menace + can't block).
pub fn rubblebelt_braggart() -> CardDefinition {
    CardDefinition {
        name: "Rubblebelt Braggart",
        cost: crate::mana::cost(&[crate::mana::generic(4), crate::mana::r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![on_attack(Effect::If {
            cond: Predicate::Not(Box::new(Predicate::SourceIsSuspected)),
            then: Box::new(Effect::MayDo {
                description: "Suspect this creature?".into(),
                body: Box::new(Effect::Suspect { what: Selector::This }),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Gearbane Orangutan — {2}{R} 2/2 Ape. Reach. When it enters, choose one —
/// destroy up to one target artifact; or sacrifice an artifact for two +1/+1
/// counters on it.
pub fn gearbane_orangutan() -> CardDefinition {
    CardDefinition {
        name: "Gearbane Orangutan",
        cost: crate::mana::cost(&[crate::mana::generic(2), crate::mana::r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ape], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Artifact) },
            Effect::MaySacrifice {
                description: "Sacrifice an artifact for two +1/+1 counters?".into(),
                filter: R::Artifact,
                count: Value::ONE,
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                }),
                else_: None,
            },
        ]))],
        ..Default::default()
    }
}
