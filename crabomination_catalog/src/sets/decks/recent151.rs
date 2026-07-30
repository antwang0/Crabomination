//! A cross-set wave (DSK / OTJ / BLB): a Delirium self-reanimator, mana dorks,
//! a saddled Mount, an additional-cost draw-burn, a graveyard-hate flyer, and a
//! life-matters Bat. All ride existing primitives. Tests in
//! `crabomination/src/tests/recent151.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    MayPlayDuration, Predicate, SelectionRequirement as R, Selector, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{attacks_while_saddled, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, LibraryPosition, ManaPayload, PlayerRef,
    ZoneDest,
};
use crate::mana::{Color, b, cost, g, generic, r};

/// Resurrected Cultist — {2}{B} 4/1. Delirium: {2}{B}{B}, sorcery-speed, return
/// this from your graveyard to the battlefield with a finality counter.
pub fn resurrected_cultist() -> CardDefinition {
    CardDefinition {
        name: "Resurrected Cultist",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b(), b()]),
            from_graveyard: true,
            sorcery_speed: true,
            condition: Some(Predicate::DeliriumActive {
                who: PlayerRef::You,
            }),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Finality,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Overgrown Zealot — {1}{G} 0/4. {T}: add one mana of any color. (The
/// turn-face-up-only ramp mode is omitted — the engine has no such spend gate.)
pub fn overgrown_zealot() -> CardDefinition {
    CardDefinition {
        name: "Overgrown Zealot",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Intrepid Stablemaster — {1}{G} 2/2 Reach. {T}: add {G}. (The Mount/Vehicle
/// ramp mode is omitted — the engine has no such spend gate.)
pub fn intrepid_stablemaster() -> CardDefinition {
    CardDefinition {
        name: "Intrepid Stablemaster",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Green, Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gila Courser — {2}{R} 4/2 Mount. Saddle 1. Whenever it attacks while saddled,
/// impulse the top card (playable until the end of your next turn).
pub fn gila_courser() -> CardDefinition {
    CardDefinition {
        name: "Gila Courser",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Mount],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Saddle(1)],
        triggered_abilities: vec![attacks_while_saddled(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::ONE,
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            pay_own_cost: false,
            uncast_penalty: None,
        })],
        ..Default::default()
    }
}

/// Grab the Prize — {1}{R} Sorcery. Additional cost: discard a card. Draw two;
/// if the discarded card wasn't a land, deal 2 to each opponent.
pub fn grab_the_prize() -> CardDefinition {
    CardDefinition {
        name: "Grab the Prize",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::You,
                amount: Value::ONE,
                random: false,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::If {
                cond: Predicate::DiscardedNonlandThisEffect {
                    who: PlayerRef::You,
                },
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Malevolent Chandelier — {6} 4/4 Construct with flying. {2}: put target card
/// from a graveyard on the bottom of its owner's library. Sorcery-speed.
pub fn malevolent_chandelier() -> CardDefinition {
    CardDefinition {
        name: "Malevolent Chandelier",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sorcery_speed: true,
            effect: Effect::Move {
                what: target_filtered(R::InGraveyard),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::Bottom,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Moonstone Harbinger — {2}{B} 1/3 with flying and deathtouch. Once each turn,
/// when you gain or lose life on your turn, Bats you control get +1/+0 and gain
/// deathtouch until end of turn.
pub fn moonstone_harbinger() -> CardDefinition {
    let payoff = || {
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Bat).and(R::ControlledByYou),
                ),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Bat).and(R::ControlledByYou),
                ),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
        ])
    };
    let on_life = |kind| TriggeredAbility {
        event: EventSpec {
            once_per_turn: true,
            ..EventSpec::new(kind, EventScope::YourControl)
        },
        effect: payoff(),
    };
    CardDefinition {
        name: "Moonstone Harbinger",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bat, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![on_life(EventKind::LifeGained), on_life(EventKind::LifeLost)],
        ..Default::default()
    }
}
