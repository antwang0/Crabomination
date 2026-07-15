//! Gap batch — DSK Delirium + OTJ Mounts/evasion, all on existing primitives.
//! Tests in `tests/recent230.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::attacks_while_saddled;
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, Value,
};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, r};

/// Wickerfolk Thresher — {3}{G} 5/4 Artifact Creature — Scarecrow. Delirium —
/// Whenever this attacks, if four or more card types are among cards in your
/// graveyard, look at the top card of your library; if it's a land you may put
/// it onto the battlefield, otherwise put it into your hand.
pub fn wickerfolk_thresher() -> CardDefinition {
    CardDefinition {
        name: "Wickerfolk Thresher",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Scarecrow], ..Default::default() },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::DeliriumActive { who: PlayerRef::You }),
            effect: Effect::RevealTopLandToBattlefieldElseHand { who: PlayerRef::You },
        }],
        ..Default::default()
    }
}

/// Resilient Roadrunner — {1}{R} 2/2 Bird. Haste, protection from Coyotes.
/// {3}: This creature can't be blocked this turn except by creatures with haste.
pub fn resilient_roadrunner() -> CardDefinition {
    CardDefinition {
        name: "Resilient Roadrunner",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste, Keyword::ProtectionFromCreatureType(CreatureType::Coyote)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::CantBeBlockedExceptBy(Box::new(R::HasKeyword(Keyword::Haste))),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Giant Beaver — {3}{G} 4/4 Beaver Mount. Vigilance. Whenever this attacks
/// while saddled, put a +1/+1 counter on target creature that saddled it this
/// turn (approximated as a creature you control). Saddle 3.
pub fn giant_beaver() -> CardDefinition {
    CardDefinition {
        name: "Giant Beaver",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beaver, CreatureType::Mount],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Saddle(3)],
        triggered_abilities: vec![attacks_while_saddled(Effect::AddCounter {
            what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Ornery Tumblewagg — {2}{G} 2/2 Brushwagg Mount. At the beginning of combat
/// on your turn, put a +1/+1 counter on target creature. Whenever this attacks
/// while saddled, double the number of +1/+1 counters on target creature.
/// Saddle 2.
pub fn ornery_tumblewagg() -> CardDefinition {
    CardDefinition {
        name: "Ornery Tumblewagg",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Brushwagg, CreatureType::Mount],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Saddle(2)],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::AddCounter {
                    what: Selector::TargetFiltered { slot: 0, filter: R::Creature },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            attacks_while_saddled(Effect::DoubleCountersOnEach {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature },
                kind: CounterType::PlusOnePlusOne,
            }),
        ],
        ..Default::default()
    }
}
