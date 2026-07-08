//! Modern Horizons 2 sweep, batch 7 — aftermath split, player hexproof,
//! sacrifice-provenance riders, discard-hand mana, per-counter activation
//! discounts. Tests in `tests/mh2f.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement, Selector, SplitCard, SplitHalf, Subtypes,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{draw, target_any, target_filtered};
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate, ZoneDest, ZoneRef};
use crate::mana::{b, cost, g, generic, r, u, w};

use SelectionRequirement as R;

/// Road // Ruin — {2}{G} instant: fetch a basic tapped // {1}{R}{R}
/// aftermath sorcery: damage a creature equal to your land count.
pub fn road_ruin() -> CardDefinition {
    CardDefinition {
        name: "Road // Ruin",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(1), r(), r()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::DealDamage {
                    amount: Value::count(Selector::EachMatching {
                        zone: ZoneRef::Battlefield,
                        filter: R::Land.and(R::ControlledByYou),
                    }),
                    to: target_filtered(R::Creature),
                },
            },
            fuse: false,
            aftermath: true,
        })),
        ..Default::default()
    }
}

/// Ethersworn Sphinx — {7}{W}{U} 4/4 flying, affinity for artifacts, cascade.
pub fn ethersworn_sphinx() -> CardDefinition {
    CardDefinition {
        name: "Ethersworn Sphinx",
        cost: cost(&[generic(7), w(), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sphinx], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        affinity_filter: Some(R::Artifact),
        triggered_abilities: vec![crate::effect::shortcut::cascade(9)],
        ..Default::default()
    }
}

/// Blossoming Calm — {W} instant. You gain hexproof until your next turn
/// and 2 life. Rebound.
pub fn blossoming_calm() -> CardDefinition {
    CardDefinition {
        name: "Blossoming Calm",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Rebound],
        effect: Effect::Seq(vec![
            Effect::GainHexproofUntilYourNextTurn { who: PlayerRef::You },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Foundry Helix — {1}{R}{W} instant. Sacrifice a permanent as a cost;
/// 4 damage to any target, gain 4 life if the fodder was an artifact.
pub fn foundry_helix() -> CardDefinition {
    CardDefinition {
        name: "Foundry Helix",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Permanent,
            count: 1,
        }],
        effect: Effect::Seq(vec![
            Effect::DealDamage { amount: Value::Const(4), to: target_any() },
            Effect::If {
                cond: Predicate::SacrificedWasArtifact,
                then: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(4),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Diamond Lion — {2} 2/2. {T}, Discard your hand, Sacrifice: add three
/// mana of any one color. Instant speed.
pub fn diamond_lion() -> CardDefinition {
    CardDefinition {
        name: "Diamond Lion",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            discard_hand_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(3)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Deepwood Denizen — {2}{G} 3/2 vigilance. {5}{G}, {T}: draw a card; {1}
/// less per +1/+1 counter on creatures you control.
pub fn deepwood_denizen() -> CardDefinition {
    CardDefinition {
        name: "Deepwood Denizen",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g()]),
            tap_cost: true,
            cost_reduction_per_counter: Some((
                CounterType::PlusOnePlusOne,
                R::Creature.and(R::ControlledByYou),
            )),
            effect: draw(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mount Velus Manticore — {2}{R}{R} 3/4. Combat on your turn: you may
/// discard a card; X damage to any target, X = its card types.
pub fn mount_velus_manticore() -> CardDefinition {
    CardDefinition {
        name: "Mount Velus Manticore",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Manticore], ..Default::default() },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::YourControl,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::MayDiscard {
                description: "discard a card to lob it?".into(),
                count: Value::ONE,
                then: Box::new(Effect::DealDamage {
                    amount: Value::LastDiscardedCardTypes,
                    to: target_any(),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Breathless Knight — {1}{W}{B} 2/2 flying, lifelink. A creature entering
/// from a graveyard (or cast from one) grows it.
pub fn breathless_knight() -> CardDefinition {
    CardDefinition {
        name: "Breathless Knight",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::All(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    },
                    Predicate::TriggerSourceEnteredFromGraveyard,
                ])),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}
