//! Ravnica (RAV) gap wave 14: Razia's damage-redirect ability, riding the new
//! `Effect::RedirectNextDamage`. Tests in `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R,
    Subtypes, Supertype, Value,
};
use crate::effect::{Effect, Selector};
use crate::mana::{cost, generic, r, w};

/// Razia, Boros Archangel — {4}{R}{R}{W}{W} 6/3 Angel with flying, vigilance,
/// haste. {T}: The next 3 damage that would be dealt to target creature you
/// control this turn is dealt to another target creature instead.
pub fn razia_boros_archangel() -> CardDefinition {
    CardDefinition {
        name: "Razia, Boros Archangel",
        cost: cost(&[generic(4), r(), r(), w(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 6,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::RedirectNextDamage {
                target: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
