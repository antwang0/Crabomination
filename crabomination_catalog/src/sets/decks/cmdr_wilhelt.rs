//! Commander: the cards the **Undead Unleashed** precon (MIC, Wilhelt, the
//! Rotcleaver) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_wilhelt.rs`.
//!
//! Residuals (each also on its card):
//! - **Hordewing Skaab** — draws for the opponents dealt combat damage this
//!   turn, not only those its Zombies' batch damaged.
//! - **Hour of Eternity** — the copies are Zombies in addition to their types.
//! - **Shadow Kin** — copies the greatest-power creature card milled (the
//!   engine's pick).
//! - **Rooftop Storm** — Zombie creature spells cast from hand only.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, investigate, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, u, x, Color, ManaCost};
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

fn spell(name: &'static str, mana: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn zombie() -> R {
    R::HasCreatureType(CreatureType::Zombie)
}

fn your_zombies() -> R {
    R::Creature.and(zombie()).and(R::ControlledByYou)
}

fn zombie_token(decayed: bool) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        keywords: if decayed { vec![Keyword::Decayed] } else { vec![] },
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    })
}

fn make_zombies(who: PlayerRef, count: Value, decayed: bool) -> Effect {
    Effect::CreateToken { who, count, definition: zombie_token(decayed) }
}

fn graveyard(who: PlayerRef, filter: R) -> Selector {
    Selector::CardsInZone { who, zone: Zone::Graveyard, filter }
}

/// "Enchant player" Curse.
fn curse(name: &'static str, mana: ManaCost, triggered: TriggeredAbility) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities: vec![triggered],
        ..Default::default()
    }
}

/// "Whenever you sacrifice [filter], …"
fn on_your_sacrifice(filter: R, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter }),
        effect,
    }
}

/// Cleaver Skaab — {3}{U} 2/4 Zombie Horror. {3}, {T}, sacrifice another
/// Zombie: two tokens that are copies of the sacrificed creature.
pub fn cleaver_skaab() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3)]),
            sac_other_filter: Some((R::Creature.and(zombie()), 1)),
            effect: Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::Const(2),
                source: Selector::SacrificedCard,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
            ..Default::default()
        }],
        ..creature("Cleaver Skaab", cost(&[generic(3), u()]), vec![CreatureType::Zombie, CreatureType::Horror], 2, 4)
    }
}

/// Curse of Unbinding — {6}{U} Aura Curse. At the beginning of enchanted
/// player's upkeep, they reveal until a creature card; you get it, the rest
/// go to their graveyard.
pub fn curse_of_unbinding() -> CardDefinition {
    curse(
        "Curse of Unbinding",
        cost(&[generic(6), u()]),
        TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::IsTurnOf(PlayerRef::EnchantedPlayer)),
            effect: Effect::RevealUntilFind {
                who: PlayerRef::EnchantedPlayer,
                find: R::Creature,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                cap: Value::Const(1000),
                life_per_revealed: 0,
                miss_dest: Default::default(),
            },
        },
    )
}

/// Curse of the Restless Dead — {2}{B} Aura Curse. Whenever a land enchanted
/// player controls enters, you create a 2/2 black Zombie with decayed.
pub fn curse_of_the_restless_dead() -> CardDefinition {
    curse(
        "Curse of the Restless Dead",
        cost(&[generic(2), b()]),
        TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land },
                Predicate::SamePlayer(
                    PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                    PlayerRef::EnchantedPlayer,
                ),
            ])),
            effect: make_zombies(PlayerRef::You, Value::ONE, true),
        },
    )
}

/// Dark Salvation — {X}{X}{B} Sorcery. Target player creates X 2/2 Zombies,
/// then up to one target creature gets −1/−1 for each Zombie that player
/// controls.
pub fn dark_salvation() -> CardDefinition {
    let shrink = || {
        Value::Diff(
            Box::new(Value::Const(0)),
            Box::new(Value::PermanentCountControlledByMatching(PlayerRef::Target(0), R::Creature.and(zombie()))),
        )
    };
    spell(
        "Dark Salvation",
        cost(&[x(), x(), b()]),
        Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::Target(0),
                    count: Value::XFromCost,
                    definition: zombie_token(false),
                },
                Effect::PumpPT {
                    what: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                    power: shrink(),
                    toughness: shrink(),
                    duration: Duration::EndOfTurn,
                },
            ])),
        },
    )
}

/// Eloise, Nephalia Sleuth — {3}{U}{B} 4/4 legendary Human Rogue. Another
/// creature of yours dies: investigate. You sacrifice a token: surveil 1.
pub fn eloise_nephalia_sleuth() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource },
                ),
                effect: investigate(1),
            },
            on_your_sacrifice(R::IsToken, Effect::Surveil { who: PlayerRef::You, amount: Value::ONE }),
        ],
        ..creature(
            "Eloise, Nephalia Sleuth",
            cost(&[generic(3), u(), b()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            4,
            4,
        )
    }
}

/// Empty the Laboratory — {X}{U}{U} Sorcery. Sacrifice X Zombies, then reveal
/// until that many Zombie creature cards; they enter, the rest go to the
/// bottom in a random order.
pub fn empty_the_laboratory() -> CardDefinition {
    spell(
        "Empty the Laboratory",
        cost(&[x(), u(), u()]),
        Effect::Seq(vec![
            Effect::Sacrifice { who: Selector::You, count: Value::XFromCost, filter: R::Creature.and(zombie()) },
            Effect::RevealUntilMatchingToBattlefield {
                filter: R::Creature.and(zombie()),
                count: Value::SacrificedThisResolutionBy { who: PlayerRef::You, filter: zombie() },
                rest_bottom: true,
            },
        ]),
    )
}

/// Ghouls' Night Out — {3}{B}{B} Sorcery. For each player, a creature card
/// from their graveyard enters under your control as a black Zombie with
/// decayed.
pub fn ghouls_night_out() -> CardDefinition {
    spell(
        "Ghouls' Night Out",
        cost(&[generic(3), b(), b()]),
        Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachPlayer),
                body: Box::new(Effect::MoveChosen {
                    from: graveyard(PlayerRef::Triggerer, R::Creature),
                    filter: None,
                    count: Value::ONE,
                    up_to: false,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
            Effect::BecomeColor {
                what: Selector::LastMoved,
                colors: vec![Color::Black],
                duration: Duration::Permanent,
                additive: true,
            },
            Effect::AddCreatureTypes {
                what: Selector::LastMoved,
                creature_types: vec![CreatureType::Zombie],
                duration: Duration::Permanent,
            },
            Effect::GrantKeyword { what: Selector::LastMoved, keyword: Keyword::Decayed, duration: Duration::Permanent },
        ]),
    )
}

/// Gorex, the Tombshell — {6}{B}{B} 4/4 legendary deathtouch Zombie Turtle.
/// You may exile any number of creature cards from your graveyard as you cast
/// it, {2} less each. Attacking or dying: a random card exiled with it goes to
/// its owner's hand.
pub fn gorex_the_tombshell() -> CardDefinition {
    let return_one = || Effect::Move {
        what: Selector::TakeRandom { inner: Box::new(Selector::CardExiledWithSource), count: Box::new(Value::ONE) },
        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Deathtouch],
        graveyard_exile_discount: Some((R::Creature, 2)),
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: return_one() },
            TriggeredAbility { event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource), effect: return_one() },
        ],
        ..creature(
            "Gorex, the Tombshell",
            cost(&[generic(6), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Turtle],
            4,
            4,
        )
    }
}

/// Havengul Runebinder — {2}{U}{U} 2/2 Human Wizard. {2}{U}, {T}, exile a
/// creature card from your graveyard: a 2/2 Zombie, then a +1/+1 counter on
/// each Zombie you control.
pub fn havengul_runebinder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), u()]),
            exile_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                make_zombies(PlayerRef::You, Value::ONE, false),
                Effect::AddCounter {
                    what: Selector::EachPermanent(your_zombies()),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Havengul Runebinder",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Hordewing Skaab — {4}{U} 3/3 flying Zombie Horror. Other Zombies you
/// control have flying. Your Zombies' combat damage to opponents: you may draw
/// that many (one per opponent) and discard that many. (Residual: counts every
/// opponent dealt combat damage this turn.)
pub fn hordewing_skaab() -> CardDefinition {
    let n = || Value::PlayersDealtCombatDamageThisTurn(PlayerRef::EachOpponent);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Other Zombies you control have flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(your_zombies().and(R::OtherThanSource)),
                keyword: Keyword::Flying,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: zombie() })
                .once_per_batch_across_players(),
            effect: Effect::MayDo {
                description: "Draw a card per opponent dealt damage, then discard that many?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: n() },
                    Effect::Discard { who: Selector::You, amount: n(), random: false },
                ])),
            },
        }],
        ..creature("Hordewing Skaab", cost(&[generic(4), u()]), vec![CreatureType::Zombie, CreatureType::Horror], 3, 3)
    }
}

/// Hour of Eternity — {X}{X}{U}{U}{U} Sorcery. Exile X target creature cards
/// from your graveyard; a 4/4 black Zombie token copy of each. (Residual:
/// Zombie in addition to the copied types.)
pub fn hour_of_eternity() -> CardDefinition {
    spell(
        "Hour of Eternity",
        cost(&[x(), x(), u(), u(), u()]),
        Effect::TargetsExactlyX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 20,
                min_targets: 0,
                filter: R::Creature.and(R::InYourGraveyard),
                effect: Box::new(Effect::Seq(vec![
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: Selector::Target(0),
                        extra_creature_types: vec![CreatureType::Zombie],
                        extra_card_types: vec![],
                        override_pt: Some((4, 4)),
                        override_colors: Some(vec![Color::Black]),
                        enters_tapped: false,
                        non_legendary: false,
                        legendary: false,
                        extra_keywords: vec![],
                    },
                    Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile },
                ])),
            }),
        },
    )
}

/// Prowling Geistcatcher — {3}{B} 2/4 Human Rogue. You sacrifice another
/// creature: exile it (a token: a +1/+1 counter instead). Leaving: every card
/// exiled with it returns under your control.
pub fn prowling_geistcatcher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_your_sacrifice(
                R::Creature.and(R::OtherThanSource),
                Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsToken },
                    then: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                    else_: Box::new(Effect::Move { what: Selector::TriggerSource, to: ZoneDest::ExileWithSourceStamp }),
                },
            ),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::CardExiledWithSource,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            },
        ],
        ..creature(
            "Prowling Geistcatcher",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            2,
            4,
        )
    }
}

/// Ravenous Rotbelly — {4}{B} 4/5 Zombie Horror. ETB: you may sacrifice up to
/// three Zombies; each opponent sacrifices that many creatures.
pub fn ravenous_rotbelly() -> CardDefinition {
    let others = || Value::CountOf(Box::new(Selector::EachPermanent(your_zombies().and(R::OtherThanSource))));
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice up to three other Zombies?".into(),
            filter: R::Creature.and(zombie()).and(R::OtherThanSource),
            count: Value::Min(Box::new(Value::Const(3)), Box::new(others())),
            then: Box::new(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::SacrificedThisResolutionBy { who: PlayerRef::You, filter: zombie() },
                filter: R::Creature,
            }),
            else_: None,
        })],
        ..creature("Ravenous Rotbelly", cost(&[generic(4), b()]), vec![CreatureType::Zombie, CreatureType::Horror], 4, 5)
    }
}

/// Rooftop Storm — {5}{U} Enchantment. You may pay {0} rather than the mana
/// cost for Zombie creature spells you cast. (Residual: from hand.)
pub fn rooftop_storm() -> CardDefinition {
    CardDefinition {
        name: "Rooftop Storm",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "You may pay {0} rather than pay the mana cost for Zombie creature spells you cast.",
            effect: StaticEffect::CastFilteredSpellsFree { filter: R::Creature.and(zombie()) },
        }],
        ..Default::default()
    }
}

/// Ruthless Deathfang — {4}{U}{B} 4/4 flying Dragon. Whenever you sacrifice a
/// creature, target opponent sacrifices a creature.
pub fn ruthless_deathfang() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_your_sacrifice(
            R::Creature,
            Effect::Sacrifice { who: target_filtered(R::OpponentPlayer), count: Value::ONE, filter: R::Creature },
        )],
        ..creature("Ruthless Deathfang", cost(&[generic(4), u(), b()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Shadow Kin — {3}{U} 2/2 flash Shapeshifter. Your upkeep: each player mills
/// three; you may exile a creature card milled this way and become a copy of
/// it, keeping this ability. (Residual: the engine picks the greatest power.)
pub fn shadow_kin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(3) },
                Effect::MayDo {
                    description: "Exile a milled creature card and become a copy of it?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: Selector::TakeGreatestPower {
                                inner: Box::new(Selector::MatchingAmong {
                                    inner: Box::new(Selector::LastMoved),
                                    filter: R::Creature.and(R::InGraveyard),
                                }),
                                count: Box::new(Value::ONE),
                            },
                            to: ZoneDest::Exile,
                        },
                        Effect::BecomeCopyOf {
                            what: Selector::This,
                            source: Selector::ExiledThisResolution { filter: R::Creature },
                            extra_creature_types: vec![],
                            keep_own_triggered: true,
                            keep_own_activated: false,
                        },
                    ])),
                },
            ]),
        }],
        ..creature("Shadow Kin", cost(&[generic(3), u()]), vec![CreatureType::Shapeshifter], 2, 2)
    }
}

/// Tomb Tyrant — {3}{B} 3/3 Zombie Noble. Other Zombies you control get
/// +1/+1. {2}{B}, {T}, sacrifice a creature: a random Zombie creature card
/// from your graveyard returns — only on your turn, with three or more there.
pub fn tomb_tyrant() -> CardDefinition {
    let gy_zombies = || graveyard(PlayerRef::You, R::Creature.and(zombie()));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Zombies you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(your_zombies().and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), b()]),
            sac_other_filter: Some((R::Creature, 1)),
            sac_other_may_be_source: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::ValueAtLeast(Value::CountOf(Box::new(gy_zombies())), Value::Const(3)),
            ])),
            effect: Effect::Move {
                what: Selector::TakeRandom { inner: Box::new(gy_zombies()), count: Box::new(Value::ONE) },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature("Tomb Tyrant", cost(&[generic(3), b()]), vec![CreatureType::Zombie, CreatureType::Noble], 3, 3)
    }
}

/// Undead Alchemist — {3}{U} 4/2 Zombie. Your Zombies' combat damage to a
/// player mills them instead. A creature card milled from an opponent's
/// library is exiled and you create a 2/2 Zombie.
pub fn undead_alchemist() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If a Zombie you control would deal combat damage to a player, instead that player mills that many cards.",
            effect: StaticEffect::CombatDamageToPlayersBecomesMill { filter: zombie() },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardMilled, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::Seq(vec![
                Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile },
                make_zombies(PlayerRef::You, Value::ONE, false),
            ]),
        }],
        ..creature("Undead Alchemist", cost(&[generic(3), u()]), vec![CreatureType::Zombie], 4, 2)
    }
}
