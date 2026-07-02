//! Tribal-anthem + spellslinger batch: chosen-type anthems (Obelisk of Urd,
//! Radiant Destiny), team pumps (Tempered Steel, Fires of Yavimaya), a
//! magecraft pinger (Gelectrode), and utility artifacts/creatures. Tests in
//! `tests/recent82.rs`.

use crate::card::{
    ActivatedAbility, CardType, CardDefinition, CreatureType, Keyword, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes,
};
use crate::effect::shortcut::{magecraft_self_untap, target_any, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, Value};
use crate::mana::{cost, generic, g, r, u, w, x};

/// Alloy Myr — {3} 2/2 Myr artifact creature. {T}: Add one mana of any color.
pub fn alloy_myr() -> CardDefinition {
    CardDefinition {
        name: "Alloy Myr",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Myr], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Courier's Capsule — {1}{U} Artifact. {1}{U}, {T}, Sacrifice this: Draw two.
pub fn couriers_capsule() -> CardDefinition {
    CardDefinition {
        name: "Courier's Capsule",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            sac_cost: true,
            effect: crate::effect::shortcut::draw(2),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ballista Squad — {3}{W} 2/2 Human Rebel. {X}{W}, {T}: This deals X damage to
/// target creature. (Printed "attacking or blocking" restriction dropped.)
pub fn ballista_squad() -> CardDefinition {
    CardDefinition {
        name: "Ballista Squad",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rebel],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), w()]),
            tap_cost: true,
            effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::XFromCost },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gelectrode — {1}{U}{R} 0/1 Weird. {T}: 1 damage to any target. Whenever you
/// cast an instant or sorcery, untap this creature.
pub fn gelectrode() -> CardDefinition {
    CardDefinition {
        name: "Gelectrode",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Weird], ..Default::default() },
        power: 0,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(1) },
            ..Default::default()
        }],
        triggered_abilities: vec![magecraft_self_untap()],
        ..Default::default()
    }
}

/// Rally the Peasants — {2}{W} Instant. Creatures you control get +2/+0 until
/// end of turn. Flashback {2}{R}.
pub fn rally_the_peasants() -> CardDefinition {
    CardDefinition {
        name: "Rally the Peasants",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(cost(&[generic(2), r()]))],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Tempered Steel — {1}{W}{W} Enchantment. Artifact creatures you control get
/// +2/+2.
pub fn tempered_steel() -> CardDefinition {
    CardDefinition {
        name: "Tempered Steel",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Artifact creatures you control get +2/+2.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Artifact.and(R::Creature).and(R::ControlledByYou),
                ),
                power: 2,
                toughness: 2,
            },
        }],
        ..Default::default()
    }
}

/// Radiant Destiny — {2}{W} Enchantment. As it enters, choose a creature type.
/// Creatures you control of the chosen type get +1/+1. (Ascend + the "with
/// city's blessing, they also have vigilance" rider are dropped.)
pub fn radiant_destiny() -> CardDefinition {
    CardDefinition {
        name: "Radiant Destiny",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::NameCreatureType {
            what: Selector::This,
        })],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control of the chosen type get +1/+1.",
            effect: StaticEffect::AnthemForChosenType {
                power: 1,
                toughness: 1,
                exclude_source: false,
                opponents: false, per_counter: None },
        }],
        ..Default::default()
    }
}

/// Fires of Yavimaya — {1}{R}{G} Enchantment. Creatures you control have haste.
/// Sacrifice this: target creature gets +2/+2 until end of turn.
pub fn fires_of_yavimaya() -> CardDefinition {
    CardDefinition {
        name: "Fires of Yavimaya",
        cost: cost(&[generic(1), r(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Haste,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}


