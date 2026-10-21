//! Commander: the single cards that stood between an EDHREC average deck and
//! a complete pod seat, seventh file (`cmdr_edhrec6` is the sixth) — the
//! one-card gaps of EDHREC's top-600 commanders. Tests in
//! `tests/recent_b/cmdr_edhrec7.rs`.

use crate::card::{
    Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, target_filtered};
use crate::effect::{
    ActivatedAbility, Duration, Effect, ExiledPlaySpend, LibraryPosition, ManaPayload, PlayerRef, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, x, Color, ManaCost};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn token(name: &str, p: i32, t: i32, color: Color, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors: vec![color],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn make(n: i32, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(def) }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn at_your(step: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl), effect }
}

/// Midnight Guard — {2}{W} 2/3 Human Soldier. Another creature entering
/// untaps it.
pub fn midnight_guard() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::OtherThanSource),
            }),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        ..creature("Midnight Guard", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 3)
    }
}

/// Skithiryx, the Blight Dragon — {3}{B}{B} 4/4 flying, infect; {B}: haste;
/// {B}{B}: regenerate.
pub fn skithiryx_the_blight_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Infect],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b(), b()]),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            },
        ],
        ..legend(
            "Skithiryx, the Blight Dragon",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Dragon, CreatureType::Skeleton],
            4,
            4,
        )
    }
}

/// Wyrm's Crossing Patrol — {W} 1/1 Human Soldier with myriad (CR 702.116).
pub fn wyrms_crossing_patrol() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Myriad,
        }],
        ..creature("Wyrm's Crossing Patrol", cost(&[w()]), vec![CreatureType::Human, CreatureType::Soldier], 1, 1)
    }
}

/// Gravelighter — {2}{B} 2/2 flying Spirit. Entering: a card if a creature
/// died this turn, otherwise each player sacrifices a creature.
pub fn gravelighter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE },
            then: Box::new(draw(1)),
            else_: Box::new(Effect::Sacrifice { who: Selector::Player(PlayerRef::EachPlayer), count: Value::ONE, filter: R::Creature }),
        })],
        ..creature("Gravelighter", cost(&[generic(2), b()]), vec![CreatureType::Spirit], 2, 2)
    }
}

/// Fateful Showdown — {2}{R}{R} Instant. Damage to any target equal to your
/// hand size, then discard your hand and draw that many.
pub fn fateful_showdown() -> CardDefinition {
    spell(
        "Fateful Showdown",
        cost(&[generic(2), r(), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::HandSizeOf(PlayerRef::You) },
            Effect::DiscardHandDrawThatMany { who: Selector::You },
        ]),
    )
}

/// Denethor, Ruling Steward — {1}{W}{B} 2/4 legendary Human Noble. Your end
/// step after one of your creatures died makes a 1/1 Human Soldier; {2},
/// sacrifice another creature: drain each opponent for 1.
pub fn denethor_ruling_steward() -> CardDefinition {
    let died = || Predicate::CreaturesDiedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(died()),
            effect: Effect::If {
                cond: died(),
                then: Box::new(make(1, token("Human Soldier", 1, 1, Color::White, vec![CreatureType::Human, CreatureType::Soldier]))),
                else_: Box::new(Effect::Noop),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: crate::effect::shortcut::drain(1),
            ..Default::default()
        }],
        ..legend("Denethor, Ruling Steward", cost(&[generic(1), w(), b()]), vec![CreatureType::Human, CreatureType::Noble], 2, 4)
    }
}

/// Alrund's Epiphany — {5}{U}{U} Sorcery, foretell {4}{U}{U}. Two 1/1 flying
/// Birds and an extra turn; it exiles itself.
pub fn alrunds_epiphany() -> CardDefinition {
    let bird = TokenDefinition { keywords: vec![Keyword::Flying], ..token("Bird", 1, 1, Color::Blue, vec![CreatureType::Bird]) };
    CardDefinition {
        foretell_cost: Some(cost(&[generic(4), u(), u()])),
        exile_on_resolve: true,
        ..spell(
            "Alrund's Epiphany",
            cost(&[generic(5), u(), u()]),
            CardType::Sorcery,
            Effect::Seq(vec![make(2, bird), Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE }]),
        )
    }
}

/// Raiders' Wake — {3}{B} Enchantment. An opponent discarding loses 2 life;
/// raid — your end step after attacking, a target opponent discards.
pub fn raiders_wake() -> CardDefinition {
    let raided = || Predicate::PlayerAttackedThisTurn { who: PlayerRef::You };
    CardDefinition {
        name: "Raiders' Wake",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::OpponentControl),
                effect: Effect::LoseLife { who: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(2) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(raided()),
                effect: Effect::If {
                    cond: raided(),
                    then: Box::new(Effect::Discard {
                        who: target_filtered(R::OpponentPlayer),
                        amount: Value::ONE,
                        random: false,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Tamiyo's Journal — {5} legendary Artifact — Book. Your upkeep investigates;
/// {T}, sacrifice three Clues: tutor any card to hand.
pub fn tamiyos_journal() -> CardDefinition {
    CardDefinition {
        name: "Tamiyo's Journal",
        cost: cost(&[generic(5)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![crate::card::ArtifactSubtype::Book], ..Default::default() },
        triggered_abilities: vec![at_your(TurnStep::Upkeep, crate::effect::shortcut::investigate(1))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::HasArtifactSubtype(crate::card::ArtifactSubtype::Clue), 3)),
            effect: Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tippy-Toe, Terrific Partner — {3}{G} 2/4 Squirrel Hero. Each of your
/// token creations brings an additional Food; your end step draws a card if
/// you gained life this turn.
pub fn tippy_toe_terrific_partner() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "If you would create one or more tokens, instead create those tokens plus an additional Food token.",
            StaticEffect::TokenCreationAddsToken { definition: crabomination_base::tokens::food_token() },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::ValueAtLeast(Value::LifeGainedThisTurn(PlayerRef::You), Value::ONE)),
            effect: draw(1),
        }],
        ..legend("Tippy-Toe, Terrific Partner", cost(&[generic(3), g()]), vec![CreatureType::Squirrel, CreatureType::Hero], 2, 4)
    }
}

/// Enchanted Evening — {3}{W/U}{W/U} Enchantment. All permanents are
/// enchantments in addition to their other types (layer 4).
pub fn enchanted_evening() -> CardDefinition {
    CardDefinition {
        name: "Enchanted Evening",
        cost: cost(&[generic(3), hybrid(Color::White, Color::Blue), hybrid(Color::White, Color::Blue)]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![static_ab(
            "All permanents are enchantments in addition to their other types.",
            StaticEffect::AddCardTypeToMatching {
                applies_to: Selector::EachPermanent(R::Permanent),
                card_type: CardType::Enchantment,
                artifact_subtype: None,
            },
        )],
        ..Default::default()
    }
}

/// Deranged Assistant — {1}{U} 1/1 Human Wizard. {T}, mill a card: add {C}.
pub fn deranged_assistant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mill_cost: 1,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
            ..Default::default()
        }],
        ..creature("Deranged Assistant", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 1)
    }
}

/// Flamekin Herald — {2}{R} 3/2 Elemental Wizard. Commander spells you cast
/// have cascade (CR 702.85a: the cast triggers, below the spell's own mana
/// value).
pub fn flamekin_herald() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::IsCommander,
            }),
            effect: Effect::Cascade { max_mv: Value::ManaValueOf(Box::new(Selector::TriggerSource)), filter: None },
        }],
        ..creature("Flamekin Herald", cost(&[generic(2), r()]), vec![CreatureType::Elemental, CreatureType::Wizard], 3, 2)
    }
}

/// Serra the Benevolent — {2}{W}{W} planeswalker, loyalty 4. +2 pumps your
/// fliers; −3 makes a 4/4 Angel; −6 is the Worship emblem.
pub fn serra_the_benevolent() -> CardDefinition {
    let angel = TokenDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..token("Angel", 4, 4, Color::White, vec![CreatureType::Angel])
    };
    CardDefinition {
        name: "Serra the Benevolent",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Serra], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::HasKeyword(Keyword::Flying))),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            LoyaltyAbility { loyalty_cost: -3, effect: make(1, angel), ..Default::default() },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Serra the Benevolent".into(),
                    triggered: vec![],
                    statics: vec![static_ab(
                        "If you control a creature, damage that would reduce your life total to less than 1 reduces it to 1 instead.",
                        StaticEffect::DamageWontReduceControllerLifeBelowOne { requires_creature: true },
                    )],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Greensleeves, Maro-Sorcerer — {3}{G}{G} */* Elemental Sorcerer.
/// Protection from planeswalkers and from Wizards; P/T = lands you control;
/// landfall makes a 3/3 Badger.
pub fn greensleeves_maro_sorcerer() -> CardDefinition {
    let lands = || Value::PermanentCountControlledByMatching(PlayerRef::You, R::Land);
    CardDefinition {
        keywords: vec![
            Keyword::ProtectionFromCardType(CardType::Planeswalker),
            Keyword::ProtectionFromCreatureType(CreatureType::Wizard),
        ],
        static_abilities: vec![static_ab(
            "Greensleeves's power and toughness are each equal to the number of lands you control.",
            StaticEffect::SelfBasePtFromValue { power: lands(), toughness: lands() },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: make(1, token("Badger", 3, 3, Color::Green, vec![CreatureType::Badger])),
        }],
        ..legend(
            "Greensleeves, Maro-Sorcerer",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Elemental, CreatureType::Sorcerer],
            0,
            0,
        )
    }
}

/// Nest of Scarabs — {2}{B} Enchantment. Putting -1/-1 counters on a
/// creature makes that many 1/1 Insects.
pub fn nest_of_scarabs() -> CardDefinition {
    CardDefinition {
        name: "Nest of Scarabs",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::MinusOneMinusOne), EventScope::YouPutCounters)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::TriggerEventAmount,
                definition: Arc::new(token("Insect", 1, 1, Color::Black, vec![CreatureType::Insect])),
            },
        }],
        ..Default::default()
    }
}

/// Astarion, the Decadent — {4}{W}{B} 4/4 deathtouch, lifelink. Your end
/// step: an opponent loses what they lost this turn, or you gain what you
/// gained.
pub fn astarion_the_decadent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        triggered_abilities: vec![at_your(
            TurnStep::End,
            Effect::ChooseMode(vec![
                Effect::LoseLife {
                    who: target_filtered(R::OpponentPlayer),
                    amount: Value::LifeLostThisTurn(PlayerRef::Target(0)),
                },
                Effect::GainLife { who: Selector::You, amount: Value::LifeGainedThisTurn(PlayerRef::You) },
            ]),
        )],
        ..legend(
            "Astarion, the Decadent",
            cost(&[generic(4), w(), b()]),
            vec![CreatureType::Vampire, CreatureType::Elf, CreatureType::Rogue],
            4,
            4,
        )
    }
}

/// Jaws, Relentless Predator — {3}{R}{R} 5/5 trample, haste Shark. Its
/// combat damage to a player makes that many Blood; a noncreature artifact
/// sacrificed or destroyed pings each opponent.
pub fn jaws_relentless_predator() -> CardDefinition {
    let ping = || Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE };
    let gone = |kind: EventKind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: R::Artifact.and(R::Noncreature),
        }),
        effect: ping(),
    };
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::TriggerEventAmount,
                    definition: Arc::new(crabomination_base::tokens::blood_token()),
                },
            },
            gone(EventKind::PermanentSacrificed),
            gone(EventKind::PermanentDestroyedByEffect),
        ],
        ..legend("Jaws, Relentless Predator", cost(&[generic(3), r(), r()]), vec![CreatureType::Shark], 5, 5)
    }
}

/// Decadent Dragon // Expensive Taste — {2}{R}{R} 4/4 flying, trample;
/// attacking makes a Treasure. Adventure {2}{B}: exile the top two cards of
/// an opponent's library face down; you may play them.
pub fn decadent_dragon() -> CardDefinition {
    let steal = || Effect::ExileTopFaceDownGrantPlay {
        library: PlayerRef::Target(0),
        grantee: PlayerRef::You,
        spend: ExiledPlaySpend::AnyType,
        cast_only: false,
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![on_attack(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(crabomination_base::tokens::treasure_token()),
        })],
        adventure: Some(Box::new(Adventure {
            name: "Expensive Taste",
            cost: cost(&[generic(2), b()]),
            card_types: vec![CardType::Instant],
            effect: Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::Seq(vec![steal(), steal()])),
            },
        })),
        ..creature("Decadent Dragon", cost(&[generic(2), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Twilight Diviner — {2}{B} 3/3 Elf Cleric. ETB surveil 2; once each turn,
/// another creature of yours entering from (or cast from) a graveyard is
/// copied.
pub fn twilight_diviner() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::All(vec![
                        Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                        Predicate::Any(vec![Predicate::TriggerSourceEnteredFromGraveyard, Predicate::CastFromGraveyard]),
                    ]))
                    .once_per_turn(),
                effect: copy_of(Selector::TriggerSource, false),
            },
        ],
        ..creature("Twilight Diviner", cost(&[generic(2), b()]), vec![CreatureType::Elf, CreatureType::Cleric], 3, 3)
    }
}

fn copy_of(source: Selector, non_legendary: bool) -> Effect {
    Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count: Value::ONE,
        source,
        extra_creature_types: vec![],
        extra_card_types: vec![],
        override_pt: None,
        override_colors: None,
        enters_tapped: false,
        non_legendary,
        legendary: false,
        extra_keywords: vec![],
        no_mana_cost: false,
        enters_with_counters: None,
        remove_keywords: vec![],
    }
}

/// Profane Transfusion — {6}{B}{B}{B} Sorcery. Two target players exchange
/// life totals; you make an X/X Horror, X the difference between them.
pub fn profane_transfusion() -> CardDefinition {
    let life = |slot| Value::LifeOf(PlayerRef::Target(slot));
    let gap = || {
        Value::Max(Box::new(Value::Diff(Box::new(life(0)), Box::new(life(1)))), Box::new(Value::Diff(Box::new(life(1)), Box::new(life(0)))))
    };
    let horror = TokenDefinition {
        name: "Horror".into(),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Horror], ..Default::default() },
        dynamic_pt: Some((gap(), gap())),
        ..Default::default()
    };
    spell(
        "Profane Transfusion",
        cost(&[generic(6), b(), b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::ExchangeLifeTotals {
                a: Selector::TargetFiltered { slot: 0, filter: R::Player },
                b: Selector::TargetFiltered { slot: 1, filter: R::Player },
            },
            make(1, horror),
        ]),
    )
}

/// Impostor Syndrome — {4}{U}{U} Enchantment. A nontoken creature of yours
/// dealing combat damage to a player is copied, not legendary.
pub fn impostor_syndrome() -> CardDefinition {
    CardDefinition {
        name: "Impostor Syndrome",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::IsToken.negate()) },
            ),
            effect: copy_of(Selector::TriggerSource, true),
        }],
        ..Default::default()
    }
}

/// Black Panther, Wakandan King — {G}{W} 2/2 first strike. A creature of
/// yours entering grows one of your lands; {3}: move a land's +1/+1 counters
/// onto a creature, gaining that much life and drawing.
pub fn black_panther_wakandan_king() -> CardDefinition {
    let land = || Selector::TargetFiltered { slot: 0, filter: R::Land.and(R::ControlledByYou) };
    let on_land = || Value::CountersOn { what: Box::new(Selector::Target(0)), kind: CounterType::PlusOnePlusOne };
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::AddCounter { what: land(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            // The life and the draw read the land's counters before they move
            // (the amount moved is what the land held).
            effect: Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::ValueAtLeast(on_land(), Value::ONE),
                    then: Box::new(Effect::Seq(vec![
                        Effect::GainLife { who: Selector::You, amount: on_land() },
                        draw(1),
                    ])),
                    else_: Box::new(Effect::Noop),
                },
                Effect::MoveCounters {
                    from: land(),
                    to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                    counter: CounterType::PlusOnePlusOne,
                    amount: on_land(),
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Black Panther, Wakandan King",
            cost(&[g(), w()]),
            vec![CreatureType::Human, CreatureType::Noble, CreatureType::Hero],
            2,
            2,
        )
    }
}

/// Spectrum Sentinel — {1} 1/2 artifact Soldier, protection from
/// multicolored. A nonbasic land entering under an opponent gains you 1.
pub fn spectrum_sentinel() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::ProtectionFromMulticolored],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsNonbasicLand }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..creature("Spectrum Sentinel", cost(&[generic(1)]), vec![CreatureType::Soldier], 1, 2)
    }
}

/// Botanical Brawler — {G}{W} 0/0 trample, enters with two +1/+1 counters.
/// The first +1/+1 counters on another permanent of yours each turn grow it.
pub fn botanical_brawler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource })
                .with_per_subject_cap(1),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature(
            "Botanical Brawler",
            cost(&[g(), w()]),
            vec![CreatureType::Elemental, CreatureType::Warrior],
            0,
            0,
        )
    }
}

/// Kirol, Attentive First-Year — {1}{R/W}{R/W} 3/3 Vampire Cleric. Tap two
/// creatures: copy a triggered ability you control, once each turn.
pub fn kirol_attentive_first_year() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::Creature, 2)),
            once_per_turn: true,
            effect: Effect::CopyAbility {
                what: target_filtered(R::HasTriggeredAbilityOnStack.and(R::ControlledByYou)),
                times: Value::ONE,
            },
            ..Default::default()
        }],
        ..legend(
            "Kirol, Attentive First-Year",
            cost(&[generic(1), hybrid(Color::Red, Color::White), hybrid(Color::Red, Color::White)]),
            vec![CreatureType::Vampire, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// The Ten Rings — {8} legendary artifact. Maximum hand size ten; your end
/// step draws you up to ten.
pub fn the_ten_rings() -> CardDefinition {
    let short = || Predicate::Not(Box::new(Predicate::ValueAtLeast(Value::HandSizeOf(PlayerRef::You), Value::Const(10))));
    CardDefinition {
        name: "The Ten Rings",
        cost: cost(&[generic(8)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![static_ab("Your maximum hand size is ten.", StaticEffect::ControllerMaxHandSize(10))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(short()),
            effect: Effect::If {
                cond: short(),
                then: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::Diff(Box::new(Value::Const(10)), Box::new(Value::HandSizeOf(PlayerRef::You))),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Oswald Fiddlebender — {1}{W} 2/2 Gnome Artificer. {W}, {T}, sacrifice an
/// artifact: tutor an artifact one mana value higher onto the battlefield.
pub fn oswald_fiddlebender() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            sorcery_speed: true,
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Artifact.and(R::ManaValueEqualsSacrificedPlus(1)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..legend("Oswald Fiddlebender", cost(&[generic(1), w()]), vec![CreatureType::Gnome, CreatureType::Artificer], 2, 2)
    }
}

/// Vow of Torment — {2}{B} Aura. +2/+2, menace, can't attack you or your
/// planeswalkers.
pub fn vow_of_torment() -> CardDefinition {
    CardDefinition {
        name: "Vow of Torment",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Menace, Keyword::CantAttackAuraController],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Toralf, God of Fury // Toralf's Hammer — {2}{R}{R} 5/4 trample God;
/// excess noncombat damage to an opponent's creature or planeswalker is
/// redirected as fresh damage to any other target. The back is a {1}{R}
/// Equipment granting "{1}{R}, {T}, unattach: 3 damage, the Hammer returns
/// to hand" and +3/+0 to a legendary bearer.
pub fn toralf_god_of_fury() -> CardDefinition {
    let hammer = CardDefinition {
        name: "Toralf's Hammer",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1), r()]))],
        equipped_bonus: Some(EquipBonus {
            conditional_pt: Some((
                3,
                0,
                Predicate::EntityMatches {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    filter: R::HasSupertype(Supertype::Legendary),
                },
            )),
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                tap_cost: true,
                unattach_cost: true,
                effect: Effect::Seq(vec![
                    Effect::DealDamage { to: target_filtered(R::Any), amount: Value::Const(3) },
                    Effect::Move {
                        what: Selector::AttachmentGranting,
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::AttachmentGranting))),
                    },
                ]),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtExcessDamage { noncombat_only: true }, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.or(R::Planeswalker) },
            ),
            effect: Effect::DealDamage {
                to: target_filtered(R::Any.and(R::OtherThanTriggerSubject)),
                amount: Value::TriggerEventAmount,
            },
        }],
        back_face: Some(Box::new(hammer)),
        ..legend("Toralf, God of Fury", cost(&[generic(2), r(), r()]), vec![CreatureType::God], 5, 4)
    }
}

/// Saheeli, the Sun's Brilliance — {U}{R} 2/2 Human Artificer. {U}{R}, {T}:
/// a hasty artifact copy of another creature or artifact you control,
/// sacrificed at the next end step.
pub fn saheeli_the_suns_brilliance() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), r()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: target_filtered(R::Creature.or(R::Artifact).and(R::ControlledByYou).and(R::OtherThanSource)),
                    extra_creature_types: vec![],
                    extra_card_types: vec![CardType::Artifact],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![Keyword::Haste],
                    no_mana_cost: false,
                    enters_with_counters: None,
                    remove_keywords: vec![],
                },
                Effect::SacrificeLastCreatedTokensAtNextEndStep,
            ]),
            ..Default::default()
        }],
        ..legend(
            "Saheeli, the Sun's Brilliance",
            cost(&[u(), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            2,
        )
    }
}

/// Avengers Assemble! — {4}{W} flash Enchantment. Heroes you control get
/// +2/+2; each end step draws you a card if a Hero of yours attacked or
/// entered this turn.
pub fn avengers_assemble() -> CardDefinition {
    let hero = || R::HasCreatureType(CreatureType::Hero);
    let assembled = || {
        Predicate::Any(vec![
            Predicate::AttackedWithCreatureMatching { who: PlayerRef::You, filter: hero() },
            Predicate::CreatureEnteredThisTurnMatching { who: PlayerRef::You, filter: hero() },
        ])
    };
    CardDefinition {
        name: "Avengers Assemble!",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        static_abilities: vec![static_ab(
            "Heroes you control get +2/+2.",
            StaticEffect::PumpPT { applies_to: Selector::EachPermanent(hero().and(R::ControlledByYou)), power: 2, toughness: 2 },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(assembled()),
            effect: Effect::If { cond: assembled(), then: Box::new(draw(1)), else_: Box::new(Effect::Noop) },
        }],
        ..Default::default()
    }
}

/// Kíli the Resourceful — {1}{W} 1/2 Dwarf Scout, storied. With an enduring
/// story the first equip each turn may cost {0}; another Dwarf or Equipment
/// of yours entering draws a card, once each turn.
pub fn kili_the_resourceful() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            static_ab(
                "Storied (If you control three or more artifacts, legendaries, and/or Sagas, you have an enduring story for the rest of the game.)",
                StaticEffect::Storied,
            ),
            static_ab(
                "As long as you have an enduring story, you may pay {0} rather than pay the equip cost of the first equip ability you activate each turn.",
                StaticEffect::EquipmentYouControlEquipZeroWhile {
                    condition: Predicate::All(vec![
                        Predicate::HasEnduringStory { who: PlayerRef::You },
                        Predicate::Not(Box::new(Predicate::ValueAtLeast(
                            Value::EquipsActivatedThisTurn(PlayerRef::You),
                            Value::ONE,
                        ))),
                    ]),
                },
            ),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Dwarf).or(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
                })
                .once_per_turn(),
            effect: draw(1),
        }],
        ..legend("Kíli the Resourceful", cost(&[generic(1), w()]), vec![CreatureType::Dwarf, CreatureType::Scout], 1, 2)
    }
}

/// Gixian Puppeteer — {3}{B} 4/3 Phyrexian Warlock. Your second draw each
/// turn drains each opponent for 2; dying returns another creature card
/// with mana value 3 or less from your graveyard.
pub fn gixian_puppeteer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ]),
            },
            on_dies(Effect::Move {
                what: target_filtered(
                    R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMost(3)).and(R::OtherThanSource),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        ],
        ..creature("Gixian Puppeteer", cost(&[generic(3), b()]), vec![CreatureType::Phyrexian, CreatureType::Warlock], 4, 3)
    }
}

/// Expel from Orazca — {1}{U} Instant, ascend. Bounce a nonland permanent;
/// with the city's blessing you may put it on top of its owner's library.
pub fn expel_from_orazca() -> CardDefinition {
    let it = || target_filtered(R::Permanent.and(R::Nonland));
    let owner = || PlayerRef::OwnerOf(Box::new(Selector::Target(0)));
    CardDefinition {
        static_abilities: vec![crate::sets::ascend()],
        ..spell(
            "Expel from Orazca",
            cost(&[generic(1), u()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Ascend { who: PlayerRef::You },
                Effect::If {
                    cond: Predicate::HasCityBlessing { who: PlayerRef::You },
                    then: Box::new(Effect::MayDoElse {
                        description: "Put it on top of its owner's library instead?".into(),
                        body: Box::new(Effect::Move {
                            what: it(),
                            to: ZoneDest::Library { who: owner(), pos: LibraryPosition::Top },
                        }),
                        else_: Box::new(Effect::Move { what: it(), to: ZoneDest::Hand(owner()) }),
                    }),
                    else_: Box::new(Effect::Move { what: it(), to: ZoneDest::Hand(owner()) }),
                },
            ]),
        )
    }
}

/// Death's Oasis — {W}{B}{G} Enchantment. A nontoken creature of yours dying
/// mills two, then returns a lesser creature card; {1}, sacrifice: gain life
/// equal to your greatest creature mana value.
pub fn deaths_oasis() -> CardDefinition {
    CardDefinition {
        name: "Death's Oasis",
        cost: cost(&[w(), b(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::IsToken.negate(),
            }),
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(2) },
                // The controller picks the returned card (CR 608.2d).
                Effect::ChooseOneAmong {
                    what: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: R::Creature.and(R::ManaValueLessThanEventAmount),
                    },
                    chooser: PlayerRef::You,
                    chosen: Box::new(Effect::Move {
                        what: Selector::SeparatedPile { chosen: true },
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                    other: Box::new(Effect::Noop),
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::ManaValueOf(Box::new(Selector::GreatestManaValueControlledMatching {
                    who: PlayerRef::You,
                    filter: R::Creature,
                })),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dance of the Dead — {1}{B} Aura. Reanimates a creature card tapped under
/// your control at +1/+1; it doesn't untap normally, but its controller may
/// pay {1}{B} in their upkeep to untap it. Leaving sacrifices it.
pub fn dance_of_the_dead() -> CardDefinition {
    let danced = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Dance of the Dead",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        static_abilities: vec![static_ab(
            "Enchanted creature doesn't untap during its controller's untap step.",
            StaticEffect::PreventUntap { applies_to: danced() },
        )],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(R::Creature.from_any_graveyard()),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    },
                    Effect::Attach { what: Selector::This, to: Selector::LastMoved },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::SacrificeSelected { what: danced() },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                    .with_filter(Predicate::ActivePlayerControls(Box::new(danced()))),
                effect: Effect::MayPayBy {
                    who: PlayerRef::ActivePlayer,
                    description: "Pay {1}{B} to untap the danced creature?".into(),
                    mana_cost: cost(&[generic(1), b()]),
                    body: Box::new(Effect::Untap { what: danced(), up_to: None }),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Ugin's Labyrinth — Land. Imprint a colorless card with mana value 7+
/// from hand; {T}: {C}, or {C}{C} with a card imprinted; {T}: return the
/// imprinted card to its owner's hand.
pub fn ugins_labyrinth() -> CardDefinition {
    CardDefinition {
        name: "Ugin's Labyrinth",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Imprint: exile a colorless card with mana value 7 or greater from your hand?".into(),
            body: Box::new(Effect::ExileTaggedWithSource {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Hand,
                        filter: R::Colorless.and(R::ManaValueAtLeast(7)),
                    },
                    Value::ONE,
                ),
            }),
        })],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Sum(vec![
                        Value::ONE,
                        Value::Min(Box::new(Value::CountOf(Box::new(Selector::CardExiledWithSource))), Box::new(Value::ONE)),
                    ])),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Move {
                    what: Selector::CardExiledWithSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::CardExiledWithSource))),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Embalmer's Tools — {2} Artifact. Activated abilities of creature cards in
/// your graveyard cost {1} less; tap an untapped Zombie you control: target
/// player mills a card.
pub fn embalmers_tools() -> CardDefinition {
    CardDefinition {
        name: "Embalmer's Tools",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![static_ab(
            "Activated abilities of creature cards in your graveyard cost {1} less to activate.",
            StaticEffect::GraveyardCreatureAbilitiesCostLess { amount: 1 },
        )],
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::Creature.and(R::HasCreatureType(CreatureType::Zombie))),
            effect: Effect::Mill { who: target_filtered(R::Player), amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ascend from Avernus — {X}{W}{W}{W} Sorcery. Every creature and
/// planeswalker card with mana value X or less in your graveyard returns;
/// then it exiles itself.
pub fn ascend_from_avernus() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Ascend from Avernus",
            cost(&[x(), w(), w(), w()]),
            CardType::Sorcery,
            Effect::ReturnAllMatchingFromGraveyardToBattlefield {
                who: PlayerRef::You,
                filter: R::Creature.or(R::Planeswalker).and(R::ManaValueAtMostXFromCost),
                sacrifice_eot: false,
            },
        )
    }
}

/// Sigarda's Splendor — {2}{W}{W} Enchantment. Notes your life total as it
/// enters; your upkeep draws if you're at or above the note, then re-notes;
/// white spells you cast gain you 1.
pub fn sigardas_splendor() -> CardDefinition {
    let note = || Effect::NoteAmountOnSource { amount: Value::LifeOf(PlayerRef::You) };
    CardDefinition {
        name: "Sigarda's Splendor",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        as_enters_effect: Some(note()),
        triggered_abilities: vec![
            at_your(
                TurnStep::Upkeep,
                Effect::Seq(vec![
                    Effect::If {
                        cond: Predicate::ValueAtLeast(Value::LifeOf(PlayerRef::You), Value::RememberedAmountOfSource),
                        then: Box::new(draw(1)),
                        else_: Box::new(Effect::Noop),
                    },
                    note(),
                ]),
            ),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellMatches(R::HasColor(Color::White))),
                effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

/// Neera, Wild Mage — {4}{U}{R} 2/7 Human Elf Shaman. Once each turn, a spell
/// you cast may go to the bottom of its owner's library; if it does, you dig
/// to your next nonland card and may cast it free, the misses going to the
/// bottom in a random order. The dig is cascade's walk with no mana value
/// cap (CR 702.85a): the passed-over cards end on the bottom the same way,
/// they're just exiled rather than revealed while it walks.
pub fn neera_wild_mage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).once_per_turn(),
            effect: Effect::If {
                cond: Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSpellOnStack },
                then: Box::new(Effect::MayDo {
                    description: "Put the spell on the bottom of its owner's library and dig for a free spell?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::MoveSpellToZone {
                            what: Selector::TriggerSource,
                            zone: crate::effect::CounteredSpellZone::OwnerLibraryBottom,
                        },
                        Effect::Cascade { max_mv: Value::Const(1000), filter: None },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..legend(
            "Neera, Wild Mage",
            cost(&[generic(4), u(), r()]),
            vec![CreatureType::Human, CreatureType::Elf, CreatureType::Shaman],
            2,
            7,
        )
    }
}

/// Jirina, Dauntless General — {W}{B} 2/2 Human Soldier. ETB exiles a target
/// player's graveyard; sacrifice it: Humans you control gain hexproof and
/// indestructible until end of turn.
pub fn jirina_dauntless_general() -> CardDefinition {
    let humans = || Selector::EachPermanent(R::Creature.and(R::HasCreatureType(CreatureType::Human)).and(R::ControlledByYou));
    let grant = |keyword| Effect::GrantKeyword { what: humans(), keyword, duration: Duration::EndOfTurn };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::TargetPlayerThen {
            filter: R::Player,
            then: Box::new(Effect::Move {
                what: Selector::CardsInZone { who: PlayerRef::Target(0), zone: Zone::Graveyard, filter: R::Any },
                to: ZoneDest::Exile,
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Seq(vec![grant(Keyword::Hexproof), grant(Keyword::Indestructible)]),
            ..Default::default()
        }],
        ..legend("Jirina, Dauntless General", cost(&[w(), b()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}
