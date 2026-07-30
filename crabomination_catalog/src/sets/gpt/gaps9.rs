//! Guildpact (GPT) closure — the last four gap cards. Tests in
//! `classic_sets/gpt`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, StaticAbility,
    Subtypes,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, Predicate, Selector, StaticEffect, TriggeredAbility,
};
use crate::mana::{Color, cost, generic, hybrid, r, u};

fn types(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}

/// Aetherplasm — {2}{U}{U} 1/1. When it blocks, you may bounce it and drop a
/// creature from hand in as the replacement blocker.
pub fn aetherplasm() -> CardDefinition {
    CardDefinition {
        name: "Aetherplasm",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Illusion]),
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Return this and deploy a blocker from hand?".into(),
                body: Box::new(Effect::ReturnSelfDeployBlocker),
            },
        }],
        ..Default::default()
    }
}

/// Djinn Illuminatus — {5}{U/R}{U/R} 3/5 flying. Your instants and sorceries
/// have replicate for their own mana cost.
pub fn djinn_illuminatus() -> CardDefinition {
    let ur = || hybrid(Color::Blue, Color::Red);
    CardDefinition {
        name: "Djinn Illuminatus",
        cost: cost(&[generic(5), ur(), ur()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Djinn]),
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Each instant and sorcery spell you cast has replicate.",
            effect: StaticEffect::YourISSpellsHaveReplicate,
        }],
        ..Default::default()
    }
}

/// Ink-Treader Nephilim — {R}{G}{W}{U} 3/3. A spell aimed only at it is copied
/// for every other creature it could hit.
pub fn ink_treader_nephilim() -> CardDefinition {
    CardDefinition {
        name: "Ink-Treader Nephilim",
        cost: cost(&[r(), crate::mana::g(), crate::mana::w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Nephilim]),
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Instant)
                        .or(R::HasCardType(CardType::Sorcery))
                        .and(R::SpellTargetsOnlySource),
                },
            ),
            effect: Effect::CopySpellForEachOtherLegalCreature {
                what: Selector::TriggerSource,
            },
        }],
        ..Default::default()
    }
}

/// Mimeofacture — {3}{U} Sorcery with replicate {3}{U}. Steal a copy of target
/// permanent an opponent controls out of their own library.
pub fn mimeofacture() -> CardDefinition {
    CardDefinition {
        name: "Mimeofacture",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Replicate(cost(&[generic(3), u()]))],
        effect: Effect::SearchOpponentLibraryForSameName {
            what: target_filtered(R::Permanent.and(R::ControlledByOpponent)),
        },
        ..Default::default()
    }
}
