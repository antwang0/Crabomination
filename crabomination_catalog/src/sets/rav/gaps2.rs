//! Ravnica (RAV) gap wave 2: vanilla/french-vanilla creatures and simple
//! activated-ability bodies filling the `set_gaps.py rav` remainder.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, LandType,
    SelectionRequirement as R, Selector, StaticAbility, Subtypes, Value,
};
use crate::effect::shortcut::{on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w};

/// Glass Golem — {5} 6/2 Golem artifact creature (vanilla).
pub fn glass_golem() -> CardDefinition {
    CardDefinition {
        name: "Glass Golem",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            ..Default::default()
        },
        power: 6,
        toughness: 2,
        ..Default::default()
    }
}

/// Goliath Spider — {6}{G}{G} 7/6 Spider with reach.
pub fn goliath_spider() -> CardDefinition {
    CardDefinition {
        name: "Goliath Spider",
        cost: cost(&[generic(6), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider],
            ..Default::default()
        },
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Reach],
        ..Default::default()
    }
}

/// Grayscaled Gharial — {U} 1/1 Crocodile with islandwalk.
pub fn grayscaled_gharial() -> CardDefinition {
    CardDefinition {
        name: "Grayscaled Gharial",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Crocodile],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        ..Default::default()
    }
}

/// Centaur Safeguard — {2}{G/W} 3/1 Centaur Warrior. When it dies, you may gain
/// 3 life.
pub fn centaur_safeguard() -> CardDefinition {
    CardDefinition {
        name: "Centaur Safeguard",
        cost: cost(&[generic(2), hybrid(Color::Green, Color::White)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::MayDo {
            description: "Gain 3 life".into(),
            body: Box::new(Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            }),
        })],
        ..Default::default()
    }
}

/// Greater Forgeling — {3}{R}{R} 3/4 Elemental. `{1}{R}: +3/-3 until end of turn.`
pub fn greater_forgeling() -> CardDefinition {
    CardDefinition {
        name: "Greater Forgeling",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Goblin Fire Fiend — {3}{R} 1/1 Goblin Berserker with haste that must be
/// blocked if able. `{R}: +1/+0 until end of turn.`
pub fn goblin_fire_fiend() -> CardDefinition {
    CardDefinition {
        name: "Goblin Fire Fiend",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Berserker],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste, Keyword::MustBeBlocked],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Blazing Archon — {6}{W}{W}{W} 5/6 Archon with flying. Creatures can't attack
/// you.
pub fn blazing_archon() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Blazing Archon",
        cost: cost(&[
            generic(6),
            crate::mana::w(),
            crate::mana::w(),
            crate::mana::w(),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Archon],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Creatures can't attack you.",
            effect: StaticEffect::CreaturesCantAttackController {
                protect_planeswalkers: false,
                filter: None,
            },
        }],
        ..Default::default()
    }
}

/// Sell-Sword Brute — {1}{R} 2/2 Human Mercenary. When it dies, it deals 2
/// damage to you.
pub fn sell_sword_brute() -> CardDefinition {
    CardDefinition {
        name: "Sell-Sword Brute",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![on_dies(Effect::DealDamage {
            to: Selector::You,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Screeching Griffin — {3}{W} 2/2 Griffin with flying. `{R}: Target creature
/// can't block this creature this turn.`
pub fn screeching_griffin() -> CardDefinition {
    CardDefinition {
        name: "Screeching Griffin",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Griffin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::CantBlockSourceThisTurn {
                target: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Roofstalker Wight — {1}{B} 2/1 Zombie. `{1}{U}: This creature gains flying
/// until end of turn.`
pub fn roofstalker_wight() -> CardDefinition {
    CardDefinition {
        name: "Roofstalker Wight",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sewerdreg — {3}{B}{B} 3/3 Spirit with swampwalk. `Sacrifice this creature:
/// Exile target card from a graveyard.`
pub fn sewerdreg() -> CardDefinition {
    CardDefinition {
        name: "Sewerdreg",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::InGraveyard),
                to: ZoneDest::Exile,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Infectious Host — {2}{B} 1/1 Zombie. When it dies, target player loses 2 life.
pub fn infectious_host() -> CardDefinition {
    CardDefinition {
        name: "Infectious Host",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::LoseLife {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Loxodon Gatekeeper — {2}{W}{W} 2/3 Elephant Soldier. Artifacts, creatures,
/// and lands your opponents control enter tapped.
pub fn loxodon_gatekeeper() -> CardDefinition {
    CardDefinition {
        name: "Loxodon Gatekeeper",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Artifacts, creatures, and lands your opponents control enter tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::EachPermanent(
                    R::ControlledByOpponent.and(R::Artifact.or(R::Creature).or(R::Land)),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Oathsworn Giant — {4}{W}{W} 3/4 Giant Soldier with vigilance. Other
/// creatures you control get +0/+2 and have vigilance.
pub fn oathsworn_giant() -> CardDefinition {
    let others =
        || Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource));
    CardDefinition {
        name: "Oathsworn Giant",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![
            StaticAbility {
                description: "Other creatures you control get +0/+2.",
                effect: StaticEffect::PumpPT {
                    applies_to: others(),
                    power: 0,
                    toughness: 2,
                },
            },
            StaticAbility {
                description: "… and have vigilance.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: others(),
                    keyword: Keyword::Vigilance,
                },
            },
        ],
        ..Default::default()
    }
}
