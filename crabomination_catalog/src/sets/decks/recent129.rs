//! A second WOE wave: Celebration, Roles, Adventure, and evasion payoffs.
//! Reuses existing primitives throughout. Tests in
//! `crabomination/src/tests/recent129.rs`.

use crate::card::{
    ActivatedAbility, Adventure, CardDefinition, CardType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef};
use crate::mana::{Color, cost, generic, r, u, w};

fn rat_token() -> TokenDefinition {
    TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat],
            ..Default::default()
        },
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    }
}

/// Moonshaker Cavalry — {5}{W}{W}{W} 6/6 Spirit Knight with flying. ETB:
/// creatures you control gain flying and get +X/+X where X = creatures you
/// control, until end of turn.
pub fn moonshaker_cavalry() -> CardDefinition {
    let team = Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    let count = Value::CountOf(Box::new(Selector::ControlledBy {
        who: PlayerRef::You,
        filter: R::Creature,
    }));
    CardDefinition {
        name: "Moonshaker Cavalry",
        cost: cost(&[generic(5), w(), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GrantKeyword {
                what: team.clone(),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: team,
                power: count.clone(),
                toughness: count,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Water Wings — {1}{U} Instant. Until end of turn, target creature you control
/// has base power and toughness 4/4 and gains flying and hexproof.
pub fn water_wings() -> CardDefinition {
    CardDefinition {
        name: "Water Wings",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::SetBasePT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(4),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Werefox Bodyguard — {1}{W}{W} 2/2 Elf Fox Knight. Flash. ETB: exile up to one
/// other target non-Fox creature until this leaves. {1}{W}, Sacrifice this: gain
/// 2 life.
pub fn werefox_bodyguard() -> CardDefinition {
    CardDefinition {
        name: "Werefox Bodyguard",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Fox, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                R::Creature
                    .and(R::OtherThanSource)
                    .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Fox)))),
            ),
            return_to: ExileReturnZone::Battlefield,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            sac_cost: true,
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Grand Ball Guest — {1}{R} 2/2 Human Peasant. Celebration — +1/+1 and trample
/// while two or more nonland permanents entered under your control this turn.
pub fn grand_ball_guest() -> CardDefinition {
    CardDefinition {
        name: "Grand Ball Guest",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Celebration — Grand Ball Guest gets +1/+1 and has trample as long as two or more nonland permanents entered under your control this turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::CelebrationActive {
                    who: PlayerRef::You,
                },
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Trample],
            },
        }],
        ..Default::default()
    }
}

/// Ratcatcher Trainee // Pest Problem — {1}{R} 2/1 Human Peasant; Adventure
/// {2}{R} Instant creates two 1/1 Rats that can't block. (The "during your turn,
/// has first strike" static is dropped.)
pub fn ratcatcher_trainee() -> CardDefinition {
    CardDefinition {
        name: "Ratcatcher Trainee",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        adventure: Some(Box::new(Adventure {
            name: "Pest Problem",
            cost: cost(&[generic(2), r()]),
            card_types: vec![CardType::Instant],
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: Box::new(rat_token()),
            },
        })),
        ..Default::default()
    }
}

/// Twisted Fealty — {2}{R} Sorcery. Gain control of target creature until end of
/// turn; untap it and it gains haste. Create a Wicked Role token attached to up
/// to one target creature.
pub fn twisted_fealty() -> CardDefinition {
    let wicked_role = TokenDefinition {
        name: "Wicked".into(),
        card_types: vec![CardType::Enchantment],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Role],
            ..Default::default()
        },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PermanentLeavesBattlefield,
                EventScope::SelfSource,
            ),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Twisted Fealty",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature),
                to: Some(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::CreateTokenAttachedTo {
                target: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature,
                },
                definition: Box::new(wicked_role),
            },
        ]),
        ..Default::default()
    }
}
