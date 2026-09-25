//! Commander: the cards the **Doom Prevails** precon (MSC, Doctor Doom, King
//! of Latveria) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_doom.rs`.
//!
//! Residuals (each also on its card):
//! - **Kang Dynasty** — the draw rider reads "goaded creatures your opponents
//!   control dealing combat damage off your turn", so a creature someone
//!   else goaded draws you a card too.
//! - **Lady Loki, Agent of Chaos** — "your first … spell each turn" counts
//!   from when it is on the battlefield (`once_per_turn`).
//! - **Superior Foes of Spider-Man** — the exiled card is playable this
//!   turn, not "until you exile another card with this".
//! - **Extract Power** — the exiled cards are face up.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{etb, melee, mint_treasures, on_attack, on_you_attack, target_filtered, unearth};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, b, cost, generic, r, u};
use crate::sets::{tap_add_any_color, tap_add_colorless};
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

fn artifact_legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Artifact, CardType::Creature], ..legend(name, mana, types, p, t) }
}

fn villain() -> R {
    R::HasCreatureType(CreatureType::Villain)
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn make(token: TokenDefinition, n: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: Arc::new(token) }
}

fn connive_self() -> Effect {
    Effect::Connive { what: Selector::This, amount: Value::ONE }
}

/// "Whenever a creature you control connives" (CR 701.50).
fn on_connive(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::Connived, EventScope::YourControl), effect }
}

/// "Whenever you draw your second card each turn" (CR 121).
fn second_draw(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::SecondCardDrawnThisTurn, EventScope::YourControl), effect }
}

fn villain_cost_reduction() -> StaticAbility {
    StaticAbility {
        description: "Villain spells you cast cost {1} less to cast.",
        effect: StaticEffect::CostReduction { filter: villain(), amount: 1 },
    }
}

/// A 2/1 black Villain with menace (HYDRA's rank and file).
fn hydra_token() -> TokenDefinition {
    TokenDefinition {
        name: "Villain".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Villain], ..Default::default() },
        keywords: vec![Keyword::Menace],
        ..Default::default()
    }
}

/// A 2/2 colorless Robot Villain artifact creature.
fn robot_token() -> TokenDefinition {
    TokenDefinition {
        name: "Robot Villain".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot, CreatureType::Villain], ..Default::default() },
        ..Default::default()
    }
}

/// "For each opponent, [verb] up to one target [filter] that player
/// controls" (CR 601.2c).
fn per_opponent(filter: R, effect: Effect) -> Effect {
    Effect::ForEachOpponentTarget {
        body: Box::new(Effect::ApplyToTargets {
            max_targets: 8,
            min_targets: 0,
            filter: filter.and(R::ControlledByOpponent),
            effect: Box::new(effect),
        }),
    }
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

/// Abomination, World Ravager — menace, trample; mayhem {4}{R}.
pub fn abomination_world_ravager() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Trample, Keyword::Mayhem(cost(&[generic(4), r()]))],
        ..legend(
            "Abomination, World Ravager",
            cost(&[generic(7), r()]),
            vec![CreatureType::Gamma, CreatureType::Berserker, CreatureType::Villain],
            10,
            10,
        )
    }
}

/// Age of Ultron — I: for each opponent, destroy up to one target
/// nonartifact creature they control. II: a 2/2 Robot Villain per opponent.
/// III: your artifact creatures gain deathtouch and a +1/+1 counter.
pub fn age_of_ultron() -> CardDefinition {
    let bots = || Selector::EachPermanent(yours(R::Artifact.and(R::Creature)));
    saga(
        "Age of Ultron",
        cost(&[generic(4), b()]),
        vec![
            (1, per_opponent(R::Creature.and(R::Artifact.negate()), Effect::Destroy { what: Selector::Target(0) })),
            (2, make(robot_token(), Value::OpponentCount)),
            (
                3,
                Effect::Seq(vec![
                    Effect::GrantKeyword { what: bots(), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
                    Effect::AddCounter { what: bots(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                ]),
            ),
        ],
    )
}

/// Archnemesis — enchant opponent; attacking them drains 2 and draws you a
/// card; a player attacking you may take it.
pub fn archnemesis() -> CardDefinition {
    CardDefinition {
        name: "Archnemesis",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::OpponentPlayer) },
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(
                    Predicate::AttackedDefenderWithCountAtLeast {
                        who: PlayerRef::You,
                        defender: PlayerRef::EnchantedPlayer,
                        at_least: 1,
                        include_planeswalkers: false,
                    },
                ),
                effect: Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::Player(PlayerRef::EnchantedPlayer), amount: Value::Const(2) },
                    draw(1),
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer).with_filter(
                    Predicate::AttackedDefenderWithCountAtLeast {
                        who: PlayerRef::ActivePlayer,
                        defender: PlayerRef::You,
                        at_least: 1,
                        include_planeswalkers: false,
                    },
                ),
                effect: Effect::MayDo {
                    description: "Attach Archnemesis to the attacking player?".into(),
                    body: Box::new(Effect::Attach {
                        what: Selector::This,
                        to: Selector::Player(PlayerRef::ActivePlayer),
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Baron Strucker, HYDRA Overlord — Villain spells cost {1} less; another
/// Villain of yours entering may connive, once each turn.
pub fn baron_strucker_hydra_overlord() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![villain_cost_reduction()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: villain() })
                .once_per_turn(),
            effect: Effect::MayDo {
                description: "Have it connive?".into(),
                body: Box::new(Effect::Connive { what: Selector::TriggerSource, amount: Value::ONE }),
            },
        }],
        ..legend(
            "Baron Strucker, HYDRA Overlord",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Villain],
            2,
            2,
        )
    }
}

/// Batroc the Leaper — multikicker {2}; a +1/+1 counter per kick, then
/// damage equal to his power to each of up to that many targets.
pub fn batroc_the_leaper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Multikicker(cost(&[generic(2)]))],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::TimesKicked)),
        triggered_abilities: vec![etb(Effect::CapTargetsAt {
            amount: Value::TimesKicked,
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Creature.or(R::Player).or(R::Planeswalker),
                effect: Box::new(Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                }),
            }),
        })],
        ..legend("Batroc the Leaper", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Villain], 2, 2)
    }
}

/// Damocles Base, Sword of Kang — flying, deathtouch Vehicle (crew 3); its
/// combat damage gives that player a villainous choice: sacrifice a nontoken
/// creature, or lose 2 life while you draw two.
pub fn damocles_base_sword_of_kang() -> CardDefinition {
    CardDefinition {
        name: "Damocles Base, Sword of Kang",
        cost: cost(&[generic(4), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch, Keyword::Crew(3)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            // The options run with the choosing player as "you" (CR 701.55).
            effect: Effect::VillainousChoice {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                option_a: Box::new(Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Creature.and(R::NotToken),
                }),
                option_b: Box::new(Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
                    Effect::Draw {
                        who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::This))),
                        amount: Value::Const(2),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Doctor Doom, King of Latveria — discarding land cards drains each
/// opponent 2; at the beginning of combat on your turn, a target Villain of
/// yours gains menace and connives.
pub fn doctor_doom_king_of_latveria() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land })
                    .once_per_batch(),
                effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_filtered(yours(villain())),
                        keyword: Keyword::Menace,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Connive { what: Selector::Target(0), amount: Value::ONE },
                ]),
            },
        ],
        ..legend(
            "Doctor Doom, King of Latveria",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Noble, CreatureType::Villain],
            3,
            3,
        )
    }
}

/// Doom's Time Platform — whenever you attack, a target nonland card from
/// your graveyard is exiled with two time counters and gains suspend.
pub fn dooms_time_platform() -> CardDefinition {
    CardDefinition {
        name: "Doom's Time Platform",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![on_you_attack(Effect::GrantSuspend {
            what: target_filtered(R::Nonland.from_your_graveyard()),
            time_counters: 2,
        })],
        ..Default::default()
    }
}

/// Endless Ranks of HYDRA — a 2/1 menace Villain per opponent; your
/// commander entering or attacking may buy it back from your graveyard for
/// {1}{B}.
pub fn endless_ranks_of_hydra() -> CardDefinition {
    let buy_back = || Effect::MayPay {
        description: "Pay {1}{B} to return Endless Ranks of HYDRA to your hand?".into(),
        mana_cost: cost(&[generic(1), b()]),
        body: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) }),
        else_: None,
    };
    let commander = || R::IsCommander.and(R::ControlledByYou);
    CardDefinition {
        name: "Endless Ranks of HYDRA",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: make(hydra_token(), Value::OpponentCount),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::FromYourGraveyard)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: commander() }),
                effect: buy_back(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::FromYourGraveyard).with_filter(
                    Predicate::AttackedWithCreatureMatching { who: PlayerRef::You, filter: commander() },
                ),
                effect: buy_back(),
            },
        ],
        ..Default::default()
    }
}

/// Extract Power — exile the top card of each library; you may play them
/// free while they stay exiled. Residual: they're exiled face up.
pub fn extract_power() -> CardDefinition {
    CardDefinition {
        name: "Extract Power",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::TopOfLibrary { who: PlayerRef::EachPlayer, count: Value::ONE },
                to: ZoneDest::Exile,
            },
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: MayPlayDuration::WhileExiled,
                to_owner: false,
                exile_after: false,
                pay_own_cost: false,
                any_color: false,
            },
        ]),
        ..Default::default()
    }
}

/// Glorious Purpose — a creature of yours conniving gets a +1/+1 counter and
/// adds a plan counter here; the sixth sacrifices it to cast any of the top
/// four free, the rest to hand.
pub fn glorious_purpose() -> CardDefinition {
    CardDefinition {
        name: "Glorious Purpose",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Plan], ..Default::default() },
        triggered_abilities: vec![
            on_connive(Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::AddCounter { what: Selector::This, kind: CounterType::Plan, amount: Value::ONE },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CounterAdded(CounterType::Plan), EventScope::SelfSource)
                    .with_filter(Predicate::SourceHasCountersAtLeast { counter: CounterType::Plan, n: 6 }),
                effect: Effect::Seq(vec![
                    Effect::SacrificeSource,
                    Effect::ExileLinked {
                        what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::Const(4) },
                    },
                    Effect::CastAnyOrderWithoutPaying {
                        what: Selector::CardExiledWithSource,
                        source_zone: Zone::Exile,
                        filter: None,
                        cap: None,
                        total_mana_value: None,
                    },
                    Effect::Move { what: Selector::CardExiledWithSource, to: ZoneDest::Hand(PlayerRef::You) },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Helmut Zemo, Mastermind — attacking, you may cast a target instant or
/// sorcery card with mana value up to his power from your graveyard (exiled
/// after); if you do, a +1/+1 counter on him.
pub fn helmut_zemo_mastermind() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::CastWithoutPayingImmediate {
                what: target_filtered(
                    R::HasCardType(CardType::Instant)
                        .or(R::HasCardType(CardType::Sorcery))
                        .and(R::ManaValueAtMostSourcePower)
                        .from_your_graveyard(),
                ),
                source_zone: Zone::Graveyard,
                exile_after: true,
                copy: false,
                reduce_generic: 0,
                pay_own_cost: true,
            },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::IsSpellOnStack },
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..legend(
            "Helmut Zemo, Mastermind",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Noble, CreatureType::Villain],
            2,
            2,
        )
    }
}

/// Iron Monger, Sadistic Tycoon — flying; a creature of yours conniving puts
/// a +1/+1 counter on each Villain you control.
pub fn iron_monger_sadistic_tycoon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_connive(Effect::AddCounter {
            what: Selector::EachPermanent(yours(villain())),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..artifact_legend(
            "Iron Monger, Sadistic Tycoon",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Villain],
            2,
            2,
        )
    }
}

/// Kang Dynasty — I, II: for each opponent, tap and goad up to one target
/// creature they control; those creatures' combat damage draws you a card
/// until your next turn. III: target creature of yours gets +1/+1 per card
/// in your hand and can't be blocked. Residual: the draw rider reads any
/// goaded creature an opponent controls, off your turn.
pub fn kang_dynasty() -> CardDefinition {
    let goad = || {
        per_opponent(
            R::Creature,
            Effect::Seq(vec![Effect::Tap { what: Selector::Target(0) }, Effect::Goad { what: Selector::Target(0) }]),
        )
    };
    let hand = || Value::HandSizeOf(PlayerRef::You);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer)
                .dealt_by(R::IsGoaded.and(R::ControlledByOpponent))
                .with_filter(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
            effect: draw(1),
        }],
        ..saga(
            "Kang Dynasty",
            cost(&[generic(3), u()]),
            vec![
                (1, goad()),
                (2, goad()),
                (
                    3,
                    Effect::Seq(vec![
                        Effect::PumpPT {
                            what: target_filtered(yours(R::Creature)),
                            power: hand(),
                            toughness: hand(),
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword {
                            what: Selector::Target(0),
                            keyword: Keyword::Unblockable,
                            duration: Duration::EndOfTurn,
                        },
                    ]),
                ),
            ],
        )
    }
}

/// Kang Prime — flying; entering or attacking, exile from the top until a
/// nonland card, which gets two time counters and suspend.
pub fn kang_prime() -> CardDefinition {
    let rift = || {
        Effect::Seq(vec![
            Effect::ExileTopUntilNonland { who: PlayerRef::You },
            Effect::GrantSuspend { what: Selector::ExiledThisResolution { filter: R::Nonland }, time_counters: 2 },
        ])
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(rift()), on_attack(rift())],
        ..legend("Kang Prime", cost(&[generic(3), u(), b()]), vec![CreatureType::Human, CreatureType::Villain], 3, 5)
    }
}

/// Kang, Temporal Tyrant — attacking, he connives; your second card drawn
/// each turn drains each opponent 1.
pub fn kang_temporal_tyrant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(connive_self()),
            second_draw(Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::ONE,
            }),
        ],
        ..legend(
            "Kang, Temporal Tyrant",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Human, CreatureType::Villain],
            3,
            4,
        )
    }
}

/// Killmonger, Ruthless Usurper — trample; attacking, +1/+0 per artifact the
/// defending player controls; his combat damage makes that player sacrifice
/// an artifact and you a Treasure.
pub fn killmonger_ruthless_usurper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            on_attack(Effect::PumpPT {
                what: Selector::This,
                power: Value::count(Selector::EachPermanent(R::Artifact.and(R::ControlledByDefendingPlayer))),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::TriggerEventPlayer),
                        count: Value::ONE,
                        filter: R::Artifact,
                    },
                    mint_treasures(1),
                ]),
            },
        ],
        ..legend(
            "Killmonger, Ruthless Usurper",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Villain],
            3,
            3,
        )
    }
}

/// Klaw, Master of Sound — deathtouch; playing a card from exile makes him
/// indestructible this turn; his combat damage exiles that player's top
/// card, which you may play with any mana while it stays exiled.
pub fn klaw_master_of_sound() -> CardDefinition {
    let indestructible = || Effect::GrantKeyword {
        what: Selector::This,
        keyword: Keyword::Indestructible,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellFromExile),
                effect: indestructible(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::EnteredFromExileThisTurn },
                ),
                effect: indestructible(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::LookTopExileOneFaceDownMayPlay {
                    who: PlayerRef::TriggerEventPlayer,
                    count: Value::ONE,
                    rest_to_graveyard: false,
                },
            },
        ],
        ..legend(
            "Klaw, Master of Sound",
            cost(&[generic(2), b()]),
            vec![CreatureType::Elemental, CreatureType::Rogue, CreatureType::Villain],
            3,
            3,
        )
    }
}

/// Lady Loki, Agent of Chaos — your first instant, sorcery or Villain spell
/// each turn is exiled; exile from the top until a nonland card, deal each
/// opponent the mana-value difference, and you may cast that card free.
/// Residual: "first each turn" counts from when she is in play.
pub fn lady_loki_agent_of_chaos() -> CardDefinition {
    let spell_mv = || Value::ManaValueOf(Box::new(Selector::TriggerSource));
    let found = || Value::LastExiledManaValue;
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).or(villain()),
                })
                .once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::ExileTopUntilNonland { who: PlayerRef::You },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Max(
                        Box::new(Value::Diff(Box::new(spell_mv()), Box::new(found()))),
                        Box::new(Value::Diff(Box::new(found()), Box::new(spell_mv()))),
                    ),
                },
                Effect::CastWithoutPayingImmediate {
                    what: Selector::ExiledThisResolution { filter: R::Nonland },
                    source_zone: Zone::Exile,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                },
                Effect::ExileSpellLinked { what: Selector::TriggerSource },
            ]),
        }],
        ..legend(
            "Lady Loki, Agent of Chaos",
            cost(&[generic(5), r()]),
            vec![CreatureType::God, CreatureType::Sorcerer, CreatureType::Villain],
            5,
            5,
        )
    }
}

/// Living Laser — haste; attacking, a nonlegendary token copy tapped and
/// attacking per card you discarded this turn, exiled at the next end step.
pub fn living_laser() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::CardsDiscardedThisTurn(PlayerRef::You),
                source: Selector::This,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: true,
                non_legendary: true,
                legendary: false,
                extra_keywords: vec![],
            },
            Effect::JoinCombatAttacking { what: Selector::LastCreatedTokens },
            Effect::ExileLastCreatedTokensAtNextEndStep,
        ]))],
        ..legend("Living Laser", cost(&[generic(4), r()]), vec![CreatureType::Elemental, CreatureType::Villain], 4, 4)
    }
}

/// Loki's Scepter — ETB: gain control of target creature until end of turn,
/// untap it, it becomes a Villain and gains haste; {T}: any color.
pub fn lokis_scepter() -> CardDefinition {
    CardDefinition {
        name: "Loki's Scepter",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainControl { what: target_filtered(R::Creature), to: None, duration: Duration::EndOfTurn },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::AddCreatureTypes {
                what: Selector::Target(0),
                creature_types: vec![CreatureType::Villain],
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
        ]))],
        activated_abilities: vec![tap_add_any_color()],
        ..Default::default()
    }
}

/// Loki, the Deceiver — attacking, a tapped and attacking nonlegendary
/// Illusion token copy of another target Villain of yours, sacrificed at the
/// next end step; your Villains' combat damage to a player draws you a card.
pub fn loki_the_deceiver() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: target_filtered(yours(villain()).and(R::OtherThanSource)),
                    extra_creature_types: vec![CreatureType::Illusion],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: true,
                    non_legendary: true,
                    legendary: false,
                    extra_keywords: vec![],
                },
                Effect::JoinCombatAttacking { what: Selector::LastCreatedTokens },
                Effect::SacrificeLastCreatedTokensAtNextEndStep,
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .dealt_by(villain())
                    .once_per_batch(),
                effect: draw(1),
            },
        ],
        ..legend(
            "Loki, the Deceiver",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::God, CreatureType::Sorcerer, CreatureType::Villain],
            4,
            4,
        )
    }
}

/// Madame Hydra — each Villain spell you cast makes a 2/1 menace Villain.
pub fn madame_hydra() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: villain() }),
            effect: make(hydra_token(), Value::ONE),
        }],
        ..legend("Madame Hydra", cost(&[generic(2), b(), r()]), vec![CreatureType::Human, CreatureType::Villain], 2, 3)
    }
}

/// Molecule Man — nonland cards in your hand have miracle {0}: your first
/// draw each turn, if nonland, may be cast free.
pub fn molecule_man() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Nonland },
                Predicate::ValueEquals(Value::CardsDrawnThisTurn(PlayerRef::You), Value::ONE),
            ])),
            effect: Effect::GrantMiracle { what: Selector::TriggerSource, cost: ManaCost::default() },
        }],
        ..legend("Molecule Man", cost(&[generic(6)]), vec![CreatureType::Human, CreatureType::Villain], 5, 5)
    }
}

/// Moonstone, Harsh Mistress — flying; whenever you discard a card, you may
/// exile it to play it through the end of your next turn.
pub fn moonstone_harsh_mistress() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Exile the discarded card to play it until the end of your next turn?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile },
                    Effect::GrantMayPlay {
                        what: Selector::TriggerSource,
                        duration: MayPlayDuration::EndOfControllersNextTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    },
                ])),
            },
        }],
        ..legend(
            "Moonstone, Harsh Mistress",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Doctor, CreatureType::Villain],
            2,
            4,
        )
    }
}

/// Prowler, Clawed Thief — menace; another Villain of yours entering makes
/// it connive.
pub fn prowler_clawed_thief() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: villain() }),
            effect: connive_self(),
        }],
        ..legend(
            "Prowler, Clawed Thief",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Villain],
            2,
            3,
        )
    }
}

/// Puppet Master, String Puller — whenever you attack, goad target creature
/// an opponent controls and it can't block this turn; goaded creatures
/// connecting with your opponents make you a Treasure.
pub fn puppet_master_string_puller() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_you_attack(Effect::Seq(vec![
                Effect::Goad { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::CantBlock, duration: Duration::EndOfTurn },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer)
                    .dealt_by(R::IsGoaded)
                    .with_filter(Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer })
                    .once_per_batch(),
                effect: mint_treasures(1),
            },
        ],
        ..legend(
            "Puppet Master, String Puller",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Artificer, CreatureType::Villain],
            2,
            4,
        )
    }
}

/// Red Ghost, Intangible Genius — ward {2}, can't be blocked; your second
/// draw each turn makes a 3/3 hasty red Ape Villain.
pub fn red_ghost_intangible_genius() -> CardDefinition {
    let ape = TokenDefinition {
        name: "Ape Villain".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ape, CreatureType::Villain], ..Default::default() },
        keywords: vec![Keyword::Haste],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::generic(2)), Keyword::Unblockable],
        triggered_abilities: vec![second_draw(make(ape, Value::ONE))],
        ..legend(
            "Red Ghost, Intangible Genius",
            cost(&[generic(2), u(), r()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Villain],
            2,
            3,
        )
    }
}

/// Stilt-Man, Towering Terror — reach; your Villains' combat damage to a
/// player steals a target noncreature, nonland permanent of theirs until the
/// end of your next turn, and it can't be sacrificed meanwhile.
pub fn stilt_man_towering_terror() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .dealt_by(villain())
                .once_per_batch(),
            effect: Effect::Seq(vec![
                Effect::GainControl {
                    what: target_filtered(
                        R::Permanent.and(R::Noncreature).and(R::Nonland).and(R::ControlledByTriggerPlayer),
                    ),
                    to: None,
                    duration: Duration::UntilEndOfYourNextTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantBeSacrificed,
                    duration: Duration::UntilEndOfYourNextTurn,
                },
            ]),
        }],
        ..legend(
            "Stilt-Man, Towering Terror",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Villain],
            4,
            2,
        )
    }
}

/// Superior Foes of Spider-Man — trample; casting a spell of mana value 4+
/// may exile your top card to play. Residual: playable this turn only.
pub fn superior_foes_of_spider_man() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::ManaValueAtLeast(4) },
            ),
            effect: Effect::MayDo {
                description: "Exile the top card of your library to play it?".into(),
                body: Box::new(Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                }),
            },
        }],
        ..creature(
            "Superior Foes of Spider-Man",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Villain],
            3,
            3,
        )
    }
}

/// The Frightful Four — menace; an opponent's first noncreature spell each
/// turn costs them life equal to its mana value.
pub fn the_frightful_four() -> CardDefinition {
    let caster = || PlayerRef::ControllerOf(Box::new(Selector::TriggerSource));
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Noncreature },
                Predicate::ValueEquals(Value::NoncreatureSpellsCastThisTurn(caster()), Value::ONE),
            ])),
            effect: Effect::LoseLife {
                who: Selector::Player(caster()),
                amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..legend("The Frightful Four", cost(&[generic(3), b()]), vec![CreatureType::Villain], 4, 4)
    }
}

/// The Squadron Sinister — flying, haste; other Villains you control get
/// +2/+2 with flying and haste; mayhem {3}{U}{R}.
pub fn the_squadron_sinister() -> CardDefinition {
    let others = || Selector::EachPermanent(yours(villain().and(R::Creature)).and(R::OtherThanSource));
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::Mayhem(cost(&[generic(3), u(), r()]))],
        static_abilities: vec![
            StaticAbility {
                description: "Other Villains you control get +2/+2.",
                effect: StaticEffect::PumpPT { applies_to: others(), power: 2, toughness: 2 },
            },
            StaticAbility {
                description: "Other Villains you control have flying.",
                effect: StaticEffect::GrantKeyword { applies_to: others(), keyword: Keyword::Flying },
            },
            StaticAbility {
                description: "Other Villains you control have haste.",
                effect: StaticEffect::GrantKeyword { applies_to: others(), keyword: Keyword::Haste },
            },
        ],
        ..legend(
            "The Squadron Sinister",
            cost(&[generic(5), u(), r()]),
            vec![CreatureType::Human, CreatureType::Villain],
            5,
            5,
        )
    }
}

/// Titania, Proud Pummeler — first strike; melee; other creatures you
/// control have melee.
pub fn titania_proud_pummeler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![melee()],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have melee.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: yours(R::Creature).and(R::OtherThanSource),
                ability: Box::new(melee()),
            },
        }],
        ..legend(
            "Titania, Proud Pummeler",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Villain],
            3,
            3,
        )
    }
}

/// Tombstone, Career Criminal — ETB return target Villain card from your
/// graveyard to hand; Villain spells cost {1} less.
pub fn tombstone_career_criminal() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![villain_cost_reduction()],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(villain().from_your_graveyard()),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..legend(
            "Tombstone, Career Criminal",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Villain],
            2,
            2,
        )
    }
}

/// Tri-Sentinel, Act of Vengeance — flying, menace, trample; ETB, for each
/// opponent, 3 damage to up to one target creature they control; unearth
/// {7}.
pub fn tri_sentinel_act_of_vengeance() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Menace, Keyword::Trample],
        triggered_abilities: vec![etb(per_opponent(
            R::Creature,
            Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(3) },
        ))],
        activated_abilities: vec![unearth(cost(&[generic(7)]))],
        ..artifact_legend(
            "Tri-Sentinel, Act of Vengeance",
            cost(&[generic(7)]),
            vec![CreatureType::Robot, CreatureType::Villain],
            7,
            7,
        )
    }
}

/// Typhoid Mary, Fractured — attacking, one mode at random (your pick if you
/// discarded this turn): a Treasure; draw a card; drain each opponent 2.
pub fn typhoid_mary_fractured() -> CardDefinition {
    let modes = || {
        vec![
            mint_treasures(1),
            draw(1),
            Effect::Drain { from: Selector::Player(PlayerRef::EachOpponent), to: Selector::You, amount: Value::Const(2) },
        ]
    };
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::If {
            cond: Predicate::DiscardedThisTurn { who: PlayerRef::You },
            then: Box::new(Effect::ChooseMode(modes())),
            else_: Box::new(Effect::ChooseModeAtRandom(modes())),
        })],
        ..legend(
            "Typhoid Mary, Fractured",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Mutant, CreatureType::Villain],
            3,
            3,
        )
    }
}

/// Ultron, Unlimited — flying; attacking, he connives; a creature of yours
/// conniving may pay {1} for a 2/2 Robot Villain.
pub fn ultron_unlimited() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            on_attack(connive_self()),
            on_connive(Effect::MayPay {
                description: "Pay {1} for a 2/2 Robot Villain?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(make(robot_token(), Value::ONE)),
                else_: None,
            }),
        ],
        ..artifact_legend(
            "Ultron, Unlimited",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Robot, CreatureType::Villain],
            2,
            2,
        )
    }
}

/// Villainous Hideout — {T}: {C}; {T}: any color for Villain spells and
/// Villain sources' abilities; {3},{T}: target Villain you control connives,
/// sorcery speed.
pub fn villainous_hideout() -> CardDefinition {
    CardDefinition {
        name: "Villainous Hideout",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                        SpendRestriction::CreatureOfTypeOrItsAbility(CreatureType::Villain),
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                sorcery_speed: true,
                effect: Effect::Connive { what: target_filtered(yours(villain())), amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
