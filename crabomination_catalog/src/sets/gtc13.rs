//! Gatecrash (GTC) wave 13: a Dimir Cipher edict-drain and an X-tapper. Tests
//! in `classic_sets/gtc`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, SelectionRequirement as R, Subtypes,
    Value,
};
use crate::effect::shortcut::{extort, target_filtered};
use crate::effect::{Effect, Selector};
use crate::mana::{b, cost, generic, u, x};

/// Undercity Plague — {4}{B}{B} Sorcery. Target player loses 1 life, discards a
/// card, then sacrifices a permanent of their choice. Cipher.
pub fn undercity_plague() -> CardDefinition {
    let victim = || target_filtered(R::Player);
    CardDefinition {
        name: "Undercity Plague",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: victim(),
                amount: Value::ONE,
            },
            Effect::Discard {
                who: victim(),
                amount: Value::ONE,
                random: false,
            },
            Effect::Sacrifice {
                who: Selector::Target(0),
                count: Value::ONE,
                filter: R::Permanent,
            },
            Effect::Cipher,
        ]),
        ..Default::default()
    }
}

/// Thrull Parasite — {B} 1/1 Thrull. Extort; {T}, Pay 2 life: remove a counter
/// from target nonland permanent.
pub fn thrull_parasite() -> CardDefinition {
    CardDefinition {
        name: "Thrull Parasite",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thrull],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![extort()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 2,
            effect: Effect::RemoveAnyCounter {
                what: target_filtered(R::Permanent.and(R::Not(Box::new(R::Land)))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gridlock — {X}{U} Instant. Tap X target nonland permanents.
pub fn gridlock() -> CardDefinition {
    CardDefinition {
        name: "Gridlock",
        cost: cost(&[x(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::TapUpToValue {
            count: Value::XFromCost,
            filter: R::Permanent.and(R::Not(Box::new(R::Land))),
            skip_untap: false,
            exact: true,
        },
        ..Default::default()
    }
}
