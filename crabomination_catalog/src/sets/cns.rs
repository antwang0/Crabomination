//! Conspiracy (CNS / CN2) — CR 315 conspiracy cards. They start in the
//! command zone and never leave; hidden-agenda ones start face down with a
//! secretly chosen card name (`GameState::seat_conspiracy`). Tests in
//! `classic_sets/cns`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::{Effect, PlayerRef, StaticEffect, Value};
use crate::game::types::TurnStep;
use crate::mana::Color;

fn conspiracy(name: &'static str) -> CardDefinition {
    CardDefinition { name, card_types: vec![CardType::Conspiracy], ..Default::default() }
}

/// "At the beginning of the first upkeep of the game, …"
fn first_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
            .with_filter(Predicate::ValueAtMost(Value::TurnNumber, Value::ONE)),
        effect,
    }
}

fn token(name: &str, p: i32, t: i32, ct: CreatureType, colors: Vec<Color>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
        ..Default::default()
    }
}

// ── Face-up conspiracies ───────────────────────────────────────────────────

/// Power Play — you are the starting player.
pub fn power_play() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You are the starting player.",
            effect: StaticEffect::ControllerIsStartingPlayer,
        }],
        ..conspiracy("Power Play")
    }
}

/// Hymn of the Wilds — a creature discount bought with your instants and
/// sorceries.
pub fn hymn_of_the_wilds() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "The first creature spell you cast each turn costs {1} less.",
                effect: StaticEffect::CostReductionFirstCreatureSpell { amount: 1 },
            },
            StaticAbility {
                description: "You can't cast instant or sorcery spells.",
                effect: StaticEffect::ControllerCantCastInstantsOrSorceries,
            },
        ],
        ..conspiracy("Hymn of the Wilds")
    }
}

/// Weight Advantage — your creatures hit as hard as they are tough.
pub fn weight_advantage() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Your creatures assign combat damage equal to their toughness.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature,
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::AssignsCombatDamageByToughness],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..conspiracy("Weight Advantage")
    }
}

/// Sentinel Dispatch — a free wall on the game's first upkeep.
pub fn sentinel_dispatch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![first_upkeep(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                card_types: vec![CardType::Artifact, CardType::Creature],
                keywords: vec![Keyword::Defender],
                ..token("Construct", 1, 1, CreatureType::Construct, vec![])
            },
        })],
        ..conspiracy("Sentinel Dispatch")
    }
}

/// Hold the Perimeter — you get a blocker, everyone else gets a Goblin that
/// can't block.
pub fn hold_the_perimeter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            first_upkeep(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    keywords: vec![Keyword::Defender],
                    ..token("Soldier", 1, 2, CreatureType::Soldier, vec![Color::White])
                },
            }),
            first_upkeep(Effect::CreateToken {
                who: PlayerRef::EachOpponent,
                count: Value::ONE,
                definition: TokenDefinition {
                    keywords: vec![Keyword::CantBlock],
                    ..token("Goblin", 1, 1, CreatureType::Goblin, vec![Color::Red])
                },
            }),
        ],
        ..conspiracy("Hold the Perimeter")
    }
}

// ── Hidden agenda ──────────────────────────────────────────────────────────

/// Brago's Favor — spells with the chosen name cost {1} less.
pub fn bragos_favor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Spells with the chosen name you cast cost {1} less.",
            effect: StaticEffect::NamedSpellCostReduction { amount: 1 },
        }],
        ..conspiracy("Brago's Favor")
    }
}

/// Immediate Action — creatures with the chosen name have haste.
pub fn immediate_action() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with the chosen name have haste.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::NamedBySource),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Haste],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..conspiracy("Immediate Action")
    }
}

/// Iterative Analysis — casting an instant or sorcery with the chosen name
/// draws you a card.
pub fn iterative_analysis() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: crate::effect::Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Instant)
                        .or(R::HasCardType(CardType::Sorcery))
                        .and(R::NamedBySource),
                },
            ),
            effect: Effect::Draw { who: crate::effect::Selector::You, amount: Value::ONE },
        }],
        ..conspiracy("Iterative Analysis")
    }
}

/// Muzzio's Preparations — creatures with the chosen name enter bigger.
pub fn muzzios_preparations() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with the chosen name enter with an extra \
                          +1/+1 counter.",
            effect: StaticEffect::MatchingEntersWithExtraCounters {
                filter: R::Creature.and(R::NamedBySource),
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: 1,
            },
        }],
        ..conspiracy("Muzzio's Preparations")
    }
}
