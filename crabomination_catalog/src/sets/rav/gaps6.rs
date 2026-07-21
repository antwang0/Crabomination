//! Ravnica (RAV) gap wave 6: repeatable board-sweepers, a pair of Transmute
//! creatures, and a couple of convoke/anthem spells. Tests in
//! `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    Keyword, SelectionRequirement as R, Subtypes, Value,
};
use crate::effect::shortcut::{etb, target_filtered, transmute};
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::mana::{b, cost, generic, r, u};

/// Hammerfist Giant — {4}{R}{R} 5/4 Giant Warrior. {T}: deals 4 damage to each
/// creature without flying and each player.
pub fn hammerfist_giant() -> CardDefinition {
    CardDefinition {
        name: "Hammerfist Giant",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::EachPermanent(
                        R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                    ),
                    amount: Value::Const(4),
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(4),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Blockbuster — {3}{R}{R} Enchantment. {1}{R}, Sacrifice this: it deals 3
/// damage to each tapped creature and each player.
pub fn blockbuster() -> CardDefinition {
    CardDefinition {
        name: "Blockbuster",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature.and(R::Tapped)),
                    amount: Value::Const(3),
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(3),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Flight of Fancy — {3}{U} Aura. Enchant creature. When it enters, draw two
/// cards. Enchanted creature has flying.
pub fn flight_of_fancy() -> CardDefinition {
    CardDefinition {
        name: "Flight of Fancy",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::Const(2) })],
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Flying], ..Default::default() }),
        ..Default::default()
    }
}

/// Dimir House Guard — {3}{B} 2/3 Skeleton with fear. Sacrifice a creature:
/// Regenerate this. Transmute {1}{B}{B}.
pub fn dimir_house_guard() -> CardDefinition {
    CardDefinition {
        name: "Dimir House Guard",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Skeleton], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Fear],
        activated_abilities: vec![
            ActivatedAbility {
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            },
            transmute(cost(&[generic(1), b(), b()]), 4),
        ],
        ..Default::default()
    }
}

/// Ethereal Usher — {5}{U} 2/3 Spirit. {U}, {T}: target creature can't be
/// blocked this turn. Transmute {1}{U}{U}.
pub fn ethereal_usher() -> CardDefinition {
    CardDefinition {
        name: "Ethereal Usher",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 3,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[u()]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            transmute(cost(&[generic(1), u(), u()]), 6),
        ],
        ..Default::default()
    }
}


