//! A Bloomburrow common/uncommon wave — cards missing from the catalog that
//! ride existing primitives (Offspring, Expend, Gift-free value, count-matters
//! ETBs). Tests in `crabomination/src/tests/recent157.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_dies, on_you_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u};

/// Darkstar Augur — {2}{B} 2/3 Bat Warlock with flying and Offspring {B}. At
/// the beginning of your upkeep, put the top card of your library into your
/// hand and lose life equal to its mana value.
pub fn darkstar_augur() -> CardDefinition {
    CardDefinition {
        name: "Darkstar Augur",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bat, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Offspring(cost(&[b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::ManaValueOf(Box::new(Selector::TopOfLibrary {
                        who: PlayerRef::You,
                        count: Value::ONE,
                    })),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Honored Dreyleader — {2}{G} 1/1 Squirrel Warrior with trample. Enters with a
/// +1/+1 counter for each other Squirrel and/or Food you control, and gains one
/// whenever another Squirrel or Food you control enters.
pub fn honored_dreyleader() -> CardDefinition {
    let squirrel_or_food =
        R::HasCreatureType(CreatureType::Squirrel).or(R::HasArtifactSubtype(ArtifactSubtype::Food));
    let count = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(R::Any)),
        filter: squirrel_or_food
            .clone()
            .and(R::ControlledByYou)
            .and(R::OtherThanSource),
    };
    CardDefinition {
        name: "Honored Dreyleader",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: Selector::This,
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: count,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: squirrel_or_food,
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Fecund Greenshell — {3}{G}{G} 4/6 Elemental Turtle with reach. Creatures you
/// control get +2/+2 while you control ten or more lands. Whenever this or
/// another creature you control with toughness greater than its power enters,
/// look at the top card of your library — put a land onto the battlefield
/// tapped, otherwise into your hand.
pub fn fecund_greenshell() -> CardDefinition {
    CardDefinition {
        name: "Fecund Greenshell",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Turtle],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        keywords: vec![Keyword::Reach],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Creatures you control get +2/+2 while you control ten or more lands.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::ValueAtLeast(
                    Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(R::Any)),
                        filter: R::Land.and(R::ControlledByYou),
                    },
                    Value::Const(10),
                ),
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: 2,
                toughness: 2,
                keywords: vec![],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::ToughnessGreaterThanPower),
                }),
            effect: Effect::RevealTopLandToBattlefieldElseHand {
                who: PlayerRef::You,
            },
        }],
        ..Default::default()
    }
}

/// Hazardroot Herbalist — {2}{G} 1/4 Rabbit Druid. Whenever you attack, target
/// creature you control gets +1/+0 until end of turn; if it's a token, it also
/// gains deathtouch until end of turn.
pub fn hazardroot_herbalist() -> CardDefinition {
    CardDefinition {
        name: "Hazardroot Herbalist",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![on_you_attack(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::IsToken,
                },
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Deathtouch,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..Default::default()
    }
}

/// Rust-Shield Rampager — {3}{G} 4/4 Raccoon Warrior with Offspring {2}. Can't
/// be blocked by creatures with power 2 or less.
pub fn rust_shield_rampager() -> CardDefinition {
    CardDefinition {
        name: "Rust-Shield Rampager",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Raccoon, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![
            Keyword::Offspring(cost(&[generic(2)])),
            Keyword::CantBeBlockedByPowerAtMost(2),
        ],
        ..Default::default()
    }
}

/// Seedpod Squire — {3}{W/U} 3/3 Bird Scout with flying. Whenever it attacks,
/// target creature you control without flying gets +1/+1 until end of turn.
pub fn seedpod_squire() -> CardDefinition {
    CardDefinition {
        name: "Seedpod Squire",
        cost: cost(&[generic(3), hybrid(Color::White, Color::Blue)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_you_attack(Effect::PumpPT {
            what: target_filtered(
                R::Creature
                    .and(R::ControlledByYou)
                    .and(R::HasKeyword(Keyword::Flying).negate()),
            ),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Steampath Charger — {1}{R} 2/1 Lizard Warlock with Offspring {2}. When it
/// dies, it deals 1 damage to target player.
pub fn steampath_charger() -> CardDefinition {
    CardDefinition {
        name: "Steampath Charger",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Offspring(cost(&[generic(2)]))],
        triggered_abilities: vec![on_dies(Effect::DealDamage {
            to: target_filtered(R::Player),
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Treeguard Duo — {3}{G} 3/4 Frog Rabbit. When it enters, target creature you
/// control gains vigilance and gets +X/+X until end of turn, where X is the
/// number of creatures you control.
pub fn treeguard_duo() -> CardDefinition {
    let creatures = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(R::Any)),
        filter: R::Creature.and(R::ControlledByYou),
    };
    CardDefinition {
        name: "Treeguard Duo",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Rabbit],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: creatures.clone(),
                toughness: creatures,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Junkblade Bruiser — {3}{R/G}{R/G} 4/5 Raccoon Berserker with trample.
/// Whenever you expend 4, it gets +2/+1 until end of turn.
pub fn junkblade_bruiser() -> CardDefinition {
    CardDefinition {
        name: "Junkblade Bruiser",
        cost: cost(&[
            generic(3),
            hybrid(Color::Red, Color::Green),
            hybrid(Color::Red, Color::Green),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Raccoon, CreatureType::Berserker],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Expend, EventScope::YourControl)
                .with_filter(Predicate::ExpendReached(4)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Waterspout Warden — {2}{U} 3/2 Frog Soldier. Whenever it attacks, if another
/// creature entered the battlefield under your control this turn, it gains
/// flying until end of turn.
pub fn waterspout_warden() -> CardDefinition {
    CardDefinition {
        name: "Waterspout Warden",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![on_you_attack(Effect::If {
            cond: Predicate::AnotherCreatureEnteredThisTurn {
                who: PlayerRef::You,
            },
            then: Box::new(Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}
