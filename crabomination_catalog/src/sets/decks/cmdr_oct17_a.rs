//! Commander: the 2026-10-17 (`01Htv6cF`) rescan's two-card gaps — Yidaro,
//! General Traag, Zara, Yuriko Hope from the Shadows, Ertha Jo. Tests in
//! `tests/recent_b/cmdr_oct17_a.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec,
    Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{backup, etb, target_any, target_filtered, token_copy_of};
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, r, u, x};

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

fn chandra(name: &'static str, mana: ManaCost, loyalty: u32, abilities: Vec<LoyaltyAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Chandra], ..Default::default() },
        base_loyalty: loyalty,
        loyalty_abilities: abilities,
        ..Default::default()
    }
}

fn loyalty(loyalty_cost: i32, effect: Effect) -> LoyaltyAbility {
    LoyaltyAbility { loyalty_cost, effect, ..Default::default() }
}

fn add_red(n: i32) -> Effect {
    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Red, Value::Const(n)) }
}

/// "You may cast red spells from among them this turn" over the cards this
/// resolution exiled — a cast grant at their own cost, lands excluded.
fn may_cast_red_exiled() -> Effect {
    let red = || Selector::ExiledThisResolution { filter: R::HasColor(Color::Red) };
    Effect::Seq(vec![
        Effect::GrantMayPlay {
            what: red(),
            duration: crate::card::MayPlayDuration::EndOfThisTurn,
            to_owner: false,
            exile_after: false,
            pay_own_cost: true,
            any_color: false,
        },
        Effect::GrantedCastOnly { what: red() },
    ])
}

/// Granitic Titan — {4}{R}{R} 5/4 Elemental. Menace. Cycling {2}.
pub fn granitic_titan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Cycling(cost(&[generic(2)]))],
        ..creature("Granitic Titan", cost(&[generic(4), r(), r()]), vec![CreatureType::Elemental], 5, 4)
    }
}

/// Quakefoot Cyclops — {4}{R} 4/4. ETB: up to two target creatures can't
/// block this turn. Cycling {1}{R}; when you cycle it, target creature can't
/// block this turn.
pub fn quakefoot_cyclops() -> CardDefinition {
    let cant_block = || Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::CantBlock, duration: Duration::EndOfTurn };
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(1), r()]))],
        triggered_abilities: vec![
            etb(Effect::ApplyToTargets { min_targets: 0, max_targets: 2, filter: R::Creature, effect: Box::new(cant_block()) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..creature("Quakefoot Cyclops", cost(&[generic(4), r()]), vec![CreatureType::Cyclops], 4, 4)
    }
}

/// General Traag, Heart of Stone — {3}{R}{R} legendary 4/3 artifact creature.
/// Trample. ETB: you may sacrifice another artifact; when you do, it deals 4
/// damage to target creature.
pub fn general_traag_heart_of_stone() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice another artifact to deal 4 damage to a creature?".into(),
            filter: R::Artifact.and(R::OtherThanSource),
            count: Value::ONE,
            then: Box::new(Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) }),
            else_: None,
        })],
        ..creature(
            "General Traag, Heart of Stone",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Soldier],
            4,
            3,
        )
    }
}

/// Earth Tremor — {3}{R} Instant. Damage to target creature or planeswalker
/// equal to the number of lands you control.
pub fn earth_tremor() -> CardDefinition {
    CardDefinition {
        name: "Earth Tremor",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature.or(R::Planeswalker)),
            amount: Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(R::Any)),
                filter: R::Land.and(R::ControlledByYou),
            },
        },
        ..Default::default()
    }
}

/// Grafted Identity — {2}{U}{U} Aura. Additional cost: sacrifice a creature.
/// You control enchanted creature; it gets +1/+1.
pub fn grafted_identity() -> CardDefinition {
    CardDefinition {
        name: "Grafted Identity",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent { filter: R::Creature.and(R::ControlledByYou), count: 1 }],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains { what: Selector::AttachedTo(Box::new(Selector::This)) })],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        ..Default::default()
    }
}

/// Discontinuity — {3}{U}{U}{U} Instant. Costs {2}{U}{U} less during your
/// turn. End the turn.
pub fn discontinuity() -> CardDefinition {
    CardDefinition {
        name: "Discontinuity",
        cost: cost(&[generic(3), u(), u(), u()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![
            StaticAbility {
                description: "During your turn, this spell costs {2}{U}{U} less to cast.",
                effect: StaticEffect::SelfCostReducedDuringYourTurn { amount: 2 },
            },
            StaticAbility {
                description: "During your turn, this spell costs {2}{U}{U} less to cast.",
                effect: StaticEffect::SelfColoredCostReducedIf {
                    condition: Predicate::IsTurnOf(PlayerRef::You),
                    less: cost(&[u(), u()]),
                },
            },
        ],
        effect: Effect::EndTheTurn,
        ..Default::default()
    }
}

/// Call to Heel — {1}{U} Instant. Return target creature to its owner's
/// hand; its controller draws a card.
pub fn call_to_heel() -> CardDefinition {
    CardDefinition {
        name: "Call to Heel",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Draw { who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))), amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Candlekeep Inspiration — {4}{U} Sorcery. Until end of turn, creatures you
/// control have base P/T X/X, X = cards you own in exile and in your
/// graveyard that are instants, sorceries, and/or have an Adventure.
pub fn candlekeep_inspiration() -> CardDefinition {
    let spellish = || R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).or(R::HasAdventure);
    let x = Value::Sum(vec![
        Value::count(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Exile, filter: spellish() }),
        Value::count(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: spellish() }),
    ]);
    CardDefinition {
        name: "Candlekeep Inspiration",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SetBasePT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: x.clone(),
            toughness: x,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Chandra, Bold Pyromancer — {4}{R}{R}, loyalty 5. +1: add {R}{R}, 2 damage
/// to target player. −3: 3 damage to target creature or planeswalker. −7: 10
/// damage to target player and each creature and planeswalker they control.
pub fn chandra_bold_pyromancer() -> CardDefinition {
    chandra(
        "Chandra, Bold Pyromancer",
        cost(&[generic(4), r(), r()]),
        5,
        vec![
            loyalty(
                1,
                Effect::Seq(vec![add_red(2), Effect::DealDamage { to: target_filtered(R::Player), amount: Value::Const(2) }]),
            ),
            loyalty(-3, Effect::DealDamage { to: target_filtered(R::Creature.or(R::Planeswalker)), amount: Value::Const(3) }),
            loyalty(
                -7,
                Effect::Seq(vec![
                    Effect::DealDamage { to: target_filtered(R::Player), amount: Value::Const(10) },
                    Effect::DealDamage {
                        to: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature.or(R::Planeswalker) },
                        amount: Value::Const(10),
                    },
                ]),
            ),
        ],
    )
}

/// Chandra, Dressed to Kill — {1}{R}{R}, loyalty 3. +1: add {R}, 1 damage to
/// up to one target player or planeswalker. +1: exile your top card; if it's
/// red, you may cast it this turn. −7: exile your top five, you may cast red
/// spells from among them this turn, and an emblem: whenever you cast a red
/// spell, it deals X damage to any target (X = mana spent on that spell).
pub fn chandra_dressed_to_kill() -> CardDefinition {
    let exile_top = |n: i32| Effect::ExileTopOfLibrary {
        who: Selector::You,
        amount: Value::Const(n),
        link_to_source: false,
        face_down: false,
    };
    chandra(
        "Chandra, Dressed to Kill",
        cost(&[generic(1), r(), r()]),
        3,
        vec![
            loyalty(
                1,
                Effect::Seq(vec![
                    add_red(1),
                    Effect::ApplyToTargets {
                        min_targets: 0,
                        max_targets: 1,
                        filter: R::Player.or(R::Planeswalker),
                        effect: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::ONE }),
                    },
                ]),
            ),
            loyalty(1, Effect::Seq(vec![exile_top(1), may_cast_red_exiled()])),
            loyalty(
                -7,
                Effect::Seq(vec![
                    exile_top(5),
                    may_cast_red_exiled(),
                    Effect::CreateEmblem {
                        who: PlayerRef::You,
                        name: "Chandra, Dressed to Kill".into(),
                        statics: vec![],
                        triggered: vec![TriggeredAbility {
                            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                                what: Selector::TriggerSource,
                                filter: R::HasColor(Color::Red),
                            }),
                            effect: Effect::DealDamage { to: target_any(), amount: Value::CastSpellManaSpent },
                        }],
                    },
                ]),
            ),
        ],
    )
}

/// Shifting Loyalties — {5}{U} Sorcery. Exchange control of two target
/// permanents that share a card type.
pub fn shifting_loyalties() -> CardDefinition {
    CardDefinition {
        name: "Shifting Loyalties",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ExchangeControl {
            a: Selector::TargetFiltered { slot: 0, filter: R::Permanent },
            b: Selector::TargetFiltered {
                slot: 1,
                filter: R::Permanent.and(R::OtherThanTargetSlot(0)).and(R::SharesCardTypeWithTargetSlot(0)),
            },
        },
        ..Default::default()
    }
}

/// Hoshi Sato, Exolinguist — {4}{U} legendary 3/3. Federation — ETB: look at
/// the top X (X = creature types among non-Borg creatures you control), one
/// to hand, the rest on the bottom in a random order.
pub fn hoshi_sato_exolinguist() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(LookPick {
            rest_bottom_random: true,
            who: PlayerRef::You,
            count: Value::DistinctCreatureTypesAmongYourNonBorgCreatures,
            ..Default::default()
        })))],
        ..creature(
            "Hoshi Sato, Exolinguist",
            cost(&[generic(4), u()]),
            vec![CreatureType::Human, CreatureType::Scientist],
            3,
            3,
        )
    }
}

/// Saiba Cryptomancer — {1}{U} 0/1 Moonfolk Ninja. Flash. Backup 1
/// (hexproof).
pub fn saiba_cryptomancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Hexproof],
        triggered_abilities: vec![backup(1, vec![Keyword::Hexproof])],
        ..creature("Saiba Cryptomancer", cost(&[generic(1), u()]), vec![CreatureType::Moonfolk, CreatureType::Ninja], 0, 1)
    }
}

/// Rick Jones, Destined Sidekick — {G} legendary 0/3. {3}, {T}: mill four;
/// you may put a Hero or enchantment card from among them into your hand.
pub fn rick_jones_destined_sidekick() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::MillThenToHand {
                amount: Value::Const(4),
                filter: R::HasCreatureType(CreatureType::Hero).or(R::HasCardType(CardType::Enchantment)),
                otherwise: None,
            },
            ..Default::default()
        }],
        ..creature("Rick Jones, Destined Sidekick", cost(&[g()]), vec![CreatureType::Human, CreatureType::Advisor], 0, 3)
    }
}

/// Crop Sigil — {G} Enchantment. Upkeep: you may mill a card. Delirium —
/// {2}{G}, sacrifice it: return up to one target creature card and up to one
/// target land card from your graveyard to your hand.
pub fn crop_sigil() -> CardDefinition {
    let back = |slot: u8, kind: CardType| Effect::Move {
        what: Selector::TargetFiltered { slot, filter: R::HasCardType(kind).and(R::InYourGraveyard) },
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        name: "Crop Sigil",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Mill a card?".into(),
                body: Box::new(Effect::Mill { who: Selector::You, amount: Value::ONE }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            sac_cost: true,
            condition: Some(Predicate::DeliriumActive { who: PlayerRef::You }),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![back(0, CardType::Creature), back(1, CardType::Land)])),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mirkwood Channeler — {3}{G} 3/3 Elf Druid. At the beginning of combat on
/// your turn, target Elf you control gains trample and gets +X/+X until end
/// of turn (X = Forests you control).
pub fn mirkwood_channeler() -> CardDefinition {
    let forests = || Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(R::Any)),
        filter: R::HasLandType(LandType::Forest).and(R::ControlledByYou),
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::HasCreatureType(CreatureType::Elf))),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
                Effect::PumpPT { what: Selector::Target(0), power: forests(), toughness: forests(), duration: Duration::EndOfTurn },
            ]),
        }],
        ..creature("Mirkwood Channeler", cost(&[generic(3), g()]), vec![CreatureType::Elf, CreatureType::Druid], 3, 3)
    }
}

/// Contest of Claws — {1}{G} Sorcery. Target creature you control deals
/// damage equal to its power to another target creature; if excess damage
/// was dealt this way, discover X (X = that excess).
pub fn contest_of_claws() -> CardDefinition {
    CardDefinition {
        name: "Contest of Claws",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamageEqualToPower {
                source: target_filtered(R::Creature.and(R::ControlledByYou)),
                target: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::OtherThanTargetSlot(0)) },
            },
            Effect::If {
                cond: Predicate::ExcessDamageDealtThisResolution,
                then: Box::new(Effect::Discover { n: Value::ExcessDamageDealtThisResolution, filter: None }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Supplant Form — {4}{U}{U} Instant. Return target creature to its owner's
/// hand; you create a token that's a copy of it. The copy is minted first so
/// it reads the creature's battlefield copiable values (its last known
/// information once bounced).
pub fn supplant_form() -> CardDefinition {
    CardDefinition {
        name: "Supplant Form",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            token_copy_of(PlayerRef::You, Value::ONE, target_filtered(R::Creature)),
            Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))) },
        ]),
        ..Default::default()
    }
}

/// Linessa, Zephyr Mage — {3}{U} legendary 3/3. {X}{U}{U}, {T}: return target
/// creature with mana value X to its owner's hand. Grandeur — discard another
/// Linessa: target player returns a creature they control to its owner's
/// hand, then an artifact, an enchantment, and a land.
pub fn linessa_zephyr_mage() -> CardDefinition {
    let returns = |filter: R| Effect::OpponentChoosesPermanentThen {
        filter,
        body: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
        chooser: Some(PlayerRef::Target(0)),
        theirs: true,
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[x(), u(), u()]),
                tap_cost: true,
                effect: Effect::Move {
                    what: target_filtered(R::Creature.and(R::ManaValueExactlyXFromCost)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                ..Default::default()
            },
            ActivatedAbility {
                discard_cost: Some((R::HasName("Linessa, Zephyr Mage".to_string()), 1)),
                effect: Effect::ApplyToTargets {
                    min_targets: 1,
                    max_targets: 1,
                    filter: R::Player,
                    effect: Box::new(Effect::Seq(vec![
                        returns(R::Creature),
                        returns(R::Artifact),
                        returns(R::Enchantment),
                        returns(R::Land),
                    ])),
                },
                ..Default::default()
            },
        ],
        ..creature("Linessa, Zephyr Mage", cost(&[generic(3), u()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 3)
    }
}
