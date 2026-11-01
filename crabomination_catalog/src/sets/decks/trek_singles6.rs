//! Star Trek (TRK/TRC) — the sixth batch of EDHREC Star Trek closers: the two
//! missing cards of Uhura, Jean-Luc, Wesley Crusher and B'Elanna Torres'
//! single-commander average decks (scan of 2026-11-01). Tests in
//! `tests/recent_b/trek_singles6.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{Color, ManaCost, cost, generic, r, u, w};

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn faerie() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Faerie".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    })
}

/// Data, Professor of Humanities — {3}{U} legendary 3/4 Android Scientist
/// artifact creature. Seek out — your second draw each turn makes two 1/1
/// blue flying Faeries.
pub fn data_professor_of_humanities() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: faerie() },
        }],
        ..legend(
            "Data, Professor of Humanities",
            cost(&[generic(3), u()]),
            vec![CreatureType::Android, CreatureType::Scientist],
            3,
            4,
        )
    }
}

/// Uhura, Xenolinguist — {4}{U} legendary 3/4 Human Officer. A non-Human
/// creature entering (anyone's) lets you pay {1} to draw.
pub fn uhura_xenolinguist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Human)))),
                },
            ),
            effect: Effect::MayPay {
                description: "Pay {1} to draw a card?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: None,
            },
        }],
        ..legend("Uhura, Xenolinguist", cost(&[generic(4), u()]), vec![CreatureType::Human, CreatureType::Officer], 3, 4)
    }
}

/// Jean-Luc, Mon Capitaine — {2}{U}{R}{W} legendary 5/3 flying Human
/// Officer. Federation — your Faeries enter with a +1/+1 counter per creature
/// type among non-Borg creatures you already control.
pub fn jean_luc_mon_capitaine() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Faeries you control enter with an additional +1/+1 counter for each creature type among \
                          non-Borg creatures you already control.",
            effect: StaticEffect::MatchingEntersWithCountersPer {
                filter: R::HasCreatureType(CreatureType::Faerie),
                kind: CounterType::PlusOnePlusOne,
                value: Value::DistinctCreatureTypesAmongYourOtherNonBorgCreatures,
            },
        }],
        ..legend(
            "Jean-Luc, Mon Capitaine",
            cost(&[generic(2), u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Officer],
            5,
            3,
        )
    }
}

/// Wesley Crusher, Starfleet Cadet — {2}{U}{U} legendary 2/2 Human Officer,
/// flash. Entering, choose two: soft-counter a noncreature spell ({2}); bounce
/// a creature with mana value 2 or less; a creature can't be blocked this
/// turn; a player draws two then discards two.
pub fn wesley_crusher_starfleet_cadet() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::ChooseN {
            up_to: None,
            picks: vec![2, 3],
            modes: vec![
                Effect::CounterUnlessPaid {
                    what: target_filtered(R::IsSpellOnStack.and(R::Noncreature)),
                    mana_cost: cost(&[generic(2)]),
                    exile: false,
                    extra_generic: None,
                    if_paid: None,
                },
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::ManaValueAtMost(2))),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
                Effect::Seq(vec![
                    Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(2) },
                    Effect::Discard { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(2), random: false },
                ]),
            ],
        })],
        ..legend(
            "Wesley Crusher, Starfleet Cadet",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Human, CreatureType::Officer],
            2,
            2,
        )
    }
}

/// Steal Artifact — {2}{U}{U} Aura, enchant artifact. You control enchanted
/// artifact.
pub fn steal_artifact() -> CardDefinition {
    CardDefinition {
        name: "Steal Artifact",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Artifact) },
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: Selector::attached_to(Selector::This),
        })],
        ..Default::default()
    }
}

/// The TRK campus cycle: enters tapped unless you control two or more other
/// lands; entering untapped, choose one — `boost` or surveil 1.
fn campus(name: &'static str, color: Color, boost: Effect) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless you control two or more other lands.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Land.and(R::ControlledByYou).and(R::OtherThanSource)),
                    n: Value::Const(2),
                },
            },
        }],
        // CR 603.2 — "when this land enters untapped": the entry's state.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::Untapped }),
            effect: Effect::ChooseMode(vec![boost, Effect::Surveil { who: PlayerRef::You, amount: Value::ONE }]),
        }],
        activated_abilities: vec![crate::sets::tap_add(color)],
        ..Default::default()
    }
}

/// Ni'Var Science Academy — land, {T}: {U}; entering untapped: target
/// creature gains flying until end of turn, or surveil 1.
pub fn ni_var_science_academy() -> CardDefinition {
    campus(
        "Ni'Var Science Academy",
        Color::Blue,
        Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
    )
}

/// Q Continuum — land, {T}: {G}; entering untapped: target creature gets
/// +1/+1 and vigilance until end of turn, or surveil 1.
pub fn q_continuum() -> CardDefinition {
    campus(
        "Q Continuum",
        Color::Green,
        Effect::Seq(vec![
            Effect::PumpPT { what: target_filtered(R::Creature), power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
        ]),
    )
}

/// Spock, Voice of Reason — {2}{U} legendary 3/2 Vulcan Human Scientist.
/// {2},{T}: draw a card per nontoken permanent you control that entered this
/// turn.
pub fn spock_voice_of_reason() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::ControlledByYou.and(R::EnteredThisTurn).and(R::Not(Box::new(R::IsToken))),
                ))),
            },
            ..Default::default()
        }],
        ..legend(
            "Spock, Voice of Reason",
            cost(&[generic(2), u()]),
            vec![CreatureType::Vulcan, CreatureType::Human, CreatureType::Scientist],
            3,
            2,
        )
    }
}

/// Q, Your Illustrious Host — {W}{U}{B}{R}{G} legendary 5/5 Q. Tap three
/// untapped Faeries you control: exile from the top until a nonland card; you
/// may cast it without paying its mana cost.
pub fn q_your_illustrious_host() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((R::HasCreatureType(CreatureType::Faerie).and(R::ControlledByYou), 3)),
            effect: Effect::Seq(vec![
                Effect::ExileTopUntilNonland { who: PlayerRef::You },
                Effect::CastWithoutPayingImmediate {
                    what: Selector::ExiledThisResolution { filter: R::Nonland },
                    source_zone: crate::card::Zone::Exile,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Q, Your Illustrious Host",
            cost(&[w(), u(), crate::mana::b(), r(), crate::mana::g()]),
            vec![CreatureType::Q],
            5,
            5,
        )
    }
}
