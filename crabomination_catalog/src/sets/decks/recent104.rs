//! Long-deferred primitive cards (TODO.md "need new primitives" list):
//! Pulmonic Sliver, Twilight Prophet, Goblin Welder, Gilt-Leaf Archdruid.
//! Tests in `tests/recent104.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement, Selector, StaticAbility, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Predicate, StaticEffect};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Pulmonic Sliver — {3}{W}{W} 3/3 Sliver. All Sliver creatures have flying.
/// All Slivers may go to their owner's library top instead of the graveyard.
pub fn pulmonic_sliver() -> CardDefinition {
    let slivers = SelectionRequirement::HasCreatureType(CreatureType::Sliver);
    CardDefinition {
        name: "Pulmonic Sliver",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sliver],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        static_abilities: vec![
            StaticAbility {
                description: "All Sliver creatures have flying.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(slivers.clone()),
                    keyword: Keyword::Flying,
                },
            },
            StaticAbility {
                description: "All Slivers may be put on top of their owner's library instead of \
                              a graveyard.",
                effect: StaticEffect::DiesToLibraryTopInstead { filter: slivers },
            },
        ],
        ..Default::default()
    }
}

/// Twilight Prophet — {2}{B}{B} 2/4 Vampire Cleric. Flying, Ascend; at your
/// upkeep with the city's blessing, reveal top to hand and drain its MV.
pub fn twilight_prophet() -> CardDefinition {
    CardDefinition {
        name: "Twilight Prophet",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Ascend {
                    who: PlayerRef::You,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::Ascend {
                    who: PlayerRef::You,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                )
                .with_filter(Predicate::HasCityBlessing {
                    who: PlayerRef::You,
                }),
                effect: Effect::RevealTopToHandLoseMv {
                    who: PlayerRef::EachOpponent,
                    you_gain: true,
                },
            },
        ],
        ..Default::default()
    }
}

/// Goblin Welder — {R} 1/1 Goblin Artificer. {T}: target artifact's
/// controller swaps it with an artifact card in their graveyard (auto-pick:
/// highest mana value).
pub fn goblin_welder() -> CardDefinition {
    CardDefinition {
        name: "Goblin Welder",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::WeldArtifacts {
                what: target_filtered(SelectionRequirement::Artifact),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Paradox Haze — {2}{U} Enchantment. At the first upkeep of your turn you
/// get an additional upkeep step (CR 500.9; enchant-player is modeled as a
/// controller-scoped enchantment).
pub fn paradox_haze() -> CardDefinition {
    CardDefinition {
        name: "Paradox Haze",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            )
            .with_filter(Predicate::IsFirstUpkeepThisTurn),
            effect: Effect::AdditionalUpkeepStep { count: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Gilt-Leaf Archdruid — {3}{G}{G} 3/3 Elf Druid. Cast a Druid spell: draw.
/// Tap seven untapped Druids you control: steal target player's lands.
pub fn gilt_leaf_archdruid() -> CardDefinition {
    let druids = SelectionRequirement::Creature
        .and(SelectionRequirement::HasCreatureType(CreatureType::Druid));
    CardDefinition {
        name: "Gilt-Leaf Archdruid",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(SelectionRequirement::HasCreatureType(
                    CreatureType::Druid,
                )),
            ),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((druids, 7)),
            effect: Effect::GainControl {
                what: Selector::ControlledBy {
                    who: PlayerRef::Target(0),
                    filter: SelectionRequirement::Land,
                },
                to: None,
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
