//! OTJ (Outlaws of Thunder Junction) gap batch — on existing primitives:
//! Ferocification (begin-combat modal pump), Freestrider Lookout (crime-gated
//! land dig), and Fleeting Reflection (hexproof + untap + become-a-copy). Tests
//! in `crabomination/src/tests/recent183.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, Selector};
use crate::game::TurnStep;
use crate::mana::{cost, g, generic, r, u};

/// Ferocification — {2}{R} Enchantment. At the beginning of combat on your turn,
/// choose one — target creature you control gets +2/+0, or gains menace and
/// haste until end of turn.
pub fn ferocification() -> CardDefinition {
    CardDefinition {
        name: "Ferocification",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::ChooseMode(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_filtered(R::Creature.and(R::ControlledByYou)),
                        keyword: Keyword::Menace,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            ]),
        }],
        ..Default::default()
    }
}

/// Freestrider Lookout — {2}{G} 3/3 Human Rogue with reach. Whenever you commit
/// a crime, look at the top five cards; you may put a land onto the battlefield
/// tapped, rest on the bottom in a random order. Once each turn.
pub fn freestrider_lookout() -> CardDefinition {
    CardDefinition {
        name: "Freestrider Lookout",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::LookTopPutMatchingOntoBattlefield {
                count: Value::Const(5),
                filter: R::Land,
                then: None,
                max: Some(1),
                tapped: true,
                exile_rest: false,
            },
        }],
        ..Default::default()
    }
}

/// Fleeting Reflection — {1}{U} Instant. Target creature you control gains
/// hexproof until end of turn and untaps; until end of turn it becomes a copy of
/// up to one other target creature. (The copy target is modeled as required.)
pub fn fleeting_reflection() -> CardDefinition {
    CardDefinition {
        name: "Fleeting Reflection",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::BecomeCopyOfFor {
                what: Selector::Target(0),
                source: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                duration: Duration::EndOfTurn,
                non_legendary: false,
            },
        ]),
        ..Default::default()
    }
}
