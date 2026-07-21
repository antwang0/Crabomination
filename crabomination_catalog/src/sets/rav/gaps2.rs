//! Ravnica (RAV) gap wave 2: vanilla/french-vanilla creatures and simple
//! activated-ability bodies filling the `set_gaps.py rav` remainder.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, LandType, Selector,
    Subtypes, Value,
};
use crate::effect::shortcut::on_dies;
use crate::effect::{Duration, Effect};
use crate::mana::{cost, g, generic, hybrid, r, u, Color};

/// Glass Golem — {5} 6/2 Golem artifact creature (vanilla).
pub fn glass_golem() -> CardDefinition {
    CardDefinition {
        name: "Glass Golem",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
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
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
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
        subtypes: Subtypes { creature_types: vec![CreatureType::Crocodile], ..Default::default() },
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
            body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
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
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
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
        cost: cost(&[generic(6), crate::mana::w(), crate::mana::w(), crate::mana::w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Archon], ..Default::default() },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Creatures can't attack you.",
            effect: StaticEffect::CreaturesCantAttackController { protect_planeswalkers: false },
        }],
        ..Default::default()
    }
}
