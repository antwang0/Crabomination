//! DMU/SNC gap batch — modal ETB, kicker discard, affinity bodies, Domain
//! burn, scry dork, combat tricks, an alliance dork, and a casualty dig. All on
//! existing primitives. Tests in `tests/recent_b/recent268.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{LookPick, Duration, Effect, PlayerRef, Selector, Value};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Aether Channeler — {2}{U} 2/1 Human Wizard. ETB, choose one: a 1/1 white
/// flying Bird, bounce another nonland permanent, or draw a card.
pub fn aether_channeler() -> CardDefinition {
    let bird = TokenDefinition {
        name: "Bird".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Aether Channeler",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: bird,
            },
            Effect::Move {
                what: target_filtered(R::Nonland.and(R::OtherThanSource)),
                to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(
                    0,
                )))),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]))],
        ..Default::default()
    }
}

/// Aggressive Sabotage — {2}{B} Sorcery, kicker {R}. Target player discards two
/// cards; if kicked, deals 3 damage to that player.
pub fn aggressive_sabotage() -> CardDefinition {
    CardDefinition {
        name: "Aggressive Sabotage",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Kicker(cost(&[r()]))],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: target_filtered(R::Player),
                amount: Value::Const(2),
                random: false,
            },
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(3),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Argivian Phalanx — {5}{W} 4/4 Human Kor Soldier. Affinity for creatures,
/// vigilance.
pub fn argivian_phalanx() -> CardDefinition {
    CardDefinition {
        name: "Argivian Phalanx",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Kor,
                CreatureType::Soldier,
            ],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        affinity_filter: Some(R::Creature.and(R::ControlledByYou)),
        ..Default::default()
    }
}

/// Artillery Blast — {1}{W} Instant. Domain — deals X damage to target tapped
/// creature, where X is 1 plus the number of basic land types you control.
pub fn artillery_blast() -> CardDefinition {
    CardDefinition {
        name: "Artillery Blast",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::Tapped)),
            amount: Value::Sum(vec![Value::ONE, Value::DomainCount(PlayerRef::You)]),
        },
        ..Default::default()
    }
}

/// Automatic Librarian — {3} 3/2 Construct artifact creature. ETB: scry 2.
pub fn automatic_librarian() -> CardDefinition {
    CardDefinition {
        name: "Automatic Librarian",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Antagonize — {1}{R} Instant. Target creature gets +4/+3 until end of turn.
pub fn antagonize() -> CardDefinition {
    CardDefinition {
        name: "Antagonize",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(4),
            toughness: Value::Const(3),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Attended Socialite — {1}{G} 2/1 Elf Druid. Alliance — whenever another
/// creature you control enters, this gets +1/+1 until end of turn.
pub fn attended_socialite() -> CardDefinition {
    CardDefinition {
        name: "Attended Socialite",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Backup Agent — {1}{W} 1/1 Human Citizen. ETB: put a +1/+1 counter on target
/// creature.
pub fn backup_agent() -> CardDefinition {
    CardDefinition {
        name: "Backup Agent",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: crate::card::CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Angelic Observer — {5}{W} 3/3 Angel Advisor. Affinity for Citizens, flying.
pub fn angelic_observer() -> CardDefinition {
    CardDefinition {
        name: "Angelic Observer",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Advisor],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        affinity_filter: Some(R::HasCreatureType(CreatureType::Citizen).and(R::ControlledByYou)),
        ..Default::default()
    }
}

/// Armor of Shadows — {B} Instant. Until end of turn, target creature gets
/// +1/+0 and gains indestructible.
pub fn armor_of_shadows() -> CardDefinition {
    CardDefinition {
        name: "Armor of Shadows",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Arms of Hadar — {3}{B} Sorcery. Creatures target player controls get -2/-2
/// until end of turn.
pub fn arms_of_hadar() -> CardDefinition {
    CardDefinition {
        name: "Arms of Hadar",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::PumpPT {
            what: Selector::ControlledBy {
                who: PlayerRef::Target(0),
                filter: R::Creature,
            },
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// A Little Chat — {1}{U} Instant, casualty 1. Look at the top two cards of
/// your library; put one into your hand and the other on the bottom.
pub fn a_little_chat() -> CardDefinition {
    CardDefinition {
        name: "A Little Chat",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Casualty(1)],
        effect: Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(2),
    ..Default::default()
})),
        ..Default::default()
    }
}
