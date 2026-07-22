//! Gatecrash (GTC) wave 9: an Equipment, a ping artifact, and two on-death
//! Auras. Tests in `classic_sets/gtc`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype,
    EquipBonus, Effect, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
    Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{target_any, target_filtered};
use crate::effect::{PlayerRef, Selector};
use crate::mana::{b, cost, generic, w, Color};

fn aura() -> Subtypes {
    Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() }
}

/// Skyblinder Staff — {1} Equipment. Equipped creature gets +1/+0 and can't be
/// blocked by creatures with flying. Equip {3}.
pub fn skyblinder_staff() -> CardDefinition {
    CardDefinition {
        name: "Skyblinder Staff",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::HasKeyword(Keyword::Flying)))],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Razortip Whip — {2} Artifact. {1}, {T}: deal 1 damage to any target.
pub fn razortip_whip() -> CardDefinition {
    CardDefinition {
        name: "Razortip Whip",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Murder Investigation — {1}{W} Aura. Enchant creature you control. When
/// enchanted creature dies, create X 1/1 white Soldiers, X = its power.
pub fn murder_investigation() -> CardDefinition {
    CardDefinition {
        name: "Murder Investigation",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::PowerOf(Box::new(Selector::TriggerSource)),
                definition: TokenDefinition {
                    name: "Soldier".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Soldier],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Dying Wish — {1}{B} Aura. Enchant creature you control. When enchanted
/// creature dies, target player loses X life and you gain X life, X = its power.
pub fn dying_wish() -> CardDefinition {
    CardDefinition {
        name: "Dying Wish",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Drain {
                from: target_filtered(R::Player),
                to: Selector::You,
                amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..Default::default()
    }
}
