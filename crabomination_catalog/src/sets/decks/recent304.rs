//! Dissension batch 3: symmetric card-draw/discard, multicolored-matters
//! combat tricks, and dual-target removal. Tests in `recent_b/recent_304`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, Selector, Subtypes,
    Value,
};
use crate::effect::shortcut::{draw, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef};
use crate::mana::{b, cost, g, generic, r, u};

/// Rakdos Ragemutt — {3}{B}{R} 3/3 Elemental Dog with lifelink and haste.
pub fn rakdos_ragemutt() -> CardDefinition {
    CardDefinition {
        name: "Rakdos Ragemutt",
        cost: cost(&[generic(3), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Dog],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Lifelink, Keyword::Haste],
        ..Default::default()
    }
}

/// Delirium Skeins — {2}{B} Sorcery. Each player discards three cards.
pub fn delirium_skeins() -> CardDefinition {
    CardDefinition {
        name: "Delirium Skeins",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Discard {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::Const(3),
            random: false,
        },
        ..Default::default()
    }
}

/// Vision Skeins — {1}{U} Instant. Each player draws two cards.
pub fn vision_skeins() -> CardDefinition {
    CardDefinition {
        name: "Vision Skeins",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Draw {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Psychotic Fury — {1}{R} Instant. Target multicolored creature gains double
/// strike until end of turn. Draw a card.
pub fn psychotic_fury() -> CardDefinition {
    CardDefinition {
        name: "Psychotic Fury",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::Multicolored)),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
        ..Default::default()
    }
}

/// Might of the Nephilim — {1}{G} Instant. Target creature gets +2/+2 until end
/// of turn for each of its colors.
pub fn might_of_the_nephilim() -> CardDefinition {
    let bonus = Value::Times(
        Box::new(Value::Const(2)),
        Box::new(Value::ColorCountOf(Box::new(Selector::Target(0)))),
    );
    CardDefinition {
        name: "Might of the Nephilim",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: bonus.clone(),
            toughness: bonus,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Stomp and Howl — {2}{G} Sorcery. Destroy target artifact and target
/// enchantment.
pub fn stomp_and_howl() -> CardDefinition {
    CardDefinition {
        name: "Stomp and Howl",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Artifact,
                },
            },
            Effect::Destroy {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Enchantment,
                },
            },
        ]),
        ..Default::default()
    }
}
