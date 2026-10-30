//! Commander: the top 600's four-card gaps nobody had claimed — Magar of the
//! Magic Strings, Narci, Fable Singer and Yennett, Cryptic Sovereign
//! (`cmdr_edhrec17` is the previous batch). Tests in
//! `tests/recent_b/cmdr_edhrec19.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, LookPick, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, ManaCost};

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

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

fn exiled() -> Selector {
    Selector::CardExiledWithSource
}

fn non_saga_permanent() -> R {
    R::PermanentCard.and(R::Not(Box::new(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga))))
}

// ── Magar of the Magic Strings ───────────────────────────────────────────────

/// Magar of the Magic Strings — {1}{B}{R} 3/3 legendary Minotaur Performer.
/// {1}{B}{R}: put target instant or sorcery card from your graveyard onto the
/// battlefield face down as a 3/3 that casts a free copy of it whenever it
/// deals combat damage to a player, and is exiled if it would leave.
pub fn magar_of_the_magic_strings() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), r()]),
            effect: Effect::PutFaceDownAsNotedSpell {
                what: target_filtered(instant_or_sorcery().and(R::InYourGraveyard)),
                on_hit: Box::new(Effect::MayDo {
                    description: "Create a copy of the noted card and cast it free?".into(),
                    body: Box::new(Effect::CopyCardAndCastFree { what: Selector::This }),
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Magar of the Magic Strings",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Minotaur, CreatureType::Performer],
            3,
            3,
        )
    }
}

/// Azra Oddsmaker — {1}{B}{R} 3/3 Azra Warrior. At the beginning of combat on
/// your turn, you may discard a card; if you do, whenever the chosen creature
/// deals combat damage to a player this turn, you draw two cards.
pub fn azra_oddsmaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            // CR 608.2d — "if you do, choose a creature": any creature, picked
            // on resolution after the discard, not targeted.
            effect: Effect::MayDiscard {
                description: "Discard a card to draw two when a chosen creature connects?".into(),
                count: Value::ONE,
                then: Box::new(Effect::ChooseOneAmong {
                    what: Selector::EachPermanent(R::Creature),
                    chooser: PlayerRef::You,
                    chosen: Box::new(Effect::WithTargets {
                        what: Selector::SeparatedPile { chosen: true },
                        body: Box::new(Effect::WhenTargetDealsCombatDamageToPlayerThisTurn {
                            slot: 0,
                            body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
                            this_combat: false,
                        }),
                    }),
                    other: Box::new(Effect::Noop),
                }),
                else_: None,
            },
        }],
        ..creature(
            "Azra Oddsmaker",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Azra, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Sorin's Vengeance — {4}{B}{B}{B} sorcery. 10 damage to target player or
/// planeswalker; you gain 10 life.
pub fn sorins_vengeance() -> CardDefinition {
    spell(
        "Sorin's Vengeance",
        cost(&[generic(4), b(), b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Player.or(R::Planeswalker)), amount: Value::Const(10) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(10) },
        ]),
    )
}

/// Worst Fears — {7}{B} sorcery. You control target player during their next
/// turn (CR 723.1). Exile Worst Fears.
pub fn worst_fears() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Worst Fears",
            cost(&[generic(7), b()]),
            CardType::Sorcery,
            Effect::TargetPlayerThen {
                filter: R::Player,
                then: Box::new(Effect::ControlPlayerNextTurn { who: PlayerRef::Target(0) }),
            },
        )
    }
}

// ── Narci, Fable Singer ──────────────────────────────────────────────────────

/// Tale of Tinúviel — {3}{W}{W} Saga. I: target creature you control gains
/// indestructible for as long as you control this Saga. II: return target
/// creature card from your graveyard to the battlefield. III: up to two
/// target creatures you control gain lifelink until end of turn.
pub fn tale_of_tinuviel() -> CardDefinition {
    saga(
        "Tale of Tinúviel",
        cost(&[generic(3), w(), w()]),
        vec![
            (
                1,
                Effect::GrantKeyword {
                    what: target_filtered(mine()),
                    keyword: Keyword::Indestructible,
                    duration: Duration::WhileYouControlSource,
                },
            ),
            (
                2,
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ),
            (
                3,
                Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: mine(),
                    effect: Box::new(Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Lifelink,
                        duration: Duration::EndOfTurn,
                    }),
                },
            ),
        ],
    )
}

/// The Aesir Escape Valhalla — {2}{G} Saga. I: exile a permanent card from
/// your graveyard (the priciest) and gain life equal to its mana value. II:
/// that many +1/+1 counters on target creature you control. III: return this
/// Saga and the exiled card to their owner's hand.
pub fn the_aesir_escape_valhalla() -> CardDefinition {
    let mv = || Value::ManaValueOf(Box::new(exiled()));
    saga(
        "The Aesir Escape Valhalla",
        cost(&[generic(2), g()]),
        vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::best_of(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: R::PermanentCard,
                        }),
                        to: ZoneDest::ExileWithSourceStamp,
                    },
                    Effect::GainLife { who: Selector::You, amount: mv() },
                ]),
            ),
            (
                2,
                Effect::AddCounter { what: target_filtered(mine()), kind: CounterType::PlusOnePlusOne, amount: mv() },
            ),
            (
                3,
                Effect::Seq(vec![
                    Effect::Move { what: exiled(), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
                    Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
                ]),
            ),
        ],
    )
}

/// The Creation of Avacyn — {1}{B}{B} Saga. I: tutor a card into exile face
/// down. II: turn it face up; if it's a creature card, lose life equal to its
/// mana value. III: you may put it onto the battlefield if it's a creature
/// card; otherwise it goes to its owner's hand.
pub fn the_creation_of_avacyn() -> CardDefinition {
    let is_creature = || Predicate::EntityMatches { what: exiled(), filter: R::Creature };
    saga(
        "The Creation of Avacyn",
        cost(&[generic(1), b(), b()]),
        vec![
            (1, Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::ExileWithSourceStamp }),
            (
                2,
                Effect::If {
                    cond: is_creature(),
                    then: Box::new(Effect::LoseLife {
                        who: Selector::You,
                        amount: Value::ManaValueOf(Box::new(exiled())),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ),
            (
                3,
                Effect::If {
                    cond: is_creature(),
                    then: Box::new(Effect::MayDoElse {
                        description: "Put the exiled creature card onto the battlefield?".into(),
                        body: Box::new(Effect::Move {
                            what: exiled(),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        }),
                        else_: Box::new(Effect::Move { what: exiled(), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
                    }),
                    else_: Box::new(Effect::Move { what: exiled(), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
                },
            ),
        ],
    )
}

/// The World Spell — {5}{G}{G} Saga, read ahead. I, II: look at the top seven;
/// you may put a non-Saga permanent card among them into your hand, the rest
/// on the bottom in a random order. III: put up to two non-Saga permanent
/// cards from your hand onto the battlefield.
pub fn the_world_spell() -> CardDefinition {
    let dig = || {
        Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(7),
            pick_filter: Some(non_saga_permanent()),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        }))
    };
    CardDefinition {
        read_ahead: true,
        ..saga(
            "The World Spell",
            cost(&[generic(5), g(), g()]),
            vec![
                (1, dig()),
                (2, dig()),
                (
                    3,
                    Effect::PutFromHandOntoBattlefield {
                        who: PlayerRef::You,
                        filter: non_saga_permanent(),
                        count: Value::Const(2),
                        tapped: false,
                        haste: false,
                        sacrifice_eot: false,
                        return_eot: false,
                        then: None,
                    },
                ),
            ],
        )
    }
}

// ── Yennett, Cryptic Sovereign ───────────────────────────────────────────────

/// Hidetsugu and Kairi — {2}{U}{U}{B} 5/4 legendary Ogre Demon Dragon, flying.
/// ETB: draw three, then put two cards from your hand on top. Dies: exile your
/// top card; target opponent loses life equal to its mana value, and if it's
/// an instant or sorcery you may cast it free.
pub fn hidetsugu_and_kairi() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                Effect::ChooseFromHandToTopOfLibrary { who: PlayerRef::You, count: Value::Const(2) },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::ExileTopOfLibrary {
                        who: Selector::You,
                        amount: Value::ONE,
                        link_to_source: false,
                        face_down: false,
                    },
                    Effect::LoseLife {
                        who: target_filtered(R::OpponentPlayer),
                        amount: Value::ManaValueOf(Box::new(Selector::LastMoved)),
                    },
                    Effect::If {
                        cond: Predicate::EntityMatches { what: Selector::LastMoved, filter: instant_or_sorcery() },
                        then: Box::new(Effect::CastWithoutPayingImmediate {
                            what: Selector::LastMoved,
                            source_zone: Zone::Exile,
                            exile_after: false,
                            copy: false,
                            reduce_generic: 0,
                            pay_own_cost: false,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..creature(
            "Hidetsugu and Kairi",
            cost(&[generic(2), u(), u(), b()]),
            vec![CreatureType::Ogre, CreatureType::Demon, CreatureType::Dragon],
            5,
            4,
        )
    }
}

/// Scholar of the Lost Trove — {5}{U}{U} 5/5 Sphinx, flying. ETB: you may cast
/// target instant, sorcery or artifact card from your graveyard free; an
/// instant or sorcery cast this way is exiled instead of going to a graveyard.
pub fn scholar_of_the_lost_trove() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CastWithoutPayingImmediate {
            what: target_filtered(instant_or_sorcery().or(R::Artifact).and(R::InYourGraveyard)),
            source_zone: Zone::Graveyard,
            exile_after: true,
            copy: false,
            reduce_generic: 0,
            pay_own_cost: false,
        })],
        ..creature("Scholar of the Lost Trove", cost(&[generic(5), u(), u()]), vec![CreatureType::Sphinx], 5, 5)
    }
}

/// Vesuvan Drifter — {2}{U} 2/4 Shapeshifter, flying. You may look at your top
/// card any time. At the beginning of each combat, you may reveal it; if it's
/// a creature card, this becomes a copy of it until end of turn, except it
/// has flying.
pub fn vesuvan_drifter() -> CardDefinition {
    let top = || Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You may look at the top card of your library any time.",
            effect: StaticEffect::MayLookAtOwnLibraryTop,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer),
            effect: Effect::If {
                cond: Predicate::EntityMatches { what: top(), filter: R::Creature },
                then: Box::new(Effect::MayDo {
                    description: "Reveal your top card and become a copy of it until end of turn?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::BecomeCopyOfFor {
                            what: Selector::This,
                            source: top(),
                            duration: Duration::EndOfTurn,
                            non_legendary: false,
                        },
                        Effect::GrantKeyword {
                            what: Selector::This,
                            keyword: Keyword::Flying,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature("Vesuvan Drifter", cost(&[generic(2), u()]), vec![CreatureType::Shapeshifter], 2, 4)
    }
}

/// Brilliant Ultimatum — {W}{W}{U}{U}{U}{B}{B} sorcery. Exile your top five;
/// an opponent separates them into two piles; you play a land and cast any of
/// the spells from one pile free, during the resolution (2008-10-01 ruling).
pub fn brilliant_ultimatum() -> CardDefinition {
    let pile = || Selector::SeparatedPile { chosen: true };
    spell(
        "Brilliant Ultimatum",
        cost(&[w(), w(), u(), u(), u(), b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::ExileTopOfLibrary { who: Selector::You, amount: Value::Const(5), link_to_source: false, face_down: false },
            Effect::SeparateIntoPiles {
                what: Selector::LastMoved,
                splitter: PlayerRef::OpponentOf(Box::new(PlayerRef::You)),
                chooser: PlayerRef::You,
                chosen: Box::new(Effect::Seq(vec![
                    Effect::PlayLandAmongNow { what: pile() },
                    Effect::CastAnyOrderWithoutPaying {
                        what: pile(),
                        source_zone: Zone::Exile,
                        filter: None,
                        cap: None,
                        total_mana_value: None,
                        exile_after: false,
                    },
                ])),
                other: Box::new(Effect::Noop),
            },
        ]),
    )
}
