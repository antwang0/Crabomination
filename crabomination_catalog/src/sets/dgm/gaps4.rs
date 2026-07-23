//! Dragon's Maze (DGM) gap cards, wave 4 — the remaining primitive-blocked
//! cards. Tests in `classic_sets/dgm`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility,
};
use crate::effect::{Duration, Selector, StaticEffect};
use crate::game::TurnStep;
use crate::mana::{b, cost, generic, r, u, w};

/// Notion Thief — {2}{U}{B} 3/1 Human Rogue. Flash. If an opponent would draw a
/// card except the first one they draw in each of their draw steps, instead that
/// player skips that draw and you draw a card.
pub fn notion_thief() -> CardDefinition {
    CardDefinition {
        name: "Notion Thief",
        cost: cost(&[generic(2), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "If an opponent would draw a card except the first each draw step, you draw instead.",
            effect: StaticEffect::OpponentExtraDrawsRedirected,
        }],
        ..Default::default()
    }
}

/// Boros Battleshaper — {5}{R}{W} 5/5 Minotaur Soldier. At the beginning of each
/// combat, up to one target creature attacks or blocks this combat if able and
/// up to one target creature can't attack or block this combat. ("Attacks or
/// blocks if able" rides MustAttack+MustBlock; "if able" picks the applicable
/// one.)
pub fn boros_battleshaper() -> CardDefinition {
    let grant = |slot: u8, kw: Keyword| Effect::GrantKeyword {
        what: Selector::TargetFiltered { slot, filter: R::Creature },
        keyword: kw,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Boros Battleshaper",
        cost: cost(&[generic(5), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Soldier],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    grant(0, Keyword::MustAttack),
                    grant(0, Keyword::MustBlock),
                    grant(1, Keyword::CantAttack),
                    grant(1, Keyword::CantBlock),
                ])),
            },
        }],
        ..Default::default()
    }
}
