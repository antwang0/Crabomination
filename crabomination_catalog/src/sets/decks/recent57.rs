//! Go-wide white: token anthems, mass pumps, Soldier/Spirit makers, and two
//! Equipment. Tests in `tests/recent57.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CreatureType, Effect, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, spell_mastery_gate, target_filtered};
use crate::effect::{Duration, PlayerRef};
use crate::mana::{Color, b, cost, generic, hybrid, w, x};

fn spirit_flyer(colors: Vec<Color>) -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn soldier_token() -> TokenDefinition {
    TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Requiem Angel — {5}{W} 5/5 Angel with flying. Whenever another non-Spirit
/// creature you control dies, create a 1/1 white flying Spirit.
pub fn requiem_angel() -> CardDefinition {
    CardDefinition {
        name: "Requiem Angel",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Spirit)
                        .negate()
                        .and(R::OtherThanSource),
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(spirit_flyer(vec![Color::White])),
            },
        }],
        ..Default::default()
    }
}

/// Angel of the Dawn — {4}{W} 3/3 Angel with flying. ETB: creatures you control
/// get +1/+1 and gain vigilance until end of turn.
pub fn angel_of_the_dawn() -> CardDefinition {
    let team = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Angel of the Dawn",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: team(),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: team(),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Elderfang Disciple — {1}{B} 1/1 Elf Cleric. ETB: each opponent discards a card.
pub fn elderfang_disciple() -> CardDefinition {
    CardDefinition {
        name: "Elderfang Disciple",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
            random: false,
        })],
        ..Default::default()
    }
}

/// Martial Coup — {X}{W}{W} Sorcery. Create X 1/1 white Soldiers; if X ≥ 5,
/// destroy all other creatures. (Modeled as destroy-then-create — same end
/// board as the printed create-then-destroy-others.)
pub fn martial_coup() -> CardDefinition {
    CardDefinition {
        name: "Martial Coup",
        cost: cost(&[x(), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(5)),
                then: Box::new(Effect::Destroy {
                    what: Selector::EachPermanent(R::Creature),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: Box::new(soldier_token()),
            },
        ]),
        ..Default::default()
    }
}

/// Beckon Apparition — {W/B} Instant. Exile target card from a graveyard;
/// create a 1/1 white-and-black flying Spirit.
pub fn beckon_apparition() -> CardDefinition {
    CardDefinition {
        name: "Beckon Apparition",
        cost: cost(&[hybrid(Color::White, Color::Black)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(R::InGraveyard),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(spirit_flyer(vec![Color::White, Color::Black])),
            },
        ]),
        ..Default::default()
    }
}

/// Kytheon's Tactics — {1}{W}{W} Sorcery. Creatures you control get +2/+1 until
/// end of turn; spell mastery (2+ I/S in your graveyard) → they also gain
/// vigilance.
pub fn kytheons_tactics() -> CardDefinition {
    let team = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Kytheon's Tactics",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: team(),
                power: Value::Const(2),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: spell_mastery_gate(),
                then: Box::new(Effect::GrantKeyword {
                    what: team(),
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Rally the Ranks — {1}{W} Enchantment. As it enters, choose a creature type;
/// creatures you control of that type get +1/+1.
pub fn rally_the_ranks() -> CardDefinition {
    CardDefinition {
        name: "Rally the Ranks",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::NameCreatureType {
            what: Selector::This,
        })],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control of the chosen type get +1/+1.",
            effect: StaticEffect::AnthemForChosenType {
                all_players: false,
                power: 1,
                toughness: 1,
                exclude_source: false,
                opponents: false,
                per_counter: None,
            },
        }],
        ..Default::default()
    }
}

/// Captain's Claws — {2} Equipment. Equipped creature gets +1/+0; whenever it
/// attacks, create a 1/1 white Kor Ally that's tapped and attacking. Equip {1}.
pub fn captains_claws() -> CardDefinition {
    let ally = TokenDefinition {
        name: "Kor Ally".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Ally],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Captain's Claws",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            triggered_abilities: vec![on_attack(Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(ally),
                cleanup: Default::default(),
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Ancestral Blade — {1}{W} Equipment. ETB: create a 1/1 white Soldier and
/// attach to it. Equipped creature gets +1/+1. Equip {1}.
pub fn ancestral_blade() -> CardDefinition {
    CardDefinition {
        name: "Ancestral Blade",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(soldier_token()),
            },
            Effect::Attach {
                what: Selector::This,
                to: Selector::LastCreatedToken,
            },
        ]))],
        ..Default::default()
    }
}
