//! OTJ/DSK gap batch — a spell-hush self-animating enchantment and a
//! Treasure-attack Mercenary. Tests in `tests/recent_b/recent274.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R,
    Subtypes, Supertype, TriggeredAbility,
};
use crate::card::{ArtifactSubtype, EventKind, EventScope, EventSpec, Predicate};
use crate::effect::shortcut::target_any;
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, r, u};

/// Emergent Haunting — {1}{U} Enchantment. At your end step, if you haven't cast
/// a spell from your hand this turn and this isn't a creature, it becomes a 3/3
/// blue Spirit with flying in addition to its types. {2}{U}: Surveil 1.
pub fn emergent_haunting() -> CardDefinition {
    CardDefinition {
        name: "Emergent Haunting",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::NoSpellCastFromHandThisTurn {
                who: PlayerRef::You,
            }),
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::Creature.negate(),
                },
                then: Box::new(Effect::Seq(vec![
                    Effect::AddCardTypeIndefinitely {
                        what: Selector::This,
                        card_type: CardType::Creature,
                        until_eot: false,
                    },
                    Effect::SetBasePT {
                        what: Selector::This,
                        power: Value::Const(3),
                        toughness: Value::Const(3),
                        duration: Duration::Permanent,
                    },
                    Effect::AddCreatureTypes {
                        what: Selector::This,
                        creature_types: vec![CreatureType::Spirit],
                        duration: Duration::Permanent,
                    },
                    Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::Flying,
                        duration: Duration::Permanent,
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Jolene, Plundering Pugilist — {1}{R}{G} 4/2 Human Mercenary. Whenever you
/// attack with one or more creatures with power 4+, create a Treasure. {1}{R},
/// Sacrifice a Treasure: Jolene deals 1 damage to any target.
pub fn jolene_plundering_pugilist() -> CardDefinition {
    CardDefinition {
        name: "Jolene, Plundering Pugilist",
        cost: cost(&[generic(1), r(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource).with_filter(
                Predicate::AttackedWithCreatureMatching {
                    who: PlayerRef::You,
                    filter: R::PowerAtLeast(4),
                },
            ),
            effect: crate::effect::shortcut::mint_treasures(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Treasure), 1)),
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
