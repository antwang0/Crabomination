//! Gatecrash (GTC) wave 12: a modal X-burn and Domri Rade. Tests in
//! `classic_sets/gtc`.

use crate::card::{
    CardDefinition, CardType, Keyword, PlaneswalkerSubtype, SelectionRequirement as R,
    StaticAbility, Subtypes, Supertype, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, LoyaltyAbility, PlayerRef, Selector, StaticEffect};
use crate::mana::{cost, g, r, x};

/// Clan Defiance — {X}{R}{G} Sorcery. Choose one or more: deal X to target
/// creature with flying; deal X to target creature without flying; deal X to
/// target player or planeswalker.
pub fn clan_defiance() -> CardDefinition {
    let dmg = |filter: R| Effect::DealDamage { to: target_filtered(filter), amount: Value::XFromCost };
    CardDefinition {
        name: "Clan Defiance",
        cost: cost(&[x(), r(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseModesCast {
            modes: vec![
                dmg(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                dmg(R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying))))),
                dmg(R::Player.or(R::Planeswalker)),
            ],
            min: 1,
            max: 3,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Domri Rade — {1}{R}{G} Planeswalker (loyalty 3). +1 reveal-a-creature dig,
/// −2 fight, −7 anthem-keyword emblem.
pub fn domri_rade() -> CardDefinition {
    let your_creatures = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    let anthem = |keyword: Keyword| StaticAbility {
        description: "Creatures you control have granted keywords",
        effect: StaticEffect::GrantKeyword { applies_to: your_creatures(), keyword },
    };
    CardDefinition {
        name: "Domri Rade",
        cost: cost(&[crate::mana::generic(1), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Domri], ..Default::default() },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                // "Look at the top card; if it's a creature, you may put it into
                // your hand." Modeled as reveal-take-if-creature (a non-creature
                // is bottomed rather than left on top).
                effect: Effect::RevealTopTakeMatchingToHand {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    filter: R::Creature,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Fight {
                    attacker: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                    defender: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Domri Rade".into(),
                    triggered: vec![],
                    statics: vec![
                        anthem(Keyword::DoubleStrike),
                        anthem(Keyword::Trample),
                        anthem(Keyword::Hexproof),
                        anthem(Keyword::Haste),
                    ],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
