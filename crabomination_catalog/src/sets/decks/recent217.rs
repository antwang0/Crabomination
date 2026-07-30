//! Bloomburrow / Foundations creature batch — Serra Redeemer (small-creature
//! counter payoff), Wandertale Mentor (expend + dual mana), and Starseer Mentor
//! (life-swing Punisher). Tests in `tests/recent217.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Predicate, Selector, Value,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, w};

/// Serra Redeemer — {3}{W}{W} 2/4 Angel Soldier. Flying. Whenever another
/// creature you control with power 2 or less enters, put two +1/+1 counters on
/// that creature.
pub fn serra_redeemer() -> CardDefinition {
    CardDefinition {
        name: "Serra Redeemer",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::PowerAtMost(2)),
                }),
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Wandertale Mentor — {R}{G} 2/2 Raccoon Bard. Whenever you expend 4, put a
/// +1/+1 counter on it. {T}: Add {R} or {G}.
pub fn wandertale_mentor() -> CardDefinition {
    let tap_for = |c: Color| ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colors(vec![c]),
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Wandertale Mentor",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Raccoon, CreatureType::Bard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Expend, EventScope::YourControl)
                .with_filter(Predicate::ExpendReached(4)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![tap_for(Color::Red), tap_for(Color::Green)],
        ..Default::default()
    }
}

/// Starseer Mentor — {3}{W}{B} 3/5 Bat Warlock. Flying, vigilance. At your end
/// step, if you gained or lost life this turn, each opponent loses 3 life unless
/// they sacrifice a nonland permanent or discard a card. (Printed "target
/// opponent" modeled as each-opponent, per the aristocrat convention.)
pub fn starseer_mentor() -> CardDefinition {
    CardDefinition {
        name: "Starseer Mentor",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bat, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::Any(vec![
                Predicate::PlayerGainedLifeThisTurn {
                    who: PlayerRef::You,
                },
                Predicate::PlayerLostLifeThisTurn {
                    who: PlayerRef::You,
                },
            ])),
            effect: Effect::Punisher {
                // Each opponent chooses: the options run with the chooser as
                // controller (`You` = that opponent), so they sac/discard their
                // own; the `otherwise` payoff runs with the source's controller
                // as context, draining each opponent.
                chooser: Selector::Player(PlayerRef::EachOpponent),
                options: vec![
                    Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::You),
                        count: Value::Const(1),
                        filter: R::Nonland,
                    },
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::You),
                        amount: Value::Const(1),
                        random: false,
                    },
                ],
                otherwise: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                }),
            },
        }],
        ..Default::default()
    }
}
