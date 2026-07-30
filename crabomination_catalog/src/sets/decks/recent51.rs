//! Black aristocrats/recursion and a green morbid pump. Tests in
//! `tests/recent51.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, Effect,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{on_dies, target_filtered};
use crate::effect::{PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic};
use crabomination_base::tokens::treasure_token;

fn etb(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect,
    }
}

/// Liliana's Standard Bearer — {2}{B} 3/1 Zombie Knight with flash. ETB draw X,
/// where X is the number of creatures that died under your control this turn.
pub fn lilianas_standard_bearer() -> CardDefinition {
    CardDefinition {
        name: "Liliana's Standard Bearer",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::ControllerCreaturesDiedThisTurn,
        })],
        ..Default::default()
    }
}

/// Skullport Merchant — {2}{B} 1/4 Dwarf Citizen. ETB create a Treasure. {1}{B},
/// sacrifice another creature or a Treasure: draw a card.
pub fn skullport_merchant() -> CardDefinition {
    CardDefinition {
        name: "Skullport Merchant",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Citizen],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: treasure_token(),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: Some((
                R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Treasure)),
                1,
            )),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bone Picker — {3}{B} 3/2 Bird with flying and deathtouch. Costs {3} less to
/// cast if a creature died this turn.
pub fn bone_picker() -> CardDefinition {
    CardDefinition {
        name: "Bone Picker",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {3} less to cast if a creature died this turn.",
            effect: StaticEffect::SelfCostReducedIfCreatureDiedThisTurn { amount: 3 },
        }],
        ..Default::default()
    }
}

/// Driver of the Dead — {3}{B} 3/2 Vampire. When it dies, return target creature
/// card with mana value 2 or less from your graveyard to the battlefield.
pub fn driver_of_the_dead() -> CardDefinition {
    CardDefinition {
        name: "Driver of the Dead",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![on_dies(Effect::Move {
            what: target_filtered(
                R::Creature
                    .and(R::InYourGraveyard)
                    .and(R::ManaValueAtMost(2))
                    .and(R::OtherThanSource),
            ),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        })],
        ..Default::default()
    }
}

/// Gixian Infiltrator — {1}{B} 2/1 Phyrexian Human. Whenever you sacrifice
/// another permanent, put a +1/+1 counter on it.
pub fn gixian_infiltrator() -> CardDefinition {
    CardDefinition {
        name: "Gixian Infiltrator",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Human],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::OtherThanSource,
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Hunger of the Howlpack — {G} Instant. Put a +1/+1 counter on target creature;
/// three instead if a creature died this turn (morbid).
pub fn hunger_of_the_howlpack() -> CardDefinition {
    CardDefinition {
        name: "Hunger of the Howlpack",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::CreaturesDiedThisTurnTotalAtLeast {
                at_least: Value::Const(1),
            },
            then: Box::new(Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            }),
            else_: Box::new(Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        },
        ..Default::default()
    }
}
