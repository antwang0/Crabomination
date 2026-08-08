//! A final OTJ batch: outlaw/crime payoffs, plot, and clean removal reusing
//! existing primitives. Tests in `tests/recent126.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, Predicate, SelectionRequirement as R,
    Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{drain, etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, Value, ZoneDest,
};
use crate::mana::{b, cost, generic, r, u, w};

/// Mine Raider — {2}{R} 3/2 Human Rogue with trample. ETB: if you control
/// another outlaw, create a Treasure.
pub fn mine_raider() -> CardDefinition {
    CardDefinition {
        name: "Mine Raider",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    R::IsOutlaw.and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                n: Value::ONE,
            },
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(crabomination_base::tokens::treasure_token()),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Scorching Shot — {R}{R} Sorcery. Deals 5 damage to target creature.
pub fn scorching_shot() -> CardDefinition {
    CardDefinition {
        name: "Scorching Shot",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::Const(5),
        },
        ..Default::default()
    }
}

/// Peerless Ropemaster — {4}{U} 4/4 Human Rogue. ETB: return up to one target
/// tapped creature to its owner's hand.
pub fn peerless_ropemaster() -> CardDefinition {
    CardDefinition {
        name: "Peerless Ropemaster",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Creature.and(R::Tapped)),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..Default::default()
    }
}

/// Spring Splasher — {1}{U} 2/1 Frog Beast. Whenever it attacks, target creature
/// the defending player controls gets -3/-0 until end of turn.
pub fn spring_splasher() -> CardDefinition {
    CardDefinition {
        name: "Spring Splasher",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-3),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Raven of Fell Omens — {1}{B} 1/2 Bird with flying. Whenever you commit a
/// crime, each opponent loses 1 life and you gain 1 (once each turn).
pub fn raven_of_fell_omens() -> CardDefinition {
    CardDefinition {
        name: "Raven of Fell Omens",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl)
                .once_per_turn(),
            effect: drain(1),
        }],
        ..Default::default()
    }
}

/// Stagecoach Security — {4}{W} 4/5 Human Soldier with Plot {3}{W}. ETB:
/// creatures you control get +1/+1 and gain vigilance until end of turn.
pub fn stagecoach_security() -> CardDefinition {
    CardDefinition {
        name: "Stagecoach Security",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        plot_cost: Some(cost(&[generic(3), w()])),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}
