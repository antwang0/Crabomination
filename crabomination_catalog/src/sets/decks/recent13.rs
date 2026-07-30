//! A thirteenth wave — assorted recent-set singles (graveyard hate, top-of-
//! library value, oil-counter clocks, a Triome, a manland). Tests in
//! `crabomination/src/tests/recent13.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, SelectionRequirement, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value, Zone,
};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, u};

/// Misery's Shadow — {1}{B} Shade 2/2. If a creature an opponent controls would
/// die, exile it instead. {1}: this creature gets +1/+1 until end of turn.
pub fn miserys_shadow() -> CardDefinition {
    CardDefinition {
        name: "Misery's Shadow",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shade],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "If a creature an opponent controls would die, exile it instead.",
            effect: StaticEffect::ExileDyingOpponentCreatures { when_you_do: None },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Glarb, Calamity's Augur — {B}{G}{U} Legendary Frog Wizard Noble 2/4 with
/// deathtouch. You may play lands and cast spells with mana value 4 or greater
/// from the top of your library. {T}: Surveil 2.
pub fn glarb_calamitys_augur() -> CardDefinition {
    CardDefinition {
        name: "Glarb, Calamity's Augur",
        cost: cost(&[b(), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Frog,
                CreatureType::Wizard,
                CreatureType::Noble,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Deathtouch],
        static_abilities: vec![StaticAbility {
            description: "You may play lands and cast spells with mana value 4 or greater from the top of your library.",
            effect: StaticEffect::PlayFromLibraryTop {
                filter: SelectionRequirement::Land.or(SelectionRequirement::ManaValueAtLeast(4)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Archfiend of the Dross — {2}{B}{B} Phyrexian Demon 6/6 flying. Enters with
/// four oil counters. At your upkeep, remove an oil counter; if it then has
/// none, you lose the game. Whenever a creature an opponent controls dies, its
/// controller loses 2 life.
pub fn archfiend_of_the_dross() -> CardDefinition {
    CardDefinition {
        name: "Archfiend of the Dross",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Demon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::Oil, Value::Const(4))),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::Seq(vec![
                    Effect::RemoveCounter {
                        what: Selector::This,
                        kind: CounterType::Oil,
                        amount: Value::ONE,
                    },
                    Effect::If {
                        cond: Predicate::ValueAtMost(
                            Value::CountersOn {
                                what: Box::new(Selector::This),
                                kind: CounterType::Oil,
                            },
                            Value::Const(0),
                        ),
                        then: Box::new(Effect::LoseGame {
                            who: PlayerRef::You,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
            TriggeredAbility {
                // "Its controller loses 2 life." The scope already restricts to
                // opponents' creatures, so the loser is an opponent; `EachOpponent`
                // is exact in 1v1 (a dead creature's `ControllerOf` no longer
                // resolves from the graveyard).
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
            },
        ],
        ..Default::default()
    }
}

/// Seeds of Renewal — {6}{G} Sorcery with Undaunted. Return up to two target
/// cards from your graveyard to your hand, then exile Seeds of Renewal. (The
/// two returns auto-pick from the graveyard — no multi-target prompt.)
pub fn seeds_of_renewal() -> CardDefinition {
    CardDefinition {
        name: "Seeds of Renewal",
        cost: cost(&[generic(6), g()]),
        card_types: vec![CardType::Sorcery],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each opponent.",
            effect: StaticEffect::SelfCostReducedPerOpponent { per: 1 },
        }],
        effect: Effect::Move {
            what: Selector::take(
                Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Any,
                },
                Value::Const(2),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        exile_on_resolve: true,
        ..Default::default()
    }
}

/// Spara's Headquarters — Triome land (Forest Plains Island). Taps for G/W/U,
/// enters tapped, Cycling {3}.
pub fn sparas_headquarters() -> CardDefinition {
    super::modern::triome(
        "Spara's Headquarters",
        [LandType::Forest, LandType::Plains, LandType::Island],
        [Color::Green, Color::White, Color::Blue],
    )
}

/// Mishra's Foundry — colorless manland. {T}: Add {C}. {2}: becomes a 2/2
/// Assembly-Worker until end of turn (still a land). (The "pump an attacking
/// Assembly-Worker" rider is dropped, as on Mishra's Factory.)
pub fn mishras_foundry() -> CardDefinition {
    super::lands::colorless_manland(
        "Mishra's Foundry",
        cost(&[generic(2)]),
        2,
        2,
        vec![CreatureType::AssemblyWorker],
        vec![],
    )
}
