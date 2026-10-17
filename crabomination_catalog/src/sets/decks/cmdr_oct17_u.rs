//! Commander: 2026-10-17 (`3chree`) closers, fifth batch — one-deck closers
//! from the EDHREC long tail. Tests in `tests/recent_b/cmdr_oct17_u.rs`.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec,
    ExileReturnZone, Keyword, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{
    dash, encore, etb, if_discarded, investigate_by, on_attack, on_dies, target_any,
    target_filtered,
};
use crate::effect::{
    Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

/// Heroes Remembered — {6}{W}{W}{W} sorcery: gain 20 life. Suspend 10—{W}.
pub fn heroes_remembered() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(10, cost(&[w()]))],
        ..spell(
            "Heroes Remembered",
            cost(&[generic(6), w(), w(), w()]),
            CardType::Sorcery,
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(20),
            },
        )
    }
}

/// A.I.M. Scientists — {3}{U} 3/3 Human Scientist Villain. ETB: it connives.
/// Basic landcycling {2}.
pub fn aim_scientists() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Typecycling(Box::new((
            cost(&[generic(2)]),
            R::IsBasicLand,
        )))],
        triggered_abilities: vec![etb(Effect::Connive {
            what: Selector::This,
            amount: Value::ONE,
        })],
        ..creature(
            "A.I.M. Scientists",
            cost(&[generic(3), u()]),
            vec![
                CreatureType::Human,
                CreatureType::Scientist,
                CreatureType::Villain,
            ],
            3,
            3,
        )
    }
}

/// Panther Pounce — {W} instant. Target player investigates. Target creature
/// gets +1/+0 and gains flying until end of turn. Untap it.
pub fn panther_pounce() -> CardDefinition {
    let it = || Selector::TargetFiltered {
        slot: 1,
        filter: R::Creature,
    };
    spell(
        "Panther Pounce",
        cost(&[w()]),
        CardType::Instant,
        Effect::Seq(vec![
            // Slot 0 — the player — is declared by the token's `who`.
            investigate_by(PlayerRef::Target(0), Value::ONE),
            Effect::PumpPT {
                what: it(),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: it(),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(1),
                up_to: None,
            },
        ]),
    )
}

/// Vivien of the Arkbow — {4}{G}{G} planeswalker, loyalty 5. +2: two +1/+1
/// counters on up to one creature. −3: your creature deals damage equal to its
/// power to a creature you don't control. −9: your creatures +4/+4 and trample.
pub fn vivien_of_the_arkbow() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype};
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Vivien of the Arkbow",
        cost: cost(&[generic(4), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Vivien],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::AddCounter {
                        what: target_filtered(R::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::DealDamageEqualToPower {
                    source: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::ControlledByYou),
                    },
                    target: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::ControlledByYou.negate()),
                    },
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -9,
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: yours(),
                        power: Value::Const(4),
                        toughness: Value::Const(4),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: yours(),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Rhovanion Rampager — {2}{B} 3/2 Wolf. Attacks: you may sacrifice another
/// creature for +1/+1 counters equal to its power. Dies: amass Goblins X (X = its power).
pub fn rhovanion_rampager() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::MayDo {
                description: "Sacrifice another creature for +1/+1 counters equal to its power?"
                    .into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::SacrificeAndRemember {
                        who: PlayerRef::You,
                        filter: R::Creature.and(R::OtherThanSource),
                    },
                    Effect::If {
                        cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                        then: Box::new(Effect::AddCounter {
                            what: Selector::This,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::SacrificedPower,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            }),
            on_dies(Effect::Amass {
                who: PlayerRef::You,
                count: Value::PowerOf(Box::new(Selector::This)),
                extra_type: Some(CreatureType::Goblin),
            }),
        ],
        ..creature(
            "Rhovanion Rampager",
            cost(&[generic(2), b()]),
            vec![CreatureType::Wolf],
            3,
            2,
        )
    }
}

/// Blessed Hippogriff // Tyr's Blessing — {3}{W} 2/3 flying Hippogriff; attacks:
/// target attacking creature without flying gains flying. Adventure {W}
/// instant: target creature gains indestructible until end of turn.
pub fn blessed_hippogriff() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(
                R::IsAttacking.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
            ),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        adventure: Some(Box::new(Adventure {
            name: "Tyr's Blessing",
            cost: cost(&[w()]),
            card_types: vec![CardType::Instant],
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        })),
        ..creature(
            "Blessed Hippogriff",
            cost(&[generic(3), w()]),
            vec![CreatureType::Hippogriff],
            2,
            3,
        )
    }
}

/// Ecstatic Awakener // Awoken Demon — {B} 1/1 Human Wizard. {2}{B}, sacrifice
/// another creature: draw a card, then transform it (once each turn). Back: 4/4 black Demon.
pub fn ecstatic_awakener() -> CardDefinition {
    let demon = CardDefinition {
        name: "Awoken Demon",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        color_indicator: vec![Color::Black],
        power: 4,
        toughness: 4,
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_other_filter: Some((R::Creature.and(R::ControlledByYou), 1)),
            once_per_turn: true,
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::Transform {
                    what: Selector::This,
                },
            ]),
            ..Default::default()
        }],
        back_face: Some(Box::new(demon)),
        ..creature(
            "Ecstatic Awakener",
            cost(&[b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Elaborate Firecannon — {2} artifact; doesn't untap in your untap step.
/// {4},{T}: 2 damage to any target. Upkeep: you may discard a card; if you do, untap it.
pub fn elaborate_firecannon() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Elaborate Firecannon",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "This artifact doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::This,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Discard a card to untap Elaborate Firecannon?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::ONE,
                        random: false,
                    },
                    if_discarded(Effect::Untap {
                        what: Selector::This,
                        up_to: None,
                    }),
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Urborg Repossession — {B} sorcery, kicker {1}{G}. Return target creature card
/// from your graveyard to hand; gain 2. Kicked: also another target permanent card.
pub fn urborg_repossession() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1), g()]))],
        ..spell(
            "Urborg Repossession",
            cost(&[b()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::InYourGraveyard),
                    },
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::Move {
                        what: Selector::TargetFiltered {
                            slot: 1,
                            filter: R::PermanentCard
                                .and(R::InYourGraveyard)
                                .and(R::OtherThanTargetSlot(0)),
                        },
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Sadistic Obsession — {3}{B} Aura. Enchanted creature has "{B}, {T}: Put a
/// -1/-1 counter on target creature."
pub fn sadistic_obsession() -> CardDefinition {
    CardDefinition {
        name: "Sadistic Obsession",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has \"{B}, {T}: Put a -1/-1 counter on target creature.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    mana_cost: cost(&[b()]),
                    tap_cost: true,
                    effect: Effect::AddCounter {
                        what: target_filtered(R::Creature),
                        kind: CounterType::MinusOneMinusOne,
                        amount: Value::ONE,
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Whiteout — {1}{G} instant: all creatures lose flying until end of turn.
/// Sacrifice a snow land: return this card from your graveyard to your hand.
pub fn whiteout() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            sac_other_filter: Some((
                R::Land
                    .and(R::HasSupertype(Supertype::Snow))
                    .and(R::ControlledByYou),
                1,
            )),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..spell(
            "Whiteout",
            cost(&[generic(1), g()]),
            CardType::Instant,
            Effect::LoseKeyword {
                what: Selector::EachPermanent(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Halo-Charged Skaab — {4}{U} 4/4 Zombie. ETB: each player mills two, then you
/// may put an instant, sorcery, or battle card from your graveyard on top of your library.
pub fn halo_charged_skaab() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(2),
            },
            Effect::MoveChosen {
                from: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::HasCardType(CardType::Instant)
                        .or(R::HasCardType(CardType::Sorcery))
                        .or(R::HasCardType(CardType::Battle)),
                },
                filter: None,
                count: Value::ONE,
                up_to: true,
                to: ZoneDest::Library {
                    who: PlayerRef::You,
                    pos: LibraryPosition::Top,
                },
            },
        ]))],
        ..creature(
            "Halo-Charged Skaab",
            cost(&[generic(4), u()]),
            vec![CreatureType::Zombie],
            4,
            4,
        )
    }
}

/// Defang — {1}{W} Aura, enchant creature. Prevent all damage that would be
/// dealt by enchanted creature (Temporal Isolation's static).
pub fn defang() -> CardDefinition {
    CardDefinition {
        name: "Defang",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt by enchanted creature.",
            effect: StaticEffect::PreventAllDamageByEnchanted,
        }],
        ..Default::default()
    }
}

/// Fork in the Road — {1}{G} sorcery. Up to two basic lands: one to hand, the
/// other to the graveyard, then shuffle (Final Parting's two-search shape).
pub fn fork_in_the_road() -> CardDefinition {
    spell(
        "Fork in the Road",
        cost(&[generic(1), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Graveyard,
            },
        ]),
    )
}

/// Iron Suitcase — {1} artifact. ETB: scry 2. {3}: becomes a 3/3 Construct
/// artifact creature with flying until end of turn.
pub fn iron_suitcase() -> CardDefinition {
    CardDefinition {
        name: "Iron Suitcase",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(3),
                creature_types: vec![CreatureType::Construct],
                keywords: vec![Keyword::Flying],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Surtland Frostpyre — land, enters tapped, {T}: {R}. {2}{U}{U}{R}, {T},
/// sacrifice, sorcery speed: scry 2, then 2 damage to each creature.
pub fn surtland_frostpyre() -> CardDefinition {
    CardDefinition {
        name: "Surtland Frostpyre",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(Color::Red, Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                mana_cost: cost(&[generic(2), u(), u(), r()]),
                effect: Effect::Seq(vec![
                    Effect::Scry {
                        who: PlayerRef::You,
                        amount: Value::Const(2),
                    },
                    Effect::DealDamage {
                        to: Selector::EachPermanent(R::Creature),
                        amount: Value::Const(2),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Cement Shoes — {1} Equipment, equip {2}. Equipped creature gets +3/+3, has
/// "At the beginning of your end step, tap this creature," and doesn't untap.
pub fn cement_shoes() -> CardDefinition {
    CardDefinition {
        name: "Cement Shoes",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            ..Default::default()
        }),
        static_abilities: vec![
            StaticAbility {
                description: "Equipped creature has \"At the beginning of your end step, tap this creature.\"",
                effect: StaticEffect::GrantTriggeredAbility {
                    filter: R::IsHostOfSource,
                    ability: Box::new(TriggeredAbility {
                        event: EventSpec::new(
                            EventKind::StepBegins(TurnStep::End),
                            EventScope::YourControl,
                        ),
                        effect: Effect::Tap {
                            what: Selector::This,
                        },
                    }),
                },
            },
            StaticAbility {
                description: "Equipped creature doesn't untap during its controller's untap step.",
                effect: StaticEffect::PreventUntap {
                    applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                },
            },
        ],
        ..Default::default()
    }
}

/// Wild Shape — {G} instant. Choose one: until end of turn your target creature
/// becomes a base 1/3 hexproof Turtle, 1/5 reach Spider, or 3/3 trample Elephant.
pub fn wild_shape() -> CardDefinition {
    let shape = |p: i32, t: i32, ty: CreatureType, kw: Keyword| {
        Effect::Seq(vec![
            Effect::SetBasePT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(p),
                toughness: Value::Const(t),
                duration: Duration::EndOfTurn,
            },
            Effect::BecomeCreatureType {
                what: Selector::Target(0),
                creature_types: vec![ty],
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: kw,
                duration: Duration::EndOfTurn,
            },
        ])
    };
    spell(
        "Wild Shape",
        cost(&[g()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            shape(1, 3, CreatureType::Turtle, Keyword::Hexproof),
            shape(1, 5, CreatureType::Spider, Keyword::Reach),
            shape(3, 3, CreatureType::Elephant, Keyword::Trample),
        ]),
    )
}

/// Word of Undoing — {U} instant. Return target creature and all white Auras
/// you own attached to it to their owners' hands (Soul Nova's attachment shape).
pub fn word_of_undoing() -> CardDefinition {
    spell(
        "Word of Undoing",
        cost(&[u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move {
                what: Selector::MatchingAmong {
                    inner: Box::new(Selector::AttachedToMe(Box::new(Selector::Target(0)))),
                    filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
                        .and(R::HasColor(Color::White))
                        .and(R::OwnedByYou),
                },
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        ]),
    )
}

/// Pyre-Sledge Arsonist — {2}{R} 2/2 Lizard Shaman. {1}, {T}: X damage to any
/// target, X = permanents you've sacrificed this turn.
pub fn pyre_sledge_arsonist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::PermanentsSacrificedThisTurn(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature(
            "Pyre-Sledge Arsonist",
            cost(&[generic(2), r()]),
            vec![CreatureType::Lizard, CreatureType::Shaman],
            2,
            2,
        )
    }
}

/// Skyrider Patrol — {2}{G}{U} 2/3 flying Elf Scout. Beginning of combat on your
/// turn: may pay {G}{U}; when you do, +1/+1 counter on another target creature
/// you control and it gains flying until end of turn.
pub fn skyrider_patrol() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::MayPay {
                description: "Pay {G}{U} to grow and lift another creature?".into(),
                mana_cost: cost(&[g(), u()]),
                body: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::Seq(vec![
                        Effect::AddCounter {
                            what: target_filtered(
                                R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                            ),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        },
                        Effect::GrantKeyword {
                            what: Selector::Target(0),
                            keyword: Keyword::Flying,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                }),
                else_: None,
            },
        }],
        ..creature(
            "Skyrider Patrol",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Elf, CreatureType::Scout],
            2,
            3,
        )
    }
}

/// Biblioplex Kraken — {4}{U} 4/5 Kraken. ETB: scry 3. Attacks: you may return
/// another creature you control to its owner's hand; if you do, it can't be
/// blocked this turn.
pub fn biblioplex_kraken() -> CardDefinition {
    let others =
        || Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource));
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(3),
            }),
            on_attack(Effect::MayDo {
                description: "Return another creature you control to make this unblockable?".into(),
                body: Box::new(Effect::If {
                    cond: Predicate::ValueAtLeast(Value::count(others()), Value::ONE),
                    then: Box::new(Effect::Seq(vec![
                        Effect::MoveChosen {
                            from: others(),
                            filter: None,
                            count: Value::ONE,
                            up_to: false,
                            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                        },
                        Effect::GrantKeyword {
                            what: Selector::This,
                            keyword: Keyword::Unblockable,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                }),
            }),
        ],
        ..creature(
            "Biblioplex Kraken",
            cost(&[generic(4), u()]),
            vec![CreatureType::Kraken],
            4,
            5,
        )
    }
}

/// Attended Healer — {3}{W} 2/3 Kor Cleric. First life gain each turn: a 1/1
/// white Cat. {2}{W}: another target Cleric gains lifelink until end of turn.
pub fn attended_healer() -> CardDefinition {
    let cat = Arc::new(TokenDefinition {
        name: "Cat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        ..Default::default()
    });
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl)
                .with_filter(Predicate::FirstLifeGainThisTurn {
                    who: PlayerRef::You,
                })
                .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: cat,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    R::HasCreatureType(CreatureType::Cleric).and(R::OtherThanSource),
                ),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Attended Healer",
            cost(&[generic(3), w()]),
            vec![CreatureType::Kor, CreatureType::Cleric],
            2,
            3,
        )
    }
}

/// Yuan-Ti Fang-Blade — {2}{B} 2/2 Snake Rogue, deathtouch. Combat damage to
/// a player: venture into the dungeon.
pub fn yuan_ti_fang_blade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Venture,
        }],
        ..creature(
            "Yuan-Ti Fang-Blade",
            cost(&[generic(2), crate::mana::b()]),
            vec![CreatureType::Snake, CreatureType::Rogue],
            2,
            2,
        )
    }
}

/// Bonded Fetch — {2}{U} 0/2 Homunculus. Defender, haste. {T}: draw a card,
/// then discard a card.
pub fn bonded_fetch() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Bonded Fetch",
            cost(&[generic(2), u()]),
            vec![CreatureType::Homunculus],
            0,
            2,
        )
    }
}

/// Runic Shot — {W} sorcery, kicker {U}. Destroy target tapped creature; if
/// kicked, scry 2.
pub fn runic_shot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[u()]))],
        ..spell(
            "Runic Shot",
            cost(&[w()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Destroy {
                    what: target_filtered(R::Creature.and(R::Tapped)),
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::Scry {
                        who: PlayerRef::You,
                        amount: Value::Const(2),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Academy Raider — {2}{R} 1/1 Human Warrior. Intimidate. Combat damage to a
/// player: you may discard a card; if you do, draw a card.
pub fn academy_raider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Intimidate],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Discard a card? If you do, draw a card.".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::ONE,
                        random: false,
                    },
                    if_discarded(Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    }),
                ])),
            },
        }],
        ..creature(
            "Academy Raider",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            1,
            1,
        )
    }
}

/// Vaultbreaker — {3}{R} 4/2 Orc Rogue. Attacks: you may discard a card; if you
/// do, draw a card. Dash {2}{R}.
pub fn vaultbreaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Discard a card? If you do, draw a card.".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
                if_discarded(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
            ])),
        })],
        alternative_cost: Some(dash(cost(&[generic(2), r()]))),
        ..creature(
            "Vaultbreaker",
            cost(&[generic(3), r()]),
            vec![CreatureType::Orc, CreatureType::Rogue],
            4,
            2,
        )
    }
}

/// Auriok Edgewright — {W}{W} 2/2 Human Soldier. Metalcraft — double strike
/// while you control three or more artifacts.
pub fn auriok_edgewright() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Metalcraft — double strike while you control three or more artifacts.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::DoubleStrike,
                condition: Predicate::MetalcraftActive {
                    who: PlayerRef::You,
                },
            },
        }],
        ..creature(
            "Auriok Edgewright",
            cost(&[w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Tomb Robber — {2}{B} 1/1 Human Pirate. Menace. {1}, Discard a card: it
/// explores.
pub fn tomb_robber() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Explore {
                who: Selector::This,
            },
            ..Default::default()
        }],
        ..creature(
            "Tomb Robber",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            1,
            1,
        )
    }
}

/// D'Avenant Trapper — {2}{W} 3/2 Human Archer. Whenever you cast a historic
/// spell, tap target creature an opponent controls.
pub fn davenant_trapper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::historic())),
            effect: Effect::Tap {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            },
        }],
        ..creature(
            "D'Avenant Trapper",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Archer],
            3,
            2,
        )
    }
}

/// Exquisite Huntmaster — {3}{B} 4/2 Elf Warrior. Dies: create a 1/1 green Elf
/// Warrior. Encore {4}{B}.
pub fn exquisite_huntmaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Elf Warrior".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            },
        }],
        activated_abilities: vec![encore(cost(&[generic(4), b()]))],
        ..creature(
            "Exquisite Huntmaster",
            cost(&[generic(3), b()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            4,
            2,
        )
    }
}

/// Vault Guardsman — {5}{W} 3/4 Artifact Creature — Human Advisor. Convoke.
/// ETB: exile target artifact or creature an opponent controls until it leaves.
pub fn vault_guardsman() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Convoke],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Artifact.or(R::Creature).and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..creature(
            "Vault Guardsman",
            cost(&[generic(5), w()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            3,
            4,
        )
    }
}

/// Trollhide — {2}{G} Aura. Enchanted creature gets +2/+2 and has
/// "{1}{G}: Regenerate this creature."
pub fn trollhide() -> CardDefinition {
    CardDefinition {
        name: "Trollhide",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(1), g()]),
                effect: Effect::Regenerate {
                    what: Selector::This,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Aberrant Researcher // Perfected Form — {3}{U} 3/2 flier. Upkeep: mill a
/// card; if it was an instant or sorcery, transform into a 5/4 flying Insect Horror.
pub fn aberrant_researcher() -> CardDefinition {
    let perfected_form = CardDefinition {
        name: "Perfected Form",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Horror],
            ..Default::default()
        },
        color_indicator: vec![Color::Blue],
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::Mill {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CardsMilledThisEffectMatching {
                            filter: R::HasCardType(CardType::Instant)
                                .or(R::HasCardType(CardType::Sorcery)),
                        },
                        Value::ONE,
                    ),
                    then: Box::new(Effect::Transform {
                        what: Selector::This,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        back_face: Some(Box::new(perfected_form)),
        ..creature(
            "Aberrant Researcher",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Insect],
            3,
            2,
        )
    }
}

/// Brambleback Brute — {2}{R} 4/5 Giant Warrior; enters with two -1/-1
/// counters. {1}{R}, remove a counter from it: target creature can't block
/// this turn. Sorcery speed.
pub fn brambleback_brute() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::MinusOneMinusOne, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            remove_counter_among_filter: Some((None, 1, R::OtherThanSource.negate())),
            sorcery_speed: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Brambleback Brute",
            cost(&[generic(2), r()]),
            vec![CreatureType::Giant, CreatureType::Warrior],
            4,
            5,
        )
    }
}
