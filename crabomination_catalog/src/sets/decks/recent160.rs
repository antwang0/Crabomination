//! A Foundations wave of straightforward catalog gaps — Raid, loot, tokens, and
//! draw/sacrifice payoffs, all on shipped primitives. Tests in
//! `crabomination/src/tests/recent160.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_dies, target_any};
use crate::effect::{Effect, PlayerRef};
use crate::mana::{Color, b, cost, generic, r, u, w};

/// Erudite Wizard — {2}{U} 2/3 Human Wizard. Whenever you draw your second card
/// each turn, put a +1/+1 counter on it.
pub fn erudite_wizard() -> CardDefinition {
    CardDefinition {
        name: "Erudite Wizard",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_turn: true,
                ..EventSpec::new(EventKind::CardDrawn, EventScope::YourControl).with_filter(
                    Predicate::PlayerDrewAtLeastThisTurn {
                        who: PlayerRef::You,
                        n: 2,
                    },
                )
            },
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Gorehorn Raider — {4}{R} 4/4 Minotaur Pirate. Raid — when it enters, if you
/// attacked this turn, it deals 2 damage to any target.
pub fn gorehorn_raider() -> CardDefinition {
    CardDefinition {
        name: "Gorehorn Raider",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Pirate],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerAttackedThisTurn {
                who: PlayerRef::You,
            },
            then: Box::new(Effect::DealDamage {
                to: target_any(),
                amount: Value::Const(2),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Gutless Plunderer — {2}{B} 2/2 Skeleton Pirate with deathtouch. Raid — when it
/// enters, if you attacked this turn, look at the top three cards of your
/// library, keep one on top, and put the rest into your graveyard.
pub fn gutless_plunderer() -> CardDefinition {
    CardDefinition {
        name: "Gutless Plunderer",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerAttackedThisTurn {
                who: PlayerRef::You,
            },
            then: Box::new(Effect::LookTopKeepOneRestToGraveyard {
                count: Value::Const(3),
                who: Some(PlayerRef::You),
                exile_rest: false,
                rest_bottom_random: false,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Hinterland Sanctifier — {W} 1/2 Rabbit Cleric. Whenever another creature you
/// control enters, you gain 1 life.
pub fn hinterland_sanctifier() -> CardDefinition {
    CardDefinition {
        name: "Hinterland Sanctifier",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Hungry Ghoul — {1}{B} 2/2 Zombie. {1}, Sacrifice another creature: put a
/// +1/+1 counter on it.
pub fn hungry_ghoul() -> CardDefinition {
    CardDefinition {
        name: "Hungry Ghoul",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Icewind Elemental — {4}{U} 3/4 Elemental with flying. When it enters, draw a
/// card, then discard a card.
pub fn icewind_elemental() -> CardDefinition {
    CardDefinition {
        name: "Icewind Elemental",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::ONE,
                random: false,
            },
        ]))],
        ..Default::default()
    }
}

/// Infestation Sage — {B} 1/1 Elf Warlock. When it dies, create a 1/1 black and
/// green Insect creature token with flying.
pub fn infestation_sage() -> CardDefinition {
    CardDefinition {
        name: "Infestation Sage",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Insect".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Black, Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Insect],
                    ..Default::default()
                },
                keywords: vec![Keyword::Flying],
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

/// Prideful Parent — {2}{W} 2/2 Cat with vigilance. When it enters, create a
/// 1/1 white Cat creature token.
pub fn prideful_parent() -> CardDefinition {
    CardDefinition {
        name: "Prideful Parent",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Cat".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Cat],
                    ..Default::default()
                },
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

/// Firespitter Whelp — {2}{R} 2/2 Dragon with flying. Whenever you cast a
/// noncreature or Dragon spell, it deals 1 damage to each opponent.
pub fn firespitter_whelp() -> CardDefinition {
    CardDefinition {
        name: "Firespitter Whelp",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(
                    R::Creature
                        .negate()
                        .or(R::HasCreatureType(CreatureType::Dragon)),
                ),
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Guarded Heir — {5}{W} 1/1 Human Noble with lifelink. When it enters, create
/// two 3/3 white Knight creature tokens.
pub fn guarded_heir() -> CardDefinition {
    CardDefinition {
        name: "Guarded Heir",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: TokenDefinition {
                name: "Knight".into(),
                power: 3,
                toughness: 3,
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Knight],
                    ..Default::default()
                },
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}
