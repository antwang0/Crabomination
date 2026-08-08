//! Deferred follow-ups cleared: the draw-second-card payoff + set-base-P/T
//! pump (Jolrael), a land-count intervening-if (Loyal Warhound), pay-up-to-X
//! reflexive draw (Well of Lost Dreams, new `Effect::MayPayGenericUpTo`), and
//! enters-with-counters-per-other-creature + remove-counter activation
//! (Custodi Soulbinders). Tests in `tests/recent60.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{Color, cost, g, generic, w};

/// Jolrael, Mwonvuli Recluse — {1}{G} 1/2 Legendary Human Druid. Whenever you
/// draw your second card each turn, create a 2/2 green Cat. {4}{G}{G}: Until end
/// of turn, creatures you control have base P/T X/X (X = cards in your hand).
pub fn jolrael_mwonvuli_recluse() -> CardDefinition {
    let cat = TokenDefinition {
        name: "Cat".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Jolrael, Mwonvuli Recluse",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                .with_filter(Predicate::PlayerDrewAtLeastThisTurn {
                    who: PlayerRef::You,
                    n: 2,
                })
                .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(cat),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g(), g()]),
            effect: Effect::SetBasePT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::HandSizeOf(PlayerRef::You),
                toughness: Value::HandSizeOf(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Loyal Warhound — {1}{W} 3/1 Dog with vigilance. ETB, if an opponent controls
/// more lands than you, search for a basic Plains onto the battlefield tapped.
pub fn loyal_warhound() -> CardDefinition {
    CardDefinition {
        name: "Loyal Warhound",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::OpponentControlsMoreLandsThanYou,
            then: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::HasLandType(LandType::Plains).and(R::HasSupertype(Supertype::Basic)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Well of Lost Dreams — {4} Artifact. Whenever you gain life, you may pay {X}
/// (X ≤ life gained); if you do, draw X cards.
pub fn well_of_lost_dreams() -> CardDefinition {
    CardDefinition {
        name: "Well of Lost Dreams",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::MayPayGenericUpTo {
                max: Value::TriggerEventAmount,
                body: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::TriggerEventAmount,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Custodi Soulbinders — {3}{W} 0/0 Human Cleric. Enters with X +1/+1 counters
/// (X = other creatures on the battlefield). {2}{W}, Remove a +1/+1 counter:
/// create a 1/1 white flying Spirit.
pub fn custodi_soulbinders() -> CardDefinition {
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Custodi Soulbinders",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        // "other creatures" = all creatures minus this one (already on the
        // battlefield when the enters-with count is evaluated).
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::Diff(
                Box::new(Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Creature,
                )))),
                Box::new(Value::Const(1)),
            ),
        )),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(spirit),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
