//! Assorted commons wave — reach + discard-pump, landwalk, regeneration,
//! conditional haste + firebreathing, a tap-ping, and vanilla bodies. All ride
//! existing engine primitives. Tests in `tests/recent70.rs`.

use crate::card::SelectionRequirement as R;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, LandType, Predicate,
    StaticAbility, StaticEffect, Subtypes,
};
use crate::effect::shortcut::target_any;
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::mana::{cost, g, generic, r, u, w};

/// Krosan Archer — {3}{G} 2/3 Centaur Archer. Reach. {G}, Discard a card: it
/// gets +0/+2 until end of turn.
pub fn krosan_archer() -> CardDefinition {
    CardDefinition {
        name: "Krosan Archer",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Archer],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(0),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dwarven Grunt — {R} 1/1 Dwarf. Mountainwalk.
pub fn dwarven_grunt() -> CardDefinition {
    CardDefinition {
        name: "Dwarven Grunt",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Landwalk(LandType::Mountain)],
        ..Default::default()
    }
}

/// Vengeful Firebrand — {3}{R} 5/2 Elemental Warrior. Has haste while a Warrior
/// card is in your graveyard. {R}: gets +1/+0 until end of turn.
pub fn vengeful_firebrand() -> CardDefinition {
    CardDefinition {
        name: "Vengeful Firebrand",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Has haste while a Warrior card is in your graveyard.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtLeast(
                    Value::CardsInGraveyardMatching {
                        who: PlayerRef::You,
                        filter: R::HasCreatureType(CreatureType::Warrior),
                    },
                    Value::Const(1),
                ),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Haste],
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
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

/// Anaba Shaman — {3}{R} 2/2 Minotaur Shaman. {R}, {T}: it deals 1 damage to
/// any target.
pub fn anaba_shaman() -> CardDefinition {
    CardDefinition {
        name: "Anaba Shaman",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            tap_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Balduvian Barbarians — {1}{R}{R} 3/2 Human Barbarian (vanilla).
pub fn balduvian_barbarians() -> CardDefinition {
    CardDefinition {
        name: "Balduvian Barbarians",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Barbarian],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        ..Default::default()
    }
}

/// Zephyr Falcon — {1}{U} 1/1 Bird. Flying, vigilance.
pub fn zephyr_falcon() -> CardDefinition {
    CardDefinition {
        name: "Zephyr Falcon",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..Default::default()
    }
}

/// Regal Unicorn — {2}{W} 2/3 Unicorn (vanilla).
pub fn regal_unicorn() -> CardDefinition {
    CardDefinition {
        name: "Regal Unicorn",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Unicorn], ..Default::default() },
        power: 2,
        toughness: 3,
        ..Default::default()
    }
}
