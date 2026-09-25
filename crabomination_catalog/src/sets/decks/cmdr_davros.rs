//! Commander: the cards the **Masters of Evil** precon (WHO, Davros, Dalek
//! Creator) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_davros.rs`.
//!
//! Residuals (each also on its card):
//! - **Day of the Moon** — only the latest chosen name is goaded.
//! - **Doomsday Confluence** — each of the X modes is chosen as it resolves.
//! - **Genesis of the Daleks** — chapter IV counts the Daleks it destroyed,
//!   not every Dalek that died this turn.
//! - **Rassilon, the War President** — noncreature spells cast from exile
//!   don't have conspire.
//! - **The Master, Multiplied** — your triggered abilities can still make you
//!   sacrifice or exile your creature tokens.
//! - **The Sound of Drums** — the enchanted creature's combat damage isn't
//!   doubled.
//! - **The Toymaker's Trap** — the guess is a match or a miss; the life lost
//!   is the chosen number's worth, and numbers may repeat.
//! - **Time Reaper** — the life is gained whether or not a card moved.
//! - **Vislor Turlough** — goaded for the rest of the game, not only while
//!   the opponent controls it.
//! - **Weeping Angel** — its combat damage to a creature is dealt, then the
//!   creature is shuffled away.
//! - **Zygon Infiltrator** — the copy lasts until end of turn, not while the
//!   target stays tapped.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EntersAsCopy, EventKind, EventScope, EventSpec, Keyword, LandType, MayPlayDuration,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{afflict, battalion, etb, investigate, myriad, on_attack, on_cast, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, r, u, x, Color, ManaCost};
use crate::sets::tap_add_any_color;
use crabomination_base::tokens::clue_token;

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

fn artifact_creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Artifact, CardType::Creature], ..creature(name, mana, types, p, t) }
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

fn vehicle(name: &'static str, mana: ManaCost, p: i32, t: i32, crew: u32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: p,
        toughness: t,
        keywords: vec![Keyword::Crew(crew)],
        ..Default::default()
    }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn dalek() -> R {
    R::HasCreatureType(CreatureType::Dalek)
}

fn artifact_creatures() -> R {
    R::Artifact.and(R::Creature)
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

/// The card's controller, from inside a villainous-choice option (whose
/// "you" is the chooser, CR 701.55).
fn me() -> PlayerRef {
    PlayerRef::ControllerOf(Box::new(Selector::This))
}

/// Run `body` as the card's controller, from inside a villainous choice.
fn as_me(body: Effect) -> Effect {
    Effect::EachPlayerDoes { who: me(), body: Box::new(body) }
}

fn villainous(who: Selector, a: Effect, b: Effect) -> Effect {
    Effect::VillainousChoice { who, option_a: Box::new(a), option_b: Box::new(b) }
}

fn step(s: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(s), EventScope::YourControl), effect }
}

/// A 3/3 black Dalek artifact creature with menace.
fn dalek_token() -> TokenDefinition {
    TokenDefinition {
        name: "Dalek".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dalek], ..Default::default() },
        keywords: vec![Keyword::Menace],
        ..Default::default()
    }
}

fn make_dalek(who: PlayerRef, n: Value) -> Effect {
    Effect::CreateToken { who, count: n, definition: Arc::new(dalek_token()) }
}

/// "Whenever an opponent casts a creature spell, this isn't a creature until
/// end of turn" (Weeping Angel and its tokens).
fn quantum_locked() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(trigger_is(R::Creature)),
        effect: Effect::LoseCardTypeUntilEot { what: Selector::This, card_type: CardType::Creature },
    }
}

/// Turn face down as a 2/2 Cyberman artifact creature under your control.
fn cyberman(what: Selector, tapped: bool) -> Effect {
    Effect::PutFaceDownAsCyberman { what, tapped }
}

fn shuffle_into_owners_library(what: Selector) -> Effect {
    Effect::Move {
        what,
        to: ZoneDest::Library { who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))), pos: LibraryPosition::Shuffled },
    }
}

/// "For each opponent, [effect] up to one target [filter] that player
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

/// Ashad, the Lone Cyberman — your first nonlegendary artifact spell each
/// turn has casualty 2; sacrificing another creature grows it.
pub fn ashad_the_lone_cyberman() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "The first nonlegendary artifact spell you cast each turn has casualty 2.",
            effect: StaticEffect::FirstNonlegendaryArtifactSpellHasCasualty(2),
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl)
                .with_filter(Predicate::Not(Box::new(Predicate::TriggerSourceIsSelf))),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..legend(artifact_creature(
            "Ashad, the Lone Cyberman",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Cyberman],
            3,
            3,
        ))
    }
}

/// Auton Soldier — may enter as a copy of any creature, except not
/// legendary, an artifact, and with myriad.
pub fn auton_soldier() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            non_legendary: true,
            extra_card_types: vec![CardType::Artifact],
            extra_triggered: vec![myriad()],
            ..Default::default()
        }),
        ..artifact_creature(
            "Auton Soldier",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Alien, CreatureType::Soldier],
            0,
            0,
        )
    }
}

/// Blink — I, III: a target creature's owner shuffles it into their library
/// and investigates. II, IV: a 2/2 first strike, vigilance Alien Angel that
/// stops being a creature when an opponent casts one.
pub fn blink() -> CardDefinition {
    let angel = TokenDefinition {
        name: "Alien Angel".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Alien, CreatureType::Angel], ..Default::default() },
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance],
        triggered_abilities: vec![quantum_locked()],
        ..Default::default()
    };
    let shuffle = || {
        Effect::Seq(vec![
            shuffle_into_owners_library(target_filtered(R::Creature)),
            Effect::CreateToken {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                count: Value::ONE,
                definition: Arc::new(clue_token()),
            },
        ])
    };
    let angel_token = || Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(angel.clone()) };
    CardDefinition {
        saga_chapters: vec![(1, shuffle()), (2, angel_token()), (3, shuffle()), (4, angel_token())],
        ..saga("Blink", cost(&[generic(2), u(), b()]), vec![])
    }
}

/// Clockwork Droid — may exert as it attacks: then it can't be blocked this
/// turn and you scry 1.
pub fn clockwork_droid() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Exert],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Exerted, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
                Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
            ]),
        }],
        ..artifact_creature("Clockwork Droid", cost(&[generic(2)]), vec![CreatureType::Robot], 3, 1)
    }
}

/// Cult of Skaro — attacking, one at random: counters on your artifact
/// creatures, draw two, a Dalek, or each opponent loses 4.
pub fn cult_of_skaro() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::ChooseModeAtRandom(vec![
            Effect::AddCounter {
                what: Selector::EachPermanent(yours(artifact_creatures())),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            draw(2),
            make_dalek(PlayerRef::You, Value::ONE),
            Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(4) },
        ]))],
        ..legend(artifact_creature(
            "Cult of Skaro",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Dalek],
            4,
            4,
        ))
    }
}

/// Cyber Conversion — turn target creature face down as a 2/2 Cyberman
/// artifact creature.
pub fn cyber_conversion() -> CardDefinition {
    spell("Cyber Conversion", cost(&[u(), u()]), CardType::Instant, cyberman(target_filtered(R::Creature), false))
}

/// Cyberman Patrol — your artifact creatures have afflict 3.
pub fn cyberman_patrol() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Artifact creatures you control have afflict 3.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: yours(artifact_creatures()),
                ability: Box::new(afflict(3)),
            },
        }],
        ..artifact_creature("Cyberman Patrol", cost(&[generic(2)]), vec![CreatureType::Cyberman], 2, 2)
    }
}

/// Cybermat — skulk; unblocked, +X/+0 per attacking artifact creature.
pub fn cybermat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Skulk],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::CountOf(Box::new(Selector::EachPermanent(artifact_creatures().and(R::IsAttacking)))),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..artifact_creature("Cybermat", cost(&[generic(2)]), vec![CreatureType::Robot], 2, 1)
    }
}

/// Cybermen Squadron — your nonlegendary artifact creatures have myriad.
pub fn cybermen_squadron() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Nonlegendary artifact creatures you control have myriad.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: yours(artifact_creatures()).and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary)))),
                ability: Box::new(myriad()),
            },
        }],
        ..artifact_creature("Cybermen Squadron", cost(&[generic(7)]), vec![CreatureType::Cyberman], 5, 5)
    }
}

/// Cybership — flying Vehicle (crew 4); its combat damage puts the top two
/// cards of that player's library onto the battlefield face down under your
/// control as 2/2 Cybermen.
pub fn cybership() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Crew(4)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: cyberman(
                Selector::TopOfLibrary { who: PlayerRef::TriggerEventPlayer, count: Value::Const(2) },
                false,
            ),
        }],
        ..vehicle("Cybership", cost(&[generic(6)]), 8, 8, 4)
    }
}

/// Dalek Drone — flying, menace; entering destroys a target creature an
/// opponent controls and that player loses 3.
pub fn dalek_drone() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Menace],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(3),
            },
        ]))],
        ..artifact_creature("Dalek Drone", cost(&[generic(3), b(), b()]), vec![CreatureType::Dalek], 3, 3)
    }
}

/// Davros, Dalek Creator — menace; your end step makes a Dalek if an
/// opponent lost 3+ life this turn, then each such opponent faces a
/// villainous choice: you draw a card, or they discard one.
pub fn davros_dalek_creator() -> CardDefinition {
    let bled = |who: PlayerRef| Predicate::ValueAtLeast(Value::LifeLostThisTurn(who), Value::Const(3));
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![step(
            TurnStep::End,
            Effect::Seq(vec![
                Effect::If {
                    cond: bled(PlayerRef::EachOpponent),
                    then: Box::new(make_dalek(PlayerRef::You, Value::ONE)),
                    else_: Box::new(Effect::Noop),
                },
                Effect::ForEachOpponent {
                    body: Box::new(Effect::If {
                        cond: bled(PlayerRef::Triggerer),
                        then: Box::new(villainous(
                            Selector::Player(PlayerRef::Triggerer),
                            Effect::Draw { who: Selector::Player(me()), amount: Value::ONE },
                            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                        )),
                        else_: Box::new(Effect::Noop),
                    }),
                },
            ]),
        )],
        ..legend(artifact_creature(
            "Davros, Dalek Creator",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Alien, CreatureType::Scientist],
            3,
            4,
        ))
    }
}

/// Day of the Moon — each chapter names a creature card and goads every
/// creature with that name. Residual: only the latest name is goaded.
pub fn day_of_the_moon() -> CardDefinition {
    let chapter = || {
        Effect::Seq(vec![
            Effect::NameCard { what: Selector::This, restrict_to: Some(R::Creature) },
            Effect::Goad { what: Selector::EachPermanent(R::Creature.and(R::NamedBySource)) },
        ])
    };
    saga("Day of the Moon", cost(&[generic(2), r()]), vec![(1, chapter()), (2, chapter()), (3, chapter())])
}

/// Death in Heaven — I, II: target player mills two, then exiles their
/// graveyard. III: every creature card exiled with it becomes a face-down
/// 2/2 Cyberman under your control.
pub fn death_in_heaven() -> CardDefinition {
    let harvest = || {
        Effect::Seq(vec![
            Effect::Mill { who: target_filtered(R::Player), amount: Value::Const(2) },
            Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::Target(0),
                    zone: Zone::Graveyard,
                    filter: R::Any,
                },
                to: ZoneDest::ExileWithSourceStamp,
            },
        ])
    };
    saga(
        "Death in Heaven",
        cost(&[generic(3), b()]),
        vec![
            (1, harvest()),
            (2, harvest()),
            (3, cyberman(Selector::MatchingAmong { inner: Box::new(Selector::CardExiledWithSource), filter: R::Creature }, false)),
        ],
    )
}

/// Delete — X damage to each nonartifact creature and each player.
pub fn delete() -> CardDefinition {
    spell(
        "Delete",
        cost(&[x(), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::Artifact)))),
                amount: Value::XFromCost,
            },
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::XFromCost },
        ]),
    )
}

/// Don't Blink — this turn, creatures that would enter from exile (or after
/// being cast from exile) are shuffled into their owners' libraries instead;
/// cycling {2}.
pub fn dont_blink() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..spell("Don't Blink", cost(&[generic(1), u()]), CardType::Instant, Effect::CreaturesFromExileShuffleThisTurn)
    }
}

/// Doomsday Confluence — choose X, repeats allowed: each player sacrifices a
/// nonartifact creature; a Dalek; each opponent discards. Residual: each
/// mode is chosen as it resolves.
pub fn doomsday_confluence() -> CardDefinition {
    spell(
        "Doomsday Confluence",
        cost(&[x(), x(), b()]),
        CardType::Sorcery,
        Effect::Repeat {
            count: Value::XFromCost,
            body: Box::new(Effect::ChooseMode(vec![
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    count: Value::ONE,
                    filter: R::Creature.and(R::Not(Box::new(R::Artifact))),
                },
                make_dalek(PlayerRef::You, Value::ONE),
                Effect::Discard { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE, random: false },
            ])),
        },
    )
}

/// Ensnared by the Mara — each opponent faces a villainous choice: they
/// exile until a nonland card you may cast free, or they exile their top four
/// and take damage equal to the total mana value.
pub fn ensnared_by_the_mara() -> CardDefinition {
    spell(
        "Ensnared by the Mara",
        cost(&[generic(2), r(), r()]),
        CardType::Sorcery,
        Effect::ForEachOpponent {
            body: Box::new(villainous(
                Selector::Player(PlayerRef::Triggerer),
                as_me(Effect::ExileTopUntilNonlandMayPlay {
                    who: PlayerRef::Triggerer,
                    duration: MayPlayDuration::EndOfThisTurn,
                    free: true,
                    hand_unless_mv_below: None,
                    grant_to_exiling_player: false,
                }),
                Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::Const(4) },
                        to: ZoneDest::Exile,
                    },
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::You),
                        amount: Value::TotalManaValueOf(Box::new(Selector::ExiledThisResolution { filter: R::Any })),
                    },
                ]),
            )),
        },
    )
}

/// Exterminate! — replicate: tap an untapped Dalek; destroy target creature,
/// its controller loses 3.
pub fn exterminate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ReplicateTap(Box::new(dalek()))],
        ..spell(
            "Exterminate!",
            cost(&[generic(2), b()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Creature) },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(3),
                },
            ]),
        )
    }
}

/// Genesis of the Daleks — I–III: a Dalek per lore counter. IV: target
/// opponent chooses — destroy all Daleks and each of your opponents loses
/// their total power, or destroy all non-Daleks. Residual: IV counts the
/// Daleks it destroyed.
pub fn genesis_of_the_daleks() -> CardDefinition {
    let muster = || make_dalek(PlayerRef::You, Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Lore });
    saga(
        "Genesis of the Daleks",
        cost(&[generic(4), b(), b()]),
        vec![
            (1, muster()),
            (2, muster()),
            (3, muster()),
            (
                4,
                villainous(
                    target_filtered(R::Player.and(R::OpponentPlayer)),
                    Effect::Seq(vec![
                        Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(dalek())) },
                        Effect::ForEach {
                            selector: Selector::DestroyedThisResolution { filter: dalek() },
                            body: Box::new(Effect::LoseLife {
                                who: Selector::Player(PlayerRef::OpponentOf(Box::new(me()))),
                                amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                            }),
                        },
                    ]),
                    Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(dalek())))) },
                ),
            ),
        ],
    )
}

/// Great Intelligence's Plan — draw three; target opponent chooses: they
/// discard three, or you may cast a spell from your hand free.
pub fn great_intelligences_plan() -> CardDefinition {
    spell(
        "Great Intelligence's Plan",
        cost(&[generic(4), u(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            draw(3),
            villainous(
                target_filtered(R::Player.and(R::OpponentPlayer)),
                Effect::Discard { who: Selector::You, amount: Value::Const(3), random: false },
                as_me(Effect::MayCastFromHandFreeMatching {
                    filter: R::Nonland,
                    max_mv: Value::Const(99),
                    else_: Box::new(Effect::Noop),
                }),
            ),
        ]),
    )
}

/// Hunted by The Family — up to four target creatures you don't control:
/// each controller chooses — it becomes a 1/1 white Human with no abilities,
/// or you create a token copy of it.
pub fn hunted_by_the_family() -> CardDefinition {
    spell(
        "Hunted by The Family",
        cost(&[generic(5), u(), u()]),
        CardType::Sorcery,
        Effect::ApplyToTargets {
            max_targets: 4,
            min_targets: 0,
            filter: R::Creature.and(R::Not(Box::new(R::ControlledByYou))),
            effect: Box::new(villainous(
                Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                Effect::Seq(vec![
                    Effect::ResetCreature {
                        what: Selector::Target(0),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        creature_types: vec![CreatureType::Human],
                        duration: Duration::Permanent,
                    },
                    Effect::BecomeColor {
                        what: Selector::Target(0),
                        colors: vec![Color::White],
                        duration: Duration::Permanent,
                        additive: false,
                    },
                ]),
                Effect::CreateTokenCopyOf {
                    who: me(),
                    count: Value::ONE,
                    source: Selector::Target(0),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
            )),
        },
    )
}

/// Laser Screwdriver — any color; {1},{T}: tap target artifact; {2},{T}:
/// surveil 1; {3},{T}: goad target creature.
pub fn laser_screwdriver() -> CardDefinition {
    let tap_for = |n: u32, effect: Effect| ActivatedAbility {
        tap_cost: true,
        mana_cost: cost(&[generic(n)]),
        effect,
        ..Default::default()
    };
    CardDefinition {
        name: "Laser Screwdriver",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            tap_add_any_color(),
            tap_for(1, Effect::Tap { what: target_filtered(R::Artifact) }),
            tap_for(2, Effect::Surveil { who: PlayerRef::You, amount: Value::ONE }),
            tap_for(3, Effect::Goad { what: target_filtered(R::Creature) }),
        ],
        ..Default::default()
    }
}

/// Midnight Crusader Shuttle — attacking, the defending player chooses: they
/// sacrifice a creature, or you take their best creature until end of turn,
/// tapped and attacking them. Crew 2.
pub fn midnight_crusader_shuttle() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(villainous(
            Selector::Player(PlayerRef::DefendingPlayer),
            Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Creature },
            as_me(Effect::ForEach {
                selector: Selector::TakeGreatestPower {
                    inner: Box::new(Selector::ControlledBy { who: PlayerRef::DefendingPlayer, filter: R::Creature }),
                    count: Box::new(Value::ONE),
                },
                body: Box::new(Effect::Seq(vec![
                    Effect::GainControl { what: Selector::TriggerSource, to: None, duration: Duration::EndOfTurn },
                    Effect::Tap { what: Selector::TriggerSource },
                    Effect::JoinCombatAttacking { what: Selector::TriggerSource },
                ])),
            }),
        ))],
        ..vehicle("Midnight Crusader Shuttle", cost(&[generic(4)]), 3, 4, 2)
    }
}

/// Missy — another nonartifact creature dying returns face down under your
/// control, tapped, as a 2/2 Cyberman; your end step gives each opponent a
/// villainous choice: your artifact creatures each deal 1 to them, or you
/// draw a card and chaos ensues.
pub fn missy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                    Predicate::Not(Box::new(Predicate::TriggerSourceIsSelf)),
                    trigger_is(R::Not(Box::new(R::Artifact))),
                ])),
                effect: cyberman(Selector::TriggerSource, true),
            },
            step(
                TurnStep::End,
                villainous(
                    Selector::Player(PlayerRef::EachOpponent),
                    Effect::ForEach {
                        selector: Selector::ControlledBy { who: me(), filter: artifact_creatures() },
                        body: Box::new(Effect::DealDamageFrom {
                            source: Selector::TriggerSource,
                            to: Selector::Player(PlayerRef::You),
                            amount: Value::ONE,
                        }),
                    },
                    as_me(Effect::Seq(vec![draw(1), Effect::ChaosEnsues { who: PlayerRef::You }])),
                ),
            ),
        ],
        ..legend(creature(
            "Missy",
            cost(&[generic(3), u(), b(), r()]),
            vec![CreatureType::TimeLord, CreatureType::Rogue],
            4,
            5,
        ))
    }
}

/// Rassilon, the War President — your upkeep loses 2 life and exiles your
/// top card, playable while it stays exiled. Residual: noncreature spells
/// cast from exile don't have conspire.
pub fn rassilon_the_war_president() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![step(
            TurnStep::Upkeep,
            Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: MayPlayDuration::WhileExiled,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
            ]),
        )],
        ..legend(creature(
            "Rassilon, the War President",
            cost(&[generic(3), u(), b()]),
            vec![CreatureType::TimeLord, CreatureType::Noble],
            3,
            4,
        ))
    }
}

/// Renegade Silent — your end step goads up to one target creature you don't
/// control, grows it, and phases it out.
pub fn renegade_silent() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![step(
            TurnStep::End,
            Effect::Seq(vec![
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature.and(R::Not(Box::new(R::ControlledByYou))),
                    effect: Box::new(Effect::Goad { what: Selector::Target(0) }),
                },
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::PhaseOut { what: Selector::This, until_source_leaves: false },
            ]),
        )],
        ..creature("Renegade Silent", cost(&[generic(3), u()]), vec![CreatureType::Alien, CreatureType::Horror], 3, 3)
    }
}

/// Sontaran General — trample, haste; battalion goads up to one target
/// creature of each opponent, and those can't block this turn.
pub fn sontaran_general() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![battalion(per_opponent(
            R::Creature,
            Effect::Seq(vec![
                Effect::Goad { what: Selector::Target(0) },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::CantBlock, duration: Duration::EndOfTurn },
            ]),
        ))],
        ..creature("Sontaran General", cost(&[generic(4), r()]), vec![CreatureType::Alien, CreatureType::Soldier], 5, 5)
    }
}

/// Sycorax Commander — first strike, haste; entering, each opponent chooses:
/// discard their hand and draw that many minus one, or take damage equal to
/// their hand size.
pub fn sycorax_commander() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        triggered_abilities: vec![etb(villainous(
            Selector::Player(PlayerRef::EachOpponent),
            Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Diff(
                        Box::new(Value::CountOf(Box::new(Selector::DiscardedThisResolution { filter: R::Any }))),
                        Box::new(Value::ONE),
                    ),
                },
            ]),
            Effect::DealDamageFrom {
                source: Selector::This,
                to: Selector::Player(PlayerRef::You),
                amount: Value::HandSizeOf(PlayerRef::You),
            },
        ))],
        ..creature(
            "Sycorax Commander",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Alien, CreatureType::Soldier],
            4,
            2,
        )
    }
}

/// The Beast, Deathless Prince — casting it steals a target creature until
/// end of turn, untapped with menace and haste; it enters tapped with six
/// stun counters; a creature dealing combat damage to its owner untaps it
/// and draws you a card.
pub fn the_beast_deathless_prince() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Stun, Value::Const(6))),
        static_abilities: vec![crate::sets::enters_tapped()],
        triggered_abilities: vec![
            on_cast(Effect::Seq(vec![
                Effect::GainControl { what: target_filtered(R::Creature), to: None, duration: Duration::EndOfTurn },
                Effect::Untap { what: Selector::Target(0), up_to: None },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Menace, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer).with_filter(
                    Predicate::SamePlayer(
                        PlayerRef::TriggerEventPlayer,
                        PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)),
                    ),
                ),
                effect: Effect::Seq(vec![Effect::Untap { what: Selector::This, up_to: None }, draw(1)]),
            },
        ],
        ..legend(creature(
            "The Beast, Deathless Prince",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Demon],
            6,
            6,
        ))
    }
}

/// The Cyber-Controller — entering, each opponent mills X and every creature
/// card milled joins you face down as a 2/2 Cyberman; your other artifact
/// creatures get +1/+1.
pub fn the_cyber_controller() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other artifact creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(yours(artifact_creatures()).and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::XFromCost },
            cyberman(Selector::MatchingAmong { inner: Box::new(Selector::LastMoved), filter: R::Creature }, false),
        ]))],
        ..legend(artifact_creature(
            "The Cyber-Controller",
            cost(&[x(), u(), u(), b()]),
            vec![CreatureType::Cyberman],
            3,
            3,
        ))
    }
}

/// The Dalek Emperor — affinity for Daleks; your other Daleks have haste;
/// your combats give each opponent a villainous choice: sacrifice a creature,
/// or you make a Dalek.
pub fn the_dalek_emperor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Affinity for Daleks.",
                effect: StaticEffect::SelfCostReducedPerPermanentMatching { filter: yours(dalek()), per: 1 },
            },
            StaticAbility {
                description: "Other Daleks you control have haste.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(yours(dalek()).and(R::OtherThanSource)),
                    keyword: Keyword::Haste,
                },
            },
        ],
        triggered_abilities: vec![step(
            TurnStep::BeginCombat,
            villainous(
                Selector::Player(PlayerRef::EachOpponent),
                Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Creature },
                make_dalek(me(), Value::ONE),
            ),
        )],
        ..legend(artifact_creature(
            "The Dalek Emperor",
            cost(&[generic(5), b(), r()]),
            vec![CreatureType::Dalek],
            6,
            6,
        ))
    }
}

/// The Flood of Mars — islandwalk; attacking, a flood counter on another
/// target creature or land: a creature becomes a copy of it, a land becomes
/// an Island too.
pub fn the_flood_of_mars() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature.or(R::Land).and(R::OtherThanSource)),
                kind: CounterType::Flood,
                amount: Value::ONE,
            },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::Creature },
                then: Box::new(Effect::BecomeCopyOf {
                    what: Selector::Target(0),
                    source: Selector::This,
                    extra_creature_types: vec![],
                    keep_own_triggered: false,
                    keep_own_activated: false,
                }),
                else_: Box::new(Effect::GainLandType {
                    what: Selector::Target(0),
                    land_type: LandType::Island,
                    duration: Duration::Permanent,
                }),
            },
        ]))],
        ..creature(
            "The Flood of Mars",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Alien, CreatureType::Zombie, CreatureType::Horror],
            3,
            3,
        )
    }
}

/// The Master, Formed Anew — casting it may exile a creature you control with
/// a takeover counter; it may enter as a copy of a creature card in exile
/// with a takeover counter.
pub fn the_master_formed_anew() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_cast(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::Move { what: target_filtered(yours(R::Creature)), to: ZoneDest::Exile },
                Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Takeover, amount: Value::ONE },
            ])),
        })],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            from_exile_with_counter: Some(CounterType::Takeover),
            ..Default::default()
        }),
        ..legend(creature(
            "The Master, Formed Anew",
            cost(&[u(), b()]),
            vec![CreatureType::TimeLord, CreatureType::Rogue],
            0,
            1,
        ))
    }
}

/// The Master, Gallifrey's End — a nontoken artifact creature of yours dying
/// may be exiled; the opponent with the most life chooses: lose 4, or you
/// create a token copy of it.
pub fn the_master_gallifreys_end() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(trigger_is(artifact_creatures().and(R::NotToken))),
            effect: Effect::MayDo {
                description: "Exile it to make them pay?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile },
                    villainous(
                        Selector::Player(PlayerRef::HighestLifeOpponent),
                        Effect::LoseLife { who: Selector::You, amount: Value::Const(4) },
                        Effect::CreateTokenCopyOf {
                            who: me(),
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
                        },
                    ),
                ])),
            },
        }],
        ..legend(creature(
            "The Master, Gallifrey's End",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::TimeLord, CreatureType::Rogue],
            4,
            3,
        ))
    }
}

/// The Master, Mesmerist — {T}: an opponent's creature with power up to his
/// gains skulk and is goaded; a creature with skulk hitting an opponent grows
/// him and draws you a card.
pub fn the_master_mesmerist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent).and(R::PowerAtMostSourcePower)),
                    keyword: Keyword::Skulk,
                    duration: Duration::EndOfTurn,
                },
                Effect::Goad { what: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                trigger_is(R::HasKeyword(Keyword::Skulk)),
                Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer },
            ])),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                draw(1),
            ]),
        }],
        ..legend(creature(
            "The Master, Mesmerist",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::TimeLord, CreatureType::Rogue],
            3,
            3,
        ))
    }
}

/// The Master, Multiplied — myriad; the legend rule doesn't apply to your
/// creature tokens. Residual: your triggered abilities can still make you
/// sacrifice or exile your creature tokens.
pub fn the_master_multiplied() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "The \"legend rule\" doesn't apply to creature tokens you control.",
            effect: StaticEffect::LegendRuleDoesntApplyToYourMatching(R::Creature.and(R::IsToken)),
        }],
        triggered_abilities: vec![myriad()],
        ..legend(creature(
            "The Master, Multiplied",
            cost(&[generic(4), b(), r()]),
            vec![CreatureType::TimeLord, CreatureType::Rogue],
            4,
            3,
        ))
    }
}

/// The Rani — entering or attacking, a Mark of the Rani Aura token (+2/+2,
/// goaded) on another target creature; a goaded creature hitting an opponent
/// investigates.
pub fn the_rani() -> CardDefinition {
    let mark = || Effect::CreateTokenAttachedTo {
        target: target_filtered(R::Creature.and(R::OtherThanSource)),
        definition: Arc::new(TokenDefinition {
            name: "Mark of the Rani".into(),
            card_types: vec![CardType::Enchantment],
            colors: vec![Color::Red],
            subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
            equipped_bonus: Some(EquipBonus { power: 2, toughness: 2, ..Default::default() }),
            static_abilities: vec![StaticAbility {
                description: "Enchanted creature is goaded.",
                effect: StaticEffect::AttachedIsGoaded,
            }],
            ..Default::default()
        }),
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(mark()),
            on_attack(mark()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                    trigger_is(R::IsGoaded),
                    Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer },
                ])),
                effect: investigate(1),
            },
        ],
        ..legend(creature(
            "The Rani",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::TimeLord, CreatureType::Scientist],
            3,
            4,
        ))
    }
}

/// The Sound of Drums — enchanted creature is goaded; {2}{R}: return it from
/// your graveyard to your hand. Residual: its combat damage isn't doubled.
pub fn the_sound_of_drums() -> CardDefinition {
    CardDefinition {
        name: "The Sound of Drums",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature is goaded.",
            effect: StaticEffect::AttachedIsGoaded,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            from_graveyard: true,
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Toymaker's Trap — your upkeep: a secret number from 1 to 5 an opponent
/// guesses; a miss loses them life and draws you a card, a match sacrifices
/// it. Residual: the life lost is the chosen number's worth, and numbers may
/// repeat.
pub fn the_toymakers_trap() -> CardDefinition {
    CardDefinition {
        name: "The Toymaker's Trap",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![step(
            TurnStep::Upkeep,
            Effect::SecretNumbersMatch {
                opponent: PlayerRef::HostileOpponent,
                max: 5,
                on_match: Box::new(Effect::SacrificeSource),
                on_miss: Box::new(Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::HostileOpponent),
                        amount: Value::ChosenNumberOfSource,
                    },
                    draw(1),
                ])),
            },
        )],
        ..Default::default()
    }
}

/// The Valeyard — opponents face each villainous choice twice; you get an
/// additional vote.
pub fn the_valeyard() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "If an opponent would face a villainous choice, they face that choice an additional time.",
                effect: StaticEffect::OpponentsFaceVillainousChoicesTwice,
            },
            StaticAbility { description: "While voting, you may vote an additional time.", effect: StaticEffect::AdditionalVotes(1) },
        ],
        ..legend(creature(
            "The Valeyard",
            cost(&[generic(2), u(), b(), r()]),
            vec![CreatureType::TimeLord, CreatureType::Noble],
            4,
            5,
        ))
    }
}

/// This Is How It Ends — target creature's owner shuffles it into their
/// library, then chooses: lose 5 life, or shuffle another creature of theirs
/// away.
pub fn this_is_how_it_ends() -> CardDefinition {
    spell(
        "This Is How It Ends",
        cost(&[generic(3), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            shuffle_into_owners_library(target_filtered(R::Creature)),
            villainous(
                Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                Effect::LoseLife { who: Selector::You, amount: Value::Const(5) },
                Effect::Move {
                    what: Selector::Take {
                        inner: Box::new(Selector::EachPermanent(R::Creature.and(R::OwnedByYou))),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Shuffled },
                },
            ),
        ]),
    )
}

/// Time Reaper — flying, haste; its combat damage puts a target face-up card
/// that player owns in exile on the bottom of their library, and you gain 3.
/// Residual: the life is gained whether or not a card moved.
pub fn time_reaper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::InExile.and(R::OwnedByDefendingPlayer)),
                    to: ZoneDest::Library { who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))), pos: LibraryPosition::Bottom },
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ]),
        }],
        ..creature("Time Reaper", cost(&[generic(3), b(), b()]), vec![CreatureType::Alien, CreatureType::Horror], 4, 4)
    }
}

/// Vislor Turlough — Doctor's companion; entering, you may give it to an
/// opponent, goaded; your end step draws one, then you lose life equal to
/// your hand. Residual: goaded for the rest of the game.
pub fn vislor_turlough() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Have an opponent gain control of Vislor Turlough?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::GainControl { what: Selector::This, to: Some(PlayerRef::HostileOpponent), duration: Duration::Permanent },
                    Effect::GoadForTheGame { what: Selector::This },
                ])),
            }),
            step(
                TurnStep::End,
                Effect::Seq(vec![draw(1), Effect::LoseLife { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You) }]),
            ),
        ],
        ..companion(creature("Vislor Turlough", cost(&[generic(3), b()]), vec![CreatureType::Rogue], 2, 5))
    }
}

/// Weeping Angel — flash, first strike, vigilance; stops being a creature
/// when an opponent casts one; its combat damage to a creature shuffles that
/// creature into its owner's library. Residual: the damage is dealt first.
pub fn weeping_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::FirstStrike, Keyword::Vigilance],
        triggered_abilities: vec![
            quantum_locked(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                        pos: LibraryPosition::Shuffled,
                    },
                },
            },
        ],
        ..artifact_creature(
            "Weeping Angel",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Alien, CreatureType::Angel],
            2,
            2,
        )
    }
}

/// Wound Reflection — at each end step, each opponent loses the life they
/// lost this turn again.
pub fn wound_reflection() -> CardDefinition {
    CardDefinition {
        name: "Wound Reflection",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::EachPlayerDoes {
                who: PlayerRef::EachOpponent,
                body: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::LifeLostThisTurn(PlayerRef::You) }),
            },
        }],
        ..Default::default()
    }
}

/// Zygon Infiltrator — sorcery speed {2}{U}: tap another target creature with
/// a stun counter and become a copy of it. Residual: the copy lasts until end
/// of turn.
pub fn zygon_infiltrator() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature.and(R::OtherThanSource)) },
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
                Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: Selector::Target(0),
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Zygon Infiltrator",
            cost(&[generic(2), u()]),
            vec![CreatureType::Alien, CreatureType::Shapeshifter, CreatureType::Soldier],
            3,
            2,
        )
    }
}
