//! Retro batch touching combat/counters/cast rules — Bog Rats (CR 509 block
//! filter), Serrated Arrows (CR 122 counter-as-cost + -1/-1), Echo, cast-a-
//! creature-spell sacrifice, end-step self-bounce. Tests in `tests/recent73.rs`.

use crate::card::SelectionRequirement as R;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword, LandType,
    Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{etb, etb_ping_any, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, Value,
    ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, w};

/// Bog Rats — {B} 1/1 Rat. Can't be blocked by Walls.
pub fn bog_rats() -> CardDefinition {
    CardDefinition {
        name: "Bog Rats",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::HasCreatureType(
            CreatureType::Wall,
        )))],
        ..Default::default()
    }
}

/// Serrated Arrows — {4} Artifact. Enters with three arrowhead counters. At the
/// beginning of your upkeep, if it has none, sacrifice it. {T}, Remove an
/// arrowhead counter: put a -1/-1 counter on target creature. (Arrowhead
/// counters are stored as charge counters.)
pub fn serrated_arrows() -> CardDefinition {
    CardDefinition {
        name: "Serrated Arrows",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Charge, Value::Const(3))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::If {
                cond: Predicate::ValueAtMost(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Charge,
                    },
                    Value::Const(0),
                ),
                then: Box::new(Effect::SacrificePermanent {
                    what: Selector::This,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Charge, 1)),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ghitu Slinger — {2}{R} 2/2 Human Nomad. Echo {2}{R}. When it enters, it
/// deals 2 damage to any target.
pub fn ghitu_slinger() -> CardDefinition {
    CardDefinition {
        name: "Ghitu Slinger",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Nomad],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Echo(cost(&[generic(2), r()]))],
        triggered_abilities: vec![etb_ping_any(2)],
        ..Default::default()
    }
}

/// Cackling Fiend — {2}{B}{B} 2/1 Zombie. When it enters, each opponent
/// discards a card.
pub fn cackling_fiend() -> CardDefinition {
    CardDefinition {
        name: "Cackling Fiend",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
            random: false,
        })],
        ..Default::default()
    }
}

/// Skittering Skirge — {B}{B} 3/2 Phyrexian Imp. Flying. When you cast a
/// creature spell, sacrifice this creature.
pub fn skittering_skirge() -> CardDefinition {
    CardDefinition {
        name: "Skittering Skirge",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Imp],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                },
            ),
            effect: Effect::SacrificePermanent {
                what: Selector::This,
            },
        }],
        ..Default::default()
    }
}

/// Highland Giant — {2}{R}{R} 3/4 Giant (vanilla).
pub fn highland_giant() -> CardDefinition {
    CardDefinition {
        name: "Highland Giant",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        ..Default::default()
    }
}

/// Shanodin Dryads — {G} 1/1 Nymph Dryad. Forestwalk.
pub fn shanodin_dryads() -> CardDefinition {
    CardDefinition {
        name: "Shanodin Dryads",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nymph, CreatureType::Dryad],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        ..Default::default()
    }
}

/// Mesa Falcon — {1}{W} 1/1 Bird. Flying. {1}{W}: gets +0/+1 until end of turn.
pub fn mesa_falcon() -> CardDefinition {
    CardDefinition {
        name: "Mesa Falcon",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(0),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Viashino Sandstalker — {1}{R}{R} 4/2 Lizard Warrior. Haste. At the
/// beginning of the end step, return it to its owner's hand.
pub fn viashino_sandstalker() -> CardDefinition {
    CardDefinition {
        name: "Viashino Sandstalker",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
        }],
        ..Default::default()
    }
}
