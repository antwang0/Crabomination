//! OTJ gap batch on existing primitives: Malcolm, the Eyes (Flurry investigate),
//! Reach for the Sky (pump Aura + dies-draw), Tomb Trawler (graveyard-to-library
//! bottom), Steer Clear (Mount-scaled combat burn). Tests in `tests/recent195.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    Keyword, Predicate, SelectionRequirement as R, Subtypes, Supertype,
};
use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
use crate::effect::shortcut::{flurry, investigate, target_filtered};
use crate::effect::{Effect, LibraryPosition, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{cost, g, generic, r, u, w};

/// Malcolm, the Eyes — {U}{R} 2/2 Siren Pirate, Flying, haste. Whenever you cast
/// your second spell each turn, investigate.
pub fn malcolm_the_eyes() -> CardDefinition {
    CardDefinition {
        name: "Malcolm, the Eyes",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Siren, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![flurry(investigate(1))],
        ..Default::default()
    }
}

/// Reach for the Sky — {3}{G} Aura with Flash. Enchanted creature gets +3/+2 and
/// has reach. When it's put into a graveyard from the battlefield, draw a card.
pub fn reach_for_the_sky() -> CardDefinition {
    CardDefinition {
        name: "Reach for the Sky",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 2,
            keywords: vec![Keyword::Reach],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PermanentLeavesBattlefield,
                EventScope::SelfSource,
            ),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Tomb Trawler — {2} 0/4 Golem artifact creature. {2}: put target card from
/// your graveyard on the bottom of your library.
pub fn tomb_trawler() -> CardDefinition {
    CardDefinition {
        name: "Tomb Trawler",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Move {
                what: target_filtered(R::InYourGraveyard),
                to: ZoneDest::Library {
                    who: PlayerRef::You,
                    pos: LibraryPosition::Bottom,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Steer Clear — {W} Instant. Deals 2 damage to target attacking or blocking
/// creature; 4 instead if you control a Mount (as you cast is approximated as at
/// resolution).
pub fn steer_clear() -> CardDefinition {
    let attacker_or_blocker = R::Creature.and(R::IsAttacking.or(R::IsBlocking));
    CardDefinition {
        name: "Steer Clear",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(
                R::HasCreatureType(CreatureType::Mount).and(R::ControlledByYou),
            )),
            then: Box::new(Effect::DealDamage {
                to: target_filtered(attacker_or_blocker.clone()),
                amount: Value::Const(4),
            }),
            else_: Box::new(Effect::DealDamage {
                to: target_filtered(attacker_or_blocker),
                amount: Value::Const(2),
            }),
        },
        ..Default::default()
    }
}
