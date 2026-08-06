//! Murders at Karlov Manor (MKM) — second gap wave. Cases, split cards, the
//! face-down payoffs, and the multicolor legends. Tests in
//! `tests/classic_sets/mkm2.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, ConditionalEquipBonus, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, SplitCard, SplitHalf, StaticAbility, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{
    cast_is_instant_or_sorcery, deal, draw, each_your_creature, etb, investigate, on_attack,
    on_dies, target_any, target_filtered, target_n,
};
use crate::effect::{
    CounteredSpellZone, Duration, Effect, LibraryPosition, LookPick, ManaPayload, PlayerRef,
    Predicate, Selector, StaticEffect, Value, ZoneDest,
};
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w};

fn creature(
    name: &'static str,
    c: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    c: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, c, types, p, t) }
}

fn case(name: &'static str, c: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Case],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Goblin Maskmaker — {R} 1/2. "Whenever this creature attacks, face-down
/// spells you cast this turn cost {1} less to cast."
pub fn goblin_maskmaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::FaceDownSpellsCostLessThisTurn { amount: 1 })],
        ..creature(
            "Goblin Maskmaker",
            cost(&[r()]),
            vec![CreatureType::Goblin, CreatureType::Citizen],
            1,
            2,
        )
    }
}

/// Tin Street Gossip — {2}{R}{G} 4/4 vigilance. "{T}: Add {R}{G}. Spend this
/// mana only to cast face-down spells or to turn creatures face up."
pub fn tin_street_gossip() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::Red, Color::Green])),
                    crate::mana::SpendRestriction::FaceDownSpellsOrTurnFaceUp,
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Tin Street Gossip",
            cost(&[generic(2), r(), g()]),
            vec![CreatureType::Lizard, CreatureType::Advisor],
            4,
            4,
        )
    }
}

fn imp_token() -> TokenDefinition {
    TokenDefinition {
        name: "Imp".into(),
        colors: vec![Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Imp], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![on_dies(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Judith, Carnage Connoisseur — {3}{B}{R} 3/4. Each of your instants and
/// sorceries either gains deathtouch and lifelink or mints a 2/2 Imp that
/// pings each opponent when it dies.
pub fn judith_carnage_connoisseur() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::ChooseMode(vec![
                Effect::GrantKeywordsToSpell {
                    what: Selector::TriggerSource,
                    keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: imp_token(),
                },
            ]),
        }],
        ..legend(
            "Judith, Carnage Connoisseur",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            3,
            4,
        )
    }
}

/// Case of the Burning Masks — {1}{R}{R} Enchantment — Case. ETB deals 3;
/// solved once three of your sources dealt damage this turn, and then it can
/// be sacrificed to exile three cards and play one.
pub fn case_of_the_burning_masks() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(deal(
            3,
            target_filtered(R::Creature.and(R::ControlledByOpponent)),
        ))],
        case: Some(Box::new(crate::card::CaseData {
            to_solve: Predicate::SourcesYouControlledDealtDamageThisTurnAtLeast(3),
            solved_activated: vec![ActivatedAbility {
                sac_cost: true,
                effect: Effect::LookTopExileOneMayPlay {
                    count: Value::Const(3),
                    who: PlayerRef::You,
                },
                ..Default::default()
            }],
            ..Default::default()
        })),
        ..case("Case of the Burning Masks", cost(&[generic(1), r(), r()]))
    }
}

/// Case of the Gorgon's Kiss — {B} Enchantment — Case. ETB destroys a damaged
/// creature; solved once three creature cards hit graveyards this turn, and
/// then it is itself a 4/4 Gorgon with deathtouch and lifelink.
pub fn case_of_the_gorgons_kiss() -> CardDefinition {
    let always = |effect| StaticAbility { description: "Solved — 4/4 Gorgon.", effect };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            min_targets: 0,
            max_targets: 1,
            filter: R::Creature.and(R::DealtDamageThisTurn),
            effect: Box::new(Effect::Destroy { what: target_n(0) }),
        })],
        case: Some(Box::new(crate::card::CaseData {
            to_solve: Predicate::CreatureCardsToGraveyardThisTurnAtLeast(3),
            solved_static: vec![
                always(StaticEffect::SelfIsCreatureIf {
                    condition: Predicate::True,
                    creature_types: vec![CreatureType::Gorgon],
                }),
                always(StaticEffect::SetBasePtIf {
                    condition: Predicate::True,
                    power: 4,
                    toughness: 4,
                }),
                always(StaticEffect::SelfHasKeywordIf {
                    keyword: Keyword::Deathtouch,
                    condition: Predicate::True,
                }),
                always(StaticEffect::SelfHasKeywordIf {
                    keyword: Keyword::Lifelink,
                    condition: Predicate::True,
                }),
            ],
            ..Default::default()
        })),
        ..case("Case of the Gorgon's Kiss", cost(&[b()]))
    }
}

/// Burden of Proof — {1}{U} Aura with flash. Enchanted creature gets +2/+2
/// while it's a Detective you control; otherwise it's a 1/1 that can't block
/// Detectives.
pub fn burden_of_proof() -> CardDefinition {
    let yours = R::HasCreatureType(CreatureType::Detective).and(R::ControlledByYou);
    CardDefinition {
        name: "Burden of Proof",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            conditional: vec![
                ConditionalEquipBonus {
                    host_filter: yours.clone(),
                    power: 2,
                    toughness: 2,
                    ..Default::default()
                },
                ConditionalEquipBonus {
                    host_filter: R::Not(Box::new(yours)),
                    set_base_pt: Some((1, 1)),
                    keywords: vec![Keyword::CantBlockCreatureType(CreatureType::Detective)],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Break Out — {R}{G} Sorcery. Look at the top six; a revealed creature of
/// mana value 2 or less enters with haste, a pricier one goes to hand, and the
/// rest are bottomed at random.
pub fn break_out() -> CardDefinition {
    CardDefinition {
        name: "Break Out",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand(Box::new(LookPick {
            count: Value::Const(6),
            pick_filter: Some(R::Creature),
            optional: true,
            rest_bottom_random: true,
            picked_matching_to_battlefield: Some(R::ManaValueAtMost(2)),
            battlefield_haste: true,
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Sudden Setback — {2}{U}{U} Instant. The owner of target spell or nonland
/// permanent puts it on their choice of the top or bottom of their library.
pub fn sudden_setback() -> CardDefinition {
    CardDefinition {
        name: "Sudden Setback",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpellToZone {
                what: target_filtered(R::IsSpellOnStack),
                zone: CounteredSpellZone::OwnerLibraryTopOrBottom,
            },
            Effect::Move {
                what: target_filtered(R::Permanent.and(R::Nonland)),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::OwnerChoice,
                },
            },
        ]),
        ..Default::default()
    }
}

/// Push // Pull — {1}{W/B} // {4}{B/R}{B/R} Sorcery // Sorcery. Push destroys a
/// tapped creature; Pull reanimates up to two creature cards from one graveyard
/// with haste and sacrifices them at the next end step.
pub fn push_pull() -> CardDefinition {
    CardDefinition {
        name: "Push // Pull",
        cost: cost(&[generic(1), hybrid(Color::White, Color::Black)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::Tapped)) },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[
                    generic(4),
                    hybrid(Color::Black, Color::Red),
                    hybrid(Color::Black, Color::Red),
                ]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![
                    Effect::ApplyToTargets {
                        min_targets: 0,
                        max_targets: 2,
                        filter: R::Creature.and(R::InGraveyard),
                        effect: Box::new(Effect::Move {
                            what: target_n(0),
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::You,
                                tapped: false,
                            },
                        }),
                    },
                    Effect::GrantKeyword {
                        what: Selector::LastMoved,
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::SacrificeAtNextEndStep { what: Selector::LastMoved },
                ]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Flotsam // Jetsam — {1}{G/U} // {4}{U/B}{U/B} Instant // Sorcery. Flotsam
/// mills three and investigates; Jetsam mills each opponent three, then casts a
/// free spell out of an opponent's graveyard.
pub fn flotsam_jetsam() -> CardDefinition {
    CardDefinition {
        name: "Flotsam // Jetsam",
        cost: cost(&[generic(1), hybrid(Color::Green, Color::Blue)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            investigate(1),
        ]),
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[
                    generic(4),
                    hybrid(Color::Blue, Color::Black),
                    hybrid(Color::Blue, Color::Black),
                ]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![
                    Effect::Mill {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(3),
                    },
                    Effect::CastAnyOrderWithoutPaying {
                        what: Selector::CardsInZone {
                            who: PlayerRef::EachOpponent,
                            zone: crate::card::Zone::Graveyard,
                            filter: R::Nonland,
                        },
                        source_zone: crate::card::Zone::Graveyard,
                        filter: None,
                        cap: Some(Value::OpponentCount),
                    },
                ]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Hustle // Bustle — {U/R} // {4}{R/G}{R/G} Instant // Sorcery. Hustle forces
/// a creature to attack or block; Bustle pumps the team +2/+2 with trample and
/// unmasks one of your face-down creatures.
pub fn hustle_bustle() -> CardDefinition {
    CardDefinition {
        name: "Hustle // Bustle",
        cost: cost(&[hybrid(Color::Blue, Color::Red)]),
        card_types: vec![CardType::Instant],
        effect: Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::MustAttackOrBlock,
            duration: Duration::EndOfTurn,
        },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[
                    generic(4),
                    hybrid(Color::Red, Color::Green),
                    hybrid(Color::Red, Color::Green),
                ]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: each_your_creature(),
                        power: Value::Const(2),
                        toughness: Value::Const(2),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: each_your_creature(),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::TurnFaceUpFree {
                        what: Selector::ControlledBy { who: PlayerRef::You, filter: R::FaceDown },
                    },
                ]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Coveted Falcon — {1}{U}{U} 1/4 flier with disguise {1}{U}. Attacking claws
/// back a permanent you own but don't control; unmasking it hands an opponent
/// up to three of your permanents and draws that many.
pub fn coveted_falcon() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Disguise(cost(&[generic(1), u()]))],
        triggered_abilities: vec![
            on_attack(Effect::GainControl {
                what: target_filtered(R::Permanent.and(R::OwnedByYou).and(R::ControlledByOpponent)),
                to: None,
                duration: Duration::Permanent,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
                effect: Effect::ApplyToTargets {
                    min_targets: 0,
                    max_targets: 3,
                    filter: R::Permanent.and(R::ControlledByYou).and(R::OtherThanSource),
                    effect: Box::new(Effect::Seq(vec![
                        Effect::GainControl {
                            what: target_n(0),
                            to: Some(PlayerRef::EachOpponent),
                            duration: Duration::Permanent,
                        },
                        draw(1),
                    ])),
                },
            },
        ],
        ..creature("Coveted Falcon", cost(&[generic(1), u(), u()]), vec![CreatureType::Bird], 1, 4)
    }
}

/// Yarus, Roar of the Old Gods — {2}{R}{G} 4/4. Your other creatures have
/// haste, connecting with a face-down creature draws, and a face-down creature
/// that dies comes straight back and flips up.
pub fn yarus_roar_of_the_old_gods() -> CardDefinition {
    let face_down_trigger = |what| Predicate::EntityMatches { what, filter: R::FaceDown };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have haste.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::OtherThanSource),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Haste],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .with_filter(face_down_trigger(Selector::TriggerSource))
                    .once_per_turn(),
                effect: draw(1),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(face_down_trigger(Selector::TriggerSource)),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)),
                            tapped: false,
                        },
                    },
                    Effect::TurnFaceUpFree { what: Selector::LastMoved },
                ]),
            },
        ],
        ..legend(
            "Yarus, Roar of the Old Gods",
            cost(&[generic(2), r(), g()]),
            vec![CreatureType::Centaur, CreatureType::Druid],
            4,
            4,
        )
    }
}

/// Illicit Masquerade — {3}{B} Enchantment with flash. Marks your creatures
/// with impostor counters; a marked creature that dies is exiled and drags a
/// creature card back out of your graveyard.
pub fn illicit_masquerade() -> CardDefinition {
    CardDefinition {
        name: "Illicit Masquerade",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: each_your_creature(),
                kind: CounterType::Impostor,
                amount: Value::ONE,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::WithCounter(CounterType::Impostor),
                    }),
                effect: Effect::Seq(vec![
                    Effect::Exile { what: Selector::TriggerSource },
                    Effect::ApplyToTargets {
                        min_targets: 0,
                        max_targets: 1,
                        filter: R::Creature.and(R::InYourGraveyard),
                        effect: Box::new(Effect::Move {
                            what: target_n(0),
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::You,
                                tapped: false,
                            },
                        }),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Blood Spatter Analysis — {B}{R} Enchantment. ETB pings a creature; every
/// death mills and stains it, and the fifth stain sacrifices it to buy back a
/// creature card.
pub fn blood_spatter_analysis() -> CardDefinition {
    CardDefinition {
        name: "Blood Spatter Analysis",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(deal(3, target_filtered(R::Creature.and(R::ControlledByOpponent)))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
                effect: Effect::Seq(vec![
                    Effect::Mill { who: Selector::You, amount: Value::ONE },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Bloodstain,
                        amount: Value::ONE,
                    },
                    Effect::If {
                        cond: Predicate::SourceHasCountersAtLeast {
                            counter: CounterType::Bloodstain,
                            n: 5,
                        },
                        then: Box::new(Effect::Seq(vec![
                            Effect::SacrificeSource,
                            Effect::ReflexiveTrigger {
                                body: Box::new(Effect::Move {
                                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                                    to: ZoneDest::Hand(PlayerRef::You),
                                }),
                            },
                        ])),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Connecting the Dots — {1}{R} Enchantment. Each attack banks the top card of
/// your library face down; cash the pile in by discarding your hand and
/// sacrificing the enchantment.
pub fn connecting_the_dots() -> CardDefinition {
    CardDefinition {
        name: "Connecting the Dots",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
            effect: Effect::ExileWithSource {
                what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::CardsInHandMatching { who: PlayerRef::You, filter: R::Any },
                    random: false,
                },
                Effect::Move {
                    what: Selector::CardExiledWithSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(
                        Selector::CardExiledWithSource,
                    ))),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lazav, Wearer of Faces — {U}{B} 2/3. Attacks exile a graveyard card and
/// investigate; cashing a Clue lets Lazav copy a creature card exiled with it.
pub fn lazav_wearer_of_faces() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::Seq(vec![
                Effect::ExileWithSource { what: target_filtered(R::InGraveyard) },
                investigate(1),
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasName("Clue".into()),
                    }),
                effect: Effect::MayDo {
                    description: "Become a copy of a creature card exiled with Lazav".into(),
                    body: Box::new(Effect::BecomeCopyOf {
                        what: Selector::This,
                        source: Selector::MatchingAmong {
                            inner: Box::new(Selector::CardExiledWithSource),
                            filter: R::Creature,
                        },
                        extra_creature_types: vec![],
                        keep_own_triggered: false,
                        keep_own_activated: false,
                    }),
                },
            },
        ],
        ..legend(
            "Lazav, Wearer of Faces",
            cost(&[u(), b()]),
            vec![CreatureType::Shapeshifter, CreatureType::Detective],
            2,
            3,
        )
    }
}

/// Niv-Mizzet, Guildpact — {W}{U}{B}{R}{G} 6/6 flier with hexproof from
/// multicolored. Combat damage to a player pays out X across a ping, a draw,
/// and life, where X counts your distinct exactly-two-color pairs.
pub fn niv_mizzet_guildpact() -> CardDefinition {
    let x = Value::DistinctTwoColorPairsControlled(PlayerRef::You);
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::HexproofFromMulticolored],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: target_any(), amount: x.clone() },
                Effect::Draw { who: Selector::Player(PlayerRef::Target(1)), amount: x.clone() },
                Effect::GainLife { who: Selector::You, amount: x },
            ]),
        }],
        ..legend(
            "Niv-Mizzet, Guildpact",
            cost(&[w(), u(), b(), r(), g()]),
            vec![CreatureType::Dragon, CreatureType::Avatar],
            6,
            6,
        )
    }
}
