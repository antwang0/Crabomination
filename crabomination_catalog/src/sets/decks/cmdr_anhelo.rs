//! Commander: the cards the **Maestros Massacre** precon (NCC, Anhelo, the
//! Painter) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_anhelo.rs`.
//!
//! Residuals (each also on its card):
//! - **Maestros Confluence** — the goad mode takes the hostile opponent's
//!   creatures rather than a targeted player's.
//! - **Parnesse, the Subtle Brush** — only your permanents get the pay-4-life
//!   tax (a targeted player doesn't), and copying a spell doesn't offer an
//!   opponent a copy.
//! - **Sinister Concierge** — the second creature it exiles is an opponent's.
//! - **Syrix, Carrier of the Flame** — its end-step check counts any card
//!   leaving your graveyard, and "you may cast this from your graveyard" is a
//!   permission for the rest of the turn.
//! - **Waste Management** — kicked, it exiles the hostile opponent's
//!   graveyard rather than a targeted player's; unkicked, its two cards may
//!   come from different graveyards.
//! - **Xander's Pact** — lands exiled this way may be played for life too.
//! - **Zndrsplt's Judgment** — you are the only friend and every opponent a
//!   foe.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{encore, etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, r, u, Color, ManaCost, SpendRestriction};

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

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

fn phoenix() -> R {
    R::HasCreatureType(CreatureType::Phoenix)
}

fn to_hand(what: Selector) -> Effect {
    Effect::Move { what, to: ZoneDest::Hand(PlayerRef::You) }
}

/// Anhelo, the Painter — deathtouch; your first instant or sorcery each
/// turn has casualty 2.
pub fn anhelo_the_painter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        static_abilities: vec![StaticAbility {
            description: "The first instant or sorcery spell you cast each turn has casualty 2.",
            effect: StaticEffect::FirstInstantSorceryHasCasualty(2),
        }],
        ..legend(
            "Anhelo, the Painter",
            cost(&[u(), b(), r()]),
            vec![CreatureType::Vampire, CreatureType::Assassin],
            1,
            3,
        )
    }
}

/// Audacious Swap — casualty 2; a nonenchantment permanent's owner shuffles
/// it away and plays or casts their top card.
pub fn audacious_swap() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Casualty(2)],
        ..spell(
            "Audacious Swap",
            cost(&[generic(3), r()]),
            CardType::Instant,
            Effect::OwnerShufflesInExilesTopPlaysOrCasts {
                what: target_filtered(R::Permanent.and(R::Not(Box::new(R::Enchantment)))),
            },
        )
    }
}

/// Body Count — spectacle {B}; draw a card per creature that died under
/// your control this turn.
pub fn body_count() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(crate::effect::shortcut::spectacle(cost(&[b()]))),
        ..spell(
            "Body Count",
            cost(&[generic(2), b()]),
            CardType::Instant,
            Effect::Draw { who: Selector::You, amount: Value::ControllerCreaturesDiedThisTurn },
        )
    }
}

/// Cormela, Glamour Thief — haste; {1}, {T}: {U}{B}{R} for instants and
/// sorceries; dying returns an instant or sorcery card.
pub fn cormela_glamour_thief() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::Blue, Color::Black, Color::Red])),
                    SpendRestriction::InstantSorceryOnly,
                ),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: to_hand(target_filtered(instant_or_sorcery().and(R::InYourGraveyard))),
        }],
        ..legend(
            "Cormela, Glamour Thief",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Vampire, CreatureType::Rogue],
            2,
            4,
        )
    }
}

/// Cryptic Pursuit — your instants and sorceries from hand manifest; a
/// face-down creature of yours dying as an instant or sorcery card is
/// exiled to cast until the end of your next turn.
pub fn cryptic_pursuit() -> CardDefinition {
    CardDefinition {
        name: "Cryptic Pursuit",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::CastSpellMatches(instant_or_sorcery().and(R::Not(Box::new(R::SpellNotCastFromHand)))),
                ),
                effect: Effect::Manifest { who: PlayerRef::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::FaceDown },
                ),
                effect: Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::TriggerSource, filter: instant_or_sorcery() },
                    then: Box::new(Effect::Seq(vec![
                        Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile },
                        Effect::GrantMayPlay {
                            what: Selector::LastMoved,
                            duration: MayPlayDuration::EndOfControllersNextTurn,
                            to_owner: false,
                            exile_after: false,
                            any_color: false,
                            pay_own_cost: true,
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Dogged Detective — surveils 2 on entering; an opponent's second draw
/// each turn may return it from your graveyard to your hand.
pub fn dogged_detective() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SecondCardDrawnThisTurn, EventScope::FromYourGraveyardAnyPlayer)
                    .from_opponent(),
                effect: Effect::MayDo {
                    description: "Return Dogged Detective to your hand?".into(),
                    body: Box::new(to_hand(Selector::This)),
                },
            },
        ],
        ..creature("Dogged Detective", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Rogue], 2, 1)
    }
}

/// Double Vision — your first instant or sorcery each turn is copied.
pub fn double_vision() -> CardDefinition {
    CardDefinition {
        name: "Double Vision",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellFirstMatchingThisTurn(instant_or_sorcery())),
            effect: Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Drawn from Dreams — look at seven, keep two, bottom the rest at random.
pub fn drawn_from_dreams() -> CardDefinition {
    spell(
        "Drawn from Dreams",
        cost(&[generic(2), u(), u()]),
        CardType::Sorcery,
        Effect::LookPickToHand(Box::new(LookPick {
            count: Value::Const(7),
            take: Some(Value::Const(2)),
            rest_bottom_random: true,
            ..Default::default()
        })),
    )
}

/// Flawless Forgery — casualty 3; exile an instant or sorcery card from an
/// opponent's graveyard and cast a copy of it for free.
pub fn flawless_forgery() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Casualty(3)],
        ..spell(
            "Flawless Forgery",
            cost(&[generic(3), u(), u()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(instant_or_sorcery().and(R::InOpponentGraveyard)),
                    to: ZoneDest::Exile,
                },
                Effect::CopyCardAndCastFree { what: Selector::Target(0) },
            ]),
        )
    }
}

/// Maestros Charm — dig five for one, drain 3, or 5 damage to a creature or
/// planeswalker.
pub fn maestros_charm() -> CardDefinition {
    spell(
        "Maestros Charm",
        cost(&[u(), b(), r()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::LookPickToHand(Box::new(LookPick {
                count: Value::Const(5),
                rest_to_graveyard: true,
                ..Default::default()
            })),
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(3),
            },
            Effect::DealDamage {
                to: target_filtered(R::Creature.or(R::Planeswalker)),
                amount: Value::Const(5),
            },
        ]),
    )
}

/// Maestros Confluence — three modes, repeats allowed: regrow a monocolored
/// instant or sorcery, -3/-3, or goad a player's creatures.
///
/// Residual: the goad mode takes the hostile opponent's creatures.
pub fn maestros_confluence() -> CardDefinition {
    spell(
        "Maestros Confluence",
        cost(&[generic(3), u(), b(), r()]),
        CardType::Sorcery,
        Effect::ChooseN {
            picks: vec![1, 1, 2],
            modes: vec![
                to_hand(target_filtered(instant_or_sorcery().and(R::Monocolored).and(R::InYourGraveyard))),
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(-3),
                    toughness: Value::Const(-3),
                    duration: Duration::EndOfTurn,
                },
                Effect::Goad {
                    what: Selector::ControlledBy { who: PlayerRef::HostileOpponent, filter: R::Creature },
                },
            ],
        },
    )
}

/// Make an Example — each opponent splits their creatures into two piles;
/// you pick one of each to be sacrificed.
pub fn make_an_example() -> CardDefinition {
    spell(
        "Make an Example",
        cost(&[generic(3), b()]),
        CardType::Sorcery,
        Effect::ForEachOpponent {
            body: Box::new(Effect::SeparateIntoPiles {
                what: Selector::ControlledBy { who: PlayerRef::Triggerer, filter: R::Creature },
                splitter: PlayerRef::Triggerer,
                chooser: PlayerRef::You,
                chosen: Box::new(Effect::SacrificeSelected { what: Selector::SeparatedPile { chosen: true } }),
                other: Box::new(Effect::Noop),
            }),
        },
    )
}

/// Parnesse, the Subtle Brush — your permanents tax an opponent's targeting
/// 4 life.
///
/// Residual: you aren't protected yourself, and copying a spell doesn't
/// offer an opponent a copy.
pub fn parnesse_the_subtle_brush() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Whenever a permanent you control becomes the target of a spell or ability an opponent \
                          controls, counter that spell or ability unless that player pays 4 life.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::ControlledByYou),
                keyword: Keyword::Ward(WardCost::Life(4)),
            },
        }],
        ..legend(
            "Parnesse, the Subtle Brush",
            cost(&[generic(2), u(), b(), r()]),
            vec![CreatureType::Vampire, CreatureType::Wizard],
            4,
            4,
        )
    }
}

/// Rekindling Phoenix — flying; dying leaves an Elemental that brings it
/// back at your next upkeep.
pub fn rekindling_phoenix() -> CardDefinition {
    let elemental = Arc::new(TokenDefinition {
        name: "Elemental".into(),
        power: 0,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                Effect::SacrificeSource,
                Effect::Move {
                    what: target_filtered(R::HasName("Rekindling Phoenix".into()).and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::GrantKeyword { what: Selector::LastMoved, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            ]),
        }],
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: elemental },
        }],
        ..creature("Rekindling Phoenix", cost(&[generic(2), r(), r()]), vec![CreatureType::Phoenix], 4, 3)
    }
}

/// Sinister Concierge — dying, it may suspend itself and a creature for
/// three turns.
///
/// Residual: the second creature is an opponent's.
pub fn sinister_concierge() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Exile Sinister Concierge with three time counters?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::GrantSuspend { what: Selector::This, time_counters: 3 },
                    Effect::ApplyToTargets {
                        max_targets: 1,
                        min_targets: 0,
                        filter: R::Creature.and(R::ControlledByOpponent),
                        effect: Box::new(Effect::GrantSuspend { what: Selector::Target(0), time_counters: 3 }),
                    },
                ])),
            },
        }],
        ..creature(
            "Sinister Concierge",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            1,
        )
    }
}

/// Skyclave Shade — kicker {2}{B}, two counters kicked; can't block;
/// landfall on your turn lets you cast it from your graveyard.
pub fn skyclave_shade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(2), b()])), Keyword::CantBlock],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::Times(Box::new(Value::Const(2)), Box::new(Value::TimesKicked)),
        )),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::FromYourGraveyard)
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::MayDo {
                description: "Cast Skyclave Shade from your graveyard this turn?".into(),
                body: Box::new(Effect::GrantMayPlay {
                    what: Selector::This,
                    duration: MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    any_color: false,
                    pay_own_cost: true,
                }),
            },
        }],
        ..creature("Skyclave Shade", cost(&[generic(1), b()]), vec![CreatureType::Shade], 3, 1)
    }
}

/// Smuggler's Buggy — hideaway 4; its combat damage to a player may cast the
/// hidden card for free, returning the Buggy to hand; crew 2.
pub fn smugglers_buggy() -> CardDefinition {
    CardDefinition {
        name: "Smuggler's Buggy",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![
            etb(Effect::Hideaway { count: Value::Const(4) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::If {
                    cond: Predicate::SelectorExists(Selector::CardExiledWithSource),
                    then: Box::new(Effect::MayDo {
                        description: "Cast the hidden card for free and return Smuggler's Buggy to hand?".into(),
                        body: Box::new(Effect::Seq(vec![
                            Effect::CastAnyOrderWithoutPaying {
                                what: Selector::CardExiledWithSource,
                                source_zone: Zone::Exile,
                                filter: None,
                                cap: Some(Value::ONE),
                                total_mana_value: None,
                            },
                            Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
                        ])),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Spellbinding Soprano — attacking makes your instants and sorceries {1}
/// cheaper this turn; encore {3}{R}.
pub fn spellbinding_soprano() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::SpellsCostLessThisTurn { filter: instant_or_sorcery(), amount: 1 })],
        activated_abilities: vec![encore(cost(&[generic(3), r()]))],
        ..creature("Spellbinding Soprano", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Bard], 2, 2)
    }
}

/// Syrix, Carrier of the Flame — flying, haste; each end step after a card
/// left your graveyard, a Phoenix of yours deals its power to any target;
/// another Phoenix of yours dying lets you cast Syrix from your graveyard.
///
/// Residual: the end-step check counts any card leaving your graveyard,
/// and the graveyard cast is a permission for the rest of the turn.
pub fn syrix_carrier_of_the_flame() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                    Predicate::CardsLeftGraveyardThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE },
                ),
                effect: Effect::DealDamageFrom {
                    source: target_filtered(phoenix().and(R::ControlledByYou)),
                    to: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.or(R::Player).or(R::Planeswalker),
                    },
                    amount: Value::PowerOf(Box::new(Selector::Target(0))),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::FromYourGraveyard).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: phoenix().and(R::OwnedByYou) },
                ),
                effect: Effect::GrantMayPlay {
                    what: Selector::This,
                    duration: MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    any_color: false,
                    pay_own_cost: true,
                },
            },
        ],
        ..legend(
            "Syrix, Carrier of the Flame",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Phoenix],
            3,
            3,
        )
    }
}

/// Waste Management — exile up to two graveyard cards (kicked: a whole
/// graveyard); a 2/2 Rogue per creature card exiled.
///
/// Residual: kicked, it takes the hostile opponent's graveyard; unkicked,
/// the two cards may come from different graveyards.
pub fn waste_management() -> CardDefinition {
    let rogue = Arc::new(TokenDefinition {
        name: "Rogue".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rogue], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(3), b()]))],
        ..spell(
            "Waste Management",
            cost(&[generic(2), b()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::ExilePlayerGraveyard { who: PlayerRef::HostileOpponent, filter: None }),
                    else_: Box::new(Effect::ApplyToTargets {
                        max_targets: 2,
                        min_targets: 0,
                        filter: R::InGraveyard,
                        effect: Box::new(Effect::Exile { what: Selector::Target(0) }),
                    }),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::count(Selector::ExiledThisResolution { filter: R::Creature }),
                    definition: rogue,
                },
            ]),
        )
    }
}

/// Xander's Pact — casualty 2; each opponent exiles their top card, and you
/// may cast those this turn paying life instead of mana.
///
/// Residual: a land exiled this way may be played too.
pub fn xanders_pact() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Casualty(2)],
        ..spell(
            "Xander's Pact",
            cost(&[generic(4), b(), b()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TopOfLibrary { who: PlayerRef::EachOpponent, count: Value::ONE },
                    to: ZoneDest::Exile,
                },
                Effect::GrantMayPlayForLife { what: Selector::LastMoved, duration: MayPlayDuration::EndOfThisTurn },
            ]),
        )
    }
}

/// Zndrsplt's Judgment — friends copy a creature of theirs; foes bounce one.
///
/// Residual: you are the only friend and every opponent a foe.
pub fn zndrsplts_judgment() -> CardDefinition {
    spell(
        "Zndrsplt's Judgment",
        cost(&[generic(4), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::GreatestPowerYouControl,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
            Effect::ForEachOpponent {
                body: Box::new(Effect::PlayerReturnsPermanentsToHand {
                    who: PlayerRef::Triggerer,
                    count: Value::ONE,
                    filter: R::Creature,
                    up_to: false,
                }),
            },
        ]),
    )
}
