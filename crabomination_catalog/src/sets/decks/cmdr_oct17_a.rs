//! Commander: the 2026-10-17 (`01Htv6cF`) rescan's two-card gaps — Yidaro,
//! General Traag, Zara, Yuriko Hope from the Shadows, Ertha Jo. Tests in
//! `tests/recent_b/cmdr_oct17_a.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec,
    Keyword, LandType, LoyaltyAbility, MayPlayDuration, PlaneswalkerSubtype, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{backup, etb, target_any, target_filtered, token_copy_of};
use crate::effect::{DelayedTriggerKind, Duration, Effect, LookPick, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, b, cost, g, generic, r, u, w, x};

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

// ── Batch 3: one-card partner-pair gaps ────────────────────────────────────

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// Refurbish — {3}{W} Sorcery. Return target artifact card from your
/// graveyard to the battlefield.
pub fn refurbish() -> CardDefinition {
    CardDefinition {
        name: "Refurbish",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}

/// Vile Manifestation — {1}{B} 0/4 Horror. +1/+0 for each card with cycling
/// in your graveyard. Cycling {2}.
pub fn vile_manifestation() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+0 for each card with cycling in your graveyard.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::count(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::HasCyclingAbility }),
                per_power: 1,
                per_toughness: 0,
            },
        }],
        ..creature("Vile Manifestation", cost(&[generic(1), b()]), vec![CreatureType::Horror], 0, 4)
    }
}

fn your_creatures() -> Value {
    Value::count(Selector::EachPermanent(R::Creature.and(R::ControlledByYou)))
}

/// Huatli, Radiant Champion — {2}{G}{W}, loyalty 3. +1: a loyalty counter per
/// creature you control. −1: target creature gets +X/+X (X = your
/// creatures). −8: emblem — whenever a creature you control enters, you may
/// draw a card.
pub fn huatli_radiant_champion() -> CardDefinition {
    CardDefinition {
        name: "Huatli, Radiant Champion",
        cost: cost(&[generic(2), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Huatli], ..Default::default() },
        base_loyalty: 3,
        loyalty_abilities: vec![
            loyalty(1, Effect::AddCounter { what: Selector::This, kind: CounterType::Loyalty, amount: your_creatures() }),
            loyalty(
                -1,
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: your_creatures(),
                    toughness: your_creatures(),
                    duration: Duration::EndOfTurn,
                },
            ),
            loyalty(
                -8,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Huatli, Radiant Champion".into(),
                    statics: vec![],
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
                        effect: Effect::MayDo {
                            description: "Draw a card?".into(),
                            body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                        },
                    }],
                },
            ),
        ],
        ..Default::default()
    }
}

/// Ajani, Wise Counselor — {3}{W}{W}, loyalty 5. +2: gain 1 life per creature
/// you control. −3: creatures you control get +2/+2. −9: X +1/+1 counters on
/// target creature (X = your life total).
pub fn ajani_wise_counselor() -> CardDefinition {
    CardDefinition {
        name: "Ajani, Wise Counselor",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Ajani], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            loyalty(2, Effect::GainLife { who: Selector::You, amount: your_creatures() }),
            loyalty(
                -3,
                Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
            ),
            loyalty(
                -9,
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::LifeOf(PlayerRef::You),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Last Chance — {R}{R} Sorcery. Take an extra turn after this one; at the
/// beginning of that turn's end step, you lose the game.
pub fn last_chance() -> CardDefinition {
    CardDefinition {
        name: "Last Chance",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::YourNextTurnsEndStep,
                body: Box::new(Effect::LoseGame { who: PlayerRef::You }),
            },
        ]),
        ..Default::default()
    }
}

/// Crush the Weak — {2}{R} Sorcery. 2 damage to each creature; a creature
/// dealt damage this way that would die this turn is exiled instead.
/// Foretell {R}.
pub fn crush_the_weak() -> CardDefinition {
    let each = || Selector::EachPermanent(R::Creature);
    CardDefinition {
        name: "Crush the Weak",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        foretell_cost: Some(cost(&[r()])),
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn { what: each() },
            Effect::DealDamage { to: each(), amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Venom Connoisseur — {1}{G} 2/2 Human Druid. Alliance: it gains deathtouch
/// until end of turn; the second time this resolves in a turn, all your
/// creatures do.
pub fn venom_connoisseur() -> CardDefinition {
    let deathtouch = |what: Selector| Effect::GrantKeyword { what, keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn };
    let mine = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::EscalatingThisTurn {
                modes: vec![
                    deathtouch(Selector::This),
                    Effect::Seq(vec![deathtouch(Selector::This), deathtouch(mine())]),
                    deathtouch(Selector::This),
                ],
            },
        }],
        ..creature("Venom Connoisseur", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Druid], 2, 2)
    }
}

/// Chronozoa — {3}{U} 3/3 flying Illusion. Vanishing 3. When it dies with no
/// time counters on it, create two tokens that are copies of it.
pub fn chronozoa() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vanishing(3)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource)
                .with_filter(Predicate::Not(Box::new(Predicate::TriggerSourceHadCounter(CounterType::Time)))),
            effect: token_copy_of(PlayerRef::You, Value::Const(2), Selector::TriggerSource),
        }],
        ..creature("Chronozoa", cost(&[generic(3), u()]), vec![CreatureType::Illusion], 3, 3)
    }
}

/// Gideon's Memorial — {1}{W} legendary artifact. Creature tokens you control
/// get +1/+0 and have vigilance. {T}: one mana of any color for planeswalker
/// spells only. {1}{W}, discard it: 4 damage to target attacking or blocking
/// creature.
pub fn gideons_memorial() -> CardDefinition {
    let tokens = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::IsToken));
    CardDefinition {
        name: "Gideon's Memorial",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "Creature tokens you control get +1/+0.",
                effect: StaticEffect::PumpPT { applies_to: tokens(), power: 1, toughness: 0 },
            },
            StaticAbility {
                description: "Creature tokens you control have vigilance.",
                effect: StaticEffect::GrantKeyword { applies_to: tokens(), keyword: Keyword::Vigilance },
            },
        ],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(Box::new(ManaPayload::AnyColors(Value::ONE)), SpendRestriction::PlaneswalkerSpellsOnly),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w()]),
                from_hand: true,
                discard_self_cost: true,
                effect: Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                    amount: Value::Const(4),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Fire Giant's Fury — {1}{R} Sorcery. Target Giant you control gets +2/+2
/// and trample this turn; whenever it deals combat damage to a player this
/// turn, exile that many cards from your top, playable until the end of your
/// next turn.
pub fn fire_giants_fury() -> CardDefinition {
    CardDefinition {
        name: "Fire Giant's Fury",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::HasCreatureType(CreatureType::Giant))),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                    effect: Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::You,
                        count: Value::TriggerEventAmount,
                        duration: MayPlayDuration::EndOfControllersNextTurn,
                        pay_any_color: false,
                        max_mana_value: None,
                        pay_own_cost: true,
                        uncast_penalty: None,
                    },
                }),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Dragon's Disciple — {1}{W} 1/3 Human Monk. Enters with a +1/+1 counter if
/// you reveal a Dragon card from your hand or control a Dragon. Dragons you
/// control have ward {1}.
pub fn dragons_disciple() -> CardDefinition {
    let dragon = || R::HasCreatureType(CreatureType::Dragon);
    CardDefinition {
        as_enters_effect: Some(Effect::If {
            cond: Predicate::Any(vec![
                Predicate::SelectorExists(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Hand, filter: dragon() }),
                Predicate::SelectorExists(Selector::ControlledBy { who: PlayerRef::You, filter: dragon() }),
            ]),
            then: Box::new(Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
            else_: Box::new(Effect::Noop),
        }),
        static_abilities: vec![StaticAbility {
            description: "Dragons you control have ward {1}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(dragon())),
                keyword: Keyword::Ward(WardCost::Mana(cost(&[generic(1)]))),
            },
        }],
        ..creature("Dragon's Disciple", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Monk], 1, 3)
    }
}

/// Captain's Hook — {3} Equipment, equip {1}. +2/+0, menace, and a Pirate;
/// whenever it becomes unattached from a permanent, destroy that permanent.
pub fn captains_hook() -> CardDefinition {
    equipment(
        "Captain's Hook",
        cost(&[generic(3)]),
        cost(&[generic(1)]),
        EquipBonus {
            power: 2,
            keywords: vec![Keyword::Menace],
            add_creature_types: vec![CreatureType::Pirate],
            destroy_host_when_unattached: true,
            ..Default::default()
        },
    )
}

/// Omnipresence — {5}{G}{G}{G} Enchantment. You may cast spells with mana
/// value at most the number of creatures you control from your hand without
/// paying their mana costs.
pub fn omnipresence() -> CardDefinition {
    CardDefinition {
        name: "Omnipresence",
        cost: cost(&[generic(5), g(), g(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "You may cast spells with mana value less than or equal to the number of creatures you control from your hand without paying their mana costs.",
            effect: StaticEffect::CastFilteredSpellsFree { filter: R::ManaValueAtMostYourCount(Box::new(R::Creature)) },
        }],
        ..Default::default()
    }
}

/// Puppet Crafting — {1}{G} Aura, enchant artifact or non-Aura enchantment.
/// It's a 5/5 Construct creature in addition to its other types. {4}{G}:
/// return this card from your graveyard to your hand.
pub fn puppet_crafting() -> CardDefinition {
    CardDefinition {
        name: "Puppet Crafting",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Artifact.or(R::Enchantment.and(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura).negate()))),
        },
        equipped_bonus: Some(EquipBonus {
            add_card_types: vec![CardType::Creature],
            add_creature_types: vec![CreatureType::Construct],
            set_base_pt: Some((5, 5)),
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g()]),
            from_graveyard: true,
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Novel Nunchaku — {2}{G} Equipment, equip {3}. ETB: attach to target
/// creature you control; when you do, it fights up to one target creature an
/// opponent controls. Equipped creature gets +1/+1 and has trample.
pub fn novel_nunchaku() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::AttachedTo(Box::new(Selector::This))),
                then: Box::new(Effect::ReflexiveTrigger {
                    body: Box::new(Effect::ApplyToTargets {
                        min_targets: 0,
                        max_targets: 1,
                        filter: R::Creature.and(R::ControlledByOpponent),
                        effect: Box::new(Effect::Fight {
                            attacker: Selector::AttachedTo(Box::new(Selector::This)),
                            defender: Selector::Target(0),
                        }),
                    }),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..equipment(
            "Novel Nunchaku",
            cost(&[generic(2), g()]),
            cost(&[generic(3)]),
            EquipBonus { power: 1, toughness: 1, keywords: vec![Keyword::Trample], ..Default::default() },
        )
    }
}
