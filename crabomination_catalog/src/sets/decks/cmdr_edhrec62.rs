//! Commander: EDHREC ranks 2001-2500, one-card gaps — Darigaaz the Igniter's
//! Khorvath's Fury, Hope of Ghirapur's Ghostfire Blade, and Istvan, Butcher of
//! Eln. Tests in `tests/recent_b/cmdr_edhrec62.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility,
    Value,
};
use crate::effect::{Effect, PlayerRef};
use crate::mana::{b, cost, generic, r};

/// Khorvath's Fury — {4}{R} sorcery. For each player, choose friend or foe.
/// Each friend discards their hand, then draws that many plus one; it deals
/// damage to each foe equal to the cards in their hand.
pub fn khorvaths_fury() -> CardDefinition {
    CardDefinition {
        name: "Khorvath's Fury",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::FriendOrFoe {
            friend: Box::new(Effect::Seq(vec![
                Effect::DiscardHandDrawThatMany { who: Selector::You },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ])),
            foe: Box::new(Effect::DealDamage {
                to: Selector::You,
                amount: Value::HandSizeOf(PlayerRef::You),
            }),
        },
        ..Default::default()
    }
}

/// Ghostfire Blade — {1} Equipment. Equipped creature gets +2/+2. Equip {3};
/// it costs {2} less if it targets a colorless creature.
pub fn ghostfire_blade() -> CardDefinition {
    CardDefinition {
        name: "Ghostfire Blade",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 2, ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "This Equipment's equip ability costs {2} less to activate if it targets a colorless creature.",
            effect: StaticEffect::EquipCostReducedIfTarget { filter: R::Colorless, amount: 2 },
        }],
        ..Default::default()
    }
}

/// Istvan, Butcher of Eln — {1}{B}{B}{B} legendary 1/3 Human. Attacks each
/// combat if able. Prevent all damage that would be dealt to it by creatures.
/// Whenever a creature an opponent controls dies, two +1/+1 counters on it.
pub fn istvan_butcher_of_eln() -> CardDefinition {
    CardDefinition {
        name: "Istvan, Butcher of Eln",
        cost: cost(&[generic(1), b(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::MustAttack],
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt to Istvan by creatures.",
            effect: StaticEffect::PreventAllDamageToThisFromCreatures,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
        }],
        ..Default::default()
    }
}
