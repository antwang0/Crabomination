//! Assorted commons wave — Rampage, morph bodies, landwalk, a landfall
//! self-buff, a tap-artifact utility creature, and a Bestow aura-creature. All
//! ride existing engine primitives. Tests in `tests/recent69.rs`.

use crate::card::SelectionRequirement as R;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EquipBonus, Keyword, LandType,
    Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{dies_gain_life, target_filtered};
use crate::effect::{Duration, Effect, EventKind, EventScope, EventSpec, Selector};
use crate::mana::{cost, g, generic, r, w};

/// Frost Giant — {3}{R}{R}{R} 4/4 Giant. Rampage 2.
pub fn frost_giant() -> CardDefinition {
    CardDefinition {
        name: "Frost Giant",
        cost: cost(&[generic(3), r(), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Rampage(2)],
        ..Default::default()
    }
}

/// Highland Game — {1}{G} 2/1 Elk. When it dies, you gain 2 life.
pub fn highland_game() -> CardDefinition {
    CardDefinition {
        name: "Highland Game",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elk],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![dies_gain_life(2)],
        ..Default::default()
    }
}

/// Rushwood Dryad — {1}{G} 2/1 Dryad. Forestwalk.
pub fn rushwood_dryad() -> CardDefinition {
    CardDefinition {
        name: "Rushwood Dryad",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dryad],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        ..Default::default()
    }
}

/// Ainok Tracker — {5}{R} 3/3 Dog Scout. First strike. Morph {4}{R}.
pub fn ainok_tracker() -> CardDefinition {
    CardDefinition {
        name: "Ainok Tracker",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![
            Keyword::FirstStrike,
            Keyword::Morph(cost(&[generic(4), r()])),
        ],
        ..Default::default()
    }
}

/// Charging Slateback — {4}{R} 4/3 Beast. Can't block. Morph {4}{R}.
pub fn charging_slateback() -> CardDefinition {
    CardDefinition {
        name: "Charging Slateback",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::CantBlock, Keyword::Morph(cost(&[generic(4), r()]))],
        ..Default::default()
    }
}

/// Auriok Transfixer — {W} 1/1 Human Scout. {W}, {T}: Tap target artifact.
pub fn auriok_transfixer() -> CardDefinition {
    CardDefinition {
        name: "Auriok Transfixer",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::Tap {
                what: target_filtered(R::Artifact),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Snapping Creeper — {2}{G} 2/3 Plant. Landfall — whenever a land you control
/// enters, it gains vigilance until end of turn.
pub fn snapping_creeper() -> CardDefinition {
    CardDefinition {
        name: "Snapping Creeper",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Nyxborn Rollicker — {R} 1/1 Enchantment Creature — Satyr. Bestow {1}{R};
/// enchanted creature gets +1/+1.
pub fn nyxborn_rollicker() -> CardDefinition {
    CardDefinition {
        name: "Nyxborn Rollicker",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Satyr],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        bestow: Some(cost(&[generic(1), r()])),
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            ..Default::default()
        }),
        ..Default::default()
    }
}
