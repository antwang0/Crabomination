//! Kaladesh artifacts / vehicles / pilots: Servo makers, crewed Vehicles,
//! Fabricate, and energy. Tests in `tests/recent62.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, fabricate, on_attack, target_filtered};
use crate::effect::{Duration, PlayerRef};
use crate::mana::{cost, g, generic, r, w};

fn servo() -> TokenDefinition {
    TokenDefinition {
        name: "Servo".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Servo],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn make_servo() -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Box::new(servo()),
    }
}

/// Servo Schematic — {2} Artifact. When it enters or is put into a graveyard
/// from the battlefield, create a 1/1 colorless Servo.
pub fn servo_schematic() -> CardDefinition {
    CardDefinition {
        name: "Servo Schematic",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            etb(make_servo()),
            TriggeredAbility {
                // "Put into a graveyard from the battlefield" — modeled via the
                // leaves-battlefield event (the artifact-death-cantrip idiom).
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: make_servo(),
            },
        ],
        ..Default::default()
    }
}

/// Cogworker's Puzzleknot — {2} Artifact. ETB: create a Servo. {1}{W}, Sacrifice
/// this: create a Servo.
pub fn cogworkers_puzzleknot() -> CardDefinition {
    CardDefinition {
        name: "Cogworker's Puzzleknot",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(make_servo())],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            sac_cost: true,
            effect: make_servo(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Renegade Freighter — {3} Vehicle 4/3, Crew 2. Whenever it attacks, it gets
/// +1/+1 and gains trample until end of turn.
pub fn renegade_freighter() -> CardDefinition {
    CardDefinition {
        name: "Renegade Freighter",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Bomat Bazaar Barge — {4} Vehicle 5/5, Crew 3. When it enters, draw a card.
pub fn bomat_bazaar_barge() -> CardDefinition {
    CardDefinition {
        name: "Bomat Bazaar Barge",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Crew(3)],
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Peema Outrider — {2}{G}{G} 3/3 Elf Artificer with trample, Fabricate 1.
pub fn peema_outrider() -> CardDefinition {
    CardDefinition {
        name: "Peema Outrider",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![fabricate(1)],
        ..Default::default()
    }
}

/// Deadeye Harpooner — {2}{W} 2/2 Dwarf Warrior. Revolt — ETB, if a permanent
/// left the battlefield under your control this turn, destroy target tapped
/// creature an opponent controls.
pub fn deadeye_harpooner() -> CardDefinition {
    CardDefinition {
        name: "Deadeye Harpooner",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::RevoltActive {
                who: PlayerRef::You,
            },
            then: Box::new(Effect::Destroy {
                what: target_filtered(R::Creature.and(R::Tapped).and(R::ControlledByOpponent)),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Gearshift Ace — {1}{W} 2/1 Dwarf Pilot with first strike. Whenever it crews a
/// Vehicle, that Vehicle gains first strike until end of turn.
pub fn gearshift_ace() -> CardDefinition {
    CardDefinition {
        name: "Gearshift Ace",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Pilot],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CrewsOrSaddles, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: Selector::TriggerSource,
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Veteran Motorist — {R}{W} 3/1 Dwarf Pilot. ETB: scry 2. Whenever it crews a
/// Vehicle, that Vehicle gets +1/+1 until end of turn.
pub fn veteran_motorist() -> CardDefinition {
    CardDefinition {
        name: "Veteran Motorist",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Pilot],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CrewsOrSaddles, EventScope::SelfSource),
                effect: Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Aether Chaser — {1}{R} 2/1 Human Artificer with first strike. ETB: you get
/// {E}{E}. Whenever it attacks, you may pay {E}{E}; if you do, create a Servo.
pub fn aether_chaser() -> CardDefinition {
    CardDefinition {
        name: "Aether Chaser",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![
            etb(Effect::AddEnergy(Value::Const(2))),
            on_attack(Effect::MayDo {
                description: "Pay {E}{E} to create a Servo".into(),
                body: Box::new(Effect::PayEnergy {
                    amount: 2,
                    then: Box::new(make_servo()),
                }),
            }),
        ],
        ..Default::default()
    }
}
