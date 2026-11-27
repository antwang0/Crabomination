//! Commander: closers for Chekov and McCoy's, Zefram Cochrane's and H.M.S.
//! Bounty's EDHREC average decks (scan of 2026-11-27) — the Whale ship and its
//! pod. Tests in `tests/recent_b/cmdr_01wr2y_a.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, StationBand, Subtypes, Supertype,
    TriggeredAbility, WardCost,
};
use crate::effect::shortcut::{etb, station, target_filtered};
use crate::effect::{Effect, PlayerRef, StaticEffect, Value, ZoneDest};
use crate::mana::{cost, g, generic, u};

fn creature(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn spacecraft(name: &'static str, mana: crate::mana::ManaCost, p: i32, t: i32, band: StationBand) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        power: p,
        toughness: t,
        activated_abilities: vec![station()],
        station: vec![band],
        ..Default::default()
    }
}

fn whale() -> R {
    R::HasCreatureType(CreatureType::Whale)
}

/// H.M.S. Bounty, Unlikely Timeship — {4}{U} legendary Spacecraft. ETB: you
/// may search for a Whale card to hand. Station; 8+: a 3/4 flier with
/// "whenever it deals combat damage to a player, draw a card; then if you
/// control a Whale and haven't begun an extra turn this game, take an extra
/// turn after this one" (CR 500.7).
pub fn h_m_s_bounty_unlikely_timeship() -> CardDefinition {
    let time_warp = Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::ONE },
        Effect::If {
            cond: Predicate::All(vec![
                Predicate::ValueAtLeast(
                    Value::count(Selector::EachPermanent(whale().and(R::ControlledByYou))),
                    Value::ONE,
                ),
                Predicate::Not(Box::new(Predicate::YouHaveBegunAnExtraTurnThisGame)),
            ]),
            then: Box::new(Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE }),
            else_: Box::new(Effect::Noop),
        },
    ]);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: whale(),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..spacecraft(
            "H.M.S. Bounty, Unlikely Timeship",
            cost(&[generic(4), u()]),
            3,
            4,
            StationBand {
                min: 8,
                keywords: vec![Keyword::Flying],
                pt: Some((3, 4)),
                triggers: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                    effect: time_warp,
                }],
                ..Default::default()
            },
        )
    }
}

/// U.S.S. Defiant, Mirror Migrant — {2} legendary Spacecraft, flash. ETB: it
/// and up to one target creature you control phase out (CR 702.26). Station;
/// 8+: a 5/5 flier.
pub fn u_s_s_defiant_mirror_migrant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::PhaseOut {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    until_source_leaves: false,
                }),
            },
            Effect::PhaseOut { what: Selector::This, until_source_leaves: false },
        ]))],
        ..spacecraft(
            "U.S.S. Defiant, Mirror Migrant",
            cost(&[generic(2)]),
            5,
            5,
            StationBand { min: 8, keywords: vec![Keyword::Flying], pt: Some((5, 5)), ..Default::default() },
        )
    }
}

/// Zefram Cochrane, Warp Pioneer — {2}{U} legendary 2/3 Human Pilot. You may
/// look at the top card of your library any time; once during each of your
/// turns you may cast an artifact spell from it. Spacecraft you control enter
/// with two additional charge counters.
pub fn zefram_cochrane_warp_pioneer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "Once during each of your turns, you may cast an artifact spell from the top of your library.",
                effect: StaticEffect::WhileYourTurn {
                    inner: Box::new(StaticEffect::PlayFromLibraryTopOncePerTurn { filter: R::Artifact.and(R::Nonland) }),
                },
            },
            StaticAbility {
                description: "Spacecraft you control enter with two additional charge counters on them.",
                effect: StaticEffect::MatchingEntersWithExtraCounters {
                    filter: R::HasArtifactSubtype(ArtifactSubtype::Spacecraft).and(R::ControlledByYou),
                    kind: CounterType::Charge,
                    amount: 2,
                },
            },
        ],
        ..creature(
            "Zefram Cochrane, Warp Pioneer",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Pilot],
            2,
            3,
        )
    }
}

/// Chekov and McCoy — {3}{G}{U} legendary 3/3 Human Pilot Doctor. Whenever you
/// put one or more +1/+1 counters on it, you may draw that many cards; do this
/// only once each turn.
pub fn chekov_and_mccoy() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![crate::effect::shortcut::once_each_turn_on_take(TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::YouPutCounters)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSource })
                .once_per_turn(),
            effect: Effect::MayDo {
                description: "Draw that many cards?".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount }),
            },
        })],
        ..creature(
            "Chekov and McCoy",
            cost(&[generic(3), g(), u()]),
            vec![CreatureType::Human, CreatureType::Pilot, CreatureType::Doctor],
            3,
            3,
        )
    }
}

/// Test of Courage — {G} instant. Choose one — exile target card from a
/// graveyard and gain 2 life; or destroy target artifact or enchantment with
/// mana value 3 or less.
pub fn test_of_courage() -> CardDefinition {
    CardDefinition {
        name: "Test of Courage",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::Exile { what: target_filtered(R::InGraveyard) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ]),
            Effect::Destroy {
                what: target_filtered(R::Artifact.or(R::Enchantment).and(R::ManaValueAtMost(3))),
            },
        ]),
        ..Default::default()
    }
}

/// Cetacean Ops — {3}{U} 3/3 Whale Officer, ward {1}. Whenever you cast a
/// noncreature spell, a +1/+1 counter on it.
pub fn cetacean_ops() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(1)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(crate::effect::shortcut::cast_is_noncreature()),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Cetacean Ops", cost(&[generic(3), u()]), vec![CreatureType::Whale, CreatureType::Officer], 3, 3)
    }
}

/// Flying Dolphin-Fish — {1}{U} 1/3 Whale Fish, flying.
pub fn flying_dolphin_fish() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature("Flying Dolphin-Fish", cost(&[generic(1), u()]), vec![CreatureType::Whale, CreatureType::Fish], 1, 3)
    }
}

/// Humpback Whales — {4}{U}{U} 5/6 Whale, vigilance, ward {2}. Basic
/// landcycling {2}.
pub fn humpback_whales() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Vigilance,
            Keyword::Ward(WardCost::Mana(cost(&[generic(2)]))),
            Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand))),
        ],
        ..creature("Humpback Whales", cost(&[generic(4), u(), u()]), vec![CreatureType::Whale], 5, 6)
    }
}

/// Long-Finned Skywhale — {2}{U}{U} 4/3 Whale, flying; it can block only
/// creatures with flying.
pub fn long_finned_skywhale() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
        ..creature("Long-Finned Skywhale", cost(&[generic(2), u(), u()]), vec![CreatureType::Whale], 4, 3)
    }
}
