//! Commander: the 2026-10-13 rescan's one-card gaps (EDHREC ranks 1-1500)
//! — Zuko, Black Vise, Garruk's Horde and friends (`cmdr_edhrec21` is the
//! previous batch). Tests in `tests/recent_b/cmdr_edhrec22.rs`.

use crate::card::{
    AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, partner_with_search, target_filtered};
use crate::effect::{
    ActivatedAbility, CounteredSpellZone, DelayedTriggerKind, Duration, Effect, PlayerRef, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color, ManaCost};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn on(kind: EventKind, scope: EventScope, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(kind, scope), effect }
}

fn source_is(kind: EventKind, scope: EventScope, filter: R, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(kind, scope)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter }),
        effect,
    }
}

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    })
}

/// Zuko, Seeking Honor — {2}{B/R} 2/2 legendary Human Noble. Firebending 1.
/// Casting a noncreature spell gives him first strike until end of turn; his
/// combat damage to a player puts a +1/+1 counter on him.
pub fn zuko_seeking_honor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Firebending(1)],
        triggered_abilities: vec![
            source_is(
                EventKind::SpellCast,
                EventScope::YourControl,
                R::Noncreature,
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
            ),
            on(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ),
        ],
        ..creature(
            "Zuko, Seeking Honor",
            cost(&[generic(2), hybrid(Color::Black, Color::Red)]),
            vec![CreatureType::Human, CreatureType::Noble],
            2,
            2,
        )
    }
}

/// Black Vise — {1} artifact. As it enters, choose an opponent; at the
/// beginning of their upkeep it deals them damage equal to their hand size
/// minus four.
pub fn black_vise() -> CardDefinition {
    CardDefinition {
        name: "Black Vise",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        as_enters_effect: Some(Effect::RememberPlayerOnSource { who: PlayerRef::HostileOpponent }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::IsTurnOf(PlayerRef::ChosenPlayerOfSource)),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ChosenPlayerOfSource),
                amount: Value::Sum(vec![
                    Value::CardsInHandMatching { who: PlayerRef::ChosenPlayerOfSource, filter: R::Any },
                    Value::Const(-4),
                ]),
            },
        }],
        ..Default::default()
    }
}

/// Garruk's Horde — {5}{G}{G} 7/7 Beast, trample. Play with the top card of
/// your library revealed; you may cast creature spells from it.
pub fn garruks_horde() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![
            static_ab("Play with the top card of your library revealed.", StaticEffect::TopOfLibraryRevealed),
            static_ab(
                "You may cast creature spells from the top of your library.",
                StaticEffect::PlayFromLibraryTop { filter: R::Creature },
            ),
        ],
        ..creature("Garruk's Horde", cost(&[generic(5), g(), g()]), vec![CreatureType::Beast], 7, 7)
    }
}

/// Mercy Killing — {2}{G/W} instant. Target creature's controller sacrifices
/// it, then creates X 1/1 green and white Elf Warriors, X = its power.
pub fn mercy_killing() -> CardDefinition {
    let elf = token(
        "Elf Warrior",
        1,
        1,
        vec![Color::Green, Color::White],
        vec![CreatureType::Elf, CreatureType::Warrior],
        vec![],
    );
    spell(
        "Mercy Killing",
        cost(&[generic(2), hybrid(Color::Green, Color::White)]),
        CardType::Instant,
        Effect::WithX {
            x: Value::PowerOf(Box::new(target_filtered(R::Creature))),
            body: Box::new(Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    count: Value::XFromCost,
                    definition: elf,
                },
                Effect::SacrificeSelected { what: Selector::Target(0) },
            ])),
        },
    )
}

/// Ley Weaver — {3}{G} 2/2 Human Druid. Partner with Lore Weaver. {T}: untap
/// two target lands.
pub fn ley_weaver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Lore Weaver".into())],
        triggered_abilities: vec![partner_with_search("Lore Weaver")],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 2,
                filter: R::Land,
                effect: Box::new(Effect::Untap { what: Selector::Target(0), up_to: None }),
            },
            ..Default::default()
        }],
        ..creature("Ley Weaver", cost(&[generic(3), g()]), vec![CreatureType::Human, CreatureType::Druid], 2, 2)
    }
}

/// Acrobatic Maneuver — {2}{W} instant. Blink target creature you control;
/// draw a card.
pub fn acrobatic_maneuver() -> CardDefinition {
    spell(
        "Acrobatic Maneuver",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::ExileAndReturnToOwner { what: target_filtered(R::Creature.and(R::ControlledByYou)) },
            draw(1),
        ]),
    )
}

/// Basri's Lieutenant — {3}{W} 3/4 Human Knight, vigilance, protection from
/// multicolored. ETB: a +1/+1 counter on target creature you control.
/// Whenever it or another creature you control dies with a +1/+1 counter on
/// it, create a 2/2 white Knight with vigilance.
pub fn basris_lieutenant() -> CardDefinition {
    let knight = token("Knight", 2, 2, vec![Color::White], vec![CreatureType::Knight], vec![Keyword::Vigilance]);
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::ProtectionFromMulticolored],
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
            on(
                EventKind::CreatureDied,
                EventScope::YourControl,
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn { what: Box::new(Selector::TriggerSource), kind: CounterType::PlusOnePlusOne },
                        Value::ONE,
                    ),
                    then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: knight }),
                    else_: Box::new(Effect::Noop),
                },
            ),
        ],
        ..creature("Basri's Lieutenant", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Knight], 3, 4)
    }
}

/// Loran, Disciple of History — {3}{W} 3/3 legendary Human Artificer.
/// Whenever it or another legendary creature you control enters, return
/// target artifact card from your graveyard to your hand.
pub fn loran_disciple_of_history() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![source_is(
            EventKind::EntersBattlefield,
            EventScope::YourControl,
            R::Creature.and(R::HasSupertype(Supertype::Legendary)),
            Effect::Move {
                what: target_filtered(R::Artifact.from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        )],
        ..creature(
            "Loran, Disciple of History",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            3,
            3,
        )
    }
}

/// Tainted Indulgence — {U}{B} instant. Draw two, then discard a card unless
/// there are five or more mana values among cards in your graveyard.
pub fn tainted_indulgence() -> CardDefinition {
    spell(
        "Tainted Indulgence",
        cost(&[u(), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            draw(2),
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::DistinctManaValuesInGraveyard(PlayerRef::You), Value::Const(5)),
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }),
            },
        ]),
    )
}

/// Delay — {1}{U} instant. Counter target spell; exile it with three time
/// counters instead, and it gains suspend if it doesn't have it.
pub fn delay() -> CardDefinition {
    spell(
        "Delay",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::CounterSpellToZone {
            what: target_filtered(R::IsSpellOnStack),
            zone: CounteredSpellZone::ExileSuspended(3),
        },
    )
}

/// Feywild Caretaker — {4}{U} 3/4 Orc Wizard. ETB: take the initiative. At
/// the beginning of your end step, if you have the initiative, create a 1/1
/// blue Faerie Dragon with flying.
pub fn feywild_caretaker() -> CardDefinition {
    let dragon = token(
        "Faerie Dragon",
        1,
        1,
        vec![Color::Blue],
        vec![CreatureType::Faerie, CreatureType::Dragon],
        vec![Keyword::Flying],
    );
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::TakeInitiative { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(Predicate::HasInitiative { who: PlayerRef::You }),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: dragon },
            },
        ],
        ..creature("Feywild Caretaker", cost(&[generic(4), u()]), vec![CreatureType::Orc, CreatureType::Wizard], 3, 4)
    }
}

/// Champion of the Path — {3}{R} 7/3 Elemental Sorcerer. Additional cost:
/// behold an Elemental and exile it. Another Elemental of yours entering deals
/// damage equal to its power to each opponent. Leaving, it returns the
/// exiled card to its owner's hand.
pub fn champion_of_the_path() -> CardDefinition {
    let elemental = || R::HasCreatureType(CreatureType::Elemental);
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::BeholdAndExile { filter: elemental() }],
        triggered_abilities: vec![
            source_is(
                EventKind::EntersBattlefield,
                EventScope::AnotherOfYours,
                elemental(),
                Effect::DealDamageFrom {
                    source: Selector::TriggerSource,
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                },
            ),
            on(
                EventKind::PermanentLeavesBattlefield,
                EventScope::SelfSource,
                Effect::Move {
                    what: Selector::CardExiledWithSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::CardExiledWithSource))),
                },
            ),
        ],
        ..creature(
            "Champion of the Path",
            cost(&[generic(3), r()]),
            vec![CreatureType::Elemental, CreatureType::Sorcerer],
            7,
            3,
        )
    }
}

/// Jace, Reality Sculptor — {3}{U}{U}, loyalty 5. +1: empower Jace X (X =
/// Islands you control). −3: until your next turn, each creature attacking
/// you or your planeswalkers gets -5/-0. 0: each opponent exiles all but the
/// bottom card of their library, with 25+ loyalty among your Jaces.
pub fn jace_reality_sculptor() -> CardDefinition {
    let jaces = Selector::EachPermanent(
        R::HasPlaneswalkerType(PlaneswalkerSubtype::Jace).and(R::ControlledByYou),
    );
    CardDefinition {
        name: "Jace, Reality Sculptor",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Jace], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::EmpowerJace {
                    who: PlayerRef::You,
                    count: Value::count(Selector::EachPermanent(
                        R::HasLandType(LandType::Island).and(R::ControlledByYou),
                    )),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::DelayUntil {
                    kind: DelayedTriggerKind::CreatureAttacksYouUntilYourNextTurn,
                    body: Box::new(Effect::PumpPT {
                        what: Selector::TriggerSource,
                        power: Value::Const(-5),
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn { what: Box::new(jaces), kind: CounterType::Loyalty },
                        Value::Const(25),
                    ),
                    then: Box::new(Effect::ExileLibraryExceptBottom { who: PlayerRef::EachOpponent, keep: Value::ONE }),
                    else_: Box::new(Effect::Noop),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ranger's Path — {3}{G} sorcery. Up to two Forest cards onto the
/// battlefield tapped.
pub fn rangers_path() -> CardDefinition {
    spell(
        "Ranger's Path",
        cost(&[generic(3), g()]),
        CardType::Sorcery,
        Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::HasLandType(LandType::Forest),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            count: Value::Const(2),
        },
    )
}

/// Tenacious Dead — {B} 1/1 Skeleton Warrior. When it dies, you may pay
/// {1}{B} to return it to the battlefield tapped under its owner's control.
pub fn tenacious_dead() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on(
            EventKind::CreatureDied,
            EventScope::SelfSource,
            Effect::MayPay {
                description: "Pay {1}{B} to return Tenacious Dead?".into(),
                mana_cost: cost(&[generic(1), b()]),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOf(Box::new(Selector::This)),
                        tapped: true,
                    },
                }),
                else_: None,
            },
        )],
        ..creature("Tenacious Dead", cost(&[b()]), vec![CreatureType::Skeleton, CreatureType::Warrior], 1, 1)
    }
}

/// The Misty Mountains Cold — {2}{R} Saga. I–IV: a Treasure; then with four
/// or more Treasures, sacrifice it and, if you do, a 6/6 red flying Dragon.
pub fn the_misty_mountains_cold() -> CardDefinition {
    let chapter = || {
        Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasArtifactSubtype(ArtifactSubtype::Treasure).and(R::ControlledByYou),
                    ),
                    n: Value::Const(4),
                },
                then: Box::new(Effect::If {
                    cond: Predicate::SelectorExists(Selector::This),
                    then: Box::new(Effect::Seq(vec![
                        Effect::SacrificePermanent { what: Selector::This },
                        Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            definition: token(
                                "Dragon",
                                6,
                                6,
                                vec![Color::Red],
                                vec![CreatureType::Dragon],
                                vec![Keyword::Flying],
                            ),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                }),
                else_: Box::new(Effect::Noop),
            },
        ])
    };
    CardDefinition {
        name: "The Misty Mountains Cold",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![(1, chapter()), (2, chapter()), (3, chapter()), (4, chapter())],
        ..Default::default()
    }
}
