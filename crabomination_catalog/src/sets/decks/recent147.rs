//! A Wilds of Eldraine (WOE) wave: a Saga, an Adventure, a Food land (the new
//! `EntersTappedUnless`), Bargain/Elf/aristocrat payoffs, and a multicolor
//! counter. All ride existing primitives. Tests in
//! `crabomination/src/tests/recent147.rs`.

use crate::card::{
    Adventure, CardDefinition, CardType, CardType::Instant, CardType::Sorcery, CounterType,
    CreatureType, EnchantmentSubtype, Keyword, LandType, Predicate, SelectionRequirement as R,
    Selector, StaticAbility, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, StaticEffect};
use crate::game::effects::{food_token, treasure_token};
use crate::mana::{Color, b, cost, g, generic, u};

use super::woe_roles::wicked_role;

/// 2/2 white Knight token with vigilance.
fn knight_vigilance_token() -> TokenDefinition {
    TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Knight],
            ..Default::default()
        },
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    }
}

/// The Witch's Vanity — {1}{B} Saga. I destroy a small opposing creature, II
/// makes a Food, III hangs a Wicked Role on a creature you control.
pub fn the_witchs_vanity() -> CardDefinition {
    CardDefinition {
        name: "The Witch's Vanity",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::Destroy {
                    what: target_filtered(
                        R::Creature
                            .and(R::ControlledByOpponent)
                            .and(R::ManaValueAtMost(2)),
                    ),
                },
            ),
            (
                2,
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: food_token(),
                },
            ),
            (
                3,
                Effect::CreateTokenAttachedTo {
                    target: target_filtered(R::Creature.and(R::ControlledByYou)),
                    definition: wicked_role(),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Imodane's Recruiter // Train Troops — {2}{R} 2/2 Human Knight. ETB gives your
/// team +1/+0 and haste. Adventure {4}{W} Sorcery: make two 2/2 vigilant Knights.
pub fn imodanes_recruiter() -> CardDefinition {
    CardDefinition {
        name: "Imodane's Recruiter",
        cost: cost(&[generic(2), crate::mana::r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]))],
        adventure: Some(Box::new(Adventure {
            name: "Train Troops",
            cost: cost(&[generic(4), crate::mana::w()]),
            card_types: vec![Sorcery],
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: knight_vigilance_token(),
            },
        })),
        ..Default::default()
    }
}

/// Gingerbread Cabin — Forest land. Enters tapped unless you control three or
/// more other Forests; when it enters untapped, create a Food.
pub fn gingerbread_cabin() -> CardDefinition {
    CardDefinition {
        name: "Gingerbread Cabin",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Forest],
            ..Default::default()
        },
        static_abilities: vec![StaticAbility {
            description: "Enters tapped unless you control three or more other Forests.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasLandType(LandType::Forest)
                            .and(R::ControlledByYou)
                            .and(R::OtherThanSource),
                    ),
                    n: Value::Const(3),
                },
            },
        }],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::This,
                filter: R::Untapped,
            },
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: food_token(),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Yeva's Forcemage — {2}{G} 2/2 Elf Shaman. ETB: target creature gets +2/+2.
pub fn yevas_forcemage() -> CardDefinition {
    CardDefinition {
        name: "Yeva's Forcemage",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Elvish Vanguard — {1}{G} 1/1 Elf Warrior. Whenever another Elf enters, grow.
pub fn elvish_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Elvish Vanguard",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Elf).and(R::OtherThanSource),
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Gnawing Vermin — {B} 1/1 Rat. ETB mills a player two; dies, an enemy creature
/// gets -1/-1.
pub fn gnawing_vermin() -> CardDefinition {
    CardDefinition {
        name: "Gnawing Vermin",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::Mill {
                who: target_filtered(R::Player),
                amount: Value::Const(2),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Hoard Robber — {1}{B} 1/3 Tiefling Rogue. Combat damage to a player makes a
/// Treasure.
pub fn hoard_robber() -> CardDefinition {
    CardDefinition {
        name: "Hoard Robber",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Tiefling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: treasure_token(),
            },
        }],
        ..Default::default()
    }
}

/// Neutralizing Blast — {1}{U} Instant. Counter target multicolored spell.
pub fn neutralizing_blast() -> CardDefinition {
    CardDefinition {
        name: "Neutralizing Blast",
        cost: cost(&[generic(1), u()]),
        card_types: vec![Instant],
        effect: Effect::CounterSpell {
            what: target_filtered(R::Multicolored),
        },
        ..Default::default()
    }
}
