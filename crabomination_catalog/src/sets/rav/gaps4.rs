//! Ravnica (RAV) gap wave 4: the Hunted cycle — cheap oversized creatures whose
//! ETB gives a target opponent a squad of tokens.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, Subtypes, TokenDefinition,
    Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef, Selector};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

fn token(
    name: &str,
    p: i32,
    t: i32,
    colors: Vec<Color>,
    ct: CreatureType,
    keywords: Vec<Keyword>,
) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes {
            creature_types: vec![ct],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Hunted Horror — {B}{B} 7/7 Horror with trample. When it enters, target
/// opponent creates two 3/3 green Centaur tokens with protection from black.
pub fn hunted_horror() -> CardDefinition {
    CardDefinition {
        name: "Hunted Horror",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horror],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::Target(0),
            count: Value::Const(2),
            definition: Box::new(token(
                "Centaur",
                3,
                3,
                vec![Color::Green],
                CreatureType::Centaur,
                vec![Keyword::Protection(Color::Black)],
            )),
        })],
        ..Default::default()
    }
}

/// Hunted Phantasm — {1}{U}{U} 4/6 Spirit that can't be blocked. When it enters,
/// target opponent creates five 1/1 red Goblin tokens.
pub fn hunted_phantasm() -> CardDefinition {
    CardDefinition {
        name: "Hunted Phantasm",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::Target(0),
            count: Value::Const(5),
            definition: Box::new(token(
                "Goblin",
                1,
                1,
                vec![Color::Red],
                CreatureType::Goblin,
                vec![],
            )),
        })],
        ..Default::default()
    }
}

/// Hunted Dragon — {3}{R}{R} 6/6 Dragon with flying and haste. When it enters,
/// target opponent creates three 2/2 white Knight tokens with first strike.
pub fn hunted_dragon() -> CardDefinition {
    CardDefinition {
        name: "Hunted Dragon",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::Target(0),
            count: Value::Const(3),
            definition: Box::new(token(
                "Knight",
                2,
                2,
                vec![Color::White],
                CreatureType::Knight,
                vec![Keyword::FirstStrike],
            )),
        })],
        ..Default::default()
    }
}

/// Hunted Lammasu — {2}{W}{W} 5/5 Lammasu with flying. When it enters, target
/// opponent creates a 4/4 black Horror token.
pub fn hunted_lammasu() -> CardDefinition {
    CardDefinition {
        name: "Hunted Lammasu",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lammasu],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::Target(0),
            count: Value::ONE,
            definition: Box::new(token(
                "Horror",
                4,
                4,
                vec![Color::Black],
                CreatureType::Horror,
                vec![],
            )),
        })],
        ..Default::default()
    }
}

/// Hunted Troll — {2}{G}{G} 8/4 Troll Warrior. When it enters, target opponent
/// creates four 1/1 blue Faerie tokens with flying. `{G}: Regenerate this.`
pub fn hunted_troll() -> CardDefinition {
    CardDefinition {
        name: "Hunted Troll",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll, CreatureType::Warrior],
            ..Default::default()
        },
        power: 8,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::Target(0),
            count: Value::Const(4),
            definition: Box::new(token(
                "Faerie",
                1,
                1,
                vec![Color::Blue],
                CreatureType::Faerie,
                vec![Keyword::Flying],
            )),
        })],
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
