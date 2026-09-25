//! Commander: the cards the **Timey-Wimey** precon (WHO, The Tenth Doctor +
//! Rose Tyler) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_tenth.rs`.
//!
//! Residuals (each also on its card):
//! - **Clockspinning** — keyword counters can't be chosen.
//! - **The Day of the Doctor** — chapter IV keeps your own greatest-power
//!   Doctors; you can't keep an opponent's.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EntersAsCopy, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, MayPlayDuration,
    SelectionRequirement as R, Selector, SplitCard, SplitHalf, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{emerge, etb, explore, investigate, on_attack, on_you_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, generic, r, u, w, Color, ManaCost};
use crabomination_base::tokens::{food_token, treasure_token};

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

fn legend(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

/// A legendary Doctor's companion (CR 702.124m).
fn companion(mut def: CardDefinition) -> CardDefinition {
    def.keywords.push(Keyword::DoctorsCompanion);
    legend(def)
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
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

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn doctor() -> R {
    R::HasCreatureType(CreatureType::Doctor)
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

fn time_travel() -> Effect {
    Effect::TimeTravel { who: PlayerRef::You }
}

fn time_counter_on_self() -> Effect {
    Effect::AddCounter { what: Selector::This, kind: CounterType::Time, amount: Value::ONE }
}

fn time_counters_on_self() -> Value {
    Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Time }
}

fn step(s: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(s), EventScope::YourControl), effect }
}

fn on_combat_damage_to_player(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource), effect }
}

/// "Whenever you sacrifice a [filter]" (Clue / Food payoffs).
fn on_you_sacrifice(filter: R, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl).with_filter(trigger_is(filter)),
        effect,
    }
}

fn clue() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Clue)
}

fn food() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Food)
}

fn token(name: &str, colors: Vec<Color>, types: Vec<CreatureType>, p: i32, t: i32) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn make(t: TokenDefinition, n: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: Arc::new(t) }
}

/// "Exile cards from the top of your library until you exile a nonland card.
/// Put [n] time counters on it. If it doesn't have suspend, it gains
/// suspend."
fn exile_until_nonland_suspend(n: u32) -> Effect {
    Effect::Seq(vec![
        Effect::ExileTopUntilNonland { who: PlayerRef::You },
        Effect::GrantSuspend { what: Selector::ExiledThisResolution { filter: R::Nonland }, time_counters: n },
    ])
}

/// Adipose Offspring — emerge {5}{W}; entering makes a 2/2 Alien, or X of
/// them if emerged (X = the sacrificed creature's toughness).
pub fn adipose_offspring() -> CardDefinition {
    let alien = || token("Alien", vec![Color::White], vec![CreatureType::Alien], 2, 2);
    CardDefinition {
        alternative_cost: Some(emerge(cost(&[generic(5), w()]))),
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::ValueAtLeast(Value::EmergeSacrificedToughness, Value::ONE),
            then: Box::new(make(alien(), Value::EmergeSacrificedToughness)),
            else_: Box::new(make(alien(), Value::ONE)),
        })],
        ..creature("Adipose Offspring", cost(&[generic(3), w()]), vec![CreatureType::Alien], 2, 2)
    }
}

/// All of History, All at Once — time travel; storm.
pub fn all_of_history_all_at_once() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Storm],
        ..spell("All of History, All at Once", cost(&[generic(2), u(), u()]), CardType::Sorcery, time_travel())
    }
}

/// Amy Pond — partner with Rory Williams; her combat damage takes that many
/// time counters off a suspended card you own; Doctor's companion.
pub fn amy_pond() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Rory Williams".into())],
        triggered_abilities: vec![on_combat_damage_to_player(Effect::RemoveTimeCountersFromSuspended {
            amount: Value::TriggerEventAmount,
        })],
        ..companion(creature("Amy Pond", cost(&[generic(2), r()]), vec![CreatureType::Human], 2, 2))
    }
}

/// As Foretold — your upkeep adds a time counter; once each turn, a spell of
/// mana value up to its counters may cost {0}.
pub fn as_foretold() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Once each turn, you may pay {0} rather than pay the mana cost for a spell you cast with mana value X or less, where X is the number of time counters on this enchantment.",
            effect: StaticEffect::ZeroCostOncePerTurnMvAtMostSourceCounters(CounterType::Time),
        }],
        triggered_abilities: vec![step(TurnStep::Upkeep, time_counter_on_self())],
        ..enchantment("As Foretold", cost(&[generic(2), u()]))
    }
}

/// Astrid Peth — entering or attacking makes a Food; sacrificing a Clue or
/// Food makes her explore.
pub fn astrid_peth() -> CardDefinition {
    let food_token_effect = || make(food_token(), Value::ONE);
    CardDefinition {
        triggered_abilities: vec![
            etb(food_token_effect()),
            on_attack(food_token_effect()),
            on_you_sacrifice(clue().or(food()), Effect::Explore { who: Selector::This }),
        ],
        ..legend(creature("Astrid Peth", cost(&[generic(1), w()]), vec![CreatureType::Human], 2, 2))
    }
}

/// Atraxi Warden — flying; entering exiles up to one target tapped creature;
/// suspend 5—{1}{W}.
pub fn atraxi_warden() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Suspend(5, cost(&[generic(1), w()]))],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Creature.and(R::Tapped),
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
        })],
        ..creature(
            "Atraxi Warden",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Alien, CreatureType::Eye],
            6,
            6,
        )
    }
}

/// Clockspinning — buyback {3}; a counter on target permanent or suspended
/// card is removed or doubled up (the caster helps its own, hurts an
/// opponent's). Residual: keyword counters can't be chosen.
pub fn clockspinning() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Buyback(cost(&[generic(3)]))],
        ..spell(
            "Clockspinning",
            cost(&[u()]),
            CardType::Instant,
            Effect::Clockspin {
                what: target_filtered(
                    R::OnBattlefield.and(R::WithAnyCounter).or(R::InExile.and(R::IsSuspended)),
                ),
            },
        )
    }
}

/// Crack in Time — vanishing 3; entering and at your first main phase, exile
/// target creature an opponent controls until it leaves.
pub fn crack_in_time() -> CardDefinition {
    let crack = || Effect::ExileUntilSourceLeaves {
        what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        return_to: ExileReturnZone::Battlefield,
    };
    CardDefinition {
        keywords: vec![Keyword::Vanishing(3)],
        triggered_abilities: vec![etb(crack()), step(TurnStep::PreCombatMain, crack())],
        ..enchantment("Crack in Time", cost(&[generic(3), w()]))
    }
}

/// Dinosaurs on a Spaceship — vigilance, trample; other Dinosaurs get +1/+1,
/// vigilance and trample; suspend 4; each time counter removed while exiled
/// makes a 2/2 flying, hasty Dinosaur.
pub fn dinosaurs_on_a_spaceship() -> CardDefinition {
    let others = || Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::Dinosaur)).and(R::OtherThanSource));
    let dino = TokenDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        ..token("Dinosaur", vec![Color::Red, Color::White], vec![CreatureType::Dinosaur], 2, 2)
    };
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Suspend(4, cost(&[generic(3), r(), w()]))],
        static_abilities: vec![
            StaticAbility {
                description: "Other Dinosaurs you control get +1/+1.",
                effect: StaticEffect::PumpPT { applies_to: others(), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "Other Dinosaurs you control have vigilance.",
                effect: StaticEffect::GrantKeyword { applies_to: others(), keyword: Keyword::Vigilance },
            },
            StaticAbility {
                description: "Other Dinosaurs you control have trample.",
                effect: StaticEffect::GrantKeyword { applies_to: others(), keyword: Keyword::Trample },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterRemoved(CounterType::Time), EventScope::SelfSource).while_suspended(),
            effect: make(dino, Value::ONE),
        }],
        ..creature(
            "Dinosaurs on a Spaceship",
            cost(&[generic(4), r(), w()]),
            vec![CreatureType::Dinosaur],
            7,
            7,
        )
    }
}

/// Donna Noble — soulbond; she or her partner being dealt damage makes her
/// deal that much to target opponent; Doctor's companion.
pub fn donna_noble() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Soulbond],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::AnyPlayer)
                .with_filter(trigger_is(R::IsSource.or(R::PairedWithSource))),
            effect: Effect::DealDamage { to: target_filtered(R::OpponentPlayer), amount: Value::TriggerEventAmount },
        }],
        ..companion(creature("Donna Noble", cost(&[generic(3), r()]), vec![CreatureType::Human], 2, 4))
    }
}

/// Ecstatic Beauty — exile your top three, playable this turn; four time
/// counters on each with suspend. Suspend 4—{R}.
pub fn ecstatic_beauty() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(4, cost(&[r()]))],
        ..spell(
            "Ecstatic Beauty",
            cost(&[generic(2), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::ExileTopOfLibrary {
                    who: Selector::You,
                    amount: Value::Const(3),
                    link_to_source: false,
                    face_down: false,
                },
                Effect::GrantMayPlay {
                    what: Selector::ExiledThisResolution { filter: R::Any },
                    duration: MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
                Effect::AddCounter {
                    what: Selector::ExiledThisResolution { filter: R::HasSuspend },
                    kind: CounterType::Time,
                    amount: Value::Const(4),
                },
            ]),
        )
    }
}

/// Everybody Lives! — creatures gain hexproof and indestructible, players
/// gain hexproof; nobody can lose life, lose or win this turn.
pub fn everybody_lives() -> CardDefinition {
    spell(
        "Everybody Lives!",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::PlayerHexproofThisTurn { who: Selector::Player(PlayerRef::EachPlayer) },
            Effect::CantLoseLifeThisTurn { who: Selector::Player(PlayerRef::EachPlayer) },
            Effect::EachPlayerDoes {
                who: PlayerRef::EachPlayer,
                body: Box::new(Effect::CantLoseThisTurn { damage_floor: false }),
            },
        ]),
    )
}

/// Everything Comes to Dust — convoke; exile all creatures but the convokers'
/// kin, all artifacts and all enchantments.
pub fn everything_comes_to_dust() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        ..spell(
            "Everything Comes to Dust",
            cost(&[generic(7), w(), w(), w()]),
            CardType::Sorcery,
            Effect::ExileAllButConvokerKin,
        )
    }
}

/// Flesh Duplicate — may enter as a copy of any creature, with vanishing 3
/// if it doesn't have vanishing.
pub fn flesh_duplicate() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            extra_keywords: vec![Keyword::Vanishing(3)],
            ..Default::default()
        }),
        ..creature(
            "Flesh Duplicate",
            cost(&[u(), u()]),
            vec![CreatureType::Shapeshifter, CreatureType::Rebel],
            0,
            0,
        )
    }
}

/// Four Knocks — vanishing 4; draw a card at your first main phase.
pub fn four_knocks() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vanishing(4)],
        triggered_abilities: vec![step(TurnStep::PreCombatMain, draw(1))],
        ..enchantment("Four Knocks", cost(&[generic(2), w()]))
    }
}

/// Idris, Soul of the TARDIS — vanishing 3; entering exiles another artifact
/// of yours until she leaves; she has its activated abilities and +X/+X (X =
/// its mana value), and its triggered abilities.
pub fn idris_soul_of_the_tardis() -> CardDefinition {
    let x = || Value::TotalManaValueOf(Box::new(Selector::CardExiledWithSource));
    CardDefinition {
        keywords: vec![Keyword::Vanishing(3)],
        static_abilities: vec![
            StaticAbility {
                description: "Idris gets +X/+X, where X is the exiled card's mana value.",
                effect: StaticEffect::PumpSelfByValue { amount: x(), per_power: 1, per_toughness: 1 },
            },
        ],
        // The borrowed abilities are stamped as the card is exiled: a static
        // reading `exiled_with` never saw a card `ExileUntilSourceLeaves`
        // links through `exiled_by`, so Idris had none.
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ExileUntilSourceLeaves {
                what: Selector::Take {
                    inner: Box::new(Selector::EachPermanent(yours(R::Artifact).and(R::OtherThanSource))),
                    count: Box::new(Value::ONE),
                },
                return_to: ExileReturnZone::Battlefield,
            },
            Effect::AcquireAbilitiesOfExiledWithSource,
        ]))],
        ..legend(creature(
            "Idris, Soul of the TARDIS",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Human, CreatureType::Incarnation],
            3,
            3,
        ))
    }
}

/// Jenny, Generated Anomaly — double strike; her combat damage makes her
/// explore.
pub fn jenny_generated_anomaly() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![on_combat_damage_to_player(explore())],
        ..legend(creature(
            "Jenny, Generated Anomaly",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::TimeLord, CreatureType::Soldier],
            2,
            3,
        ))
    }
}

/// Judoon Enforcers — trample; no more than one creature can attack you each
/// combat; suspend 6—{1}{R}{W}.
pub fn judoon_enforcers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Suspend(6, cost(&[generic(1), r(), w()]))],
        static_abilities: vec![StaticAbility {
            description: "No more than one creature can attack you each combat.",
            effect: StaticEffect::AttackerCapAgainstController { n: 1 },
        }],
        ..creature(
            "Judoon Enforcers",
            cost(&[generic(5), r(), w()]),
            vec![CreatureType::Alien, CreatureType::Rhino, CreatureType::Soldier],
            8,
            8,
        )
    }
}

/// Kate Stewart — putting time counters on a permanent of yours makes a 1/1
/// Soldier; attacking, pay {8} for +X/+X on attackers (X = your time
/// counters).
pub fn kate_stewart() -> CardDefinition {
    let x = || Value::CountersOn { what: Box::new(Selector::EachPermanent(R::ControlledByYou)), kind: CounterType::Time };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CounterAdded(CounterType::Time), EventScope::YouPutCounters)
                    .with_filter(trigger_is(R::ControlledByYou)),
                effect: make(token("Soldier", vec![Color::White], vec![CreatureType::Soldier], 1, 1), Value::ONE),
            },
            on_attack(Effect::MayPay {
                description: "Pay {8} to pump your attackers?".into(),
                mana_cost: cost(&[generic(8)]),
                body: Box::new(Effect::PumpPT {
                    what: Selector::EachPermanent(R::IsAttacking),
                    power: x(),
                    toughness: x(),
                    duration: Duration::EndOfTurn,
                }),
                else_: None,
            }),
        ],
        ..legend(creature(
            "Kate Stewart",
            cost(&[generic(1), u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Scientist],
            3,
            3,
        ))
    }
}

/// Martha Jones — entering investigates; sacrificing a Clue makes her and up
/// to one other target creature unblockable this turn; Doctor's companion.
pub fn martha_jones() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(investigate(1)),
            on_you_sacrifice(
                clue(),
                Effect::Seq(vec![
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
                    Effect::ApplyToTargets {
                        max_targets: 1,
                        min_targets: 0,
                        filter: R::Creature.and(R::OtherThanSource),
                        effect: Box::new(Effect::GrantKeyword {
                            what: Selector::Target(0),
                            keyword: Keyword::Unblockable,
                            duration: Duration::EndOfTurn,
                        }),
                    },
                ]),
            ),
        ],
        ..companion(creature("Martha Jones", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Cleric], 3, 2))
    }
}

/// RMS Titanic — flying, trample Vehicle (crew 3); its combat damage
/// sacrifices it for that many Treasures.
pub fn rms_titanic() -> CardDefinition {
    CardDefinition {
        name: "RMS Titanic",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 7,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::Crew(3)],
        triggered_abilities: vec![on_combat_damage_to_player(Effect::Seq(vec![
            Effect::SacrificeSource,
            make(treasure_token(), Value::TriggerEventAmount),
        ]))],
        ..Default::default()
    }
}

/// Regenerations Restored — twelve time counters, one off each upkeep; each
/// removal scries 1 and gains 1, and the last exiles it for an extra turn.
/// (Vanishing 12, spelled out: the last counter exiles it rather than
/// sacrificing it.)
pub fn regenerations_restored() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Time, Value::Const(12))),
        triggered_abilities: vec![
            step(
                TurnStep::Upkeep,
                Effect::RemoveCounter { what: Selector::This, kind: CounterType::Time, amount: Value::ONE },
            ),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CounterRemoved(CounterType::Time), EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                    Effect::If {
                        cond: Predicate::SourceHasCountersAtLeast { counter: CounterType::Time, n: 1 },
                        then: Box::new(Effect::Noop),
                        else_: Box::new(Effect::Seq(vec![
                            Effect::Move { what: Selector::This, to: ZoneDest::Exile },
                            Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
                        ])),
                    },
                ]),
            },
        ],
        ..enchantment("Regenerations Restored", cost(&[w(), u()]))
    }
}

/// Rory Williams — partner with Amy Pond; first strike, lifelink; cast from
/// anywhere but exile, he exiles himself with three time counters and
/// suspend, then investigates.
pub fn rory_williams() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Amy Pond".into()), Keyword::FirstStrike, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource)
                .with_filter(Predicate::Not(Box::new(Predicate::CastSpellFromExile))),
            effect: Effect::Seq(vec![
                Effect::GrantSuspend { what: Selector::This, time_counters: 3 },
                investigate(1),
            ]),
        }],
        ..legend(creature(
            "Rory Williams",
            cost(&[w(), u()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        ))
    }
}

/// Rose Tyler — +1/+1 per time counter on her; attacking, a time counter per
/// suspended card you own and per other permanent of yours with a time
/// counter; Doctor's companion.
pub fn rose_tyler() -> CardDefinition {
    let bad_wolf = Value::Sum(vec![
        Value::CountOf(Box::new(Selector::CardsInZone {
            who: PlayerRef::You,
            zone: crate::card::Zone::Exile,
            filter: R::HasSuspend.and(R::WithCounter(CounterType::Time)),
        })),
        Value::CountOf(Box::new(Selector::EachPermanent(
            yours(R::WithCounter(CounterType::Time)).and(R::OtherThanSource),
        ))),
    ]);
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Rose Tyler gets +1/+1 for each time counter on it.",
            effect: StaticEffect::PumpPTPerCounterOnSource {
                applies_to: Selector::This,
                kind: CounterType::Time,
                per_power: 1,
                per_toughness: 1,
            },
        }],
        triggered_abilities: vec![on_attack(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Time,
            amount: bad_wolf,
        })],
        ..companion(creature("Rose Tyler", cost(&[generic(1), w()]), vec![CreatureType::Human], 2, 2))
    }
}

/// Rotating Fireplace — enters tapped with a time counter; {T}: {C} per time
/// counter; {4},{T}: time travel (sorcery speed).
pub fn rotating_fireplace() -> CardDefinition {
    CardDefinition {
        name: "Rotating Fireplace",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Time, Value::ONE)),
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(time_counters_on_self()) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(4)]),
                sorcery_speed: true,
                effect: time_travel(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Run for Your Life — one or two target creatures gain haste and can't be
/// blocked except by hasty creatures; escape {2}{U}{R}, exile four.
pub fn run_for_your_life() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Escape(cost(&[generic(2), u(), r()]), 4)],
        ..spell(
            "Run for Your Life",
            cost(&[u(), r()]),
            CardType::Instant,
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 1,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::CantBeBlockedExceptBy(Box::new(R::HasKeyword(Keyword::Haste))),
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
        )
    }
}

/// Sally Sparrow — your creature spells have flash; once each turn, other
/// creatures of yours leaving investigates.
pub fn sally_sparrow() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may cast creature spells as though they had flash.",
            effect: StaticEffect::ControllerSpellsHaveFlash { filter: R::Creature },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::AnotherOfYours)
                .with_filter(trigger_is(R::Creature))
                .once_per_turn(),
            effect: investigate(1),
        }],
        ..legend(creature(
            "Sally Sparrow",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Human, CreatureType::Detective],
            2,
            3,
        ))
    }
}

/// Sibylline Soothsayer — entering reveals until a nonland card of mana
/// value 3+, exiled with three time counters and suspend.
pub fn sibylline_soothsayer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: R::Nonland.and(R::ManaValueAtLeast(3)),
                to: ZoneDest::Exile,
                cap: Value::Const(99),
                life_per_revealed: 0,
                miss_dest: crate::effect::RevealMissDest::BottomRandom,
            },
            Effect::GrantSuspend { what: Selector::LastMoved, time_counters: 3 },
        ]))],
        ..creature(
            "Sibylline Soothsayer",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            3,
            2,
        )
    }
}

/// Star Whale — flying, vigilance; your other creatures have ward {2};
/// suspend 6—{1}{U}.
pub fn star_whale() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Suspend(6, cost(&[generic(1), u()]))],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have ward {2}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours(R::Creature).and(R::OtherThanSource)),
                keyword: Keyword::Ward(WardCost::Mana(cost(&[generic(2)]))),
            },
        }],
        ..creature(
            "Star Whale",
            cost(&[generic(6), u(), u()]),
            vec![CreatureType::Alien, CreatureType::Whale],
            8,
            8,
        )
    }
}

/// The Day of the Doctor — I–III: exile from the top until a legendary card,
/// playable while this remains. IV: keep up to three Doctors and you may exile
/// every other creature for 13 damage to you. Residual: IV keeps your own
/// greatest-power Doctors.
pub fn the_day_of_the_doctor() -> CardDefinition {
    let summon = || {
        Effect::Seq(vec![
            Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: R::HasSupertype(Supertype::Legendary),
                to: ZoneDest::ExileWithSourceStamp,
                cap: Value::Const(99),
                life_per_revealed: 0,
                miss_dest: crate::effect::RevealMissDest::BottomRandom,
            },
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: MayPlayDuration::WhileExiled,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
        ])
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::EndMayPlayOnCardsExiledWithSource,
        }],
        ..saga(
            "The Day of the Doctor",
            cost(&[generic(3), r(), w()]),
            vec![
                (1, summon()),
                (2, summon()),
                (3, summon()),
                (
                    4,
                    Effect::MayDo {
                        description: "Exile every creature but up to three of your Doctors and take 13 damage?".into(),
                        body: Box::new(Effect::Seq(vec![
                            Effect::ExileOtherCreaturesKeepingUpTo { keep: doctor(), max: 3 },
                            Effect::DealDamage { to: Selector::You, amount: Value::Const(13) },
                        ])),
                    },
                ),
            ],
        )
    }
}

/// The Eleventh Doctor — his combat damage may exile a card from your hand
/// with its mana value in time counters and suspend; {2}: a creature with
/// power 3 or less can't be blocked this turn.
pub fn the_eleventh_doctor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_combat_damage_to_player(Effect::MayExileFromHandSuspended {
            who: PlayerRef::You,
            filter: R::Any,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::PowerAtMost(3))),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(creature(
            "The Eleventh Doctor",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::TimeLord, CreatureType::Doctor],
            3,
            2,
        ))
    }
}

/// The Eleventh Hour — I: tutor a Doctor. II: a Food and a 1/1 Human that
/// makes Doctor spells cost {1} less. III: a legendary Alien token copy of
/// target creature named Prisoner Zero.
pub fn the_eleventh_hour() -> CardDefinition {
    let human = TokenDefinition {
        static_abilities: vec![StaticAbility {
            description: "Doctor spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: doctor(), amount: 1 },
        }],
        ..token("Human", vec![Color::White], vec![CreatureType::Human], 1, 1)
    };
    saga(
        "The Eleventh Hour",
        cost(&[generic(3), u()]),
        vec![
            (1, Effect::Search { who: PlayerRef::You, filter: doctor(), to: ZoneDest::Hand(PlayerRef::You) }),
            (2, Effect::Seq(vec![make(food_token(), Value::ONE), make(human, Value::ONE)])),
            (
                3,
                Effect::Seq(vec![
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: target_filtered(R::Creature),
                        extra_creature_types: vec![],
                        extra_card_types: vec![],
                        override_pt: None,
                        override_colors: None,
                        enters_tapped: false,
                        non_legendary: false,
                        legendary: true,
                        extra_keywords: vec![],
                    },
                    Effect::SetCopiableNameAndTypes {
                        what: Selector::LastCreatedToken,
                        name: "Prisoner Zero",
                        creature_types: vec![CreatureType::Alien],
                    },
                ]),
            ),
        ],
    )
}

/// The Face of Boe — {T}: cast a spell with suspend from your hand for its
/// suspend cost (sorcery speed).
pub fn the_face_of_boe() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::CastFromHandPayingSuspendCost,
            ..Default::default()
        }],
        ..legend(creature(
            "The Face of Boe",
            cost(&[generic(1), u(), r(), w()]),
            vec![CreatureType::Alien, CreatureType::Advisor],
            0,
            4,
        ))
    }
}

/// The Girl in the Fireplace — I: a 1/1 Human Noble with vanishing 3 that
/// prevents all damage to itself. II: a 2/2 Horse giving your Doctors
/// horsemanship. III: this turn, your creatures' combat damage to a player
/// time travels.
pub fn the_girl_in_the_fireplace() -> CardDefinition {
    let noble = TokenDefinition {
        keywords: vec![Keyword::Vanishing(3)],
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt to this token.",
            effect: StaticEffect::PreventAllDamageToThis,
        }],
        ..token("Human Noble", vec![Color::White], vec![CreatureType::Human, CreatureType::Noble], 1, 1)
    };
    let horse = TokenDefinition {
        static_abilities: vec![StaticAbility {
            description: "Doctors you control have horsemanship.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours(doctor())),
                keyword: Keyword::Horsemanship,
            },
        }],
        ..token("Horse", vec![Color::White], vec![CreatureType::Horse], 2, 2)
    };
    saga(
        "The Girl in the Fireplace",
        cost(&[generic(2), w()]),
        vec![
            (1, make(noble, Value::ONE)),
            (2, make(horse, Value::ONE)),
            (
                3,
                Effect::GrantTriggeredAbilityThisTurnToMatching {
                    filter: yours(R::Creature),
                    trigger: Box::new(on_combat_damage_to_player(time_travel())),
                },
            ),
        ],
    )
}

/// The Moment — your upkeep adds a time counter; {2},{T}: untap a creature
/// of yours and phase it out until The Moment leaves; {3},{T}: destroy each
/// nonland permanent with mana value up to its counters, then sacrifice it.
pub fn the_moment() -> CardDefinition {
    CardDefinition {
        name: "The Moment",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![step(TurnStep::Upkeep, time_counter_on_self())],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::Seq(vec![
                    Effect::Untap { what: target_filtered(yours(R::Creature)), up_to: None },
                    Effect::PhaseOut { what: Selector::Target(0), until_source_leaves: true },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                sorcery_speed: true,
                effect: Effect::Seq(vec![
                    Effect::ForEach {
                        selector: Selector::EachPermanent(R::Nonland.and(R::OtherThanSource)),
                        body: Box::new(Effect::If {
                            cond: Predicate::ValueAtMost(
                                Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                                time_counters_on_self(),
                            ),
                            then: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
                            else_: Box::new(Effect::Noop),
                        }),
                    },
                    Effect::SacrificeSource,
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// The Ninth Doctor — haste; untapping in your untap step gives you an extra
/// upkeep step.
pub fn the_ninth_doctor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource)
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::AdditionalUpkeepStep { count: Value::ONE },
        }],
        ..legend(creature(
            "The Ninth Doctor",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::TimeLord, CreatureType::Doctor],
            2,
            4,
        ))
    }
}

/// The Pandorica — may stay tapped; {1}{W},{T}: untap another target nonland
/// permanent, then phase it out until The Pandorica untaps or leaves
/// (sorcery speed).
pub fn the_pandorica() -> CardDefinition {
    CardDefinition {
        name: "The Pandorica",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1), w()]),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Untap { what: target_filtered(R::Nonland.and(R::OtherThanSource)), up_to: None },
                Effect::PhaseOut { what: Selector::Target(0), until_source_leaves: true },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource),
            effect: Effect::PhaseInHeldBySource,
        }],
        ..Default::default()
    }
}

/// The Parting of the Ways — I: exile your top five; each nonland card gets
/// its mana value in time counters and suspend. II: time travel twice. III:
/// for each opponent, destroy up to one target artifact of theirs.
pub fn the_parting_of_the_ways() -> CardDefinition {
    saga(
        "The Parting of the Ways",
        cost(&[generic(4), r(), r()]),
        vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::ExileTopOfLibrary {
                        who: Selector::You,
                        amount: Value::Const(5),
                        link_to_source: false,
                        face_down: false,
                    },
                    Effect::GrantSuspendManaValueCounters { what: Selector::ExiledThisResolution { filter: R::Nonland } },
                ]),
            ),
            (2, Effect::Seq(vec![time_travel(), time_travel()])),
            (
                3,
                Effect::ForEachOpponentTarget {
                    body: Box::new(Effect::ApplyToTargets {
                        max_targets: 8,
                        min_targets: 0,
                        filter: R::Artifact.and(R::ControlledByOpponent),
                        effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                    }),
                },
            ),
        ],
    )
}

/// The Tenth Doctor — whenever you attack, exile from the top until a nonland
/// card, which gets three time counters and suspend; {7}: time travel three
/// times (sorcery speed).
pub fn the_tenth_doctor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_you_attack(exile_until_nonland_suspend(3))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7)]),
            sorcery_speed: true,
            effect: Effect::Repeat { count: Value::Const(3), body: Box::new(time_travel()) },
            ..Default::default()
        }],
        ..legend(creature(
            "The Tenth Doctor",
            cost(&[generic(3), u(), r()]),
            vec![CreatureType::TimeLord, CreatureType::Doctor],
            3,
            5,
        ))
    }
}

/// The War Doctor — other permanents phasing out and other cards being
/// exiled add time counters; attacking, it deals its time counters to any
/// target, exiling a creature that would die.
pub fn the_war_doctor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PhasesOut, EventScope::AnyPlayer)
                    .with_filter(Predicate::Not(Box::new(Predicate::TriggerSourceIsSelf)))
                    .once_per_batch(),
                effect: time_counter_on_self(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardExiled, EventScope::AnyPlayer)
                    .with_filter(Predicate::Not(Box::new(Predicate::TriggerSourceIsSelf)))
                    .once_per_batch(),
                effect: time_counter_on_self(),
            },
            on_attack(Effect::Seq(vec![
                Effect::ExileIfWouldDieThisTurn { what: target_filtered(R::Any) },
                Effect::DealDamage { to: Selector::Target(0), amount: time_counters_on_self() },
            ])),
        ],
        ..legend(creature(
            "The War Doctor",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::TimeLord, CreatureType::Doctor],
            3,
            5,
        ))
    }
}

/// The Wedding of River Song — draw two; you, then target opponent, may exile
/// a nonland card from hand with its mana value in time counters and
/// suspend; time travel.
pub fn the_wedding_of_river_song() -> CardDefinition {
    spell(
        "The Wedding of River Song",
        cost(&[generic(2), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            draw(2),
            Effect::MayExileFromHandSuspended { who: PlayerRef::You, filter: R::Nonland },
            Effect::LoseLife { who: target_filtered(R::OpponentPlayer), amount: Value::ZERO },
            Effect::MayExileFromHandSuspended { who: PlayerRef::Target(0), filter: R::Nonland },
            time_travel(),
        ]),
    )
}

/// Time Beetle — skulk; its combat damage time travels.
pub fn time_beetle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Skulk],
        triggered_abilities: vec![on_combat_damage_to_player(time_travel())],
        ..creature("Time Beetle", cost(&[generic(1), u()]), vec![CreatureType::Alien, CreatureType::Insect], 1, 1)
    }
}

/// Wedding Ring — entering, if cast, target opponent gets a token copy; an
/// opponent with a Wedding Ring drawing or gaining life on their turn does
/// the same for you (CR 201.2: any artifact of theirs with that name).
pub fn wedding_ring() -> CardDefinition {
    let wed = || {
        Predicate::All(vec![
            Predicate::SelectorExists(Selector::ControlledBy {
                who: PlayerRef::Triggerer,
                filter: R::Artifact.and(R::HasName("Wedding Ring".into())),
            }),
            Predicate::IsTurnOf(PlayerRef::Triggerer),
        ])
    };
    CardDefinition {
        name: "Wedding Ring",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(Predicate::SourceWasCast),
                effect: Effect::Seq(vec![
                    Effect::LoseLife { who: target_filtered(R::OpponentPlayer), amount: Value::ZERO },
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::Target(0),
                        count: Value::ONE,
                        source: Selector::This,
                        extra_creature_types: vec![],
                        extra_card_types: vec![],
                        override_pt: None,
                        override_colors: None,
                        enters_tapped: false,
                        non_legendary: false,
                        legendary: false,
                        extra_keywords: vec![],
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl).with_filter(wed()),
                effect: draw(1),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::OpponentControl).with_filter(wed()),
                effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
            },
        ],
        ..Default::default()
    }
}

/// Wibbly-wobbly, Timey-wimey — time travel, then draw a card.
pub fn wibbly_wobbly_timey_wimey() -> CardDefinition {
    spell(
        "Wibbly-wobbly, Timey-wimey",
        cost(&[generic(1), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![time_travel(), draw(1)]),
    )
}

/// Wilfred Mott — your upkeep adds a time counter, then looks at that many
/// top cards and may put a nonland permanent card of mana value 3 or less
/// onto the battlefield.
pub fn wilfred_mott() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![step(
            TurnStep::Upkeep,
            Effect::Seq(vec![
                time_counter_on_self(),
                Effect::LookTopPutMatchingOntoBattlefield {
                    count: time_counters_on_self(),
                    filter: R::Nonland.and(R::Permanent).and(R::ManaValueAtMost(3)),
                    then: None,
                    max: Some(1),
                    tapped: false,
                    exile_rest: false,
                    rest_to_graveyard: false,
                },
            ]),
        )],
        ..legend(creature(
            "Wilfred Mott",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            4,
        ))
    }
}

/// Coward // Killer — Coward: target creature can't block and becomes a
/// Coward this turn; time travel. Killer: 3 damage to target creature and
/// each other creature sharing a creature type with it.
pub fn coward_killer() -> CardDefinition {
    CardDefinition {
        name: "Coward // Killer",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
            Effect::AddCreatureTypes {
                what: Selector::Target(0),
                creature_types: vec![CreatureType::Coward],
                duration: Duration::EndOfTurn,
            },
            time_travel(),
        ]),
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(2), r(), r()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::DealDamageToTargetAndTypeSharers {
                    what: target_filtered(R::Creature),
                    amount: Value::Const(3),
                },
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Gallifrey Falls // No More — fuse. Gallifrey Falls: 4 damage to each
/// creature, exiling any that would die. No More: any number of target
/// creatures you control phase out.
pub fn gallifrey_falls_no_more() -> CardDefinition {
    CardDefinition {
        name: "Gallifrey Falls // No More",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn { what: Selector::EachPermanent(R::Creature) },
            Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::Const(4) },
        ]),
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(2), w()]),
                card_types: vec![CardType::Instant],
                effect: Effect::ApplyToTargets {
                    max_targets: 16,
                    min_targets: 0,
                    filter: yours(R::Creature),
                    effect: Box::new(Effect::PhaseOut { what: Selector::Target(0), until_source_leaves: false }),
                },
            },
            fuse: true,
            aftermath: false,
        })),
        ..Default::default()
    }
}
