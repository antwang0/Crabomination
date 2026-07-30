//! Ravnica (RAV) gap wave 15: a flash Equipment and Indentured Oaf's
//! color-scoped damage prevention. Tests in `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R,
    StaticAbility, Subtypes, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect};
use crate::mana::{Color, cost, generic, r};

/// Grifter's Blade — {3} Equipment with flash. As it enters, attach it to a
/// creature you control. Equipped creature gets +1/+1. Equip {1}.
pub fn grifters_blade() -> CardDefinition {
    CardDefinition {
        name: "Grifter's Blade",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 1,
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        ..Default::default()
    }
}

/// Indentured Oaf — {3}{R} 4/3 Ogre Warrior. Prevent all damage that this
/// creature would deal to red creatures.
pub fn indentured_oaf() -> CardDefinition {
    CardDefinition {
        name: "Indentured Oaf",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage this creature would deal to red creatures.",
            effect: StaticEffect::PreventThisDamageToColor(Color::Red),
        }],
        ..Default::default()
    }
}

/// Spectral Searchlight — {3} Artifact. {T}: Choose a player. That player adds
/// one mana of any color they choose.
pub fn spectral_searchlight() -> CardDefinition {
    CardDefinition {
        name: "Spectral Searchlight",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::Target(0),
                pool: ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Molten Sentry — {3}{R} */* Elemental. As it enters, flip a coin: heads → a
/// 5/2 with haste; tails → a 2/5 with defender. (Modeled as a printed 2/5 whose
/// ETB coin flip animates it to 5/2 haste on heads and grants defender on tails.)
pub fn molten_sentry() -> CardDefinition {
    CardDefinition {
        name: "Molten Sentry",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::FlipCoin {
            count: Value::ONE,
            on_heads: Box::new(Effect::Seq(vec![
                Effect::SetBasePT {
                    what: Selector::This,
                    power: Value::Const(5),
                    toughness: Value::Const(2),
                    duration: Duration::Permanent,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Haste,
                    duration: Duration::Permanent,
                },
            ])),
            on_tails: Box::new(Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Defender,
                duration: Duration::Permanent,
            }),
        })],
        ..Default::default()
    }
}
