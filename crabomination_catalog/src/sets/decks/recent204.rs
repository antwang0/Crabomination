//! DSK gap batch 3 — an attack-sac Equipment, a lose-abilities Toy Aura, a
//! damage-destroy edict Aura, and a manifest-dread recursion sorcery. Tests in
//! `tests/recent204.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus, Keyword,
    SelectionRequirement as R, Subtypes, Value,
};
use crate::effect::shortcut::{draw, etb, target_filtered};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, TriggeredAbility,
};
use crate::mana::{b, cost, generic, g, u};

/// Saw — {2} Artifact — Equipment. Equipped creature gets +2/+0. Whenever it
/// attacks, you may sacrifice another permanent to draw a card. Equip {2}.
pub fn saw() -> CardDefinition {
    CardDefinition {
        name: "Saw",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 0,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MaySacrifice {
                    description: "Sacrifice another permanent to draw a card.".into(),
                    filter: R::ControlledByYou,
                    count: Value::ONE,
                    then: Box::new(draw(1)),
                    else_: None,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Unable to Scream — {U} Aura. Enchanted creature loses all abilities and is a
/// 0/2 Toy artifact creature in addition to its other types.
pub fn unable_to_scream() -> CardDefinition {
    CardDefinition {
        name: "Unable to Scream",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            set_base_pt: Some((0, 2)),
            set_card_types: Some(vec![CardType::Artifact, CardType::Creature]),
            set_creature_types: Some(vec![CreatureType::Toy]),
            remove_abilities: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Sporogenic Infection — {1}{B} Aura. On enter, target player sacrifices a
/// creature. When enchanted creature is dealt damage, destroy it. (The
/// "other than enchanted creature" sacrifice clause is approximated away.)
pub fn sporogenic_infection() -> CardDefinition {
    CardDefinition {
        name: "Sporogenic Infection",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![
            etb(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Creature,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::EnchantedBySource),
                effect: Effect::Destroy { what: Selector::TriggerSource },
            },
        ],
        ..Default::default()
    }
}

/// Under the Skin — {2}{G} Sorcery. Manifest dread, then you may return a
/// permanent card from your graveyard to your hand.
pub fn under_the_skin() -> CardDefinition {
    CardDefinition {
        name: "Under the Skin",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ManifestDread { who: PlayerRef::You },
            Effect::ReturnGraveyardCardsToHand { filter: R::Permanent, max: Value::ONE },
        ]),
        ..Default::default()
    }
}
