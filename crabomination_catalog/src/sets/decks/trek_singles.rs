//! Star Trek (TRK/TRC, 2026-11-13) — the commanders and staples that close
//! EDHREC's Star Trek single-commander average decks: each of these was the
//! one card (or one of two) a deck lacked. Tests in
//! `tests/recent_b/trek_singles.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, StationBand, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, investigate, on_attack, station, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};

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

fn spacecraft(name: &'static str, mana: ManaCost, bands: Vec<StationBand>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        activated_abilities: vec![station()],
        station: bands,
        ..Default::default()
    }
}

fn spacecraft_filter() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Spacecraft)
}

fn officer_tokens(n: i32) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: Arc::new(TokenDefinition {
            name: "Officer".into(),
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Creature],
            colors: vec![Color::Red],
            subtypes: Subtypes { creature_types: vec![CreatureType::Officer], ..Default::default() },
            ..Default::default()
        }),
    }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn counter_on(what: Selector, kind: CounterType, n: i32) -> Effect {
    Effect::AddCounter { what, kind, amount: Value::Const(n) }
}

/// Captain Michael Burnham — {2}{R} legendary 3/3 Human Officer, trample.
/// Landfall: 1 damage to each opponent; the second resolution each turn also
/// puts a +1/+1 counter on her.
pub fn captain_michael_burnham() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                Effect::NthResolutionThisTurn {
                    branches: vec![Effect::Noop, counter_on(Selector::This, CounterType::PlusOnePlusOne, 1)],
                },
            ]),
        }],
        ..legend(
            "Captain Michael Burnham",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Officer],
            3,
            3,
        )
    }
}

/// Mr. Spock, Science Officer — {2}{U}{U} legendary 2/4 Vulcan Human
/// Scientist. If you would draw a card, draw two instead.
pub fn mr_spock_science_officer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If you would draw a card, draw two cards instead.",
            effect: StaticEffect::ControllerDrawsDoubledIf { condition: Predicate::True },
        }],
        ..legend(
            "Mr. Spock, Science Officer",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Vulcan, CreatureType::Human, CreatureType::Scientist],
            2,
            4,
        )
    }
}

/// Constable Odo — {3}{W/U}{W/U} legendary 4/3 Shapeshifter Detective.
/// Entering: tap target opponent's creature with two stun counters, or
/// investigate twice.
pub fn constable_odo() -> CardDefinition {
    let wu = || hybrid(Color::White, Color::Blue);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
                counter_on(Selector::Target(0), CounterType::Stun, 2),
            ]),
            investigate(2),
        ]))],
        ..legend(
            "Constable Odo",
            cost(&[generic(3), wu(), wu()]),
            vec![CreatureType::Shapeshifter, CreatureType::Detective],
            4,
            3,
        )
    }
}

/// Dathon, His Heart Open — {G}{U}{R} legendary 3/3 Tamarian Officer.
/// Casting an enchantment draws a card (a Saga also deals 2 to each
/// opponent). {2}: another counter of a kind already on target enchantment
/// you control; sorcery speed.
pub fn dathon_his_heart_open() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellMatches(R::Enchantment)),
                effect: draw(1),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::CastSpellMatches(R::HasEnchantmentSubtype(crate::card::EnchantmentSubtype::Saga)),
                ),
                effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sorcery_speed: true,
            effect: Effect::AddCounterOfEachKindOn {
                what: target_filtered(R::Enchantment.and(R::ControlledByYou)),
            },
            ..Default::default()
        }],
        ..legend(
            "Dathon, His Heart Open",
            cost(&[g(), u(), r()]),
            vec![CreatureType::Tamarian, CreatureType::Officer],
            3,
            3,
        )
    }
}

/// Deanna Troi, Psychic Counsel — {1}{W} legendary 2/2 Betazoid Human
/// Advisor. Your other creatures enter with an additional +1/+1 counter.
pub fn deanna_troi_psychic_counsel() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control enter with an additional +1/+1 counter on them.",
            effect: StaticEffect::MatchingEntersWithExtraCounters {
                filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                kind: CounterType::PlusOnePlusOne,
                amount: 1,
            },
        }],
        ..legend(
            "Deanna Troi, Psychic Counsel",
            cost(&[generic(1), w()]),
            vec![CreatureType::Betazoid, CreatureType::Human, CreatureType::Advisor],
            2,
            2,
        )
    }
}

/// Dr. McCoy, Surly Surgeon — {2}{W}{W} legendary 2/2 Human Doctor. Entering
/// puts a +1/+1 counter on another target creature you control. Another
/// countered creature of yours that would be destroyed sheds its +1/+1
/// counters and its damage instead.
pub fn dr_mccoy_surly_surgeon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(counter_on(
            target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            CounterType::PlusOnePlusOne,
            1,
        ))],
        static_abilities: vec![StaticAbility {
            description: "If another creature you control with a +1/+1 counter on it would be destroyed, instead remove all +1/+1 counters from it and all damage already dealt to it is healed.",
            effect: StaticEffect::OtherCounteredCreaturesShedCountersInsteadOfDestroyed,
        }],
        ..legend(
            "Dr. McCoy, Surly Surgeon",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Doctor],
            2,
            2,
        )
    }
}

/// D'Vana Tendi, Optimistic Ensign — {1}{G/U}{G/U} legendary 2/4 Orion
/// Scientist. {T}: untap another target permanent. {3}: surveil 1.
pub fn dvana_tendi_optimistic_ensign() -> CardDefinition {
    let gu = || hybrid(Color::Green, Color::Blue);
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Untap { what: target_filtered(R::Permanent.and(R::OtherThanSource)), up_to: None },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..legend(
            "D'Vana Tendi, Optimistic Ensign",
            cost(&[generic(1), gu(), gu()]),
            vec![CreatureType::Orion, CreatureType::Scientist],
            2,
            4,
        )
    }
}

/// Grand Nagus Zek — {2}{B} legendary 1/1 Ferengi Noble. +1/+1 per Treasure
/// you control. The first time each turn another creature dies, a Treasure.
pub fn grand_nagus_zek() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Grand Nagus Zek gets +1/+1 for each Treasure you control.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::HasArtifactSubtype(ArtifactSubtype::Treasure).and(R::ControlledByYou),
                ))),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource })
                .once_per_turn(),
            effect: crate::effect::shortcut::mint_treasures(1),
        }],
        ..legend("Grand Nagus Zek", cost(&[generic(2), b()]), vec![CreatureType::Ferengi, CreatureType::Noble], 1, 1)
    }
}

/// Hemmer, Fixing What Is Broken — {W}{U} legendary 2/3 Aenar Artificer.
/// During your turn your Spacecraft have hexproof. Becoming tapped loots.
pub fn hemmer_fixing_what_is_broken() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, Spacecraft you control have hexproof.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(spacecraft_filter().and(R::ControlledByYou)),
                    keyword: Keyword::Hexproof,
                }),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Seq(vec![draw(1), Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }]),
        }],
        ..legend(
            "Hemmer, Fixing What Is Broken",
            cost(&[w(), u()]),
            vec![CreatureType::Aenar, CreatureType::Artificer],
            2,
            3,
        )
    }
}

/// Nyota Uhura, Communications — {2}{U} legendary 2/4 Human Officer. As she
/// enters, choose a number from 0 to 10; an opponent casting a spell with that
/// mana value draws you a card.
pub fn nyota_uhura_communications() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::ChooseNumberForSource { max: 10, pays_life: false, min: 0, own_creature_pt: false }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(Predicate::ValueEquals(
                Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                Value::ChosenNumberOfSource,
            )),
            effect: draw(1),
        }],
        ..legend(
            "Nyota Uhura, Communications",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Officer],
            2,
            4,
        )
    }
}

/// The Doctor, EMH — {3}{G/W}{G/W} legendary 3/3 Illusion Doctor artifact
/// creature. Entering: a +1/+1 counter on target creature, then one on each
/// creature you control with a counter.
pub fn the_doctor_emh() -> CardDefinition {
    let gw = || hybrid(Color::Green, Color::White);
    let mut c = CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            counter_on(target_filtered(R::Creature), CounterType::PlusOnePlusOne, 1),
            counter_on(
                Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::WithAnyCounter)),
                CounterType::PlusOnePlusOne,
                1,
            ),
        ]))],
        ..legend(
            "The Doctor, EMH",
            cost(&[generic(3), gw(), gw()]),
            vec![CreatureType::Illusion, CreatureType::Doctor],
            3,
            3,
        )
    };
    c.card_types.insert(0, CardType::Artifact);
    c
}

/// U.S.S. Discovery, at Black Alert — {3}{G}{G} legendary Spacecraft.
/// Entering or attacking: discover 5. Station; 6+: 5/5 flying.
pub fn u_s_s_discovery_at_black_alert() -> CardDefinition {
    let disc = || Effect::Discover { n: Value::Const(5), filter: None };
    CardDefinition {
        triggered_abilities: vec![etb(disc()), on_attack(disc())],
        ..spacecraft(
            "U.S.S. Discovery, at Black Alert",
            cost(&[generic(3), g(), g()]),
            vec![StationBand { min: 6, keywords: vec![Keyword::Flying], pt: Some((5, 5)), ..Default::default() }],
        )
    }
}

/// U.S.S. Reliant, From Hell's Heart — {2}{B}{B} legendary Spacecraft.
/// Entering or connecting returns target creature card with mana value 4 or
/// less from your graveyard with a finality counter. Station; 10+: 4/3 flying.
pub fn u_s_s_reliant_from_hells_heart() -> CardDefinition {
    let reanimate = || {
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMost(4))),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            counter_on(Selector::LastMoved, CounterType::Finality, 1),
        ])
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(reanimate()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: reanimate(),
            },
        ],
        ..spacecraft(
            "U.S.S. Reliant, From Hell's Heart",
            cost(&[generic(2), b(), b()]),
            vec![StationBand { min: 10, keywords: vec![Keyword::Flying], pt: Some((4, 3)), ..Default::default() }],
        )
    }
}

/// Null Group Biological Assets — {2}{R} 3/1 Mutant Mercenary. First strike
/// during your turn; attacking, you may discard a card to draw one.
pub fn null_group_biological_assets() -> CardDefinition {
    CardDefinition {
        name: "Null Group Biological Assets",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mutant, CreatureType::Mercenary], ..Default::default() },
        power: 3,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has first strike.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::FirstStrike,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        triggered_abilities: vec![on_attack(Effect::MayDiscard {
            description: "Discard a card to draw a card?".into(),
            count: Value::ONE,
            then: Box::new(draw(1)),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Starship Enterprise — {5} legendary Spacecraft. Landfall: four charge
/// counters. Station; 16+: 4/6 flying, and its combat damage to a player
/// draws that many.
pub fn starship_enterprise() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: counter_on(Selector::This, CounterType::Charge, 4),
        }],
        ..spacecraft(
            "Starship Enterprise",
            cost(&[generic(5)]),
            vec![StationBand {
                min: 16,
                keywords: vec![Keyword::Flying],
                pt: Some((4, 6)),
                triggers: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                    effect: Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
                }],
                ..Default::default()
            }],
        )
    }
}

/// Deep Space 9 — {9} legendary Spacecraft. Entering and at your upkeep,
/// exile the top four cards and put every permanent card among them onto the
/// battlefield. Station; 9+: creatures you control get +3/+3.
pub fn deep_space_9() -> CardDefinition {
    let dig = || Effect::LookTopPutMatchingOntoBattlefield {
        count: Value::Const(4),
        filter: R::Permanent,
        then: None,
        max: None,
        tapped: false,
        exile_rest: true,
        rest_to_graveyard: false,
        mandatory: true,
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(dig()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: dig(),
            },
        ],
        ..spacecraft(
            "Deep Space 9",
            cost(&[generic(9)]),
            vec![StationBand {
                min: 9,
                statics: vec![StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: 3,
                    toughness: 3,
                }],
                ..Default::default()
            }],
        )
    }
}

/// U.S.S. Enterprise-A, Free Spirit — {3}{W}{W} legendary Spacecraft, ward
/// {2}. Entering makes three 1/1 red Officers. Station; 6+: 5/5 flying,
/// lifelink.
pub fn u_s_s_enterprise_a_free_spirit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![etb(officer_tokens(3))],
        ..spacecraft(
            "U.S.S. Enterprise-A, Free Spirit",
            cost(&[generic(3), w(), w()]),
            vec![StationBand {
                min: 6,
                keywords: vec![Keyword::Flying, Keyword::Lifelink],
                pt: Some((5, 5)),
                ..Default::default()
            }],
        )
    }
}

/// Captain Jean-Luc Picard — {2}{W} legendary 2/3 Human Officer. Your other
/// creatures get +1/+1. Casting a creature or Spacecraft spell makes a 1/1
/// Officer. Tap seven untapped Officers: a Spacecraft from your library
/// enters with seven charge counters.
pub fn captain_jean_luc_picard() -> CardDefinition {
    let officer = || R::Creature.and(R::HasCreatureType(CreatureType::Officer));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Creature.or(spacecraft_filter()))),
            effect: officer_tokens(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((officer(), 7)),
            effect: Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: spacecraft_filter(),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                counter_on(Selector::LastMoved, CounterType::Charge, 7),
            ]),
            ..Default::default()
        }],
        ..legend(
            "Captain Jean-Luc Picard",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Officer],
            2,
            3,
        )
    }
}

/// Worf, Intelligence Contractor — {2}{R} legendary 3/2 Klingon Warrior Spy.
/// Entering, for each opponent, steal up to one of their creatures with mana
/// value 3 or less until end of turn: untapped, a Warrior too, with haste.
pub fn worf_intelligence_contractor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 15,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent).and(R::ManaValueAtMost(3)),
                effect: Box::new(Effect::Seq(vec![
                    Effect::GainControl { what: Selector::Target(0), to: None, duration: Duration::EndOfTurn },
                    Effect::Untap { what: Selector::Target(0), up_to: None },
                    Effect::AddCreatureTypes {
                        what: Selector::Target(0),
                        creature_types: vec![CreatureType::Warrior],
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ])),
            }),
        })],
        ..legend(
            "Worf, Intelligence Contractor",
            cost(&[generic(2), r()]),
            vec![CreatureType::Klingon, CreatureType::Warrior, CreatureType::Spy],
            3,
            2,
        )
    }
}
