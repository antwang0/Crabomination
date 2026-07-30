//! BLB gap batch (wave 2) — on existing primitives: Finneas, Ace Archer
//! (attack tribal counters + power-gated draw) and Gev, Scaled Scorch (Ward +
//! Lizard-cast ping). Tests in `crabomination/src/tests/recent182.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{on_you_attack, target_filtered};
use crate::effect::{Effect, Predicate, Selector};
use crate::mana::{b, cost, g, r, w};

/// Finneas, Ace Archer — {G}{W} 2/2 legendary Rabbit Archer with vigilance and
/// reach. Whenever it attacks, put a +1/+1 counter on each other token or Rabbit
/// creature you control; then draw a card if your creatures have total power 10+.
pub fn finneas_ace_archer() -> CardDefinition {
    CardDefinition {
        name: "Finneas, Ace Archer",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Archer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance, Keyword::Reach],
        triggered_abilities: vec![on_you_attack(Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource)
                        .and(R::IsToken.or(R::HasCreatureType(CreatureType::Rabbit))),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::PowerOf(Box::new(Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou),
                    ))),
                    Value::Const(10),
                ),
                then: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..Default::default()
    }
}

/// Gev, Scaled Scorch — {B}{R} 3/2 legendary Lizard Mercenary. Ward—Pay 2 life.
/// Whenever you cast a Lizard spell, Gev deals 1 damage to target opponent.
/// (The enters-with-extra-counters static is omitted — no engine primitive yet.)
pub fn gev_scaled_scorch() -> CardDefinition {
    CardDefinition {
        name: "Gev, Scaled Scorch",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Ward(WardCost::Life(2))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Lizard),
                },
            ),
            effect: Effect::DealDamage {
                to: target_filtered(R::OpponentPlayer),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}
