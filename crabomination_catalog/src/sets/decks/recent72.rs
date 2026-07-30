//! Retro commons/uncommons — a CDA enchantress, a Zombie lord, regenerating
//! Trolls, evasion-hate, and vanilla bodies. All ride existing primitives.
//! Tests in `tests/recent72.rs`.

use crate::card::SelectionRequirement as R;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, DynamicPt, Keyword, LandType,
    StaticAbility, StaticEffect, Subtypes,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, Selector};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Yavimaya Enchantress — {2}{G} 2/2 Human Druid. Gets +1/+1 for each
/// enchantment on the battlefield.
pub fn yavimaya_enchantress() -> CardDefinition {
    CardDefinition {
        name: "Yavimaya Enchantress",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        dynamic_pt: Some(DynamicPt::EnchantmentsInPlay {
            base_p: 2,
            base_t: 2,
        }),
        ..Default::default()
    }
}

/// Zombie Master — {1}{B}{B} 2/3 Zombie. Other Zombies have swampwalk and
/// "{B}: Regenerate this permanent."
pub fn zombie_master() -> CardDefinition {
    let other_zombies = R::HasCreatureType(CreatureType::Zombie).and(R::OtherThanSource);
    CardDefinition {
        name: "Zombie Master",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![
            StaticAbility {
                description: "Other Zombie creatures have swampwalk.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(other_zombies.clone()),
                    keyword: Keyword::Landwalk(LandType::Swamp),
                },
            },
            StaticAbility {
                description: "Other Zombies have \"{B}: Regenerate this permanent.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::EachPermanent(other_zombies),
                    ability: ActivatedAbility {
                        mana_cost: cost(&[b()]),
                        effect: Effect::Regenerate {
                            what: Selector::This,
                        },
                        ..Default::default()
                    },
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Cudgel Troll — {2}{G}{G} 4/3 Troll. {G}: Regenerate this creature.
pub fn cudgel_troll() -> CardDefinition {
    CardDefinition {
        name: "Cudgel Troll",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::Regenerate {
                what: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Uthden Troll — {2}{R} 2/2 Troll. {R}: Regenerate this creature.
pub fn uthden_troll() -> CardDefinition {
    CardDefinition {
        name: "Uthden Troll",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::Regenerate {
                what: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Longbow Archer — {W}{W} 2/2 Human Soldier Archer. First strike, reach.
pub fn longbow_archer() -> CardDefinition {
    CardDefinition {
        name: "Longbow Archer",
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Soldier,
                CreatureType::Archer,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Reach],
        ..Default::default()
    }
}

/// Talruum Minotaur — {2}{R}{R} 3/3 Minotaur Berserker. Haste.
pub fn talruum_minotaur() -> CardDefinition {
    CardDefinition {
        name: "Talruum Minotaur",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        ..Default::default()
    }
}

/// Anaba Bodyguard — {3}{R} 2/3 Minotaur. First strike.
pub fn anaba_bodyguard() -> CardDefinition {
    CardDefinition {
        name: "Anaba Bodyguard",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        ..Default::default()
    }
}

/// Deadly Insect — {4}{G} 6/1 Insect. Shroud.
pub fn deadly_insect() -> CardDefinition {
    CardDefinition {
        name: "Deadly Insect",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 6,
        toughness: 1,
        keywords: vec![Keyword::Shroud],
        ..Default::default()
    }
}

/// Radjan Spirit — {3}{G} 3/2 Spirit. {T}: Target creature loses flying until
/// end of turn.
pub fn radjan_spirit() -> CardDefinition {
    CardDefinition {
        name: "Radjan Spirit",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::LoseKeywordThisTurn {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Giant Octopus — {3}{U} 3/3 Octopus (vanilla).
pub fn giant_octopus() -> CardDefinition {
    CardDefinition {
        name: "Giant Octopus",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Octopus],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        ..Default::default()
    }
}

/// Balduvian Bears — {1}{G} 2/2 Bear (vanilla).
pub fn balduvian_bears() -> CardDefinition {
    CardDefinition {
        name: "Balduvian Bears",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// Willow Elf — {G} 1/1 Elf (vanilla).
pub fn willow_elf() -> CardDefinition {
    CardDefinition {
        name: "Willow Elf",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}

/// Norwood Ranger — {G} 1/2 Elf Scout Ranger (vanilla).
pub fn norwood_ranger() -> CardDefinition {
    CardDefinition {
        name: "Norwood Ranger",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Scout, CreatureType::Ranger],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        ..Default::default()
    }
}
