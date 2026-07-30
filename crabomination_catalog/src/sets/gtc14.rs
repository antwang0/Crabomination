//! Gatecrash (GTC) wave 14: a lock-down Aura and a Battalion counter-medic.
//! Tests in `classic_sets/gtc`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{battalion, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector};
use crate::mana::{b, cost, generic, w};

/// One Thousand Lashes — {2}{W}{B} Aura. Enchant creature; it can't attack or
/// block and its activated abilities can't be activated; at the upkeep of its
/// controller, that player loses 1 life.
pub fn one_thousand_lashes() -> CardDefinition {
    let attached = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "One Thousand Lashes",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(crate::card::EquipBonus {
            keywords: vec![
                Keyword::CantAttack,
                Keyword::CantBlock,
                Keyword::CantActivateAbilities,
            ],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            )
            .with_filter(Predicate::ActivePlayerControls(Box::new(attached()))),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(attached()))),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Frontline Medic — {2}{W} 3/3 Human Cleric. Battalion: creatures you control
/// gain indestructible; Sacrifice: counter target spell with {X} in its mana
/// cost unless its controller pays {3}.
pub fn frontline_medic() -> CardDefinition {
    CardDefinition {
        name: "Frontline Medic",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![battalion(Effect::GrantKeyword {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            keyword: Keyword::Indestructible,
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack.and(R::HasXInCost)),
                mana_cost: cost(&[generic(3)]),
                exile: false,
                extra_generic: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
