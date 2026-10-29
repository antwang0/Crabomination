//! Commander: three-card gaps for Abigale Poet Laureate, Grist Voracious
//! Larva and Gandalf the Grey (`cmdr_edhrec38` is the previous batch of this
//! run). Tests in `tests/recent_b/cmdr_edhrec41.rs`.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility,
    PlaneswalkerSubtype, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, ZoneDest};
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn legendary_equipment(name: &'static str, mana: ManaCost, equip: u32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(equip)]))],
        ..Default::default()
    }
}

fn insect_token() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Insect".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Black, Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        ..Default::default()
    })
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

fn plus_one_each(what: Selector) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }
}

// ── Abigale, Poet Laureate ───────────────────────────────────────────────────

/// Generous Pup — {1}{W} 2/2 Dog, vigilance. Whenever one or more +1/+1
/// counters are put on it, a +1/+1 counter on each other creature you
/// control; only once each turn.
pub fn generous_pup() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::SelfSource)
                .once_per_turn(),
            effect: plus_one_each(yours(R::Creature.and(R::OtherThanSource))),
        }],
        ..creature("Generous Pup", cost(&[generic(1), w()]), vec![CreatureType::Dog], 2, 2)
    }
}

/// Hagra Constrictor — {2}{B} 0/0 Snake. Enters with two +1/+1 counters;
/// each creature you control with a +1/+1 counter has menace.
pub fn hagra_constrictor() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        static_abilities: vec![StaticAbility {
            description: "Each creature you control with a +1/+1 counter on it has menace.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne))),
                keyword: Keyword::Menace,
            },
        }],
        ..creature("Hagra Constrictor", cost(&[generic(2), b()]), vec![CreatureType::Snake], 0, 0)
    }
}

/// Leader's Talent — {1}{W} Class. Level 1: whenever you attack, a +1/+1
/// counter on target attacking creature. Level 2 ({2}{W}): a creature of
/// yours leaving with a counter on it gains you 2 life. Level 3 ({3}{W}):
/// whenever you cast a spell, a +1/+1 counter on each creature you control.
pub fn leaders_talent() -> CardDefinition {
    let level_up = |mana: ManaCost, from: u8| ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    CardDefinition {
        name: "Leader's Talent",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Class], ..Default::default() },
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
                effect: plus_one_each(target_filtered(R::Creature.and(R::IsAttacking))),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl).with_filter(
                    Predicate::All(vec![
                        Predicate::SourceClassLevelAtLeast(2),
                        Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::WithAnyCounter) },
                    ]),
                ),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::SourceClassLevelAtLeast(3)),
                effect: plus_one_each(yours(R::Creature)),
            },
        ],
        activated_abilities: vec![level_up(cost(&[generic(2), w()]), 1), level_up(cost(&[generic(3), w()]), 2)],
        ..Default::default()
    }
}

// ── Grist, Voracious Larva ───────────────────────────────────────────────────

/// Grist, Voracious Larva // Grist, the Plague Swarm — {G} legendary 1/2
/// Insect with deathtouch. Whenever it or another creature you control enters
/// from your graveyard (or cast from it), you may pay {G} to exile Grist and
/// return it transformed: a 3-loyalty planeswalker.
pub fn grist_voracious_larva() -> CardDefinition {
    let plague_swarm = CardDefinition {
        name: "Grist, the Plague Swarm",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Grist], ..Default::default() },
        color_indicator: vec![Color::Black, Color::Green],
        base_loyalty: 3,
        loyalty_abilities: vec![
            // +1: an Insect, mill two, deathtouch counter if a black card was milled.
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: insect_token() },
                    Effect::Mill { who: Selector::You, amount: Value::Const(2) },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(
                            Value::CardsMilledThisEffectMatching { filter: R::HasColor(Color::Black) },
                            Value::ONE,
                        ),
                        then: Box::new(Effect::AddKeywordCounter {
                            what: Selector::LastCreatedToken,
                            keyword: Keyword::Deathtouch,
                            amount: Value::ONE,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
                ..Default::default()
            },
            // −6: a 1/1 black and green Insect copy of each creature card in
            // your graveyard (CR 707.9b — the types are replaced, not added).
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::ForEach {
                    selector: Selector::EachMatching {
                        zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                        filter: R::Creature,
                    },
                    body: Box::new(Effect::Seq(vec![
                        Effect::CreateTokenCopyOf {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            source: Selector::TriggerSource,
                            extra_creature_types: vec![],
                            extra_card_types: vec![],
                            override_pt: Some((1, 1)),
                            override_colors: Some(vec![Color::Black, Color::Green]),
                            enters_tapped: false,
                            non_legendary: false,
                            legendary: false,
                            extra_keywords: vec![],
                            no_mana_cost: false,
                            enters_with_counters: None,
                            remove_keywords: vec![],
                        },
                        Effect::AmendCopiableValues {
                            what: Selector::LastCreatedToken,
                            name: None,
                            set_creature_types: Some(vec![CreatureType::Insect]),
                            add_creature_types: vec![],
                            legendary: false,
                            add_colors: vec![],
                            set_card_types: None,
                        },
                    ])),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::EnteredFromGraveyardThisTurn),
            }),
            effect: Effect::MayPay {
                description: "Pay {G} to transform Grist?".into(),
                mana_cost: cost(&[g()]),
                body: Box::new(Effect::ExileSelfReturnTransformed),
                else_: None,
            },
        }],
        back_face: Some(Box::new(plague_swarm)),
        ..legend("Grist, Voracious Larva", cost(&[g()]), vec![CreatureType::Insect], 1, 2)
    }
}

/// Amzu, Swarm's Hunger — {3}{B}{G} legendary 3/3 Insect Shaman, flying,
/// menace; other Insects you control have menace. Once each turn, when one or
/// more cards leave your graveyard, you may make a 1/1 Insect with +1/+1
/// counters equal to the greatest mana value among them.
pub fn amzu_swarms_hunger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Other Insects you control have menace.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature.and(R::HasCreatureType(CreatureType::Insect)).and(R::OtherThanSource)),
                keyword: Keyword::Menace,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            // "Do this only once each turn": spent by a taken may.
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
                .once_per_batch()
                .with_filter(crate::card::Predicate::Not(Box::new(crate::card::Predicate::SourceDoneThisTurn))),
            effect: Effect::OverTriggerBatch {
                ids: vec![],
                body: Box::new(Effect::MayDo {
                    description: "Create an Insect with counters equal to the greatest mana value among them?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::MarkDoneThisTurn,
                        Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: insect_token() },
                        Effect::AddCounter {
                            what: Selector::LastCreatedToken,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::HighestManaValueAmong(Box::new(Selector::TriggerBatch)),
                        },
                    ])),
                }),
            },
        }],
        ..legend(
            "Amzu, Swarm's Hunger",
            cost(&[generic(3), b(), g()]),
            vec![CreatureType::Insect, CreatureType::Shaman],
            3,
            3,
        )
    }
}

/// Zask, Skittering Swarmlord — {3}{G}{G} legendary 5/5 Insect. You may play
/// lands and cast Insect spells from your graveyard. Another Insect of yours
/// dying goes to the bottom of its owner's library, then you mill two.
/// {1}{B/G}: target Insect gets +1/+0 and deathtouch until end of turn.
pub fn zask_skittering_swarmlord() -> CardDefinition {
    let insect = || R::HasCreatureType(CreatureType::Insect);
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "You may play lands from your graveyard.",
                effect: StaticEffect::MayPlayLandsFromGraveyard,
            },
            StaticAbility {
                description: "You may cast Insect spells from your graveyard.",
                effect: StaticEffect::CastFromGraveyardMatching { filter: insect() },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: insect() }),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Bottom },
                },
                Effect::Mill { who: Selector::You, amount: Value::Const(2) },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), hybrid(Color::Black, Color::Green)]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(insect())),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..legend("Zask, Skittering Swarmlord", cost(&[generic(3), g(), g()]), vec![CreatureType::Insect], 5, 5)
    }
}

// ── Gandalf the Grey ─────────────────────────────────────────────────────────

/// Gandalf the Grey — {3}{U}{R} legendary 3/4 Avatar Wizard. Whenever you
/// cast an instant or sorcery spell, choose one that hasn't been chosen: tap
/// or untap a permanent; 3 damage to each opponent; copy an instant or
/// sorcery spell of yours; put Gandalf on top of its owner's library.
pub fn gandalf_the_grey() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(instant_or_sorcery())),
            effect: Effect::ChooseUnchosenMode {
                modes: vec![
                    Effect::TapOrUntap { what: target_filtered(R::Permanent) },
                    Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(3) },
                    Effect::CopySpellMayChooseTargets {
                        what: target_filtered(R::IsSpellOnStack.and(instant_or_sorcery()).and(R::ControlledByYou)),
                        count: Value::ONE,
                    },
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Library { who: PlayerRef::OwnerOf(Box::new(Selector::This)), pos: LibraryPosition::Top },
                    },
                ],
            },
        }],
        ..legend(
            "Gandalf the Grey",
            cost(&[generic(3), u(), r()]),
            vec![CreatureType::Avatar, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Glamdring — {2} legendary Equipment. Equipped creature has first strike
/// and gets +1/+0 per instant and sorcery card in your graveyard; its combat
/// damage to a player lets you cast an instant or sorcery from your hand with
/// mana value up to that damage free. Equip {3}.
pub fn glamdring() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::FirstStrike],
            scale: Some(EquipScale {
                filter: R::Any,
                per_power: 1,
                per_toughness: 0,
                count_graveyard: Some(instant_or_sorcery()),
                ..Default::default()
            }),
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MayCastFromHandFreeMatching {
                    filter: instant_or_sorcery(),
                    max_mv: Value::TriggerEventAmount,
                    else_: Box::new(Effect::Noop),
                },
            }],
            triggers_on_equipment: true,
            ..Default::default()
        }),
        ..legendary_equipment("Glamdring", cost(&[generic(2)]), 3)
    }
}

/// Glamdring, Foe-hammer // Gleam of Death — {2} legendary Equipment: your
/// instant and sorcery spells cost {X} less, X = equipped creature's power.
/// Equip {2}. Adventure {3}{U} sorcery: mill six, then put the instants and
/// sorceries milled into your hand.
pub fn glamdring_foe_hammer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {X} less to cast, where X is equipped \
                          creature's power.",
            effect: StaticEffect::CostReductionByValue {
                filter: instant_or_sorcery(),
                amount: Value::PowerOf(Box::new(Selector::AttachedTo(Box::new(Selector::This)))),
            },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Gleam of Death",
            cost: cost(&[generic(3), u()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(6) },
                Effect::Move {
                    what: Selector::MatchingAmong { inner: Box::new(Selector::LastMoved), filter: instant_or_sorcery() },
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ]),
        })),
        ..legendary_equipment("Glamdring, Foe-hammer", cost(&[generic(2)]), 2)
    }
}
