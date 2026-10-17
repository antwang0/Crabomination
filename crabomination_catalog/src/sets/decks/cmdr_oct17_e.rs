//! Commander: the 2026-10-17 (`01Htv6cF`) engine-blocked single-commander
//! gaps — Keiga's Renegade Doppelganger, Bohn's Mathise, Spider-Man 2099, and
//! the Golem decks' Precursor Golem. Tests in `tests/recent_b/cmdr_oct17_b.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_any;
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::game::TurnStep;
use crate::mana::{ManaCost, cost, generic, r, u};
use std::sync::Arc;

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

/// Renegade Doppelganger — {1}{U} 0/1 Shapeshifter. Whenever another creature
/// you control enters, you may have it become a copy of that creature until
/// end of turn (the copy carries no such ability, so it stops for the turn).
pub fn renegade_doppelganger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::MayDo {
                description: "Have Renegade Doppelganger become a copy of that creature until end of turn?".into(),
                body: Box::new(Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: Selector::TriggerSource,
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                }),
            },
        }],
        ..creature("Renegade Doppelganger", cost(&[generic(1), u()]), vec![CreatureType::Shapeshifter], 0, 1)
    }
}

/// Mathise, Surge Channeler — {2}{U} legendary 1/1. Whenever you cast an
/// instant or sorcery spell with mana value 3 or greater, roll a d20: 1-9
/// each player draws; 10-19 you draw; 20 copy that spell (new targets
/// allowed).
pub fn mathise_surge_channeler() -> CardDefinition {
    let draw = |who: PlayerRef| Effect::Draw { who: Selector::Player(who), amount: Value::ONE };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::ManaValueAtLeast(3)),
            }),
            effect: Effect::RollDie {
                sides: 20,
                count: Value::ONE,
                modifier: Value::ZERO,
                reroll_at_most: 0,
                ignore_lowest: 0,
                results: vec![
                    (1, 9, draw(PlayerRef::EachPlayer)),
                    (10, 19, draw(PlayerRef::You)),
                    (20, 20, Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE }),
                ],
                on_doubles: None,
            },
        }],
        ..creature(
            "Mathise, Surge Channeler",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Elf, CreatureType::Shaman, CreatureType::Sorcerer],
            1,
            1,
        )
    }
}

/// Spider-Man 2099 — {U}{R} legendary 2/3 double strike, vigilance. From the
/// Future — can't be cast during your first three turns. At the beginning of
/// your end step, if you've played a land or cast a spell this turn from
/// anywhere other than your hand, it deals damage equal to its power to any
/// target.
pub fn spider_man_2099() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::DoubleStrike, Keyword::Vigilance],
        cast_condition: Some(Predicate::ValueAtLeast(Value::TurnsBegunBy(PlayerRef::You), Value::Const(4))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(Predicate::Any(vec![
                Predicate::ValueAtLeast(Value::SpellsCastNotFromHandThisTurn(PlayerRef::You), Value::ONE),
                Predicate::PlayedLandNotFromHandThisTurn { who: PlayerRef::You },
            ])),
            effect: Effect::DealDamage { to: target_any(), amount: Value::PowerOf(Box::new(Selector::This)) },
        }],
        ..creature("Spider-Man 2099", cost(&[u(), r()]), vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero], 2, 3)
    }
}

/// Precursor Golem — {5} 3/3 artifact Golem. ETB: two 3/3 Golem artifact
/// creature tokens. Whenever a player casts an instant or sorcery spell that
/// targets only a single Golem, that player copies it for each other Golem it
/// could target, each copy targeting a different one.
pub fn precursor_golem() -> CardDefinition {
    let golem = || R::HasCreatureType(CreatureType::Golem);
    let token = Arc::new(TokenDefinition {
        name: "Golem".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: token }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    },
                    Predicate::SpellTargetsOnlyOne { what: Selector::TriggerSource, filter: golem() },
                ])),
                effect: Effect::CasterCopiesSpellForEachOtherMatchingTarget { what: Selector::TriggerSource, filter: golem() },
            },
        ],
        ..creature("Precursor Golem", cost(&[generic(5)]), vec![CreatureType::Golem], 3, 3)
    }
}
