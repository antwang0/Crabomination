//! A second Foundations wave — a can't-lose Angel, a Rat swarm, a Goblin lord,
//! and death-matters value. Tests in `crabomination/src/tests/recent163.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{discard, each_opponent_creature, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, RevealMissDest, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Herald of Eternal Dawn — {4}{W}{W}{W} 6/6 Angel. Flash, flying. You can't
/// lose the game and your opponents can't win the game.
pub fn herald_of_eternal_dawn() -> CardDefinition {
    CardDefinition {
        name: "Herald of Eternal Dawn",
        cost: cost(&[generic(4), w(), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You can't lose the game and your opponents can't win the game.",
            effect: StaticEffect::ControllerCantLoseGame,
        }],
        ..Default::default()
    }
}

/// Rune-Sealed Wall — {2}{U} 0/6 Wall. Defender. {T}: Surveil 1.
pub fn rune_sealed_wall() -> CardDefinition {
    CardDefinition {
        name: "Rune-Sealed Wall",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 6,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Scrawling Crawler — {3} 3/2 Phyrexian Construct. At the beginning of your
/// upkeep, each player draws a card. Whenever an opponent draws a card, they
/// lose 1 life.
pub fn scrawling_crawler() -> CardDefinition {
    CardDefinition {
        name: "Scrawling Crawler",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Construct],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::Draw {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Revenge of the Rats — {2}{B}{B} Sorcery. Create a tapped 1/1 black Rat token
/// for each creature card in your graveyard. Flashback {2}{B}{B}.
pub fn revenge_of_the_rats() -> CardDefinition {
    CardDefinition {
        name: "Revenge of the Rats",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(2), b(), b()]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::CardsInGraveyardMatching {
                who: PlayerRef::You,
                filter: R::Creature,
            },
            definition: TokenDefinition {
                name: "Rat".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Black],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Rat],
                    ..Default::default()
                },
                tapped: true,
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Spinner of Souls — {2}{G} 4/3 Spider Spirit. Reach. Whenever another nontoken
/// creature you control dies, you may reveal cards from the top of your library
/// until you reveal a creature card, put it into your hand, rest on the bottom.
pub fn spinner_of_souls() -> CardDefinition {
    CardDefinition {
        name: "Spinner of Souls",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider, CreatureType::Spirit],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken,
                },
            ),
            effect: Effect::MayDo {
                description: "Dig for a creature card".into(),
                body: Box::new(Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: R::Creature,
                    to: ZoneDest::Hand(PlayerRef::You),
                    cap: Value::Const(60),
                    miss_dest: RevealMissDest::BottomRandom,
                    life_per_revealed: 0,
                }),
            },
        }],
        ..Default::default()
    }
}

/// High-Society Hunter — {3}{B}{B} 5/3 Vampire Noble. Flying. Whenever it
/// attacks, you may sacrifice another creature to put a +1/+1 counter on it.
/// Whenever another nontoken creature dies, draw a card.
pub fn high_society_hunter() -> CardDefinition {
    CardDefinition {
        name: "High-Society Hunter",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Noble],
            ..Default::default()
        },
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            on_attack(Effect::MaySacrifice {
                description: "Sacrifice another creature to grow the Hunter?".into(),
                filter: R::Creature.and(R::OtherThanSource),
                count: Value::ONE,
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: None,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::NotToken.and(R::OtherThanSource),
                    },
                ),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Dropkick Bomber — {2}{R} 2/3 Goblin Warrior. Other Goblins you control get
/// +1/+1. {R}: Another target Goblin you control gains flying until end of turn.
/// (The granted "sacrifice on combat damage" rider is dropped.)
pub fn dropkick_bomber() -> CardDefinition {
    CardDefinition {
        name: "Dropkick Bomber",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Other Goblins you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Goblin)
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    R::HasCreatureType(CreatureType::Goblin)
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Seeker's Folly — {2}{B} Sorcery. Choose one — target opponent discards two
/// cards; or creatures your opponents control get -1/-1 until end of turn.
pub fn seekers_folly() -> CardDefinition {
    CardDefinition {
        name: "Seeker's Folly",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            discard(Selector::Player(PlayerRef::EachOpponent), 2, false),
            Effect::PumpPT {
                what: each_opponent_creature(),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}
