//! Commander: Whtz, the Bibliophile's deck cards that are legal on their
//! own (Whtz itself is a Rulebreaker and stays unseated). Tests in
//! `tests/recent_b/cmdr_edhrec35.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::{Effect, LibraryPosition, PlayerRef, ZoneDest};
use crate::mana::{cost, generic, u, w, ManaCost};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

/// Lost Auramancers — {2}{W}{W} 3/3 Human Wizard, vanishing 3. When it dies
/// with no time counters on it, you may search for an enchantment card and
/// put it onto the battlefield.
pub fn lost_auramancers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vanishing(3)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource).with_filter(Predicate::Not(Box::new(
                Predicate::TriggerSourceHadCounter(crate::card::CounterType::Time),
            ))),
            effect: Effect::MayDo {
                description: "Search for an enchantment and put it onto the battlefield?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Enchantment,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        ..creature("Lost Auramancers", cost(&[generic(2), w(), w()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 3)
    }
}

/// Reed Richards, Smartest Man — {5}{U} 2/4 legendary Human Scientist Hero,
/// reach. No maximum hand size. The first time you would draw a card each
/// turn, except your draw step's first draw, you draw four instead.
pub fn reed_richards_smartest_man() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        static_abilities: vec![
            static_ab("You have no maximum hand size.", StaticEffect::NoMaximumHandSize),
            static_ab(
                "The first time you would draw a card each turn except the first card you draw during each of your draw steps, you draw four cards instead.",
                StaticEffect::FirstExtraDrawEachTurnBecomes { count: 4 },
            ),
        ],
        ..legend(
            "Reed Richards, Smartest Man",
            cost(&[generic(5), u()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Hero],
            2,
            4,
        )
    }
}

/// Personal Tutor — {U} sorcery. Search for a sorcery card and put it on top
/// of your library.
pub fn personal_tutor() -> CardDefinition {
    spell(
        "Personal Tutor",
        cost(&[u()]),
        CardType::Sorcery,
        Effect::Search {
            who: PlayerRef::You,
            filter: R::HasCardType(CardType::Sorcery),
            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
        },
    )
}
