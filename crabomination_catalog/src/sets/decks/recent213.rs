//! Foundations (FDN) gap batch 12 — two +1/+1-counter Hydras (one on the new
//! `CounterAdded` trigger), a life-gated recursion sorcery, a land-destruction
//! utility land, a flash removal artifact, and Ajani, Caller of the Pride.
//! Tests in `tests/recent213.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, Keyword,
    LoyaltyAbility, SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, EventKind, EventScope, EventSpec, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, w, x};

/// Heroes' Bane — {3}{G}{G} 0/0 Hydra. Enters with four +1/+1 counters.
/// {2}{G}{G}: Put X +1/+1 counters on it, where X is its power.
pub fn heroes_bane() -> CardDefinition {
    CardDefinition {
        name: "Heroes' Bane",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hydra],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), g()]),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::PowerOf(Box::new(Selector::This)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wildwood Scourge — {X}{G} 0/0 Hydra. Enters with X +1/+1 counters. Whenever
/// one or more +1/+1 counters are put on another non-Hydra creature you
/// control, put a +1/+1 counter on this creature.
pub fn wildwood_scourge() -> CardDefinition {
    CardDefinition {
        name: "Wildwood Scourge",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hydra],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                EventScope::AnotherOfYours,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Hydra)))),
            }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Sanguine Indulgence — {3}{B} Sorcery. Costs {3} less if you've gained 3 or
/// more life this turn. Return up to two target creature cards from your
/// graveyard to your hand.
pub fn sanguine_indulgence() -> CardDefinition {
    CardDefinition {
        name: "Sanguine Indulgence",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Costs {3} less if you've gained 3 or more life this turn.",
            effect: crate::card::StaticEffect::SelfCostReducedIf {
                condition: Predicate::LifeGainedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(3),
                },
                amount: 3,
            },
        }],
        effect: Effect::ReturnGraveyardCardsToHand {
            filter: R::Creature,
            max: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Demolition Field — Land. {T}: Add {C}. {2}, {T}, Sacrifice this land:
/// Destroy target nonbasic land an opponent controls, then you may search your
/// library for a basic land, put it onto the battlefield, then shuffle.
pub fn demolition_field() -> CardDefinition {
    CardDefinition {
        name: "Demolition Field",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            super::super::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::Seq(vec![
                    Effect::Destroy {
                        what: target_filtered(R::IsNonbasicLand.and(R::ControlledByOpponent)),
                    },
                    Effect::Search {
                        who: PlayerRef::You,
                        filter: R::IsBasicLand,
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Goblin Firebomb — {1} Artifact. Flash; {7}, {T}, Sacrifice this artifact:
/// Destroy target permanent.
pub fn goblin_firebomb() -> CardDefinition {
    CardDefinition {
        name: "Goblin Firebomb",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Flash],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(7)]),
            effect: Effect::Destroy {
                what: target_filtered(R::Permanent),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ajani, Caller of the Pride — {1}{W}{W} Legendary Planeswalker, loyalty 4.
/// +1: Put a +1/+1 counter on up to one target creature. −3: Target creature
/// gains flying and double strike until end of turn. −8: Create X 2/2 white Cat
/// creature tokens, where X is your life total.
pub fn ajani_caller_of_the_pride() -> CardDefinition {
    let cat = TokenDefinition {
        name: "Cat".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Ajani, Caller of the Pride",
        cost: cost(&[generic(1), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_filtered(R::Creature),
                        keyword: Keyword::Flying,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::DoubleStrike,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::LifeOf(PlayerRef::You),
                    definition: cat,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
