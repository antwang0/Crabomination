//! Commander: the cards the **Forged in Stone** precon (C14, Nahiri, the
//! Lithomancer) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **Arcane Lighthouse** — the creatures lose hexproof and shroud until end
//!   of turn; a grant made later that turn is not stopped ("can't have").
//! - **Benevolent Offering** — each "choose an opponent" is the engine's pick.
//! - **Nahiri, the Lithomancer** — the +2 attaches your first Equipment, the
//!   −2 puts your first Equipment card from hand, else graveyard.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, DynamicPt,
    EquipBonus, EquipScale, EntersAsCopy, Keyword, LoyaltyAbility, PlaneswalkerSubtype,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, on_dies};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, cost, generic, w};
use crate::sets::tap_add_colorless;
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn equipment(name: &'static str, mana: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn white_token(name: &str, types: Vec<CreatureType>, p: i32, t: i32, flying: bool) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords: if flying { vec![Keyword::Flying] } else { vec![] },
        ..Default::default()
    })
}

fn spirit() -> Arc<TokenDefinition> {
    white_token("Spirit", vec![CreatureType::Spirit], 1, 1, true)
}

fn kor_soldier() -> Arc<TokenDefinition> {
    white_token("Kor Soldier", vec![CreatureType::Kor, CreatureType::Soldier], 1, 1, false)
}

fn tokens(who: PlayerRef, count: Value, definition: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who, count, definition }
}

fn your_creatures() -> Value {
    Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))))
}

/// Nahiri, the Lithomancer — +2: a Kor Soldier, and you may suit it up; −2:
/// an Equipment card from hand or graveyard onto the battlefield; −10: the
/// Stoneforged Blade.
pub fn nahiri_the_lithomancer() -> CardDefinition {
    let equipment_you_control =
        || R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou);
    let equipment_card = || R::HasArtifactSubtype(ArtifactSubtype::Equipment);
    CardDefinition {
        name: "Nahiri, the Lithomancer",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Nahiri],
            ..Default::default()
        },
        base_loyalty: 3,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Seq(vec![
                    tokens(PlayerRef::You, Value::ONE, kor_soldier()),
                    Effect::If {
                        cond: Predicate::SelectorExists(Selector::EachPermanent(
                            equipment_you_control(),
                        )),
                        then: Box::new(Effect::Attach {
                            what: Selector::Take {
                                inner: Box::new(Selector::EachPermanent(equipment_you_control())),
                                count: Box::new(Value::ONE),
                            },
                            to: Selector::LastCreatedToken,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Move {
                    what: Selector::Take {
                        inner: Box::new(Selector::Both(
                            Box::new(Selector::CardsInZone {
                                who: PlayerRef::You,
                                zone: Zone::Hand,
                                filter: equipment_card(),
                            }),
                            Box::new(Selector::CardsInZone {
                                who: PlayerRef::You,
                                zone: Zone::Graveyard,
                                filter: equipment_card(),
                            }),
                        )),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -10,
                effect: tokens(
                    PlayerRef::You,
                    Value::ONE,
                    Arc::new(TokenDefinition {
                        name: "Stoneforged Blade".into(),
                        card_types: vec![CardType::Artifact],
                        subtypes: Subtypes {
                            artifact_subtypes: vec![ArtifactSubtype::Equipment],
                            ..Default::default()
                        },
                        keywords: vec![Keyword::Indestructible, Keyword::Equip(cost(&[]))],
                        equipped_bonus: Some(EquipBonus {
                            power: 5,
                            toughness: 5,
                            keywords: vec![Keyword::DoubleStrike],
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                ),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Adarkar Valkyrie — flying, vigilance; {T}: when another target creature
/// dies this turn, it returns under your control.
pub fn adarkar_valkyrie() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::WhenTargetDiesThisTurn {
                body: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
                slot: 0,
                filter: Some(R::Creature.and(R::OtherThanSource)),
            },
            ..Default::default()
        }],
        ..creature(
            "Adarkar Valkyrie",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Angel],
            4,
            5,
        )
    }
}

/// Angel of the Dire Hour — flash, flying; cast from hand, it exiles every
/// attacking creature.
pub fn angel_of_the_dire_hour() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: crate::card::EventSpec::new(
                crate::card::EventKind::EntersBattlefield,
                crate::card::EventScope::SelfSource,
            )
            .with_filter(Predicate::CastFromHand),
            effect: Effect::Exile {
                what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
            },
        }],
        ..creature(
            "Angel of the Dire Hour",
            cost(&[generic(5), w(), w()]),
            vec![CreatureType::Angel],
            5,
            4,
        )
    }
}

/// Arcane Lighthouse — {C}; {1}, {T}: opponents' creatures lose hexproof and
/// shroud until end of turn. ⚠ Later grants that turn are not stopped.
pub fn arcane_lighthouse() -> CardDefinition {
    let theirs = || Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent));
    CardDefinition {
        name: "Arcane Lighthouse",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::LoseKeyword {
                        what: theirs(),
                        keyword: Keyword::Hexproof,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::LoseKeyword {
                        what: theirs(),
                        keyword: Keyword::Shroud,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Benevolent Offering — Spirits for you and an opponent; then life for you
/// and an opponent by creature count. ⚠ The opponents are the engine's pick.
pub fn benevolent_offering() -> CardDefinition {
    let chosen = || PlayerRef::ChosenPlayerOfSource;
    CardDefinition {
        name: "Benevolent Offering",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ChooseOpponentThen {
                then: Box::new(Effect::Seq(vec![
                    tokens(PlayerRef::You, Value::Const(3), spirit()),
                    tokens(chosen(), Value::Const(3), spirit()),
                ])),
            },
            Effect::ChooseOpponentThen {
                then: Box::new(Effect::Seq(vec![
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::Times(Box::new(your_creatures()), Box::new(Value::Const(2))),
                    },
                    Effect::GainLife {
                        who: Selector::Player(chosen()),
                        amount: Value::Times(
                            Box::new(Value::CountOf(Box::new(Selector::ControlledBy {
                                who: chosen(),
                                filter: R::Creature,
                            }))),
                            Box::new(Value::Const(2)),
                        ),
                    },
                ])),
            },
        ]),
        ..Default::default()
    }
}

/// Celestial Crusader — flash, split second, flying; other white creatures
/// get +1/+1.
pub fn celestial_crusader() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::SplitSecond, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Other white creatures get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasColor(Color::White)).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..creature(
            "Celestial Crusader",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Spirit],
            2,
            2,
        )
    }
}

/// Deploy to the Front — a Soldier per creature on the battlefield.
pub fn deploy_to_the_front() -> CardDefinition {
    CardDefinition {
        name: "Deploy to the Front",
        cost: cost(&[generic(5), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: tokens(
            PlayerRef::You,
            Value::CountOf(Box::new(Selector::EachPermanent(R::Creature))),
            white_token("Soldier", vec![CreatureType::Soldier], 1, 1, false),
        ),
        ..Default::default()
    }
}

/// Geist-Honored Monk — vigilance; */* = your creatures; two Spirits on entry.
pub fn geist_honored_monk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        dynamic_pt: Some(DynamicPt::PermanentsControlledMatching {
            base_p: 0,
            base_t: 0,
            filter: Box::new(R::Creature),
        }),
        triggered_abilities: vec![etb(tokens(PlayerRef::You, Value::Const(2), spirit()))],
        ..creature(
            "Geist-Honored Monk",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Monk],
            0,
            0,
        )
    }
}

/// Hallowed Spiritkeeper — vigilance; dying, a Spirit per creature card in
/// your graveyard.
pub fn hallowed_spiritkeeper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![on_dies(tokens(
            PlayerRef::You,
            Value::CountOf(Box::new(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: R::Creature,
            })),
            spirit(),
        ))],
        ..creature(
            "Hallowed Spiritkeeper",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Avatar],
            3,
            2,
        )
    }
}

/// Masterwork of Ingenuity — may enter as a copy of any Equipment.
pub fn masterwork_of_ingenuity() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy {
            filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment),
            ..Default::default()
        }),
        ..equipment("Masterwork of Ingenuity", cost(&[generic(1)]))
    }
}

/// Moonsilver Spear — first strike; each attack makes a 4/4 Angel. Equip {4}.
pub fn moonsilver_spear() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::FirstStrike],
            triggered_abilities: vec![on_attack(tokens(
                PlayerRef::You,
                Value::ONE,
                white_token("Angel", vec![CreatureType::Angel], 4, 4, true),
            ))],
            ..Default::default()
        }),
        ..equipment("Moonsilver Spear", cost(&[generic(4)]))
    }
}

/// Nomads' Assembly — a Kor Soldier per creature you control. Rebound.
pub fn nomads_assembly() -> CardDefinition {
    CardDefinition {
        name: "Nomads' Assembly",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Rebound],
        effect: tokens(PlayerRef::You, your_creatures(), kor_soldier()),
        ..Default::default()
    }
}

/// Strata Scythe — imprint a land from your library; +1/+1 per land on the
/// battlefield sharing its name. Equip {3}.
pub fn strata_scythe() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::Land,
            to: ZoneDest::ExileWithSourceStamp,
        })],
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                filter: R::Land,
                per_power: 1,
                per_toughness: 1,
                count_named_like_exiled_with_source: true,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..equipment("Strata Scythe", cost(&[generic(3)]))
    }
}

/// True Conviction — your creatures have double strike and lifelink.
pub fn true_conviction() -> CardDefinition {
    let grant = |keyword| StaticAbility {
        description: "Creatures you control have double strike and lifelink.",
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            keyword,
        },
    };
    CardDefinition {
        name: "True Conviction",
        cost: cost(&[generic(3), w(), w(), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![grant(Keyword::DoubleStrike), grant(Keyword::Lifelink)],
        ..Default::default()
    }
}

/// Twilight Shepherd — flying, vigilance, persist; entering, it returns what
/// went from the battlefield to your graveyard this turn.
pub fn twilight_shepherd() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Persist],
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: R::PutIntoGraveyardFromBattlefieldThisTurn,
            },
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature(
            "Twilight Shepherd",
            cost(&[generic(3), w(), w(), w()]),
            vec![CreatureType::Angel],
            5,
            5,
        )
    }
}
