//! CR 104.3 can't-lose/can't-win cluster (Angel's Grace, Platinum Angel,
//! Abyssal Persecutor, Worship) and the CR 113.11 "can't have or gain"
//! Theros Archetype cycle. Tests in `tests/recent109.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, StaticAbility, Subtypes,
};
use crate::effect::shortcut::{each_opponent_creature, each_your_creature};
use crate::effect::{Effect, StaticEffect};
use crate::mana::{b, cost, g, generic, r, u, w, ManaSymbol};

/// Angel's Grace — {W} Instant. Split second. You can't lose the game this
/// turn and your opponents can't win; damage that would drop you below 1
/// life drops you to 1 instead.
pub fn angels_grace() -> CardDefinition {
    CardDefinition {
        name: "Angel's Grace",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::SplitSecond],
        effect: Effect::CantLoseThisTurn { damage_floor: true },
        ..Default::default()
    }
}

/// Platinum Angel — {7} 4/4 Artifact Creature — Angel. Flying. You can't
/// lose the game and your opponents can't win the game.
pub fn platinum_angel() -> CardDefinition {
    CardDefinition {
        name: "Platinum Angel",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You can't lose the game and your opponents can't win the game.",
            effect: StaticEffect::ControllerCantLoseGame,
        }],
        ..Default::default()
    }
}

/// Abyssal Persecutor — {2}{B}{B} 6/6 Demon. Flying, trample. You can't win
/// the game and your opponents can't lose the game.
pub fn abyssal_persecutor() -> CardDefinition {
    CardDefinition {
        name: "Abyssal Persecutor",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "You can't win the game and your opponents can't lose the game.",
            effect: StaticEffect::ControllerCantWinGame,
        }],
        ..Default::default()
    }
}

/// Worship — {3}{W} Enchantment. If you control a creature, damage that
/// would reduce your life total to less than 1 reduces it to 1 instead.
pub fn worship() -> CardDefinition {
    CardDefinition {
        name: "Worship",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If you control a creature, damage that would reduce your life total to less than 1 reduces it to 1 instead.",
            effect: StaticEffect::DamageWontReduceControllerLifeBelowOne { requires_creature: true },
        }],
        ..Default::default()
    }
}

// ── CR 113.11 — the Theros Archetype cycle ───────────────────────────────────

/// Shared Archetype body: your creatures have `kw`; opponents' creatures
/// lose it and can't have or gain it.
fn archetype(
    name: &'static str,
    mana: &[ManaSymbol],
    types: Vec<CreatureType>,
    pt: (i32, i32),
    kw: Keyword,
    desc: (&'static str, &'static str),
) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(mana),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: pt.0,
        toughness: pt.1,
        static_abilities: vec![
            StaticAbility {
                description: desc.0,
                effect: StaticEffect::GrantKeyword {
                    applies_to: each_your_creature(),
                    keyword: kw.clone(),
                },
            },
            StaticAbility {
                description: desc.1,
                effect: StaticEffect::CantHaveKeyword {
                    applies_to: each_opponent_creature(),
                    keyword: kw,
                },
            },
        ],
        ..Default::default()
    }
}

/// Archetype of Courage — {1}{W}{W} 2/2. Your creatures have first strike;
/// opponents' lose it and can't have or gain it.
pub fn archetype_of_courage() -> CardDefinition {
    archetype(
        "Archetype of Courage",
        &[generic(1), w(), w()],
        vec![CreatureType::Human, CreatureType::Soldier],
        (2, 2),
        Keyword::FirstStrike,
        (
            "Creatures you control have first strike.",
            "Creatures your opponents control lose first strike and can't have or gain first strike.",
        ),
    )
}

/// Archetype of Imagination — {4}{U}{U} 3/2. Flying for yours; none for theirs.
pub fn archetype_of_imagination() -> CardDefinition {
    archetype(
        "Archetype of Imagination",
        &[generic(4), u(), u()],
        vec![CreatureType::Human, CreatureType::Wizard],
        (3, 2),
        Keyword::Flying,
        (
            "Creatures you control have flying.",
            "Creatures your opponents control lose flying and can't have or gain flying.",
        ),
    )
}

/// Archetype of Finality — {4}{B}{B} 2/3. Deathtouch for yours; none for theirs.
pub fn archetype_of_finality() -> CardDefinition {
    archetype(
        "Archetype of Finality",
        &[generic(4), b(), b()],
        vec![CreatureType::Gorgon],
        (2, 3),
        Keyword::Deathtouch,
        (
            "Creatures you control have deathtouch.",
            "Creatures your opponents control lose deathtouch and can't have or gain deathtouch.",
        ),
    )
}

/// Archetype of Aggression — {1}{R}{R} 3/2. Trample for yours; none for theirs.
pub fn archetype_of_aggression() -> CardDefinition {
    archetype(
        "Archetype of Aggression",
        &[generic(1), r(), r()],
        vec![CreatureType::Human, CreatureType::Warrior],
        (3, 2),
        Keyword::Trample,
        (
            "Creatures you control have trample.",
            "Creatures your opponents control lose trample and can't have or gain trample.",
        ),
    )
}

/// Archetype of Endurance — {6}{G}{G} 6/5. Hexproof for yours; none for theirs.
pub fn archetype_of_endurance() -> CardDefinition {
    archetype(
        "Archetype of Endurance",
        &[generic(6), g(), g()],
        vec![CreatureType::Boar],
        (6, 5),
        Keyword::Hexproof,
        (
            "Creatures you control have hexproof.",
            "Creatures your opponents control lose hexproof and can't have or gain hexproof.",
        ),
    )
}
