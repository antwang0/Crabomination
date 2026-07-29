//! Dragon's Maze (DGM) gap cards, wave 5 — the last cards that were each
//! blocked on a single primitive. Tests in `classic_sets/dgm`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, SplitCard, SplitHalf, StaticAbility, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Predicate, Selector, StaticEffect};
use crate::mana::{cost, g, generic, r, u, w};

/// Melek, Izzet Paragon — {4}{U}{R} 2/4 legendary Weird Wizard. Plays with the
/// top of your library revealed, casts instants and sorceries from there, and
/// copies each one cast from the library.
pub fn melek_izzet_paragon() -> CardDefinition {
    let instant_or_sorcery = R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    CardDefinition {
        name: "Melek, Izzet Paragon",
        cost: cost(&[generic(4), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Weird, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        static_abilities: vec![
            StaticAbility {
                description: "Play with the top card of your library revealed.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "You may cast instant and sorcery spells from the top of your \
                              library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: instant_or_sorcery.clone() },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::All(vec![
                    Predicate::CastSpellMatches(instant_or_sorcery),
                    Predicate::CastSpellFromLibrary,
                ])),
            effect: Effect::CopySpellMayChooseTargets {
                what: Selector::TriggerSource,
                count: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Plasm Capture — {G}{G}{U}{U} Instant. Counter target spell; at your next
/// main phase add mana of any colors equal to that spell's mana value.
pub fn plasm_capture() -> CardDefinition {
    CardDefinition {
        name: "Plasm Capture",
        cost: cost(&[g(), g(), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::AddManaAtNextMainPhase {
                amount: Value::CounteredSpellManaValue,
                any_color: true,
            },
        ]),
        ..Default::default()
    }
}

/// Goblin Test Pilot — {1}{U}{R} 0/2 Goblin Pilot Wizard with flying.
/// {T}: 2 damage to any target chosen at random.
pub fn goblin_test_pilot() -> CardDefinition {
    CardDefinition {
        name: "Goblin Test Pilot",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Goblin,
                CreatureType::Pilot,
                CreatureType::Wizard,
            ],
            ..Default::default()
        },
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: Selector::RandomAmong(
                    R::Creature.or(R::Player).or(R::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Catch // Release — {1}{U}{R} // {4}{R}{W} Sorcery split with Fuse. Catch
/// steals a creature for the turn; Release is a five-type edict on each player.
pub fn catch_release() -> CardDefinition {
    CardDefinition {
        name: "Catch // Release",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(4), r(), w()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(
                    [R::Artifact, R::Creature, R::Enchantment, R::Land, R::Planeswalker]
                        .into_iter()
                        .map(|filter| Effect::Sacrifice {
                            who: Selector::Player(crate::effect::PlayerRef::EachPlayer),
                            count: Value::ONE,
                            filter,
                        })
                        .collect(),
                ),
            },
            fuse: true,
            aftermath: false,
        })),
        ..Default::default()
    }
}
