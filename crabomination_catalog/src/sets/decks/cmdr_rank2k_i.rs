//! Commander: single cards each completing several of EDHREC's top-2000
//! decks (the 2026-10-13 rescan), ninth file. Tests in
//! `tests/recent_b/cmdr_rank2k_i.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::{ActivatedAbility, Effect, PlayerRef, ZoneDest};
use crate::mana::{cost, g, generic, u, Color, ManaCost};
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn saga(name: &'static str, mana: ManaCost, chapters: Vec<(u32, Effect)>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: chapters,
        ..Default::default()
    }
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

/// Attuma, Atlantean Warlord — {2}{U}{U} legendary 3/4 Merfolk Warrior
/// Villain; other Merfolk you control get +1/+1; whenever one or more Merfolk
/// you control attack a player, draw a card.
pub fn attuma_atlantean_warlord() -> CardDefinition {
    let merfolk = R::HasCreatureType(CreatureType::Merfolk);
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Merfolk you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(merfolk.clone()).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                // "Attack a PLAYER" (CR 506.2): not a planeswalker.
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: merfolk.and(R::IsAttackingOpponentPlayer),
                })
                .once_per_batch_per_defender(),
            effect: draw(Value::ONE),
        }],
        ..legend(
            "Attuma, Atlantean Warlord",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Warrior, CreatureType::Villain],
            3,
            4,
        )
    }
}

/// Jin-Gitaxias — {3}{U}{U} legendary 5/5 Phyrexian Praetor; ward {2}; a
/// noncreature spell of yours with mana value 3 or more draws a card; {3}{U}
/// at sorcery speed with seven or more cards in hand: exile it and return it
/// transformed. The Great Synthesis (Saga): I — draw cards equal to your hand
/// size, no maximum hand size while you control it; II — return all
/// non-Phyrexian creatures to their owners' hands; III — cast any number of
/// spells from your hand for free, then exile it and return it front face up.
pub fn jin_gitaxias() -> CardDefinition {
    let synthesis = CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You have no maximum hand size for as long as you control this Saga.",
            effect: StaticEffect::NoMaximumHandSize,
        }],
        ..saga(
            "The Great Synthesis",
            ManaCost::default(),
            vec![
                (1, draw(Value::HandSizeOf(PlayerRef::You))),
                (
                    2,
                    Effect::Move {
                        what: Selector::EachPermanent(
                            R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Phyrexian)))),
                        ),
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    },
                ),
                (
                    3,
                    Effect::Seq(vec![
                        // "Any number": one free cast offered per card in hand.
                        Effect::Repeat {
                            count: Value::HandSizeOf(PlayerRef::You),
                            body: Box::new(Effect::MayCastFromHandFreeMatching {
                                filter: R::Any,
                                max_mv: Value::Const(1_000),
                                else_: Box::new(Effect::Noop),
                            }),
                        },
                        Effect::ExileSelfReturnFrontFace,
                    ]),
                ),
            ],
        )
    };
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Noncreature.and(R::ManaValueAtLeast(3)),
            }),
            effect: draw(Value::ONE),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            sorcery_speed: true,
            condition: Some(Predicate::ValueAtLeast(Value::HandSizeOf(PlayerRef::You), Value::Const(7))),
            effect: Effect::ExileSelfReturnTransformed,
            ..Default::default()
        }],
        back_face: Some(Box::new(synthesis)),
        ..legend(
            "Jin-Gitaxias",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Phyrexian, CreatureType::Praetor],
            5,
            5,
        )
    }
}

/// The Hunger Tide Rises — {2}{G} Saga. I, II, III — a 1/1 black and green
/// Insect. IV — sacrifice any number of creatures, then search your library
/// and/or graveyard for a creature card with mana value up to the number
/// sacrificed and put it onto the battlefield.
/// Like Invasion of Ikoria, the library is always searched (and shuffled)
/// alongside the graveyard.
pub fn the_hunger_tide_rises() -> CardDefinition {
    let insect = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition {
            name: "Insect".into(),
            power: 1,
            toughness: 1,
            colors: vec![Color::Black, Color::Green],
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
            ..Default::default()
        }),
    };
    saga(
        "The Hunger Tide Rises",
        cost(&[generic(2), g()]),
        vec![
            (1, insect()),
            (2, insect()),
            (3, insect()),
            (
                4,
                Effect::Seq(vec![
                    Effect::SacrificeAnyNumber {
                        who: PlayerRef::You,
                        filter: R::Creature,
                        per_each: Box::new(Effect::Noop),
                    },
                    Effect::WithX {
                        x: Value::SacrificedCount,
                        body: Box::new(Effect::SearchZones {
                            who: PlayerRef::You,
                            zones: vec![Zone::Library, Zone::Graveyard],
                            filter: R::Creature.and(R::ManaValueAtMostXFromCost),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        }),
                    },
                ]),
            ),
        ],
    )
}
