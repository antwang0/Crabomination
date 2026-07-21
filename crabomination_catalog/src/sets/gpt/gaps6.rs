//! Guildpact (GPT) gap wave 6: a Rhystic-tax edict and a sacrifice-to-retarget
//! Goblin. Reuses `Effect::UnlessPlayerPays` and `ChooseNewTargetsForSpell`.
//! Tests in `classic_sets/gpt`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef, Selector};
use crate::mana::{cost, generic, r, u, w};

/// Spelltithe Enforcer — {3}{W}{W} 3/3 Elephant Wizard. Whenever an opponent
/// casts a spell, that player sacrifices a permanent of their choice unless
/// they pay {1}.
pub fn spelltithe_enforcer() -> CardDefinition {
    CardDefinition {
        name: "Spelltithe Enforcer",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::Triggerer,
                cost: crate::card::WardCost::generic(1),
                then: Box::new(Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::Triggerer),
                    count: Value::ONE,
                    filter: R::Permanent,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Goblin Flectomancer — {U}{R}{R} 2/2 Goblin Wizard. Sacrifice this creature:
/// You may change the targets of target instant or sorcery spell.
pub fn goblin_flectomancer() -> CardDefinition {
    CardDefinition {
        name: "Goblin Flectomancer",
        cost: cost(&[u(), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::ChooseNewTargetsForSpell {
                what: target_filtered(
                    R::IsSpellOnStack
                        .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
