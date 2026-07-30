//! LTR gap batch — Food makers, a looter, a scry-grower, Ring-tempt removal,
//! an Amass trick, and a base-P/T set. All on existing primitives. Tests in
//! `tests/recent_b/recent280.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType,
    SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::card::{EventKind, EventScope, EventSpec};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::mana::{b, cost, generic, r, u};

fn food() -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: crabomination_base::tokens::food_token(),
    }
}

/// Brandywine Farmer — {2}{G} 1/1 Halfling Peasant. When it enters or leaves the
/// battlefield, create a Food token.
pub fn brandywine_farmer() -> CardDefinition {
    CardDefinition {
        name: "Brandywine Farmer",
        cost: cost(&[generic(2), crate::mana::g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Halfling, CreatureType::Peasant],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            etb(food()),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: food(),
            },
        ],
        ..Default::default()
    }
}

/// Captain of Umbar — {2}{U} 2/3 Human Pirate. {1}, {T}: Draw a card, then
/// discard a card.
pub fn captain_of_umbar() -> CardDefinition {
    CardDefinition {
        name: "Captain of Umbar",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Chance-Met Elves — {2}{G} 3/2 Elf Warrior. Whenever you scry, put a +1/+1
/// counter on it. Once each turn.
pub fn chance_met_elves() -> CardDefinition {
    CardDefinition {
        name: "Chance-Met Elves",
        cost: cost(&[generic(2), crate::mana::g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ScriedOrSurveiled, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Cirith Ungol Patrol — {4}{B} 4/5 Orc Soldier. {1}, {T}, Sacrifice another
/// creature: Draw a card, then create a Food token.
pub fn cirith_ungol_patrol() -> CardDefinition {
    CardDefinition {
        name: "Cirith Ungol Patrol",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                food(),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Claim the Precious — {1}{B}{B} Sorcery. Destroy target creature. The Ring
/// tempts you.
pub fn claim_the_precious() -> CardDefinition {
    CardDefinition {
        name: "Claim the Precious",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Creature),
            },
            Effect::RingTempts {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}

/// Deceive the Messenger — {U} Instant. Target creature gets -3/-0 until end of
/// turn. Amass Orcs 1.
pub fn deceive_the_messenger() -> CardDefinition {
    CardDefinition {
        name: "Deceive the Messenger",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::Amass {
                who: PlayerRef::You,
                count: Value::ONE,
                extra_type: Some(CreatureType::Orc),
            },
        ]),
        ..Default::default()
    }
}

/// Dreadful as the Storm — {2}{U} Instant. Target creature has base power and
/// toughness 5/5 until end of turn. The Ring tempts you.
pub fn dreadful_as_the_storm() -> CardDefinition {
    CardDefinition {
        name: "Dreadful as the Storm",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::SetBasePT {
                what: target_filtered(R::Creature),
                power: Value::Const(5),
                toughness: Value::Const(5),
                duration: Duration::EndOfTurn,
            },
            Effect::RingTempts {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}

/// Breaking of the Fellowship — {1}{R} Sorcery. Target creature an opponent
/// controls deals damage equal to its power to another target creature that
/// player controls. The Ring tempts you.
pub fn breaking_of_the_fellowship() -> CardDefinition {
    CardDefinition {
        name: "Breaking of the Fellowship",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamageEqualToPower {
                source: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
                target: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            },
            Effect::RingTempts {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}
