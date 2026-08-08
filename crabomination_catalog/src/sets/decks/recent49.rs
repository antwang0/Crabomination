//! Graveyard-matters green/Golgari: self-milling *-creatures, recursion, and two
//! Auras. Tests in `tests/recent49.rs`.

use crate::card::EquipBonus;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic};

/// Ghoultree — {7}{G} 10/10 Zombie Treefolk. Costs {1} less for each creature
/// card in your graveyard.
pub fn ghoultree() -> CardDefinition {
    CardDefinition {
        name: "Ghoultree",
        cost: cost(&[generic(7), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Treefolk],
            ..Default::default()
        },
        power: 10,
        toughness: 10,
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each creature card in your graveyard.",
            effect: StaticEffect::SelfCostReducedPerCreatureInGraveyard,
        }],
        ..Default::default()
    }
}

/// Nyx Weaver — {1}{B}{G} 2/3 enchantment Spider, reach. At your upkeep, mill
/// two. {1}{B}{G}, exile this: return target card from your graveyard to hand.
pub fn nyx_weaver() -> CardDefinition {
    CardDefinition {
        name: "Nyx Weaver",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Mill {
                who: Selector::You,
                amount: Value::Const(2),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), g()]),
            exile_self_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::InYourGraveyard.and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Genesis — {4}{G} 4/4 Incarnation. At your upkeep, if it's in your graveyard,
/// you may pay {2}{G}. If you do, return target creature card from your graveyard
/// to your hand.
pub fn genesis() -> CardDefinition {
    CardDefinition {
        name: "Genesis",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Incarnation],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::FromYourGraveyard,
            ),
            effect: Effect::MayPay {
                description: "Pay {2}{G} to return a creature card from your graveyard.".into(),
                mana_cost: cost(&[generic(2), g()]),
                body: Box::new(Effect::Move {
                    what: target_filtered(
                        R::Creature.and(R::InYourGraveyard).and(R::OtherThanSource),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Elephant Guide — {2}{G} Aura. Enchanted creature gets +3/+3. When it dies,
/// create a 3/3 green Elephant token.
pub fn elephant_guide() -> CardDefinition {
    let elephant = TokenDefinition {
        name: "Elephant".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Elephant Guide",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: Box::new(elephant),
            },
        }],
        ..Default::default()
    }
}

/// Moldervine Cloak — {2}{G} Aura. Enchanted creature gets +3/+3. Dredge 2.
pub fn moldervine_cloak() -> CardDefinition {
    CardDefinition {
        name: "Moldervine Cloak",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Dredge(2)],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            ..Default::default()
        }),
        ..Default::default()
    }
}
