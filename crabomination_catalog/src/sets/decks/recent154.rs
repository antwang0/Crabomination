//! A cross-set wave (BLB / DSK / OTJ): an impulse Otter, a life-loss-matters
//! Gecko, an ETB value enchantment, and a graveyard-affinity Crab. All ride
//! existing primitives. Tests in `crabomination/src/tests/recent154.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, MayPlayDuration, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, Subtypes, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, StaticEffect,
};
use crate::mana::{b, cost, generic, r, u, Color};

/// Harnesser of Storms — {2}{R} 1/4 Otter Wizard. Once each turn, when you cast a
/// noncreature or Otter spell, impulse the top card (playable until end of turn).
pub fn harnesser_of_storms() -> CardDefinition {
    CardDefinition {
        name: "Harnesser of Storms",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec {
                once_per_turn: true,
                ..EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Not(Box::new(R::Creature))
                            .or(R::HasCreatureType(CreatureType::Otter)),
                    },
                )
            },
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false, pay_own_cost: false,
                uncast_penalty: None,
            },
        }],
        ..Default::default()
    }
}

/// Flamecache Gecko — {1}{R} 2/2 Lizard Warlock. ETB: if an opponent lost life
/// this turn, add {B}{R}. {1}{R}, discard a card: draw a card.
pub fn flamecache_gecko() -> CardDefinition {
    CardDefinition {
        name: "Flamecache Gecko",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent },
            then: Box::new(Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Black, Color::Red]),
            }),
            else_: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Intimidation Campaign — {1}{U}{B} Enchantment. ETB: each opponent loses 1
/// life, you gain 1, and you draw a card. (The commit-a-crime self-bounce rider
/// is omitted — the engine has no crime tracker yet.)
pub fn intimidation_campaign() -> CardDefinition {
    CardDefinition {
        name: "Intimidation Campaign",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]))],
        ..Default::default()
    }
}

/// Eddymurk Crab — {5}{U}{U} 5/5 Elemental Crab with flash. Costs {1} less per
/// instant/sorcery in your graveyard. Enters tapped if it's not your turn. ETB
/// taps up to two target creatures.
pub fn eddymurk_crab() -> CardDefinition {
    CardDefinition {
        name: "Eddymurk Crab",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental, CreatureType::Crab], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flash],
        affinity_graveyard_filter: Some(
            R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
        ),
        static_abilities: vec![StaticAbility {
            description: "Enters tapped if it's not your turn.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        triggered_abilities: vec![etb(Effect::TapUpToValue {
            count: Value::Const(2),
            filter: R::Creature,
            skip_untap: false,
        })],
        ..Default::default()
    }
}
