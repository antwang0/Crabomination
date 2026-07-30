//! Ravnica (RAV) gap wave 6: repeatable board-sweepers, a pair of Transmute
//! creatures, and a couple of convoke/anthem spells. Tests in
//! `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    Keyword, SelectionRequirement as R, Subtypes, Value,
};
use crate::effect::shortcut::{blocks, etb, on_dies, target_any, target_filtered, transmute};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, generic, r, u, w, x};

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
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
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
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Fear],
        activated_abilities: vec![
            ActivatedAbility {
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::Regenerate {
                    what: Selector::This,
                },
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
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
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

/// Cyclopean Snare — {2} Artifact. {3}, {T}: Tap target creature, then return
/// this artifact to its owner's hand.
pub fn cyclopean_snare() -> CardDefinition {
    CardDefinition {
        name: "Cyclopean Snare",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: target_filtered(R::Creature),
                },
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Festival of the Guildpact — {X}{W} Instant. Prevent the next X damage that
/// would be dealt to you this turn. Draw a card.
pub fn festival_of_the_guildpact() -> CardDefinition {
    CardDefinition {
        name: "Festival of the Guildpact",
        cost: cost(&[x(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PreventNextDamage {
                target: Selector::You,
                amount: Value::XFromCost,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Viashino Fangtail — {2}{R}{R} 3/3 Lizard Warrior. {T}: deals 1 damage to any
/// target.
pub fn viashino_fangtail() -> CardDefinition {
    CardDefinition {
        name: "Viashino Fangtail",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Undercity Shade — {4}{B} 1/1 Shade with fear. {B}: gets +1/+1 until end of
/// turn.
pub fn undercity_shade() -> CardDefinition {
    CardDefinition {
        name: "Undercity Shade",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shade],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Fear],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// War-Torch Goblin — {R} 1/1 Goblin Warrior. {R}, Sacrifice this: it deals 2
/// damage to target blocking creature.
pub fn war_torch_goblin() -> CardDefinition {
    CardDefinition {
        name: "War-Torch Goblin",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsBlocking)),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Viashino Slasher — {1}{R} 1/2 Lizard Warrior. {R}: gets +1/-1 until end of
/// turn.
pub fn viashino_slasher() -> CardDefinition {
    CardDefinition {
        name: "Viashino Slasher",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tattered Drake — {4}{U} 2/2 Zombie Drake with flying. {B}: regenerate this.
pub fn tattered_drake() -> CardDefinition {
    CardDefinition {
        name: "Tattered Drake",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Drake],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate {
                what: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Surveilling Sprite — {1}{U} 1/1 Faerie Rogue with flying. When it dies, you
/// may draw a card.
pub fn surveilling_sprite() -> CardDefinition {
    CardDefinition {
        name: "Surveilling Sprite",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::MayDo {
            description: "Draw a card".into(),
            body: Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            }),
        })],
        ..Default::default()
    }
}

/// Zephyr Spirit — {5}{U} 0/6 Spirit. When it blocks, return it to its owner's
/// hand.
pub fn zephyr_spirit() -> CardDefinition {
    CardDefinition {
        name: "Zephyr Spirit",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 0,
        toughness: 6,
        triggered_abilities: vec![blocks(Effect::Move {
            what: Selector::This,
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..Default::default()
    }
}
