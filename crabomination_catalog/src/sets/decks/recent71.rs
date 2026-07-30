//! Classic-frame commons/uncommons wave — a CDA body, a death-token, landwalk,
//! firebreathing, a colorless ping artifact, and vanilla bodies. All ride
//! existing engine primitives. Tests in `tests/recent71.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, DynamicPt, Keyword, LandType,
    Subtypes, TokenDefinition,
};
use crate::effect::shortcut::{dies_mint_token, target_any};
use crate::effect::{Duration, Effect, Selector, Value};
use crate::mana::{Color, b, cost, g, generic, r, u};

/// Nightmare — {5}{B} Nightmare Horse. Flying. Power and toughness are each
/// equal to the number of Swamps you control.
pub fn nightmare() -> CardDefinition {
    CardDefinition {
        name: "Nightmare",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare, CreatureType::Horse],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        dynamic_pt: Some(DynamicPt::BasePlusLandsOfTypeControlled {
            land_type: LandType::Swamp,
            base_p: 0,
            base_t: 0,
        }),
        ..Default::default()
    }
}

/// Rukh Egg — {2}{R} 0/3 Bird. When it dies, create a 4/4 red Bird with flying.
pub fn rukh_egg() -> CardDefinition {
    let rukh = TokenDefinition {
        name: "Bird".into(),
        power: 4,
        toughness: 4,
        colors: vec![Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Rukh Egg",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        triggered_abilities: vec![dies_mint_token(rukh, 1)],
        ..Default::default()
    }
}

/// Sabertooth Tiger — {3}{R} 2/1 Cat. First strike.
pub fn sabertooth_tiger() -> CardDefinition {
    CardDefinition {
        name: "Sabertooth Tiger",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        ..Default::default()
    }
}

/// Ironroot Treefolk — {3}{G} 3/5 Treefolk (vanilla).
pub fn ironroot_treefolk() -> CardDefinition {
    CardDefinition {
        name: "Ironroot Treefolk",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        ..Default::default()
    }
}

/// Fire Elemental — {3}{R}{R} 5/4 Elemental (vanilla).
pub fn fire_elemental() -> CardDefinition {
    CardDefinition {
        name: "Fire Elemental",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        ..Default::default()
    }
}

/// Dross Crocodile — {5}{B} 5/1 Zombie Crocodile (vanilla).
pub fn dross_crocodile() -> CardDefinition {
    CardDefinition {
        name: "Dross Crocodile",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Crocodile],
            ..Default::default()
        },
        power: 5,
        toughness: 1,
        ..Default::default()
    }
}

/// Segovian Leviathan — {3}{U} 3/3 Leviathan. Islandwalk.
pub fn segovian_leviathan() -> CardDefinition {
    CardDefinition {
        name: "Segovian Leviathan",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Leviathan],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        ..Default::default()
    }
}

/// Vampire Bats — {1}{B} 1/1 Bat. Flying. {B}: gets +1/+0 until end of turn.
/// Activate only once each turn.
pub fn vampire_bats() -> CardDefinition {
    CardDefinition {
        name: "Vampire Bats",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bat],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Durkwood Boars — {4}{G} 5/5 Boar (vanilla).
pub fn durkwood_boars() -> CardDefinition {
    CardDefinition {
        name: "Durkwood Boars",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Boar],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        ..Default::default()
    }
}

/// Wall of Spears — {2} 2/3 Artifact Creature — Wall. Defender, first strike.
pub fn wall_of_spears() -> CardDefinition {
    CardDefinition {
        name: "Wall of Spears",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Defender, Keyword::FirstStrike],
        ..Default::default()
    }
}

/// Wall of Ice — {2}{G} 0/7 Wall. Defender.
pub fn wall_of_ice() -> CardDefinition {
    CardDefinition {
        name: "Wall of Ice",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 7,
        keywords: vec![Keyword::Defender],
        ..Default::default()
    }
}

/// Rod of Ruin — {4} Artifact. {3}, {T}: deals 1 damage to any target.
pub fn rod_of_ruin() -> CardDefinition {
    CardDefinition {
        name: "Rod of Ruin",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
