//! Star Trek Commander (TRC, 2026-11-13) — the Federation Fleet precon's new
//! cards (Jean-Luc Picard, Luminary). "Seek out" is the ability word for
//! "whenever you draw your second card each turn" (CR 207.2c). Tests in
//! `tests/recent_b/trek_federation.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EntersAsCopy, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StationBand, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, station, target_filtered};
use crate::effect::{AttackingTokenCleanup, Duration, Effect, LibraryPosition, PlayerRef, StaticEffect, ZoneDest};
use crate::game::types::TurnStep;
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

/// A legendary Spacecraft: station plus its bands (CR 721).
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

/// Seek out — "whenever you draw your second card each turn" (CR 121.2's
/// per-turn draw count, the `NthCardDrawnThisTurn(2)` event).
fn seek_out(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl), effect }
}

fn officer() -> R {
    R::HasCreatureType(CreatureType::Officer)
}

fn spacecraft_filter() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Spacecraft)
}

fn officer_token() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Officer".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Officer], ..Default::default() },
        ..Default::default()
    })
}

fn counters(what: Selector, kind: CounterType, n: i32) -> Effect {
    Effect::AddCounter { what, kind, amount: Value::Const(n) }
}

/// "You and target opponent each draw two cards."
fn you_and_target_opponent_draw_two() -> Effect {
    Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        Effect::Draw { who: target_filtered(R::OpponentPlayer), amount: Value::Const(2) },
    ])
}

/// Jean-Luc Picard, Luminary — {1}{U}{R}{W} legendary 2/4 Human Officer.
/// Another historic permanent of yours with mana value 3+ entering draws a
/// card. Seek out: a +1/+1 counter on up to one other target creature and two
/// charge counters on up to one target Spacecraft.
pub fn jean_luc_picard_luminary() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::historic().and(R::ManaValueAtLeast(3)),
                    },
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
            seek_out(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    counters(
                        Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::OtherThanSource) },
                        CounterType::PlusOnePlusOne,
                        1,
                    ),
                    counters(
                        Selector::TargetFiltered { slot: 1, filter: spacecraft_filter() },
                        CounterType::Charge,
                        2,
                    ),
                ])),
            }),
        ],
        ..legend(
            "Jean-Luc Picard, Luminary",
            cost(&[generic(1), u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Officer],
            2,
            4,
        )
    }
}

/// Captain Janeway, Endgame — {1}{U}{R}{W} legendary 3/3 Human Officer. One
/// or more Officers of yours connecting draws a card and makes a 1/1 red
/// Officer. {2}, tap an untapped creature you control: unblockable this turn.
pub fn captain_janeway_endgame() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_batch: true,
                ..EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: officer() })
            },
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: officer_token() },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_permanents_cost: Some((R::Creature, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Captain Janeway, Endgame",
            cost(&[generic(1), u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Officer],
            3,
            3,
        )
    }
}

/// Ambassador K'Ehleyr — {2}{W} legendary 2/2 Klingon Human Advisor,
/// vigilance. Parley at the beginning of combat on your turn: a +1/+1 counter
/// on her for each nonland card revealed.
pub fn ambassador_kehleyr() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Parley {
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::CardsRevealedThisEffect,
                }),
            },
        }],
        ..legend(
            "Ambassador K'Ehleyr",
            cost(&[generic(2), w()]),
            vec![CreatureType::Klingon, CreatureType::Human, CreatureType::Advisor],
            2,
            2,
        )
    }
}

/// Beverly Crusher, Ship's Doctor — {2}{W} legendary 2/2 Human Doctor. Seek
/// out: a +1/+1 counter on each of up to three target creatures.
pub fn beverly_crusher_ships_doctor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![seek_out(Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(counters(Selector::Target(0), CounterType::PlusOnePlusOne, 1)),
        })],
        ..legend(
            "Beverly Crusher, Ship's Doctor",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Doctor],
            2,
            2,
        )
    }
}

/// Dr. Pulaski, Cynical Clinician — {1}{W} legendary 2/2 Human Doctor. Seek
/// out: gain 4 life.
pub fn dr_pulaski_cynical_clinician() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![seek_out(Effect::GainLife { who: Selector::You, amount: Value::Const(4) })],
        ..legend(
            "Dr. Pulaski, Cynical Clinician",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Doctor],
            2,
            2,
        )
    }
}

/// Ensign Harry Kim, Operations — {1}{W} legendary 1/2 Human Officer. Another
/// creature of yours entering with greater power grows him; {2}, remove two
/// +1/+1 counters from among your creatures: draw a card.
pub fn ensign_harry_kim_operations() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::PowerGreaterThanSource),
                },
            ),
            effect: counters(Selector::This, CounterType::PlusOnePlusOne, 1),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            remove_counter_among_filter: Some((Some(CounterType::PlusOnePlusOne), 2, R::Creature)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..legend(
            "Ensign Harry Kim, Operations",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Officer],
            1,
            2,
        )
    }
}

/// Ezri Dax, Newly Joined — {1}{W} legendary 2/2 Trill Advisor, flash. May
/// enter as a copy of a creature card put into your graveyard from the
/// battlefield this turn, keeping her name and adding legendary Trill.
pub fn ezri_dax_newly_joined() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature.and(R::InYourGraveyard).and(R::PutIntoGraveyardFromBattlefieldThisTurn),
            from_graveyards: true,
            keep_name: true,
            legendary: true,
            extra_creature_types: vec![CreatureType::Trill],
            ..Default::default()
        }),
        ..legend(
            "Ezri Dax, Newly Joined",
            cost(&[generic(1), w()]),
            vec![CreatureType::Trill, CreatureType::Advisor],
            2,
            2,
        )
    }
}

/// Guinan, Ancient Advisor — {2}{W} legendary 1/4 El-Aurian Advisor. {1}, {T}:
/// a +1/+1 counter on each creature you control that entered this turn; two
/// or more counters placed draws a card.
pub fn guinan_ancient_advisor() -> CardDefinition {
    let entered = || R::Creature.and(R::ControlledByYou).and(R::EnteredThisTurn);
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                counters(Selector::EachPermanent(entered()), CounterType::PlusOnePlusOne, 1),
                Effect::If {
                    cond: Predicate::SelectorCountAtLeast { sel: Selector::EachPermanent(entered()), n: Value::Const(2) },
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Guinan, Ancient Advisor",
            cost(&[generic(2), w()]),
            vec![CreatureType::ElAurian, CreatureType::Advisor],
            1,
            4,
        )
    }
}

/// Julian Bashir, Frontier Doctor — {2}{W} legendary 1/1 Human Doctor. Seek
/// out: a +1/+1 counter on him. Attacking returns target creature card with
/// mana value at most his power from your graveyard to the battlefield.
pub fn julian_bashir_frontier_doctor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            seek_out(counters(Selector::This, CounterType::PlusOnePlusOne, 1)),
            on_attack(Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMostSourcePower)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        ],
        ..legend(
            "Julian Bashir, Frontier Doctor",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Doctor],
            1,
            1,
        )
    }
}

/// Spot, Felis catus — {2}{W} legendary 1/1 Cat. It or another nontoken
/// legendary permanent of yours entering makes a Food. {4}, sacrifice a Food:
/// destroy target artifact.
pub fn spot_felis_catus() -> CardDefinition {
    let food = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(crabomination_base::tokens::food_token()),
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(food()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasSupertype(Supertype::Legendary).and(R::IsToken.negate()),
                    },
                ),
                effect: food(),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Food), 1)),
            effect: Effect::Destroy { what: target_filtered(R::Artifact) },
            ..Default::default()
        }],
        ..legend("Spot, Felis catus", cost(&[generic(2), w()]), vec![CreatureType::Cat], 1, 1)
    }
}

/// U.S.S. Enterprise-E, Sovereign — {3}{W}{W} Spacecraft. Entering destroys
/// all tapped creatures. Station; 8+: 7/7 flying, vigilance.
pub fn u_s_s_enterprise_e_sovereign() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::Tapped)) })],
        ..spacecraft(
            "U.S.S. Enterprise-E, Sovereign",
            cost(&[generic(3), w(), w()]),
            vec![StationBand {
                min: 8,
                keywords: vec![Keyword::Flying, Keyword::Vigilance],
                pt: Some((7, 7)),
                ..Default::default()
            }],
        )
    }
}

/// Chief Miles O'Brien — {2}{U} legendary 2/4 Human Artificer. Seek out:
/// double each kind of counter on target artifact or creature you control.
pub fn chief_miles_obrien() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![seek_out(Effect::DoubleAllCountersOn {
            what: target_filtered(R::Artifact.or(R::Creature).and(R::ControlledByYou)),
        })],
        ..legend(
            "Chief Miles O'Brien",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            4,
        )
    }
}

/// Data, Exploring Humanity — {4}{U} legendary 4/4 Android Scientist
/// artifact creature. Seek out: discover 4.
pub fn data_exploring_humanity() -> CardDefinition {
    let mut c = CardDefinition {
        triggered_abilities: vec![seek_out(Effect::Discover { n: Value::Const(4), filter: None })],
        ..legend(
            "Data, Exploring Humanity",
            cost(&[generic(4), u()]),
            vec![CreatureType::Android, CreatureType::Scientist],
            4,
            4,
        )
    };
    c.card_types.insert(0, CardType::Artifact);
    c
}

/// Geordi La Forge, Innovator — {2}{U} legendary 2/3 Human Artificer.
/// Unblockable while you control a Spacecraft; connecting draws a card.
pub fn geordi_la_forge_innovator() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Geordi La Forge can't be blocked if you control a Spacecraft.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Unblockable,
                condition: Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: spacecraft_filter(),
                }),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..legend(
            "Geordi La Forge, Innovator",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            3,
        )
    }
}

/// Kes, Evolving Telepath — {4}{U} legendary 2/6 Ocampa Advisor. Your end
/// step: a time counter on her, draw one card per time counter, then sacrifice
/// her at nine or more.
pub fn kes_evolving_telepath() -> CardDefinition {
    let time = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Time };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Seq(vec![
                counters(Selector::This, CounterType::Time, 1),
                Effect::Draw { who: Selector::You, amount: time() },
                Effect::If {
                    cond: Predicate::ValueAtLeast(time(), Value::Const(9)),
                    then: Box::new(Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::ONE,
                        filter: R::IsSource,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..legend(
            "Kes, Evolving Telepath",
            cost(&[generic(4), u()]),
            vec![CreatureType::Ocampa, CreatureType::Advisor],
            2,
            6,
        )
    }
}

/// On the Shoulders of Giants — {2}{U} enchantment. Historic creatures you
/// control have flying.
pub fn on_the_shoulders_of_giants() -> CardDefinition {
    CardDefinition {
        name: "On the Shoulders of Giants",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Historic creatures you control have flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::historic())),
                keyword: Keyword::Flying,
            },
        }],
        ..Default::default()
    }
}

/// Seven of Nine, Tertiary Adjunct — {2}{U} legendary 1/4 Borg artifact
/// creature. That Is Irrelevant — {T}: loot; discarding a nonland card this
/// way puts two charge counters on up to one target artifact (reflexive).
pub fn seven_of_nine_tertiary_adjunct() -> CardDefinition {
    let mut c = CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                Effect::If {
                    cond: Predicate::DiscardedNonlandThisEffect { who: PlayerRef::You },
                    then: Box::new(Effect::Reflexive {
                        body: Box::new(Effect::OptionalTargets {
                            min: 0,
                            body: Box::new(counters(target_filtered(R::Artifact), CounterType::Charge, 2)),
                        }),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..legend("Seven of Nine, Tertiary Adjunct", cost(&[generic(2), u()]), vec![CreatureType::Borg], 1, 4)
    };
    c.card_types.insert(0, CardType::Artifact);
    c
}

/// Temporal Tribulation — {2}{U} instant. Exile two target permanents you
/// control; they return under their owners' control at the next end step.
/// Flashback {5}{U}.
pub fn temporal_tribulation() -> CardDefinition {
    CardDefinition {
        name: "Temporal Tribulation",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(cost(&[generic(5), u()]))],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Permanent.and(R::ControlledByYou),
            effect: Box::new(Effect::ExileReturnNextEndStep { what: Selector::Target(0) }),
        },
        ..Default::default()
    }
}

/// Tuvok, Chief Tactical Officer — {2}{U} legendary 2/3 Vulcan Officer, flash.
/// Entering during the declare attackers step lets you reselect what up to
/// two target attacking creatures are attacking.
pub fn tuvok_chief_tactical_officer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::CurrentStepIs(TurnStep::DeclareAttackers)),
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::IsAttacking,
                effect: Box::new(Effect::ReselectAttackTarget { what: Selector::Target(0), players_only: false }),
            },
        }],
        ..legend(
            "Tuvok, Chief Tactical Officer",
            cost(&[generic(2), u()]),
            vec![CreatureType::Vulcan, CreatureType::Officer],
            2,
            3,
        )
    }
}

/// Ro Laren, Double Agent — {2}{R} legendary 3/2 Bajoran Rogue Officer. Seek
/// out: 2 damage to each opponent. {1}, {T}, discard a card: draw a card.
pub fn ro_laren_double_agent() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![seek_out(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..legend(
            "Ro Laren, Double Agent",
            cost(&[generic(2), r()]),
            vec![CreatureType::Bajoran, CreatureType::Rogue, CreatureType::Officer],
            3,
            2,
        )
    }
}

/// Rom and Leeta, Unlikely Pair — {3}{R} legendary 4/3 Ferengi Bajoran. Seek
/// out: two tapped Treasures.
pub fn rom_and_leeta_unlikely_pair() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![seek_out(crate::effect::shortcut::mint_tapped_treasures(2))],
        ..legend(
            "Rom and Leeta, Unlikely Pair",
            cost(&[generic(3), r()]),
            vec![CreatureType::Ferengi, CreatureType::Bajoran],
            4,
            3,
        )
    }
}

/// B'Elanna Torres, Scrapper — {1}{U}{R} legendary 2/2 Klingon Human
/// Artificer, vigilance, trample. {2}, {T}: surveil 2, then reveal the top
/// card; an artifact is drawn, otherwise two +1/+1 counters on her. (The
/// "may reveal" is always taken.)
pub fn belanna_torres_scrapper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
                Effect::RevealTopThenIf {
                    who: PlayerRef::You,
                    filter: R::Artifact,
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: Some(Box::new(counters(Selector::This, CounterType::PlusOnePlusOne, 2))),
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "B'Elanna Torres, Scrapper",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Klingon, CreatureType::Human, CreatureType::Artificer],
            2,
            2,
        )
    }
}

/// Cmdr. Troi, Outgoing Empath — {2}{U}{R} legendary 3/3 Betazoid Human
/// Advisor, vigilance. {T}: target opponent whose turn it is may cast a spell
/// from their hand for free; if they do you copy it (new targets allowed),
/// otherwise you draw a card.
pub fn cmdr_troi_outgoing_empath() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::TargetPlayerThen {
                    filter: R::OpponentPlayer.and(R::ControlledByActivePlayer),
                    then: Box::new(Effect::Noop),
                },
                Effect::PlayerMayCastFromHandFree {
                    who: PlayerRef::Target(0),
                    if_cast: Box::new(Effect::CopySpellMayChooseTargets {
                        what: Selector::Target(0),
                        count: Value::ONE,
                    }),
                    else_: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Cmdr. Troi, Outgoing Empath",
            cost(&[generic(2), u(), r()]),
            vec![CreatureType::Betazoid, CreatureType::Human, CreatureType::Advisor],
            3,
            3,
        )
    }
}

/// Commander William Riker — {1}{R}{W} legendary 3/3 Human Officer,
/// vigilance, haste. {1}, {T}: you and target opponent each draw two.
pub fn commander_william_riker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: you_and_target_opponent_draw_two(),
            ..Default::default()
        }],
        ..legend(
            "Commander William Riker",
            cost(&[generic(1), r(), w()]),
            vec![CreatureType::Human, CreatureType::Officer],
            3,
            3,
        )
    }
}

/// Reg Barclay, Fidgety Fantasist — {2}{U}{R} legendary 1/5 Human Artificer.
/// Seek out: a hasty nonlegendary Illusion artifact token copy of another
/// target creature you control, exiled at the next end step.
pub fn reg_barclay_fidgety_fantasist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![seek_out(Effect::Seq(vec![
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                extra_creature_types: vec![CreatureType::Illusion],
                extra_card_types: vec![CardType::Artifact],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: true,
                legendary: false,
                extra_keywords: vec![Keyword::Haste],
                no_mana_cost: false,
                enters_with_counters: None,
                remove_keywords: vec![],
            },
            Effect::ExileAtNextEndStep { what: Selector::LastCreatedTokens },
        ]))],
        ..legend(
            "Reg Barclay, Fidgety Fantasist",
            cost(&[generic(2), u(), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            5,
        )
    }
}

/// Tom Paris, Risk-Taker — {4}{U}{R} legendary 4/4 Human Pilot. Seek out:
/// your creatures gain flying, double strike, vigilance and haste until end
/// of turn.
pub fn tom_paris_risk_taker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![seek_out(Effect::GrantKeywords {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            keywords: vec![Keyword::Flying, Keyword::DoubleStrike, Keyword::Vigilance, Keyword::Haste],
            duration: Duration::EndOfTurn,
        })],
        ..legend(
            "Tom Paris, Risk-Taker",
            cost(&[generic(4), u(), r()]),
            vec![CreatureType::Human, CreatureType::Pilot],
            4,
            4,
        )
    }
}

/// Unification — {3}{W}{U} Saga. I — the owner of up to one target creature
/// draws two, then shuffles it into their library. II — you may put a
/// Spacecraft card from your hand or graveyard onto the battlefield. III — you
/// and target opponent each draw two.
pub fn unification() -> CardDefinition {
    let owner = || PlayerRef::OwnerOf(Box::new(Selector::Target(0)));
    CardDefinition {
        name: "Unification",
        cost: cost(&[generic(3), w(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::Player(owner()), amount: Value::Const(2) },
                        Effect::Move {
                            what: target_filtered(R::Creature),
                            to: ZoneDest::Library { who: owner(), pos: LibraryPosition::Shuffled },
                        },
                    ])),
                },
            ),
            (2, Effect::PutFromHandOrGraveyardOntoBattlefield { filter: spacecraft_filter() }),
            (3, you_and_target_opponent_draw_two()),
        ],
        ..Default::default()
    }
}

/// Vic Fontaine, One Night Only — {1}{R}{W} legendary 3/3 Illusion Performer
/// artifact creature. Seek out: one or two target creatures each get +2/+0
/// and gain lifelink until end of turn.
pub fn vic_fontaine_one_night_only() -> CardDefinition {
    let mut c = CardDefinition {
        triggered_abilities: vec![seek_out(Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 1,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
            ])),
        })],
        ..legend(
            "Vic Fontaine, One Night Only",
            cost(&[generic(1), r(), w()]),
            vec![CreatureType::Illusion, CreatureType::Performer],
            3,
            3,
        )
    };
    c.card_types.insert(0, CardType::Artifact);
    c
}

/// Worf, Son of Mogh — {1}{R}{W} legendary 3/3 Klingon Warrior Officer,
/// menace. Other Officers you control get +1/+1. Seek out: a 1/1 red Officer.
pub fn worf_son_of_mogh() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Other Officers you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(officer()).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![seek_out(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: officer_token(),
        })],
        ..legend(
            "Worf, Son of Mogh",
            cost(&[generic(1), r(), w()]),
            vec![CreatureType::Klingon, CreatureType::Warrior, CreatureType::Officer],
            3,
            3,
        )
    }
}

/// U.S.S. Defiant, Tough Little Ship — {6} Spacecraft. Entering destroys up
/// to one target artifact or enchantment. Station; 5+: 7/4 flying, hexproof.
pub fn u_s_s_defiant_tough_little_ship() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) }),
        })],
        ..spacecraft(
            "U.S.S. Defiant, Tough Little Ship",
            cost(&[generic(6)]),
            vec![StationBand {
                min: 5,
                keywords: vec![Keyword::Flying, Keyword::Hexproof],
                pt: Some((7, 4)),
                ..Default::default()
            }],
        )
    }
}

/// U.S.S. Enterprise-C, Time-Tossed — {5} Spacecraft. Entering or dying, you
/// may discard your hand and draw that many. Station; 3+: 6/6 flying.
pub fn u_s_s_enterprise_c_time_tossed() -> CardDefinition {
    let wheel = || Effect::MayDo {
        description: "Discard your hand and draw that many cards?".into(),
        body: Box::new(Effect::DiscardHandDrawThatMany { who: Selector::You }),
    };
    CardDefinition {
        triggered_abilities: vec![etb(wheel()), on_dies(wheel())],
        ..spacecraft(
            "U.S.S. Enterprise-C, Time-Tossed",
            cost(&[generic(5)]),
            vec![StationBand { min: 3, keywords: vec![Keyword::Flying], pt: Some((6, 6)), ..Default::default() }],
        )
    }
}

/// U.S.S. Prometheus, Prototype — {5} Spacecraft. Station; 4+: 6/6 flying;
/// 8+: its combat damage to an opponent is dealt again to each other opponent.
pub fn u_s_s_prometheus_prototype() -> CardDefinition {
    spacecraft(
        "U.S.S. Prometheus, Prototype",
        cost(&[generic(5)]),
        vec![
            StationBand { min: 4, keywords: vec![Keyword::Flying], pt: Some((6, 6)), ..Default::default() },
            StationBand {
                min: 8,
                triggers: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                    effect: Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachOpponentExceptTriggerer),
                        amount: Value::TriggerEventAmount,
                    },
                }],
                ..Default::default()
            },
        ],
    )
}

/// U.S.S. Stargazer, Here and There — {3} Spacecraft. Station; 6+: 4/3
/// flying, and attacking makes a tapped-and-attacking 4/3 flying Spacecraft
/// Illusion artifact creature token exiled at end of combat.
pub fn u_s_s_stargazer_here_and_there() -> CardDefinition {
    let decoy = TokenDefinition {
        name: "Spacecraft Illusion".into(),
        power: 4,
        toughness: 3,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Illusion],
            artifact_subtypes: vec![ArtifactSubtype::Spacecraft],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    spacecraft(
        "U.S.S. Stargazer, Here and There",
        cost(&[generic(3)]),
        vec![StationBand {
            min: 6,
            keywords: vec![Keyword::Flying],
            pt: Some((4, 3)),
            triggers: vec![on_attack(Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(decoy),
                cleanup: AttackingTokenCleanup::ExileAtEndOfCombat,
                defender: None,
            })],
            ..Default::default()
        }],
    )
}

/// Malfunctioning Holodeck — {2}{W} artifact, ward {2}. Entering exiles
/// target nonland permanent an opponent controls until it leaves.
pub fn malfunctioning_holodeck() -> CardDefinition {
    CardDefinition {
        name: "Malfunctioning Holodeck",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}
