//! Commander: the 2026-10-20 (`01DDeUtc`) closers — Hakim, Loreweaver's
//! Auras. Tests in `tests/recent_b/cmdr_01dd_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, Predicate, Selector};
use crate::mana::{cost, generic, u};

fn aura() -> Subtypes {
    Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() }
}

/// Crown of the Ages — {2} artifact. {4}, {T}: attach target Aura attached to
/// a creature to another creature.
pub fn crown_of_the_ages() -> CardDefinition {
    CardDefinition {
        name: "Crown of the Ages",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::Attach {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura).and(R::AttachedToCreature),
                },
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Drake Umbra — {4}{U} Aura. Enchanted creature gets +3/+3 and has flying.
/// Umbra armor.
pub fn drake_umbra() -> CardDefinition {
    CardDefinition {
        name: "Drake Umbra",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        keywords: vec![Keyword::UmbraArmor],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: 3, toughness: 3, keywords: vec![Keyword::Flying], ..Default::default() }),
        ..Default::default()
    }
}

/// Illusory Gains — {3}{U}{U} Aura. You control enchanted creature. Whenever
/// a creature an opponent controls enters, attach this Aura to it — the steal
/// follows the Aura, so the old host goes home.
pub fn illusory_gains() -> CardDefinition {
    CardDefinition {
        name: "Illusory Gains",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![
            etb(Effect::GainControlWhileSourceAttached),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::OpponentControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
                effect: Effect::Seq(vec![
                    Effect::Attach { what: Selector::This, to: Selector::TriggerSource },
                    Effect::GainControlWhileSourceAttached,
                ]),
            },
        ],
        ..Default::default()
    }
}
