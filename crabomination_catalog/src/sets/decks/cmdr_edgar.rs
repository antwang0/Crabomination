//! Commander batch — cards EDHREC lists as commonly played with **Edgar
//! Markov** that the catalog lacked (`cmdr_edgar`). Vampire tribal, the
//! W/B/R support suite (Sorins, Olivias, Elenda), choose-a-creature-type
//! kindred payoffs, and the multiplayer lands. Tests in `recent_b/cmdr_edgar`.
//!
//! Residuals (approximated or omitted clauses — each also noted on its card):
//! - **Bilbo's Gambit** — "return target spell to its owner's hand" rides the
//!   countered-spell-to-hand path (as Reprieve does), so an uncounterable spell
//!   is not bounced; the gift's Treasure goes to the first opponent in turn
//!   order (the engine's gift model has no recipient choice).
//! - **Gleaming Splendor** — "two target players" are two player slots; the
//!   engine does not require them to be distinct.
//! - **Master of Dark Rites** — the Vampire/Cleric/Demon restriction reads a
//!   creature spell's types, so a noncreature Kindred Vampire spell can't spend
//!   it.
//! - **Necropotence** — the exiled card is exiled face up, and it returns at
//!   the next end step (any player's), not strictly "your next end step".
//! - **Drana and Linvala** — the "spend mana as though it were mana of any
//!   color" rider on the borrowed abilities is dropped; the lock gates the
//!   activation path, so a creature's mana ability tapped by the auto-payer
//!   while paying a cost is not stopped.
//! - **Olivia Voldaren** — "for as long as you control Olivia" is modelled as
//!   "for as long as Olivia remains on the battlefield".
//! - **Florian, Voldaren Scion** — the exiled card may be cast while it stays
//!   exiled (the engine's look-and-exile-with-permission primitive), not
//!   "played this turn"; a land among them can't be played.
//! - **Charismatic Conqueror** — the tap choice is asked of the permanent's
//!   controller as a yes/no.
//! - **New Blood** — the text change is modelled as the stolen creature
//!   becoming a Vampire *in addition to* its types (it keeps the replaced type,
//!   and creature-type words inside its rules text are not rewritten).
//! - **Idol of Oblivion** — "you created a token this turn" reads "a token you
//!   control entered this turn", so a token that already left (or one you made
//!   for another player) doesn't count.
//! - **Edgar, Charmed Groom** — returns under the control of its controller at
//!   death (its owner in every ordinary game).
//! - **Westvale Abbey** — the transform ability's five sacrifices are "other
//!   creatures" (the land itself is not a creature, so this matches).

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EquipBonus, EventKind, EventScope, EventSpec, Gift, Keyword,
    LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R, Selector, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, on_you_attack, target_filtered};
use crate::effect::{
    CounteredSpellZone, Duration, Effect, LibraryPosition, ManaPayload,
    PlayerRef, Predicate, StaticAbility, StaticEffect, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{
    Color, ManaCost, ManaSymbol, SpendRestriction, b, cost, generic, r, w, x,
};
use crabomination_base::tokens::{blood_token, treasure_token};
use std::sync::Arc;

// ── Shared shapes ────────────────────────────────────────────────────────────

fn creature(
    name: &'static str,
    mana: ManaCost,
    power: i32,
    toughness: i32,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power,
        toughness,
        keywords,
        ..Default::default()
    }
}

fn legend(mut card: CardDefinition) -> CardDefinition {
    card.supertypes = vec![Supertype::Legendary];
    card
}

fn spell(name: &'static str, mana: ManaCost, sorcery: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if sorcery {
            CardType::Sorcery
        } else {
            CardType::Instant
        }],
        effect,
        ..Default::default()
    }
}

fn permanent(name: &'static str, mana: ManaCost, kind: CardType) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        ..Default::default()
    }
}

fn vampire() -> R {
    R::HasCreatureType(CreatureType::Vampire)
}

/// "Vampires you control" (permanents, as printed).
fn your_vampires() -> R {
    vampire().and(R::ControlledByYou)
}

/// "Other Vampires you control get +P/+T."
fn other_vampires_pump(power: i32, toughness: i32, description: &'static str) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::PumpPT {
            applies_to: Selector::EachPermanent(
                R::Creature.and(your_vampires()).and(R::OtherThanSource),
            ),
            power,
            toughness,
        },
    }
}

fn count_your_vampires() -> Value {
    Value::count(Selector::EachPermanent(your_vampires()))
}

fn vampire_token(
    power: i32,
    toughness: i32,
    colors: Vec<Color>,
    extra: Vec<CreatureType>,
    keywords: Vec<Keyword>,
    tapped: bool,
) -> TokenDefinition {
    let mut types = vec![CreatureType::Vampire];
    types.extend(extra);
    TokenDefinition {
        name: if types.len() > 1 { "Vampire Demon".into() } else { "Vampire".into() },
        power,
        toughness,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        keywords,
        tapped,
        ..Default::default()
    }
}

/// 1/1 white Vampire with lifelink (Elenda, March of the Canonized, …).
fn white_lifelinker() -> TokenDefinition {
    vampire_token(1, 1, vec![Color::White], vec![], vec![Keyword::Lifelink], false)
}

/// 1/1 white and black Vampire with lifelink (Coffin, Glass-Cast Heart).
fn wb_lifelinker() -> TokenDefinition {
    vampire_token(1, 1, vec![Color::White, Color::Black], vec![], vec![Keyword::Lifelink], false)
}

/// 2/2 black Vampire with flying (Bloodline Keeper, Sorin, Solemn Visitor).
fn black_flyer() -> TokenDefinition {
    vampire_token(2, 2, vec![Color::Black], vec![], vec![Keyword::Flying], false)
}

/// 4/3 white and black Vampire Demon with flying.
fn vampire_demon(tapped: bool) -> TokenDefinition {
    vampire_token(
        4,
        3,
        vec![Color::White, Color::Black],
        vec![CreatureType::Demon],
        vec![Keyword::Flying],
        tapped,
    )
}

fn mint(who: PlayerRef, count: Value, token: TokenDefinition) -> Effect {
    Effect::CreateToken {
        who,
        count,
        definition: Arc::new(token),
    }
}

fn mint_one(token: TokenDefinition) -> Effect {
    mint(PlayerRef::You, Value::ONE, token)
}

fn draw(who: Selector, n: Value) -> Effect {
    Effect::Draw { who, amount: n }
}

/// "Creatures you control of the chosen type" (the source's stamped type).
fn your_chosen_type() -> R {
    R::Creature.and(R::ControlledByYou).and(R::IsSourceChosenCreatureType)
}

fn tapper(mana: ManaCost, effect: Effect) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        tap_cost: true,
        effect,
        ..Default::default()
    }
}

fn add_colors(colors: Vec<Color>) -> Effect {
    Effect::AddMana {
        who: PlayerRef::You,
        pool: ManaPayload::Colors(colors),
    }
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Edgar, Ancient Bloodlord — {W}{B} Legendary Creature — Vampire Noble 2/3.
/// "Whenever another creature or planeswalker you control dies, you gain 1
/// life. {2}, Sacrifice another creature or planeswalker: Put a +1/+1 counter
/// on Edgar. He gains menace until end of turn."
pub fn edgar_ancient_bloodlord() -> CardDefinition {
    let creature_or_walker = R::Creature.or(R::Planeswalker);
    legend(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: creature_or_walker.clone(),
                }),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((creature_or_walker, 1)),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Edgar, Ancient Bloodlord",
            cost(&[w(), b()]),
            2,
            3,
            vec![CreatureType::Vampire, CreatureType::Noble],
            vec![],
        )
    })
}

/// Bloodline Recollector // Ancestral Craving — {1}{B} Creature — Vampire
/// Warlock 2/2 // {B} Instant. "At the beginning of each end step, if three or
/// more creatures died this turn, this creature becomes prepared." Prepare
/// spell: "Target player draws three cards and loses 3 life."
///
/// The intervening "if" is the event filter (re-checked at resolution by the
/// step-trigger path); "becomes prepared" is capped at one Prepared counter.
pub fn bloodline_recollector() -> CardDefinition {
    let craving = spell(
        "Ancestral Craving",
        cost(&[b()]),
        false,
        Effect::Seq(vec![
            draw(target_filtered(R::Player), Value::Const(3)),
            Effect::LoseLife {
                who: Selector::Target(0),
                amount: Value::Const(3),
            },
        ]),
    );
    CardDefinition {
        prepare_spell: Some(Arc::new(craving)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast {
                    at_least: Value::Const(3),
                }),
            effect: Effect::AddCounterCapped {
                what: Selector::This,
                kind: CounterType::Prepared,
                amount: Value::ONE,
                cap: Value::ONE,
            },
        }],
        ..creature(
            "Bloodline Recollector",
            cost(&[generic(1), b()]),
            2,
            2,
            vec![CreatureType::Vampire, CreatureType::Warlock],
            vec![],
        )
    }
}

/// Bloodline Keeper // Lord of Lineage — {2}{B}{B} Creature — Vampire 3/3,
/// flying. "{T}: Create a 2/2 black Vampire creature token with flying. {B}:
/// Transform this creature. Activate only if you control five or more
/// Vampires." Back: Lord of Lineage 5/5 flying, "Other Vampire creatures you
/// control get +2/+2" and the same {T} token ability.
pub fn bloodline_keeper() -> CardDefinition {
    let make_flyer = tapper(ManaCost::default(), mint_one(black_flyer()));
    let lord = CardDefinition {
        static_abilities: vec![other_vampires_pump(
            2,
            2,
            "Other Vampire creatures you control get +2/+2.",
        )],
        activated_abilities: vec![make_flyer.clone()],
        ..creature(
            "Lord of Lineage",
            ManaCost::default(),
            5,
            5,
            vec![CreatureType::Vampire],
            vec![Keyword::Flying],
        )
    };
    CardDefinition {
        activated_abilities: vec![
            make_flyer,
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(your_vampires()),
                    n: Value::Const(5),
                }),
                effect: Effect::Transform {
                    what: Selector::This,
                },
                ..Default::default()
            },
        ],
        back_face: Some(Box::new(lord)),
        ..creature(
            "Bloodline Keeper",
            cost(&[generic(2), b(), b()]),
            3,
            3,
            vec![CreatureType::Vampire],
            vec![Keyword::Flying],
        )
    }
}

/// Sanctum Seeker — {2}{B}{B} Creature — Vampire Knight 3/4. "Whenever a
/// Vampire you control attacks, each opponent loses 1 life and you gain 1
/// life."
pub fn sanctum_seeker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: vampire(),
                },
            ),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Sanctum Seeker",
            cost(&[generic(2), b(), b()]),
            3,
            4,
            vec![CreatureType::Vampire, CreatureType::Knight],
            vec![],
        )
    }
}

/// Master of Dark Rites — {B} Creature — Vampire Cleric 1/1. "{T}, Sacrifice
/// another creature: Add {B}{B}{B}. Spend this mana only to cast Vampire,
/// Cleric, and/or Demon spells." (The restriction reads a creature spell's
/// types — see the module residuals.)
pub fn master_of_dark_rites() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::Black, Color::Black, Color::Black])),
                    SpendRestriction::CreatureOfAnyTypes([
                        CreatureType::Vampire,
                        CreatureType::Cleric,
                        CreatureType::Demon,
                    ]),
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Master of Dark Rites",
            cost(&[b()]),
            1,
            1,
            vec![CreatureType::Vampire, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Elenda, the Dusk Rose — {2}{W}{B} Legendary Creature — Vampire Knight 1/1,
/// lifelink. "Whenever another creature dies, put a +1/+1 counter on Elenda.
/// When Elenda dies, create X 1/1 white Vampire creature tokens with
/// lifelink, where X is Elenda's power." (X reads her last-known power.)
pub fn elenda_the_dusk_rose() -> CardDefinition {
    legend(CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                    .with_filter(Predicate::Not(Box::new(Predicate::TriggerSourceIsSelf))),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            on_dies(mint(
                PlayerRef::You,
                Value::PowerOf(Box::new(Selector::This)),
                white_lifelinker(),
            )),
        ],
        ..creature(
            "Elenda, the Dusk Rose",
            cost(&[generic(2), w(), b()]),
            1,
            1,
            vec![CreatureType::Vampire, CreatureType::Knight],
            vec![Keyword::Lifelink],
        )
    })
}

/// Charismatic Conqueror — {1}{W} Creature — Vampire Soldier 2/2, vigilance.
/// "Whenever an artifact or creature an opponent controls enters untapped,
/// they may tap that permanent. If they don't, you create a 1/1 white Vampire
/// creature token with lifelink."
///
/// The controller of the entering permanent answers the yes/no (a {0}
/// `MayPayBy`); the token's recipient is the Conqueror's controller.
pub fn charismatic_conqueror() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact.or(R::Creature).and(R::Untapped),
                }),
            effect: Effect::MayPayBy {
                who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                description: "Tap that permanent? (If you don't, its opponent creates a 1/1 Vampire.)"
                    .into(),
                mana_cost: ManaCost::default(),
                body: Box::new(Effect::Tap {
                    what: Selector::TriggerSource,
                }),
                else_: Some(Box::new(mint(
                    PlayerRef::ControllerOf(Box::new(Selector::This)),
                    Value::ONE,
                    white_lifelinker(),
                ))),
            },
        }],
        ..creature(
            "Charismatic Conqueror",
            cost(&[generic(1), w()]),
            2,
            2,
            vec![CreatureType::Vampire, CreatureType::Soldier],
            vec![Keyword::Vigilance],
        )
    }
}

/// Edgar, Charmed Groom // Edgar Markov's Coffin — {2}{W}{B} Legendary
/// Creature — Vampire Noble 4/4. "Other Vampires you control get +1/+1. When
/// Edgar dies, return it to the battlefield transformed under its owner's
/// control." Back: Legendary Artifact — "At the beginning of your upkeep,
/// create a 1/1 white and black Vampire creature token with lifelink and put a
/// bloodline counter on Edgar Markov's Coffin. Then if there are three or more
/// bloodline counters on it, remove those counters and transform it."
pub fn edgar_charmed_groom() -> CardDefinition {
    let coffin = CardDefinition {
        name: "Edgar Markov's Coffin",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                mint_one(wb_lifelinker()),
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Bloodline,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::SourceHasCountersAtLeast {
                        counter: CounterType::Bloodline,
                        n: 3,
                    },
                    then: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter {
                            what: Selector::This,
                            kind: CounterType::Bloodline,
                            amount: Value::CountersOn {
                                what: Box::new(Selector::This),
                                kind: CounterType::Bloodline,
                            },
                        },
                        Effect::Transform {
                            what: Selector::This,
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    };
    legend(CardDefinition {
        static_abilities: vec![other_vampires_pump(1, 1, "Other Vampires you control get +1/+1.")],
        triggered_abilities: vec![on_dies(Effect::ExileSelfReturnTransformed)],
        back_face: Some(Box::new(coffin)),
        ..creature(
            "Edgar, Charmed Groom",
            cost(&[generic(2), w(), b()]),
            4,
            4,
            vec![CreatureType::Vampire, CreatureType::Noble],
            vec![],
        )
    })
}

/// Clavileño, First of the Blessed — {1}{W}{B} Legendary Creature — Vampire
/// Cleric 2/2. "Whenever you attack, target attacking Vampire that isn't a
/// Demon becomes a Demon in addition to its other types. It gains 'When this
/// creature dies, draw a card and create a tapped 4/3 white and black Vampire
/// Demon creature token with flying.'"
pub fn clavileno_first_of_the_blessed() -> CardDefinition {
    let target = target_filtered(
        R::Creature
            .and(R::IsAttacking)
            .and(vampire())
            .and(R::HasCreatureType(CreatureType::Demon).negate()),
    );
    legend(CardDefinition {
        triggered_abilities: vec![on_you_attack(Effect::Seq(vec![
            Effect::AddCreatureTypes {
                what: target,
                creature_types: vec![CreatureType::Demon],
                duration: Duration::Permanent,
            },
            Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(on_dies(Effect::Seq(vec![
                    draw(Selector::You, Value::ONE),
                    mint_one(vampire_demon(true)),
                ]))),
                duration: Duration::Permanent,
            },
        ]))],
        ..creature(
            "Clavileño, First of the Blessed",
            cost(&[generic(1), w(), b()]),
            2,
            2,
            vec![CreatureType::Vampire, CreatureType::Cleric],
            vec![],
        )
    })
}

/// Markov Baron — {2}{B} Creature — Vampire Noble 2/2. Convoke, lifelink.
/// "Other Vampires you control get +1/+1." Madness {2}{B}.
pub fn markov_baron() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![other_vampires_pump(1, 1, "Other Vampires you control get +1/+1.")],
        ..creature(
            "Markov Baron",
            cost(&[generic(2), b()]),
            2,
            2,
            vec![CreatureType::Vampire, CreatureType::Noble],
            vec![
                Keyword::Convoke,
                Keyword::Lifelink,
                Keyword::Madness(cost(&[generic(2), b()])),
            ],
        )
    }
}

/// Champion of Dusk — {3}{B}{B} Creature — Vampire Knight 4/4. "When this
/// creature enters, you draw X cards and you lose X life, where X is the
/// number of Vampires you control."
pub fn champion_of_dusk() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            draw(Selector::You, count_your_vampires()),
            Effect::LoseLife {
                who: Selector::You,
                amount: count_your_vampires(),
            },
        ]))],
        ..creature(
            "Champion of Dusk",
            cost(&[generic(3), b(), b()]),
            4,
            4,
            vec![CreatureType::Vampire, CreatureType::Knight],
            vec![],
        )
    }
}

/// Mavren Fein, Dusk Apostle — {2}{W} Legendary Creature — Vampire Cleric 2/2.
/// "Whenever one or more nontoken Vampires you control attack, create a 1/1
/// white Vampire creature token with lifelink."
pub fn mavren_fein_dusk_apostle() -> CardDefinition {
    legend(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource).with_filter(
                Predicate::AttackedWithCreatureMatching {
                    who: PlayerRef::You,
                    filter: vampire().and(R::NotToken),
                },
            ),
            effect: mint_one(white_lifelinker()),
        }],
        ..creature(
            "Mavren Fein, Dusk Apostle",
            cost(&[generic(2), w()]),
            2,
            2,
            vec![CreatureType::Vampire, CreatureType::Cleric],
            vec![],
        )
    })
}

/// Patron of the Vein — {4}{B}{B} Creature — Vampire Shaman 4/4, flying.
/// "When this creature enters, destroy target creature an opponent controls.
/// Whenever a creature an opponent controls dies, exile it and put a +1/+1
/// counter on each Vampire you control."
pub fn patron_of_the_vein() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Destroy {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Exile,
                    },
                    Effect::AddCounter {
                        what: Selector::EachPermanent(R::Creature.and(your_vampires())),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                ]),
            },
        ],
        ..creature(
            "Patron of the Vein",
            cost(&[generic(4), b(), b()]),
            4,
            4,
            vec![CreatureType::Vampire, CreatureType::Shaman],
            vec![Keyword::Flying],
        )
    }
}

/// Forerunner of the Legion — {2}{W} Creature — Vampire Knight 2/2. "When this
/// creature enters, you may search your library for a Vampire card, reveal it,
/// then shuffle and put that card on top. Whenever another Vampire you control
/// enters, target creature gets +1/+1 until end of turn."
pub fn forerunner_of_the_legion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Search your library for a Vampire card to put on top?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: vampire(),
                    to: ZoneDest::Library {
                        who: PlayerRef::You,
                        pos: LibraryPosition::Top,
                    },
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: vampire(),
                    }),
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..creature(
            "Forerunner of the Legion",
            cost(&[generic(2), w()]),
            2,
            2,
            vec![CreatureType::Vampire, CreatureType::Knight],
            vec![],
        )
    }
}

/// Rakish Heir — {2}{R} Creature — Vampire 2/2. "Whenever a Vampire you
/// control deals combat damage to a player, put a +1/+1 counter on it."
pub fn rakish_heir() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: vampire(),
                }),
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Rakish Heir",
            cost(&[generic(2), r()]),
            2,
            2,
            vec![CreatureType::Vampire],
            vec![],
        )
    }
}

/// Olivia Voldaren — {2}{B}{R} Legendary Creature — Vampire 3/3, flying.
/// "{1}{R}: Olivia Voldaren deals 1 damage to another target creature. That
/// creature becomes a Vampire in addition to its other types. Put a +1/+1
/// counter on Olivia Voldaren. {3}{B}{B}: Gain control of target Vampire for
/// as long as you control Olivia Voldaren." (The steal lasts while Olivia
/// remains on the battlefield.)
pub fn olivia_voldaren() -> CardDefinition {
    legend(CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                effect: Effect::Seq(vec![
                    Effect::DealDamage {
                        to: target_filtered(R::Creature.and(R::OtherThanSource)),
                        amount: Value::ONE,
                    },
                    Effect::AddCreatureTypes {
                        what: Selector::Target(0),
                        creature_types: vec![CreatureType::Vampire],
                        duration: Duration::Permanent,
                    },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), b(), b()]),
                effect: Effect::GainControlWhileSourceRemains {
                    what: target_filtered(R::Creature.and(vampire())),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Olivia Voldaren",
            cost(&[generic(2), b(), r()]),
            3,
            3,
            vec![CreatureType::Vampire],
            vec![Keyword::Flying],
        )
    })
}

/// Drana and Linvala — {1}{W}{W}{B} Legendary Creature — Vampire Angel 3/4,
/// flying, vigilance. "Activated abilities of creatures your opponents control
/// can't be activated. Drana and Linvala has all activated abilities of all
/// creatures your opponents control. You may spend mana as though it were
/// mana of any color to activate those abilities." (The any-color spend rider
/// is dropped — see the module residuals.)
pub fn drana_and_linvala() -> CardDefinition {
    legend(CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Activated abilities of creatures your opponents control can't be activated.",
                effect: StaticEffect::OpponentsCreatureAbilitiesLocked,
            },
            StaticAbility {
                description: "Drana and Linvala has all activated abilities of all creatures your opponents control.",
                effect: StaticEffect::HasActivatedAbilitiesOfOpponentCreatures,
            },
        ],
        ..creature(
            "Drana and Linvala",
            cost(&[generic(1), w(), w(), b()]),
            3,
            4,
            vec![CreatureType::Vampire, CreatureType::Angel],
            vec![Keyword::Flying, Keyword::Vigilance],
        )
    })
}

/// Oathsworn Vampire — {1}{B} Creature — Vampire Knight 2/2. "This creature
/// enters tapped. You may cast this card from your graveyard if you gained
/// life this turn."
pub fn oathsworn_vampire() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        flashback_condition: Some(Predicate::PlayerGainedLifeThisTurn { who: PlayerRef::You }),
        ..creature(
            "Oathsworn Vampire",
            cost(&[generic(1), b()]),
            2,
            2,
            vec![CreatureType::Vampire, CreatureType::Knight],
            vec![Keyword::GraveyardCast],
        )
    }
}

/// Florian, Voldaren Scion — {1}{B}{R} Legendary Creature — Vampire Noble 3/3,
/// first strike. "At the beginning of each of your postcombat main phases,
/// look at the top X cards of your library, where X is the total amount of
/// life your opponents lost this turn. Exile one of those cards and put the
/// rest on the bottom of your library in a random order. You may play the
/// exiled card this turn." (The permission lasts while the card is exiled.)
pub fn florian_voldaren_scion() -> CardDefinition {
    legend(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::PostCombatMain),
                EventScope::YourControl,
            ),
            effect: Effect::LookTopExileOneMayPlay {
                count: Value::TotalLifeLostThisTurn(PlayerRef::EachOpponent),
                who: PlayerRef::You,
            },
        }],
        ..creature(
            "Florian, Voldaren Scion",
            cost(&[generic(1), b(), r()]),
            3,
            3,
            vec![CreatureType::Vampire, CreatureType::Noble],
            vec![Keyword::FirstStrike],
        )
    })
}

/// Elenda's Hierophant — {2}{W} Creature — Vampire Cleric 1/1, flying.
/// "Whenever you gain life, put a +1/+1 counter on this creature. When this
/// creature dies, create X 1/1 white Vampire creature tokens with lifelink,
/// where X is its power."
pub fn elendas_hierophant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            on_dies(mint(
                PlayerRef::You,
                Value::PowerOf(Box::new(Selector::This)),
                white_lifelinker(),
            )),
        ],
        ..creature(
            "Elenda's Hierophant",
            cost(&[generic(2), w()]),
            1,
            1,
            vec![CreatureType::Vampire, CreatureType::Cleric],
            vec![Keyword::Flying],
        )
    }
}

/// Vampire Nocturnus — {1}{B}{B}{B} Creature — Vampire 3/3. "Play with the top
/// card of your library revealed. As long as the top card of your library is
/// black, this creature and other Vampire creatures you control get +2/+1 and
/// have flying."
pub fn vampire_nocturnus() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Play with the top card of your library revealed.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "As long as the top card of your library is black, this creature and other Vampire creatures you control get +2/+1 and have flying.",
                effect: StaticEffect::AnthemForFilterIf {
                    filter: R::Creature.and(vampire()).or(R::IsSource),
                    power: 2,
                    toughness: 1,
                    keywords: vec![Keyword::Flying],
                    condition: Predicate::EntityMatches {
                        what: Selector::TopOfLibrary {
                            who: PlayerRef::You,
                            count: Value::ONE,
                        },
                        filter: R::HasColor(Color::Black),
                    },
                    all_players: false,
                },
            },
        ],
        ..creature(
            "Vampire Nocturnus",
            cost(&[generic(1), b(), b(), b()]),
            3,
            3,
            vec![CreatureType::Vampire],
            vec![],
        )
    }
}

/// Creeping Bloodsucker — {1}{B} Creature — Vampire 1/2. "At the beginning of
/// your upkeep, this creature deals 1 damage to each opponent. You gain life
/// equal to the damage dealt this way."
pub fn creeping_bloodsucker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::DamageDealtThisResolution,
                },
            ]),
        }],
        ..creature(
            "Creeping Bloodsucker",
            cost(&[generic(1), b()]),
            1,
            2,
            vec![CreatureType::Vampire],
            vec![],
        )
    }
}

/// Carmen, Cruel Skymarcher — {3}{W}{B} Legendary Creature — Vampire Soldier
/// 2/2, flying. "Whenever a player sacrifices a permanent, put a +1/+1 counter
/// on Carmen and you gain 1 life. Whenever Carmen attacks, return up to one
/// target permanent card with mana value less than or equal to Carmen's power
/// from your graveyard to the battlefield."
pub fn carmen_cruel_skymarcher() -> CardDefinition {
    legend(CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::AnyPlayer),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                ]),
            },
            on_attack(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Move {
                    what: target_filtered(
                        R::PermanentCard
                            .and(R::InYourGraveyard)
                            .and(R::ManaValueAtMostSourcePower),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                }),
            }),
        ],
        ..creature(
            "Carmen, Cruel Skymarcher",
            cost(&[generic(3), w(), b()]),
            2,
            2,
            vec![CreatureType::Vampire, CreatureType::Soldier],
            vec![Keyword::Flying],
        )
    })
}

// ── Instants and sorceries ───────────────────────────────────────────────────

/// Bilbo's Gambit — {1}{W} Instant. "Gift a Treasure. Return target spell to
/// its owner's hand. If the gift was promised, players can't cast spells this
/// turn." (See the module residuals for the bounce and gift recipient.)
pub fn bilbos_gambit() -> CardDefinition {
    let bounce = Effect::CounterSpellToZone {
        what: target_filtered(R::IsSpellOnStack),
        zone: CounteredSpellZone::OwnerHand,
    };
    CardDefinition {
        gift: Some(Box::new(Gift {
            label: "a Treasure",
            gifted_effect: Effect::Seq(vec![
                mint(
                    PlayerRef::OpponentOf(Box::new(PlayerRef::You)),
                    Value::ONE,
                    treasure_token(),
                ),
                bounce.clone(),
                Effect::EachPlayerDoes {
                    who: PlayerRef::EachPlayer,
                    body: Box::new(Effect::PlayerCantCastMatchingThisTurn {
                        who: PlayerRef::You,
                        filter: R::Any,
                    }),
                },
            ]),
        })),
        ..spell("Bilbo's Gambit", cost(&[generic(1), w()]), false, bounce)
    }
}

/// Olivia's Wrath — {4}{B} Sorcery. "Each non-Vampire creature gets -X/-X
/// until end of turn, where X is the number of Vampires you control."
pub fn olivias_wrath() -> CardDefinition {
    let minus_x = Value::Negate(Box::new(count_your_vampires()));
    spell(
        "Olivia's Wrath",
        cost(&[generic(4), b()]),
        true,
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(vampire().negate())),
            power: minus_x.clone(),
            toughness: minus_x,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Farewell — {4}{W}{W} Sorcery. "Choose one or more — • Exile all artifacts.
/// • Exile all creatures. • Exile all enchantments. • Exile all graveyards."
pub fn farewell() -> CardDefinition {
    let exile_all = |filter: R| Effect::Exile {
        what: Selector::EachPermanent(filter),
    };
    spell(
        "Farewell",
        cost(&[generic(4), w(), w()]),
        true,
        Effect::ChooseModesCast {
            modes: vec![
                exile_all(R::Artifact),
                exile_all(R::Creature),
                exile_all(R::Enchantment),
                Effect::ExileAllGraveyards {
                    filter: None,
                    opponents_only: false,
                },
            ],
            min: 1,
            max: 4,
            allow_repeats: false,
        },
    )
}

/// Clever Concealment — {2}{W}{W} Instant, convoke. "Any number of target
/// nonland permanents you control phase out." (Up to eight target slots.)
pub fn clever_concealment() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        ..spell(
            "Clever Concealment",
            cost(&[generic(2), w(), w()]),
            false,
            Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Nonland.and(R::ControlledByYou),
                effect: Box::new(Effect::PhaseOut {
                    what: Selector::Target(0),
                    until_source_leaves: false,
                }),
            },
        )
    }
}

/// And They Shall Know No Fear — {1}{W} Instant. "Choose a creature type.
/// Creatures you control of the chosen type get +1/+0 and gain indestructible
/// until end of turn."
pub fn and_they_shall_know_no_fear() -> CardDefinition {
    spell(
        "And They Shall Know No Fear",
        cost(&[generic(1), w()]),
        false,
        Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::EachPermanent(your_chosen_type()),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(your_chosen_type()),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ])),
        },
    )
}

/// New Blood — {2}{B}{B} Sorcery. "As an additional cost to cast this spell,
/// tap an untapped Vampire you control. Gain control of target creature.
/// Change the text of that creature by replacing all instances of one creature
/// type with Vampire." (The text change is approximated — see the residuals.)
pub fn new_blood() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::TapPermanents {
            filter: vampire().and(R::Untapped),
            count: 1,
        }],
        ..spell(
            "New Blood",
            cost(&[generic(2), b(), b()]),
            true,
            Effect::Seq(vec![
                Effect::GainControl {
                    what: target_filtered(R::Creature),
                    to: None,
                    duration: Duration::Permanent,
                },
                Effect::AddCreatureTypes {
                    what: Selector::Target(0),
                    creature_types: vec![CreatureType::Vampire],
                    duration: Duration::Permanent,
                },
            ]),
        )
    }
}

/// Pact of the Serpent — {1}{B}{B} Sorcery. "Choose a creature type. Target
/// player draws X cards and loses X life, where X is the number of creatures
/// they control of the chosen type."
pub fn pact_of_the_serpent() -> CardDefinition {
    let x = Value::PermanentCountControlledByMatching(
        PlayerRef::Target(0),
        R::Creature.and(R::IsSourceChosenCreatureType),
    );
    spell(
        "Pact of the Serpent",
        cost(&[generic(1), b(), b()]),
        true,
        Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::Seq(vec![
                draw(target_filtered(R::Player), x.clone()),
                Effect::LoseLife {
                    who: Selector::Target(0),
                    amount: x,
                },
            ])),
        },
    )
}

/// Kindred Dominance — {5}{B}{B} Sorcery. "Choose a creature type. Destroy all
/// creatures that aren't of the chosen type."
pub fn kindred_dominance() -> CardDefinition {
    spell(
        "Kindred Dominance",
        cost(&[generic(5), b(), b()]),
        true,
        Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::Destroy {
                what: Selector::EachPermanent(
                    R::Creature.and(R::IsSourceChosenCreatureType.negate()),
                ),
            }),
        },
    )
}

/// Bloodline Bidding — {6}{B}{B} Sorcery, convoke. "Choose a creature type.
/// Return all creature cards of the chosen type from your graveyard to the
/// battlefield."
pub fn bloodline_bidding() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        ..spell(
            "Bloodline Bidding",
            cost(&[generic(6), b(), b()]),
            true,
            Effect::ChooseCreatureTypeThen {
                who: PlayerRef::You,
                then: Box::new(Effect::ReturnAllMatchingFromGraveyardToBattlefield {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::IsSourceChosenCreatureType),
                    sacrifice_eot: false,
                }),
            },
        )
    }
}

// ── Artifacts ────────────────────────────────────────────────────────────────

/// Orcrist, Goblin-cleaver — {3} Legendary Artifact — Equipment. "Equipped
/// creature gets +2/+2 and has trample. Whenever equipped creature deals
/// combat damage to a player, choose a creature type. Create a Treasure token
/// for each creature you control of that type." Equip {3}.
pub fn orcrist_goblin_cleaver() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Trample],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::ChooseCreatureTypeThen {
                    who: PlayerRef::You,
                    then: Box::new(mint(
                        PlayerRef::You,
                        Value::count(Selector::EachPermanent(your_chosen_type())),
                        treasure_token(),
                    )),
                },
            }],
            triggers_on_equipment: true,
            ..Default::default()
        }),
        ..permanent("Orcrist, Goblin-cleaver", cost(&[generic(3)]), CardType::Artifact)
    }
}

/// Banner of Kinship — {5} Artifact. "As this artifact enters, choose a
/// creature type. This artifact enters with a fellowship counter on it for
/// each creature you control of the chosen type. Creatures you control of the
/// chosen type get +1/+1 for each fellowship counter on this artifact."
pub fn banner_of_kinship() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::NameCreatureType {
            what: Selector::This,
        }),
        enters_with_counters: Some((
            CounterType::Fellowship,
            Value::count(Selector::EachPermanent(your_chosen_type())),
        )),
        static_abilities: vec![StaticAbility {
            description: "Creatures you control of the chosen type get +1/+1 for each fellowship counter on this artifact.",
            effect: StaticEffect::AnthemForChosenType {
                power: 1,
                toughness: 1,
                exclude_source: false,
                opponents: false,
                all_players: false,
                per_counter: Some(CounterType::Fellowship),
            },
        }],
        ..permanent("Banner of Kinship", cost(&[generic(5)]), CardType::Artifact)
    }
}

/// Idol of Oblivion — {2} Artifact. "{T}: Draw a card. Activate only if you
/// created a token this turn. {8}, {T}, Sacrifice this artifact: Create a
/// 10/10 colorless Eldrazi creature token." (The token gate reads "a token you
/// control entered this turn".)
pub fn idol_of_oblivion() -> CardDefinition {
    let eldrazi = TokenDefinition {
        name: "Eldrazi".into(),
        power: 10,
        toughness: 10,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::SelectorExists(Selector::EachPermanent(
                    R::IsToken.and(R::ControlledByYou).and(R::EnteredThisTurn),
                ))),
                effect: draw(Selector::You, Value::ONE),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(8)]),
                tap_cost: true,
                sac_cost: true,
                effect: mint_one(eldrazi),
                ..Default::default()
            },
        ],
        ..permanent("Idol of Oblivion", cost(&[generic(2)]), CardType::Artifact)
    }
}

/// Chronicle of Victory — {6} Legendary Artifact. "As Chronicle of Victory
/// enters, choose a creature type. Creatures you control of the chosen type
/// get +2/+2 and have first strike and trample. Whenever you cast a spell of
/// the chosen type, draw a card."
pub fn chronicle_of_victory() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        as_enters_effect: Some(Effect::NameCreatureType {
            what: Selector::This,
        }),
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control of the chosen type get +2/+2.",
                effect: StaticEffect::AnthemForChosenType {
                    power: 2,
                    toughness: 2,
                    exclude_source: false,
                    opponents: false,
                    all_players: false,
                    per_counter: None,
                },
            },
            StaticAbility {
                description: "Creatures you control of the chosen type have first strike.",
                effect: StaticEffect::GrantKeywordToChosenType {
                    keyword: Keyword::FirstStrike,
                    opponents: false,
                },
            },
            StaticAbility {
                description: "Creatures you control of the chosen type have trample.",
                effect: StaticEffect::GrantKeywordToChosenType {
                    keyword: Keyword::Trample,
                    opponents: false,
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::TriggerObjectIsChosenType),
            effect: draw(Selector::You, Value::ONE),
        }],
        ..permanent("Chronicle of Victory", cost(&[generic(6)]), CardType::Artifact)
    }
}

/// Glass-Cast Heart — {2}{B} Artifact. "Whenever one or more Vampires you
/// control attack, create a Blood token. {B}, {T}, Pay 1 life: Create a 1/1
/// white and black Vampire creature token with lifelink. {B}{B}, {T},
/// Sacrifice this artifact and thirteen Blood tokens: Each opponent loses 13
/// life and you gain 13 life."
pub fn glass_cast_heart() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource).with_filter(
                Predicate::AttackedWithCreatureMatching {
                    who: PlayerRef::You,
                    filter: vampire(),
                },
            ),
            effect: mint_one(blood_token()),
        }],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                tap_cost: true,
                life_cost: 1,
                effect: mint_one(wb_lifelinker()),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b(), b()]),
                tap_cost: true,
                sac_cost: true,
                sac_other_filter: Some((
                    R::IsToken.and(R::HasArtifactSubtype(ArtifactSubtype::Blood)),
                    13,
                )),
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(13),
                },
                ..Default::default()
            },
        ],
        ..permanent("Glass-Cast Heart", cost(&[generic(2), b()]), CardType::Artifact)
    }
}

/// Heirloom Blade — {3} Artifact — Equipment. "Equipped creature gets +3/+1.
/// Whenever equipped creature dies, you may reveal cards from the top of your
/// library until you reveal a creature card that shares a creature type with
/// it. Put that card into your hand and the rest on the bottom of your
/// library in a random order." Equip {1}.
///
/// The death trigger rides the equipped creature (the Skullclamp shape), so
/// "it" is the trigger's own source, read from its last-known types through
/// `SharesCreatureTypeWithSource`.
pub fn heirloom_blade() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 1,
            triggered_abilities: vec![on_dies(Effect::MayDo {
                description: "Reveal until a creature card that shares a creature type with it?"
                    .into(),
                body: Box::new(Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: R::Creature.and(R::SharesCreatureTypeWithSource),
                    to: ZoneDest::Hand(PlayerRef::You),
                    cap: Value::Const(1000),
                    life_per_revealed: 0,
                    miss_dest: crate::effect::RevealMissDest::BottomRandom,
                }),
            })],
            ..Default::default()
        }),
        ..permanent("Heirloom Blade", cost(&[generic(3)]), CardType::Artifact)
    }
}

// ── Enchantments ─────────────────────────────────────────────────────────────

/// Gleaming Splendor — {1}{W} Enchantment. "Whenever an opponent draws their
/// second card each turn, you create a Treasure token. {2}{W}: Two target
/// players each draw a card."
pub fn gleaming_splendor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SecondCardDrawnThisTurn, EventScope::OpponentControl),
            effect: mint_one(treasure_token()),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::Seq(vec![
                draw(target_filtered(R::Player), Value::ONE),
                draw(
                    Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Player,
                    },
                    Value::ONE,
                ),
            ]),
            ..Default::default()
        }],
        ..permanent("Gleaming Splendor", cost(&[generic(1), w()]), CardType::Enchantment)
    }
}

/// Necropotence — {B}{B}{B} Enchantment. "Skip your draw step. Whenever you
/// discard a card, exile that card from your graveyard. Pay 1 life: Exile the
/// top card of your library face down. Put that card into your hand at the
/// beginning of your next end step." (Face-up exile, next end step — see the
/// residuals.)
pub fn necropotence() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Skip your draw step.",
            effect: StaticEffect::ControllerSkipsDrawStep,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Exile,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TopOfLibrary {
                        who: PlayerRef::You,
                        count: Value::ONE,
                    },
                    to: ZoneDest::ExileWithSourceStamp,
                },
                // One delayed trigger per activation; each returns one card
                // still stamped as exiled with Necropotence.
                Effect::AtNextEndStep {
                    body: Box::new(Effect::Move {
                        what: Selector::CardExiledWithSource,
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..permanent("Necropotence", cost(&[b(), b(), b()]), CardType::Enchantment)
    }
}

/// Shared Animosity — {2}{R} Enchantment. "Whenever a creature you control
/// attacks, it gets +1/+0 until end of turn for each other attacking creature
/// that shares a creature type with it."
pub fn shared_animosity() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                // `SharingCreatureTypeWith` includes the attacker itself
                // whenever it has a creature type; subtract it back out.
                power: Value::NonNeg(Box::new(Value::Diff(
                    Box::new(Value::CountMatching {
                        sel: Box::new(Selector::SharingCreatureTypeWith(Box::new(
                            Selector::TriggerSource,
                        ))),
                        filter: R::IsAttacking,
                    }),
                    Box::new(Value::ONE),
                ))),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..permanent("Shared Animosity", cost(&[generic(2), r()]), CardType::Enchantment)
    }
}

/// Etchings of the Chosen — {1}{W}{B} Enchantment. "As this enchantment
/// enters, choose a creature type. Creatures you control of the chosen type
/// get +1/+1. {1}, Sacrifice a creature of the chosen type: Target creature
/// you control gains indestructible until end of turn."
pub fn etchings_of_the_chosen() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::NameCreatureType {
            what: Selector::This,
        }),
        static_abilities: vec![StaticAbility {
            description: "Creatures you control of the chosen type get +1/+1.",
            effect: StaticEffect::AnthemForChosenType {
                power: 1,
                toughness: 1,
                exclude_source: false,
                opponents: false,
                all_players: false,
                per_counter: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature.and(R::IsSourceChosenCreatureType), 1)),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..permanent(
            "Etchings of the Chosen",
            cost(&[generic(1), w(), b()]),
            CardType::Enchantment,
        )
    }
}

/// March of the Canonized — {X}{W}{W} Enchantment. "When this enchantment
/// enters, create X 1/1 white Vampire creature tokens with lifelink. At the
/// beginning of your upkeep, if your devotion to white and black is seven or
/// greater, create a 4/3 white and black Vampire Demon creature token with
/// flying."
pub fn march_of_the_canonized() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(mint(PlayerRef::You, Value::XFromCost, white_lifelinker())),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                )
                .with_filter(Predicate::ValueAtLeast(
                    Value::DevotionTo(vec![Color::White, Color::Black]),
                    Value::Const(7),
                )),
                effect: mint_one(vampire_demon(false)),
            },
        ],
        ..permanent(
            "March of the Canonized",
            cost(&[x(), w(), w()]),
            CardType::Enchantment,
        )
    }
}

/// Renewed Solidarity — {2}{W} Enchantment. "As this enchantment enters,
/// choose a creature type. Creatures you control of the chosen type get
/// +1/+0. At the beginning of your end step, for each token you control of
/// the chosen type that entered this turn, create a token that's a copy of
/// it."
pub fn renewed_solidarity() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::NameCreatureType {
            what: Selector::This,
        }),
        static_abilities: vec![StaticAbility {
            description: "Creatures you control of the chosen type get +1/+0.",
            effect: StaticEffect::AnthemForChosenType {
                power: 1,
                toughness: 0,
                exclude_source: false,
                opponents: false,
                all_players: false,
                per_counter: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    R::IsToken
                        .and(R::ControlledByYou)
                        .and(R::IsSourceChosenCreatureType)
                        .and(R::EnteredThisTurn),
                ),
                body: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::TriggerSource,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                }),
            },
        }],
        ..permanent("Renewed Solidarity", cost(&[generic(2), w()]), CardType::Enchantment)
    }
}

// ── Planeswalkers ────────────────────────────────────────────────────────────

fn sorin(name: &'static str, loyalty: u32, abilities: Vec<LoyaltyAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Sorin],
            ..Default::default()
        },
        base_loyalty: loyalty,
        loyalty_abilities: abilities,
        ..Default::default()
    }
}

fn loyalty(loyalty_cost: i32, effect: Effect) -> LoyaltyAbility {
    LoyaltyAbility {
        loyalty_cost,
        effect,
        ..Default::default()
    }
}

/// Sorin, Lord of Innistrad — {2}{W}{B} Legendary Planeswalker — Sorin, 3.
/// "+1: Create a 1/1 black Vampire creature token with lifelink. −2: You get
/// an emblem with 'Creatures you control get +1/+0.' −6: Destroy up to three
/// target creatures and/or other planeswalkers. Return each card put into a
/// graveyard this way to the battlefield under your control."
pub fn sorin_lord_of_innistrad() -> CardDefinition {
    sorin(
        "Sorin, Lord of Innistrad",
        3,
        vec![
            loyalty(
                1,
                mint_one(vampire_token(
                    1,
                    1,
                    vec![Color::Black],
                    vec![],
                    vec![Keyword::Lifelink],
                    false,
                )),
            ),
            loyalty(
                -2,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Sorin, Lord of Innistrad".into(),
                    triggered: vec![],
                    statics: vec![StaticAbility {
                        description: "Creatures you control get +1/+0.",
                        effect: StaticEffect::PumpPT {
                            applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                            power: 1,
                            toughness: 0,
                        },
                    }],
                },
            ),
            loyalty(
                -6,
                Effect::ApplyToTargets {
                    max_targets: 3,
                    min_targets: 0,
                    filter: R::Creature.or(R::Planeswalker.and(R::OtherThanSource)),
                    effect: Box::new(Effect::Seq(vec![
                        Effect::Destroy {
                            what: Selector::Target(0),
                        },
                        Effect::Move {
                            what: Selector::DestroyedThisResolution { filter: R::Any },
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::You,
                                tapped: false,
                            },
                        },
                    ])),
                },
            ),
        ],
    )
}

/// Sorin, Solemn Visitor — {2}{W}{B} Legendary Planeswalker — Sorin, 4. "+1:
/// Until your next turn, creatures you control get +1/+0 and gain lifelink.
/// −2: Create a 2/2 black Vampire creature token with flying. −6: You get an
/// emblem with 'At the beginning of each opponent's upkeep, that player
/// sacrifices a creature of their choice.'"
pub fn sorin_solemn_visitor() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    sorin(
        "Sorin, Solemn Visitor",
        4,
        vec![
            loyalty(
                1,
                Effect::Seq(vec![
                    Effect::PumpPT {
                        what: yours(),
                        power: Value::ONE,
                        toughness: Value::Const(0),
                        duration: Duration::UntilNextTurn,
                    },
                    Effect::GrantKeyword {
                        what: yours(),
                        keyword: Keyword::Lifelink,
                        duration: Duration::UntilNextTurn,
                    },
                ]),
            ),
            loyalty(-2, mint_one(black_flyer())),
            loyalty(
                -6,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Sorin, Solemn Visitor".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(
                            EventKind::StepBegins(TurnStep::Upkeep),
                            EventScope::OpponentControl,
                        ),
                        effect: Effect::Sacrifice {
                            who: Selector::Player(PlayerRef::ActivePlayer),
                            count: Value::ONE,
                            filter: R::Creature,
                        },
                    }],
                    statics: vec![],
                },
            ),
        ],
    )
}

// ── Lands ────────────────────────────────────────────────────────────────────

fn land(name: &'static str) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        ..Default::default()
    }
}

/// Battlebond "crowd" land — the body is `sets::cmdr::crowd_land`, where the
/// other seven of the ten-card cycle live: what they share is a player count,
/// not a set.
use super::super::cmdr::crowd_land;

/// Vault of Champions — Land. Enters tapped unless you have two or more
/// opponents. {T}: Add {W} or {B}.
pub fn vault_of_champions() -> CardDefinition {
    crowd_land("Vault of Champions", Color::White, Color::Black)
}

/// Luxury Suite — Land. Enters tapped unless you have two or more opponents.
/// {T}: Add {B} or {R}.
pub fn luxury_suite() -> CardDefinition {
    crowd_land("Luxury Suite", Color::Black, Color::Red)
}

/// Spectator Seating — Land. Enters tapped unless you have two or more
/// opponents. {T}: Add {R} or {W}.
pub fn spectator_seating() -> CardDefinition {
    crowd_land("Spectator Seating", Color::Red, Color::White)
}

/// Haunted Ridge — Land. "This land enters tapped unless you control two or
/// more other lands. {T}: Add {B} or {R}." (The MID/VOW slow-land frame.)
pub fn haunted_ridge() -> CardDefinition {
    super::lands::slow_land("Haunted Ridge", Color::Black, Color::Red)
}

/// Foreboding Ruins — Land. "As this land enters, you may reveal a Swamp or
/// Mountain card from your hand. If you don't, this land enters tapped. {T}:
/// Add {B} or {R}."
pub fn foreboding_ruins() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        triggered_abilities: vec![etb(Effect::IfRevealFromHand {
            filter: R::HasLandType(LandType::Swamp).or(R::HasLandType(LandType::Mountain)),
            then: Box::new(Effect::Noop),
            else_: Box::new(Effect::Tap {
                what: Selector::This,
            }),
        })],
        activated_abilities: vec![
            super::super::tap_add(Color::Black),
            super::super::tap_add(Color::Red),
        ],
        ..land("Foreboding Ruins")
    }
}

/// Fetid Heath — Land. "{T}: Add {C}. {W/B}, {T}: Add {W}{W}, {W}{B}, or
/// {B}{B}." (The Shadowmoor filter land: one activation per output.)
pub fn fetid_heath() -> CardDefinition {
    let filter = |out: Vec<Color>| ActivatedAbility {
        mana_cost: ManaCost {
            symbols: vec![ManaSymbol::Hybrid(Color::White, Color::Black)],
        },
        tap_cost: true,
        effect: add_colors(out),
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            super::super::tap_add_colorless(),
            filter(vec![Color::White, Color::White]),
            filter(vec![Color::White, Color::Black]),
            filter(vec![Color::Black, Color::Black]),
        ],
        ..land("Fetid Heath")
    }
}

/// Vault of the Archangel — Land. "{T}: Add {C}. {2}{W}{B}, {T}: Creatures you
/// control gain deathtouch and lifelink until end of turn."
pub fn vault_of_the_archangel() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        activated_abilities: vec![
            super::super::tap_add_colorless(),
            tapper(
                cost(&[generic(2), w(), b()]),
                Effect::GrantKeywords {
                    what: yours(),
                    keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
                    duration: Duration::EndOfTurn,
                },
            ),
        ],
        ..land("Vault of the Archangel")
    }
}

/// Minas Tirith — Legendary Land. "Minas Tirith enters tapped unless you
/// control a legendary creature. {T}: Add {W}. {1}{W}, {T}: Draw a card.
/// Activate only if you attacked with two or more creatures this turn."
pub fn minas_tirith() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Minas Tirith enters tapped unless you control a legendary creature.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature
                        .and(R::HasSupertype(Supertype::Legendary))
                        .and(R::ControlledByYou),
                )),
            },
        }],
        activated_abilities: vec![
            super::super::tap_add(Color::White),
            ActivatedAbility {
                condition: Some(Predicate::ValueAtLeast(
                    Value::CreaturesAttackedWithThisTurn(PlayerRef::You),
                    Value::Const(2),
                )),
                ..tapper(cost(&[generic(1), w()]), draw(Selector::You, Value::ONE))
            },
        ],
        ..land("Minas Tirith")
    }
}

/// Westvale Abbey // Ormendahl, Profane Prince — Land. "{T}: Add {C}. {5},
/// {T}, Pay 1 life: Create a 1/1 white and black Human Cleric creature token.
/// {5}, {T}, Sacrifice five creatures: Transform this land, then untap it."
/// Back: Legendary Creature — Demon 9/7, flying, lifelink, indestructible,
/// haste.
pub fn westvale_abbey() -> CardDefinition {
    let cleric = TokenDefinition {
        name: "Human Cleric".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        ..Default::default()
    };
    // Ormendahl carries a black color indicator (CR 204), which is also what
    // puts {B} in the card's color identity.
    let mut ormendahl = legend(creature(
        "Ormendahl, Profane Prince",
        ManaCost::default(),
        9,
        7,
        vec![CreatureType::Demon],
        vec![
            Keyword::Flying,
            Keyword::Lifelink,
            Keyword::Indestructible,
            Keyword::Haste,
        ],
    ));
    ormendahl.color_indicator = vec![Color::Black];
    CardDefinition {
        activated_abilities: vec![
            super::super::tap_add_colorless(),
            ActivatedAbility {
                life_cost: 1,
                ..tapper(cost(&[generic(5)]), mint_one(cleric))
            },
            ActivatedAbility {
                sac_other_filter: Some((R::Creature, 5)),
                ..tapper(
                    cost(&[generic(5)]),
                    Effect::Seq(vec![
                        Effect::Transform {
                            what: Selector::This,
                        },
                        Effect::Untap {
                            what: Selector::This,
                            up_to: None,
                        },
                    ]),
                )
            },
        ],
        back_face: Some(Box::new(ormendahl)),
        ..land("Westvale Abbey")
    }
}

/// Accursed Duneyard — Land. "{T}: Add {C}. {2}, {T}: Regenerate target Shade,
/// Skeleton, Specter, Spirit, Vampire, Wraith, or Zombie."
pub fn accursed_duneyard() -> CardDefinition {
    let undead = [
        CreatureType::Shade,
        CreatureType::Skeleton,
        CreatureType::Specter,
        CreatureType::Spirit,
        CreatureType::Vampire,
        CreatureType::Wraith,
        CreatureType::Zombie,
    ]
    .into_iter()
    .map(R::HasCreatureType)
    .reduce(|a, b| a.or(b))
    .unwrap();
    CardDefinition {
        activated_abilities: vec![
            super::super::tap_add_colorless(),
            tapper(
                cost(&[generic(2)]),
                Effect::Regenerate {
                    what: target_filtered(R::Creature.and(undead)),
                },
            ),
        ],
        ..land("Accursed Duneyard")
    }
}
