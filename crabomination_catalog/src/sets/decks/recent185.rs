//! BLB gap batch on existing primitives: Thought Shucker (threshold
//! activate-once), Shoreline Looter (unblockable combat-damage loot), Ruthless
//! Negotiation (opponent hand-exile + Flashback), Seasoned Warrenguard
//! (token-gated attack pump), and Valley Flamecaller (new
//! `StaticEffect::ControlledCreatureTypesDealExtraDamage`). Tests in
//! `crabomination/src/tests/recent185.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    TriggeredAbility, Value,
};
use crate::effect::{Effect, PlayerRef, Predicate, Selector};
use crate::mana::{b, cost, generic, r, u, w};

/// Thought Shucker — {1}{U} 1/3 Rat Rogue. Threshold — {1}{U}: Put a +1/+1
/// counter on it and draw a card. Activate only with 7+ cards in your graveyard
/// and only once.
pub fn thought_shucker() -> CardDefinition {
    CardDefinition {
        name: "Thought Shucker",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            activate_once: true,
            condition: Some(Predicate::ThresholdActive { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Shoreline Looter — {1}{U} 1/1 Rat Rogue. Can't be blocked. Threshold —
/// Whenever it deals combat damage to a player, draw a card, then discard a
/// card unless there are 7+ cards in your graveyard.
pub fn shoreline_looter() -> CardDefinition {
    CardDefinition {
        name: "Shoreline Looter",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ThresholdActive { who: PlayerRef::You },
                    then: Box::new(Effect::Noop),
                    else_: Box::new(Effect::Discard {
                        who: Selector::You,
                        amount: Value::ONE,
                        random: false,
                    }),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Ruthless Negotiation — {B} Sorcery. Target opponent exiles a card from their
/// hand. If this spell was cast from a graveyard, draw a card. Flashback {4}{B}.
pub fn ruthless_negotiation() -> CardDefinition {
    CardDefinition {
        name: "Ruthless Negotiation",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(4), b()]))],
        effect: Effect::Seq(vec![
            Effect::ExileFromHand { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            Effect::If {
                cond: Predicate::CastFromGraveyard,
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Seasoned Warrenguard — {W} 1/2 Rabbit Warrior. Whenever it attacks while you
/// control a token, it gets +2/+0 until end of turn.
pub fn seasoned_warrenguard() -> CardDefinition {
    CardDefinition {
        name: "Seasoned Warrenguard",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::SelectorExists(Selector::EachPermanent(
                    R::IsToken.and(R::ControlledByYou),
                )),
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: crate::effect::Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Valley Flamecaller — {2}{R} 3/3 Lizard Warlock. If a Lizard, Mouse, Otter,
/// or Raccoon you control would deal damage to a permanent or player, it deals
/// that much damage plus 1 instead.
pub fn valley_flamecaller() -> CardDefinition {
    CardDefinition {
        name: "Valley Flamecaller",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Your Lizards, Mice, Otters, and Raccoons deal 1 extra damage.",
            effect: StaticEffect::ControlledCreatureTypesDealExtraDamage {
                types: vec![
                    CreatureType::Lizard,
                    CreatureType::Mouse,
                    CreatureType::Otter,
                    CreatureType::Raccoon,
                ],
                amount: 1,
            },
        }],
        ..Default::default()
    }
}
