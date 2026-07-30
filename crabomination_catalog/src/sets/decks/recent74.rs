//! Retro batch — bounce-ETB, firebreathing/pump outlets, a sac-for-mana Thrull,
//! discard-pump, dual protection, and vanilla bodies. All ride existing
//! primitives. Tests in `tests/recent74.rs`.

use crate::card::SelectionRequirement as R;
use crate::card::{ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, Subtypes};
use crate::effect::shortcut::{etb, target_any};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, Value};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

/// Water Elemental — {3}{U}{U} 5/4 Elemental (vanilla).
pub fn water_elemental() -> CardDefinition {
    CardDefinition {
        name: "Water Elemental",
        cost: cost(&[generic(3), u(), u()]),
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

/// Wall of Water — {1}{U}{U} 0/5 Wall. Defender. {U}: gets +1/+0 until end of turn.
pub fn wall_of_water() -> CardDefinition {
    CardDefinition {
        name: "Wall of Water",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 5,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
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

/// Spitting Drake — {3}{R} 2/2 Drake. Flying. {R}: gets +1/+0 until end of
/// turn. Activate only once each turn.
pub fn spitting_drake() -> CardDefinition {
    CardDefinition {
        name: "Spitting Drake",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
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

/// Blood Pet — {B} 1/1 Thrull. Sacrifice this creature: Add {B}.
pub fn blood_pet() -> CardDefinition {
    CardDefinition {
        name: "Blood Pet",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thrull],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Black]),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Foul Imp — {B}{B} 2/2 Imp. Flying. When it enters, you lose 2 life.
pub fn foul_imp() -> CardDefinition {
    CardDefinition {
        name: "Foul Imp",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Imp],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::LoseLife {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Skyshroud Vampire — {3}{B}{B} 3/3 Vampire. Flying. Discard a creature card:
/// gets +2/+2 until end of turn.
pub fn skyshroud_vampire() -> CardDefinition {
    CardDefinition {
        name: "Skyshroud Vampire",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Creature, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Feral Shadow — {2}{B} 2/1 Nightstalker. Flying.
pub fn feral_shadow() -> CardDefinition {
    CardDefinition {
        name: "Feral Shadow",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightstalker],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Rowan Treefolk — {3}{G} 3/4 Treefolk (vanilla).
pub fn rowan_treefolk() -> CardDefinition {
    CardDefinition {
        name: "Rowan Treefolk",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        ..Default::default()
    }
}

/// Sabertooth Nishoba — {4}{G}{W} 5/5 Cat Beast Warrior. Trample; protection
/// from blue and from red.
pub fn sabertooth_nishoba() -> CardDefinition {
    CardDefinition {
        name: "Sabertooth Nishoba",
        cost: cost(&[generic(4), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Cat,
                CreatureType::Beast,
                CreatureType::Warrior,
            ],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![
            Keyword::Trample,
            Keyword::Protection(Color::Blue),
            Keyword::Protection(Color::Red),
        ],
        ..Default::default()
    }
}

/// Kris Mage — {R} 1/1 Human Spellshaper. {R}, {T}, Discard a card: deals 1
/// damage to any target.
pub fn kris_mage() -> CardDefinition {
    CardDefinition {
        name: "Kris Mage",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Spellshaper],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
