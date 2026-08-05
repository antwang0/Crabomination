//! Mirrodin Besieged (MBS) — the Phyrexian half of the block: Infect,
//! Metalcraft, Battle cry, Living weapon and proliferate. Tests in
//! `recent_b/mbs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EnchantmentSubtype, EquipBonus, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, LibraryPosition, PlayerRef, Selector,
    StaticEffect, Value, ZoneDest,
    shortcut::{battle_cry, draw, etb, target_any, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    c: ManaCost,
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

fn artifact_creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature(name, c, types, p, t)
    }
}

fn artifact(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn equipment(name: &'static str, c: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..artifact(name, c)
    }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

/// Metalcraft (CR 702.0 ability word) — "as long as you control three or more
/// artifacts".
fn metalcraft() -> Predicate {
    Predicate::MetalcraftActive { who: PlayerRef::You }
}

fn myr_token() -> TokenDefinition {
    TokenDefinition {
        name: "Myr".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Myr], ..Default::default() },
        ..Default::default()
    }
}

/// Living weapon (CR 702.92) — mint a 0/0 black Phyrexian Germ and attach.
fn living_weapon() -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Phyrexian Germ".into(),
                card_types: vec![CardType::Creature],
                colors: vec![Color::Black],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Phyrexian],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
    ]))
}

/// "{cost}: This creature gains `keyword` until end of turn."
fn self_grant(cost_: ManaCost, keyword: Keyword) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost_,
        effect: Effect::GrantKeyword {
            what: Selector::This,
            keyword,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

// ── White ───────────────────────────────────────────────────────────────────

/// Banishment Decree — a five-mana instant-speed "put it on top".
pub fn banishment_decree() -> CardDefinition {
    instant(
        "Banishment Decree",
        cost(&[generic(3), w(), w()]),
        Effect::Move {
            what: target_filtered(R::Artifact.or(R::Creature).or(R::Enchantment)),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::Top,
            },
        },
    )
}

/// Choking Fumes — a -1/-1 counter on every attacker.
pub fn choking_fumes() -> CardDefinition {
    instant(
        "Choking Fumes",
        cost(&[generic(2), w()]),
        Effect::AddCounter {
            what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
            kind: CounterType::MinusOneMinusOne,
            amount: Value::ONE,
        },
    )
}

/// Frantic Salvage — rebuy your artifacts off the top, then cantrip.
pub fn frantic_salvage() -> CardDefinition {
    instant(
        "Frantic Salvage",
        cost(&[generic(3), w()]),
        Effect::Seq(vec![
            Effect::Move {
                what: Selector::Take {
                    inner: Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: R::Artifact,
                    }),
                    count: Box::new(Value::Const(99)),
                },
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            },
            draw(1),
        ]),
    )
}

/// Gore Vassal — a sacrificial shrink that can also save the target.
pub fn gore_vassal() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::MinusOneMinusOne,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: R::ToughnessAtLeast(1),
                    },
                    then: Box::new(Effect::Regenerate { what: Selector::Target(0) }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Gore Vassal",
            cost(&[generic(2), w()]),
            vec![CreatureType::Phyrexian, CreatureType::Dog],
            2,
            1,
        )
    }
}

/// Loxodon Partisan — a common battle-cry body.
pub fn loxodon_partisan() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![battle_cry(1)],
        ..creature(
            "Loxodon Partisan",
            cost(&[generic(4), w()]),
            vec![CreatureType::Elephant, CreatureType::Soldier],
            3,
            4,
        )
    }
}

/// Master's Call — two Myr at instant speed.
pub fn masters_call() -> CardDefinition {
    instant(
        "Master's Call",
        cost(&[generic(2), w()]),
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: myr_token(),
        },
    )
}

/// Phyrexian Rebirth — a wrath that leaves you the biggest body on the table.
pub fn phyrexian_rebirth() -> CardDefinition {
    sorcery(
        "Phyrexian Rebirth",
        cost(&[generic(4), w(), w()]),
        Effect::Seq(vec![
            Effect::Destroy { what: Selector::EachPermanent(R::Creature) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Phyrexian Horror".into(),
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
                        ..Default::default()
                    },
                    dynamic_pt: Some((
                        Value::PermanentsDestroyedThisResolution,
                        Value::PermanentsDestroyedThisResolution,
                    )),
                    ..Default::default()
                },
            },
        ]),
    )
}

/// Priests of Norn — a vigilant infect wall.
pub fn priests_of_norn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Infect],
        ..creature(
            "Priests of Norn",
            cost(&[generic(2), w()]),
            vec![CreatureType::Phyrexian, CreatureType::Cleric],
            1,
            4,
        )
    }
}

/// Victory's Herald — the alpha-strike Angel.
pub fn victorys_herald() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GrantKeywords {
                what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                keywords: vec![Keyword::Flying, Keyword::Lifelink],
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Victory's Herald",
            cost(&[generic(3), w(), w(), w()]),
            vec![CreatureType::Angel],
            4,
            4,
        )
    }
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Cryptoplasm — the upkeep shapeshifter that keeps its own ability.
pub fn cryptoplasm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::SelfSource,
            ),
            effect: Effect::MayDo {
                description: "Become a copy of another creature?".into(),
                body: Box::new(Effect::BecomeCopyOf {
                    what: Selector::This,
                    source: target_filtered(R::Creature.and(R::OtherThanSource)),
                    extra_creature_types: vec![],
                    keep_own_triggered: true,
                }),
            },
        }],
        ..creature(
            "Cryptoplasm",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Shapeshifter],
            2,
            2,
        )
    }
}

/// Mirran Spy — every artifact you cast untaps something.
pub fn mirran_spy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(R::Artifact),
            ),
            effect: Effect::MayDo {
                description: "Untap target creature?".into(),
                body: Box::new(Effect::Untap {
                    what: target_filtered(R::Creature),
                    up_to: None,
                }),
            },
        }],
        ..creature("Mirran Spy", cost(&[generic(2), u()]), vec![CreatureType::Drone], 1, 3)
    }
}

/// Neurok Commando — an untouchable card-advantage engine.
pub fn neurok_commando() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shroud],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo { description: "Draw a card?".into(), body: Box::new(draw(1)) },
        }],
        ..creature(
            "Neurok Commando",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            2,
            1,
        )
    }
}

/// Oculus — a one-mana body that replaces itself.
pub fn oculus() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::MayDo { description: "Draw a card?".into(), body: Box::new(draw(1)) },
        }],
        ..creature(
            "Oculus",
            cost(&[generic(1), u()]),
            vec![CreatureType::Phyrexian, CreatureType::Homunculus],
            1,
            1,
        )
    }
}

/// Quicksilver Geyser — a two-for-one bounce.
pub fn quicksilver_geyser() -> CardDefinition {
    instant(
        "Quicksilver Geyser",
        cost(&[generic(4), u()]),
        Effect::ApplyToTargets {
            filter: R::Permanent.and(R::Nonland),
            max_targets: 2,
            min_targets: 0,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        },
    )
}

/// Serum Raker — a flier whose death strips both hands.
pub fn serum_raker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..creature(
            "Serum Raker",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Phyrexian, CreatureType::Drake],
            3,
            2,
        )
    }
}

/// Shimmer Myr — artifacts at instant speed.
pub fn shimmer_myr() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "You may cast artifact spells as though they had flash.",
            effect: StaticEffect::ControllerSpellsHaveFlash { filter: R::Artifact },
        }],
        ..artifact_creature("Shimmer Myr", cost(&[generic(3)]), vec![CreatureType::Myr], 2, 2)
    }
}

/// Spire Serpent — a wall that gets up once the artifacts are online.
pub fn spire_serpent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        static_abilities: vec![
            StaticAbility {
                description: "Metalcraft — +2/+2 while you control three or more artifacts.",
                effect: StaticEffect::PumpSelfIf {
                    condition: metalcraft(),
                    power: 2,
                    toughness: 2,
                    keywords: vec![],
                },
            },
            StaticAbility {
                description: "Metalcraft — it can attack as though it didn't have defender.",
                effect: StaticEffect::CanAttackIgnoringDefenderWhile { condition: metalcraft() },
            },
        ],
        ..creature("Spire Serpent", cost(&[generic(4), u()]), vec![CreatureType::Serpent], 3, 5)
    }
}

/// Steel Sabotage — the modal artifact answer.
pub fn steel_sabotage() -> CardDefinition {
    instant(
        "Steel Sabotage",
        cost(&[u()]),
        Effect::ChooseMode(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack.and(R::Artifact)) },
            Effect::Move {
                what: target_filtered(R::Artifact),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        ]),
    )
}

/// Treasure Mage — the big-artifact tutor.
pub fn treasure_mage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for a big artifact?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::Artifact.and(R::ManaValueAtLeast(6)),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..creature(
            "Treasure Mage",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Turn the Tide — a one-sided combat swing.
pub fn turn_the_tide() -> CardDefinition {
    instant(
        "Turn the Tide",
        cost(&[generic(1), u()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
            power: Value::Const(-2),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Vedalken Anatomist — a repeatable shrink with a tap/untap rider.
pub fn vedalken_anatomist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::MinusOneMinusOne,
                    amount: Value::ONE,
                },
                Effect::TapOrUntap { what: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Vedalken Anatomist",
            cost(&[generic(2), u()]),
            vec![CreatureType::Phyrexian, CreatureType::Vedalken, CreatureType::Wizard],
            1,
            2,
        )
    }
}

/// Vedalken Infuser — a charge counter every upkeep.
pub fn vedalken_infuser() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::SelfSource,
            ),
            effect: Effect::MayDo {
                description: "Put a charge counter on target artifact?".into(),
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Artifact),
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                }),
            },
        }],
        ..creature(
            "Vedalken Infuser",
            cost(&[generic(3), u()]),
            vec![CreatureType::Vedalken, CreatureType::Wizard],
            1,
            4,
        )
    }
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Caustic Hound — a six-drop that drains four on the way out.
pub fn caustic_hound() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(4),
            },
        }],
        ..creature(
            "Caustic Hound",
            cost(&[generic(5), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Dog],
            4,
            4,
        )
    }
}

/// Flensermite — infect that pays you back.
pub fn flensermite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect, Keyword::Lifelink],
        ..creature(
            "Flensermite",
            cost(&[generic(1), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Gremlin],
            1,
            1,
        )
    }
}

/// Flesh-Eater Imp — a poison finisher fed by your board.
pub fn flesh_eater_imp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Infect],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Flesh-Eater Imp",
            cost(&[generic(3), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Imp],
            2,
            2,
        )
    }
}

/// Gruesome Encore — borrow their creature for exactly one swing.
pub fn gruesome_encore() -> CardDefinition {
    sorcery(
        "Gruesome Encore",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InOpponentGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::GrantKeyword {
                what: Selector::LastMoved,
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::ExileIfWouldDieThisTurn { what: Selector::LastMoved },
            Effect::DelayUntilWithCapture {
                kind: crate::effect::DelayedTriggerKind::NextEndStep,
                capture: Selector::LastMoved,
                body: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
            },
        ]),
    )
}

/// Horrifying Revelation — a one-mana two-for-one on their resources.
pub fn horrifying_revelation() -> CardDefinition {
    sorcery(
        "Horrifying Revelation",
        cost(&[b()]),
        Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: false,
            },
            Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
        ]),
    )
}

/// Morbid Plunder — two bodies back from the yard.
pub fn morbid_plunder() -> CardDefinition {
    sorcery(
        "Morbid Plunder",
        cost(&[generic(1), b(), b()]),
        Effect::ApplyToTargets {
            filter: R::Creature.and(R::InYourGraveyard),
            max_targets: 2,
            min_targets: 0,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
    )
}

/// Nested Ghoul — every ping mints a 2/2.
pub fn nested_ghoul() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Phyrexian Zombie".into(),
                    power: 2,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Phyrexian, CreatureType::Zombie],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..creature(
            "Nested Ghoul",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Warrior],
            4,
            2,
        )
    }
}

/// Phyresis — one black mana turns any beater into a clock.
pub fn phyresis() -> CardDefinition {
    CardDefinition {
        name: "Phyresis",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Infect],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Phyrexian Vatmother — a huge infect body that poisons you too.
pub fn phyrexian_vatmother() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::SelfSource,
            ),
            effect: Effect::AddPoison { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Phyrexian Vatmother",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            4,
            5,
        )
    }
}

/// Sangromancer — a flier that eats both their creatures and their hand.
pub fn sangromancer() -> CardDefinition {
    let gain = || Effect::MayDo {
        description: "Gain 3 life?".into(),
        body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: gain(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::OpponentControl),
                effect: gain(),
            },
        ],
        ..creature(
            "Sangromancer",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Vampire, CreatureType::Shaman],
            3,
            3,
        )
    }
}

/// Scourge Servant — a big vanilla infect body.
pub fn scourge_servant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        ..creature(
            "Scourge Servant",
            cost(&[generic(4), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie],
            3,
            3,
        )
    }
}

/// Septic Rats — infect that grows once the poison is flowing.
pub fn septic_rats() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::ValueAtLeast(
                    Value::PoisonCountersOf(PlayerRef::DefendingPlayer),
                    Value::ONE,
                ),
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Septic Rats",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Rat],
            2,
            2,
        )
    }
}

/// Spread the Sickness — removal with a proliferate rider.
pub fn spread_the_sickness() -> CardDefinition {
    sorcery(
        "Spread the Sickness",
        cost(&[generic(4), b()]),
        Effect::Seq(vec![Effect::Destroy { what: target_filtered(R::Creature) }, Effect::Proliferate]),
    )
}

/// Virulent Wound — a one-mana shrink that pays a poison counter on the kill.
pub fn virulent_wound() -> CardDefinition {
    instant(
        "Virulent Wound",
        cost(&[b()]),
        Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            },
            Effect::WhenTargetDiesThisTurn {
                body: Box::new(Effect::AddPoison {
                    who: Selector::Target(0),
                    amount: Value::ONE,
                }),
                slot: 0,
                filter: None,
            },
        ]),
    )
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Blisterstick Shaman — a Shock stapled to a 2/1.
pub fn blisterstick_shaman() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DealDamage { to: target_any(), amount: Value::ONE })],
        ..creature(
            "Blisterstick Shaman",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goblin, CreatureType::Shaman],
            2,
            1,
        )
    }
}

/// Burn the Impure — removal that punishes the infect deck twice.
pub fn burn_the_impure() -> CardDefinition {
    instant(
        "Burn the Impure",
        cost(&[generic(1), r()]),
        Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(3),
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasKeyword(Keyword::Infect),
                },
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(3),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Concussive Bolt — four to the face, and metalcraft turns off their blocks.
pub fn concussive_bolt() -> CardDefinition {
    sorcery(
        "Concussive Bolt",
        cost(&[generic(3), r(), r()]),
        Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::Const(4),
            },
            Effect::If {
                cond: metalcraft(),
                then: Box::new(Effect::MatchingCantBlockThisTurn {
                    filter: R::Creature.and(R::ControlledByOpponent),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Gnathosaur — trample for an artifact.
pub fn gnathosaur() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Gnathosaur", cost(&[generic(4), r(), r()]), vec![CreatureType::Dinosaur], 5, 4)
    }
}

/// Hellkite Igniter — a hasty Dragon that scales off your artifacts.
pub fn hellkite_igniter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou))),
                    filter: R::Any,
                },
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Hellkite Igniter",
            cost(&[generic(5), r(), r()]),
            vec![CreatureType::Dragon],
            5,
            5,
        )
    }
}

/// Koth's Courier — a red body that walks past the green deck.
pub fn koths_courier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        ..creature(
            "Koth's Courier",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            2,
            3,
        )
    }
}

/// Kuldotha Flamefiend — trade an artifact for four divided damage.
pub fn kuldotha_flamefiend() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Sacrifice an artifact for four damage?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Artifact,
                },
                Effect::DealDamageDivided {
                    filter: R::Creature.or(R::Player).or(R::Planeswalker),
                    total: Value::Const(4),
                    max_targets: 4,
                    retaliate_to_source: false,
                },
            ])),
        })],
        ..creature(
            "Kuldotha Flamefiend",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Elemental],
            4,
            4,
        )
    }
}

/// Kuldotha Ringleader — a battle-cry Giant that has to swing.
pub fn kuldotha_ringleader() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustAttack],
        triggered_abilities: vec![battle_cry(1)],
        ..creature(
            "Kuldotha Ringleader",
            cost(&[generic(4), r()]),
            vec![CreatureType::Giant, CreatureType::Berserker],
            4,
            4,
        )
    }
}

/// Metallic Mastery — a Threaten for artifacts.
pub fn metallic_mastery() -> CardDefinition {
    sorcery(
        "Metallic Mastery",
        cost(&[generic(2), r()]),
        Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Artifact),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Ogre Resister — a vanilla 4/3.
pub fn ogre_resister() -> CardDefinition {
    creature("Ogre Resister", cost(&[generic(2), r(), r()]), vec![CreatureType::Ogre], 4, 3)
}

/// Rally the Forces — a combat trick for the whole team.
pub fn rally_the_forces() -> CardDefinition {
    instant(
        "Rally the Forces",
        cost(&[generic(2), r()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeywordToMatchingThisTurn {
                filter: R::Creature.and(R::IsAttacking),
                keyword: Keyword::FirstStrike,
            },
        ]),
    )
}

/// Spiraling Duelist — metalcraft double strike.
pub fn spiraling_duelist() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Metalcraft — double strike while you control three or more artifacts.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::DoubleStrike,
                condition: metalcraft(),
            },
        }],
        ..creature(
            "Spiraling Duelist",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Human, CreatureType::Berserker],
            3,
            1,
        )
    }
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Blightwidow — the infect spider.
pub fn blightwidow() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Infect],
        ..creature(
            "Blightwidow",
            cost(&[generic(3), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Spider],
            2,
            4,
        )
    }
}

/// Fangren Marauder — every artifact death is five life.
pub fn fangren_marauder() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact },
            ),
            effect: Effect::MayDo {
                description: "Gain 5 life?".into(),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(5) }),
            },
        }],
        ..creature("Fangren Marauder", cost(&[generic(5), g()]), vec![CreatureType::Beast], 5, 5)
    }
}

/// Glissa's Courier — a green body that walks past the red deck.
pub fn glissas_courier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Mountain)],
        ..creature(
            "Glissa's Courier",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            2,
            3,
        )
    }
}

/// Melira's Keepers — a 4/4 no counter can touch.
pub fn meliras_keepers() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature can't have counters put on it.",
            effect: StaticEffect::CountersCantBePlaced,
        }],
        ..creature(
            "Melira's Keepers",
            cost(&[generic(4), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Mirran Mettle — a one-mana trick that doubles under metalcraft.
pub fn mirran_mettle() -> CardDefinition {
    instant(
        "Mirran Mettle",
        cost(&[g()]),
        Effect::If {
            cond: metalcraft(),
            then: Box::new(Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(4),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            }),
        },
    )
}

/// Pistus Strike — flier removal that also poisons.
pub fn pistus_strike() -> CardDefinition {
    instant(
        "Pistus Strike",
        cost(&[generic(2), g()]),
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))) },
            Effect::AddPoison {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::ONE,
            },
        ]),
    )
}

/// Plaguemaw Beast — a proliferate engine you feed creatures.
pub fn plaguemaw_beast() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Proliferate,
            ..Default::default()
        }],
        ..creature(
            "Plaguemaw Beast",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Beast],
            4,
            3,
        )
    }
}

/// Praetor's Counsel — the whole graveyard back, and no hand size again.
pub fn praetors_counsel() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..sorcery(
            "Praetor's Counsel",
            cost(&[generic(5), g(), g(), g()]),
            Effect::Seq(vec![
                Effect::Move {
                    what: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: R::Any,
                    },
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::SetNoMaxHandSize { who: Selector::You },
            ]),
        )
    }
}

/// Quilled Slagwurm — a vanilla 8/8.
pub fn quilled_slagwurm() -> CardDefinition {
    creature(
        "Quilled Slagwurm",
        cost(&[generic(4), g(), g(), g()]),
        vec![CreatureType::Phyrexian, CreatureType::Wurm],
        8,
        8,
    )
}

/// Rot Wolf — infect that draws off every kill it set up.
pub fn rot_wolf() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::DamagedBySourceThisTurn,
                },
            ),
            effect: Effect::MayDo { description: "Draw a card?".into(), body: Box::new(draw(1)) },
        }],
        ..creature(
            "Rot Wolf",
            cost(&[generic(2), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Wolf],
            2,
            2,
        )
    }
}

/// Tangle Mantis — a vanilla trampler.
pub fn tangle_mantis() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature("Tangle Mantis", cost(&[generic(2), g(), g()]), vec![CreatureType::Insect], 3, 4)
    }
}

/// Unnatural Predation — a one-mana trample trick.
pub fn unnatural_predation() -> CardDefinition {
    instant(
        "Unnatural Predation",
        cost(&[g()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Viridian Corrupter — infect plus artifact removal.
pub fn viridian_corrupter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        triggered_abilities: vec![etb(Effect::Destroy { what: target_filtered(R::Artifact) })],
        ..creature(
            "Viridian Corrupter",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Elf, CreatureType::Shaman],
            2,
            2,
        )
    }
}

// ── Gold ────────────────────────────────────────────────────────────────────

/// Glissa, the Traitor — first strike, deathtouch, and artifact recursion.
pub fn glissa_the_traitor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::FirstStrike, Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
            effect: Effect::MayDo {
                description: "Return an artifact card from your graveyard?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..creature(
            "Glissa, the Traitor",
            cost(&[b(), g(), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Elf],
            3,
            3,
        )
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Bladed Sentinel — a Construct that can hold the fort and swing.
pub fn bladed_sentinel() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_grant(cost(&[w()]), Keyword::Vigilance)],
        ..artifact_creature(
            "Bladed Sentinel",
            cost(&[generic(4)]),
            vec![CreatureType::Construct],
            2,
            4,
        )
    }
}

/// Copper Carapace — a one-mana +2/+2 with a real drawback.
pub fn copper_carapace() -> CardDefinition {
    equipment(
        "Copper Carapace",
        cost(&[generic(1)]),
        cost(&[generic(3)]),
        EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::CantBlock],
            ..Default::default()
        },
    )
}

/// Core Prowler — an infect body that proliferates on the way out.
pub fn core_prowler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Proliferate,
        }],
        ..artifact_creature(
            "Core Prowler",
            cost(&[generic(4)]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            2,
            2,
        )
    }
}

/// Decimator Web — a four-mana drain, poison and mill.
pub fn decimator_web() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4)]),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(2),
                },
                Effect::AddPoison { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
                Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(6) },
            ]),
            ..Default::default()
        }],
        ..artifact("Decimator Web", cost(&[generic(4)]))
    }
}

/// Dross Ripper — a pumpable artifact beater.
pub fn dross_ripper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Dross Ripper",
            cost(&[generic(4)]),
            vec![CreatureType::Phyrexian, CreatureType::Dog],
            3,
            3,
        )
    }
}

/// Gust-Skimmer — a two-drop that flies for a blue mana.
pub fn gust_skimmer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_grant(cost(&[u()]), Keyword::Flying)],
        ..artifact_creature("Gust-Skimmer", cost(&[generic(2)]), vec![CreatureType::Insect], 2, 1)
    }
}

/// Hexplate Golem — a vanilla seven-drop.
pub fn hexplate_golem() -> CardDefinition {
    artifact_creature("Hexplate Golem", cost(&[generic(7)]), vec![CreatureType::Golem], 5, 7)
}

/// Lumengrid Gargoyle — a six-mana 4/4 flier.
pub fn lumengrid_gargoyle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..artifact_creature(
            "Lumengrid Gargoyle",
            cost(&[generic(6)]),
            vec![CreatureType::Gargoyle],
            4,
            4,
        )
    }
}

/// Magnetic Mine — every artifact death pings its controller.
pub fn magnetic_mine() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact.and(R::OtherThanSource),
                },
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::Const(2),
            },
        }],
        ..artifact("Magnetic Mine", cost(&[generic(4)]))
    }
}

/// Mortarpod — a living weapon that turns any creature into a Shock.
pub fn mortarpod() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![living_weapon()],
        ..equipment(
            "Mortarpod",
            cost(&[generic(2)]),
            cost(&[generic(2)]),
            EquipBonus {
                toughness: 1,
                activated_abilities: vec![ActivatedAbility {
                    sac_cost: true,
                    effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
    }
}

/// Myr Sire — a chump blocker that leaves a chump blocker.
pub fn myr_sire() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Phyrexian Myr".into(),
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Phyrexian, CreatureType::Myr],
                        ..Default::default()
                    },
                    ..myr_token()
                },
            },
        }],
        ..artifact_creature(
            "Myr Sire",
            cost(&[generic(2)]),
            vec![CreatureType::Phyrexian, CreatureType::Myr],
            1,
            1,
        )
    }
}

/// Myr Turbine — a Myr factory that tutors once it's got a crew.
pub fn myr_turbine() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: myr_token(),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                tap_n_filter: Some((R::HasCreatureType(CreatureType::Myr), 5)),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Myr)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
        ],
        ..artifact("Myr Turbine", cost(&[generic(5)]))
    }
}

/// Peace Strider — a three-mana body that gains you three.
pub fn peace_strider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(3),
        })],
        ..artifact_creature(
            "Peace Strider",
            cost(&[generic(4)]),
            vec![CreatureType::Construct],
            3,
            3,
        )
    }
}

/// Phyrexian Juggernaut — a six-mana infect Juggernaut.
pub fn phyrexian_juggernaut() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect, Keyword::MustAttack],
        ..artifact_creature(
            "Phyrexian Juggernaut",
            cost(&[generic(6)]),
            vec![CreatureType::Phyrexian, CreatureType::Juggernaut],
            5,
            5,
        )
    }
}

/// Pierce Strider — a three-mana body that drains three.
pub fn pierce_strider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LoseLife {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(3),
        })],
        ..artifact_creature(
            "Pierce Strider",
            cost(&[generic(4)]),
            vec![CreatureType::Phyrexian, CreatureType::Construct],
            3,
            3,
        )
    }
}

/// Piston Sledge — a free-to-equip +3/+1 that eats your artifacts.
pub fn piston_sledge() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        equip_sacrifice_filter: Some(R::Artifact),
        ..equipment(
            "Piston Sledge",
            cost(&[generic(3)]),
            ManaCost::default(),
            EquipBonus { power: 3, toughness: 1, ..Default::default() },
        )
    }
}

/// Plague Myr — a mana Myr with infect.
pub fn plague_myr() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::Colorless(Value::ONE),
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Plague Myr",
            cost(&[generic(2)]),
            vec![CreatureType::Phyrexian, CreatureType::Myr],
            1,
            1,
        )
    }
}

/// Psychosis Crawler — your hand is its body, and every draw drains.
pub fn psychosis_crawler() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::ControllerHandSize),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..artifact_creature(
            "Psychosis Crawler",
            cost(&[generic(5)]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            0,
            0,
        )
    }
}

/// Razorfield Rhino — metalcraft turns a 4/4 into a 6/6.
pub fn razorfield_rhino() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Metalcraft — +2/+2 while you control three or more artifacts.",
            effect: StaticEffect::PumpSelfIf {
                    condition: metalcraft(),
                    power: 2,
                    toughness: 2,
                    keywords: vec![],
                },
        }],
        ..artifact_creature(
            "Razorfield Rhino",
            cost(&[generic(6)]),
            vec![CreatureType::Rhino],
            4,
            4,
        )
    }
}

/// Rusted Slasher — regeneration paid in artifacts.
pub fn rusted_slasher() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..artifact_creature(
            "Rusted Slasher",
            cost(&[generic(4)]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            4,
            1,
        )
    }
}

/// Skinwing — a living weapon that flies.
pub fn skinwing() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![living_weapon()],
        ..equipment(
            "Skinwing",
            cost(&[generic(4)]),
            cost(&[generic(6)]),
            EquipBonus {
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Flying],
                ..Default::default()
            },
        )
    }
}

/// Spin Engine — a red mana clears a blocker out of its way.
pub fn spin_engine() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::CantBlockSourceThisTurn { target: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..artifact_creature("Spin Engine", cost(&[generic(3)]), vec![CreatureType::Construct], 3, 1)
    }
}

/// Strandwalker — a living weapon that blocks the sky.
pub fn strandwalker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![living_weapon()],
        ..equipment(
            "Strandwalker",
            cost(&[generic(5)]),
            cost(&[generic(4)]),
            EquipBonus {
                power: 2,
                toughness: 4,
                keywords: vec![Keyword::Reach],
                ..Default::default()
            },
        )
    }
}

/// Tangle Hulk — a regenerating 5/3.
pub fn tangle_hulk() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..artifact_creature(
            "Tangle Hulk",
            cost(&[generic(5)]),
            vec![CreatureType::Phyrexian, CreatureType::Beast],
            5,
            3,
        )
    }
}

/// Titan Forge — bank three charges, cash them for a 9/9.
pub fn titan_forge() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Charge, 3)),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Golem".into(),
                        power: 9,
                        toughness: 9,
                        card_types: vec![CardType::Artifact, CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Golem],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                ..Default::default()
            },
        ],
        ..artifact("Titan Forge", cost(&[generic(3)]))
    }
}

/// Viridian Claw — a cheap first-strike Equipment.
pub fn viridian_claw() -> CardDefinition {
    equipment(
        "Viridian Claw",
        cost(&[generic(2)]),
        cost(&[generic(1)]),
        EquipBonus {
            power: 1,
            keywords: vec![Keyword::FirstStrike],
            ..Default::default()
        },
    )
}

// ── Land ────────────────────────────────────────────────────────────────────

/// Contested War Zone — a land that changes hands with every hit.
pub fn contested_war_zone() -> CardDefinition {
    CardDefinition {
        name: "Contested War Zone",
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer)
                .with_filter(Predicate::PlayerDamagedThisTurn { who: PlayerRef::You }),
            effect: Effect::GainControl {
                what: Selector::This,
                to: Some(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                duration: Duration::Permanent,
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Into the Core — a hard answer to two artifacts at once.
pub fn into_the_core() -> CardDefinition {
    instant(
        "Into the Core",
        cost(&[generic(2), r(), r()]),
        Effect::ApplyToTargets {
            filter: R::Artifact,
            max_targets: 2,
            min_targets: 2,
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
        },
    )
}

/// Training Drone — a 4/4 that only fights while it's carrying something.
pub fn training_drone() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "This creature can't attack or block unless it's equipped.",
                effect: StaticEffect::SelfHasKeywordWhilePredicate {
                    keyword: Keyword::CantAttack,
                    condition: Predicate::Not(Box::new(Predicate::SourceIsEquipped)),
                },
            },
            StaticAbility {
                description: "This creature can't attack or block unless it's equipped.",
                effect: StaticEffect::SelfHasKeywordWhilePredicate {
                    keyword: Keyword::CantBlock,
                    condition: Predicate::Not(Box::new(Predicate::SourceIsEquipped)),
                },
            },
        ],
        ..artifact_creature(
            "Training Drone",
            cost(&[generic(3)]),
            vec![CreatureType::Drone],
            4,
            4,
        )
    }
}

/// Phyrexian Hydra — damage never sticks; it just shrinks instead.
pub fn phyrexian_hydra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to this creature, prevent it and put that \
                          many -1/-1 counters on it instead.",
            effect: StaticEffect::ReplaceDamageToSelfWithCounters {
                kind: CounterType::MinusOneMinusOne,
            },
        }],
        ..creature(
            "Phyrexian Hydra",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Hydra],
            7,
            7,
        )
    }
}

/// Mirrorworks — {2} copies every artifact you play.
pub fn mirrorworks() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact.and(R::NotToken),
                }),
            effect: Effect::MayPay {
                description: "Pay {2} to copy that artifact?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::TriggerSource,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    extra_keywords: vec![],
                    legendary: false,
                    non_legendary: false,
                }),
                else_: None,
            },
        }],
        ..artifact("Mirrorworks", cost(&[generic(5)]))
    }
}

/// Knowledge Pool — imprint three cards per player, then every hand-cast spell
/// is swapped for something already in the pool.
pub fn knowledge_pool() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::ExileTopOfLibrary {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(3),
                link_to_source: true,
                face_down: false,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                    .with_filter(Predicate::CastFromHand),
                effect: Effect::KnowledgePool,
            },
        ],
        ..artifact("Knowledge Pool", cost(&[generic(6)]))
    }
}

/// Mitotic Manipulation — dig seven for a second copy of something you already
/// have on the battlefield.
pub fn mitotic_manipulation() -> CardDefinition {
    sorcery(
        "Mitotic Manipulation",
        cost(&[generic(1), u(), u()]),
        Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(7),
            pick_filter: Some(R::SameNameAsAPermanent),
            to_battlefield: true,
            optional: true,
            rest_to_graveyard: false,
            take: None,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            picked_lands_to_battlefield: false,
            rest_bottom_random: false,
            rest_to_exile: false,
        },
    )
}

/// Myr Welder — imprints artifact cards out of graveyards and wields every
/// activated ability it has swallowed.
pub fn myr_welder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::ExileWithSource {
                what: target_filtered(R::Artifact.and(R::InGraveyard)),
            },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "This creature has all activated abilities of all cards exiled with it.",
            effect: StaticEffect::HasActivatedAbilitiesOfExiledWithSelf,
        }],
        ..artifact_creature("Myr Welder", cost(&[generic(3)]), vec![CreatureType::Myr], 1, 4)
    }
}

/// Galvanoth — a free instant or sorcery off the top each upkeep.
pub fn galvanoth() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::CastWithoutPayingImmediate {
                what: Selector::MatchingAmong {
                    inner: Box::new(Selector::TopOfLibrary {
                        who: PlayerRef::You,
                        count: Value::ONE,
                    }),
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                },
                source_zone: crate::card::Zone::Library,
                exile_after: false,
                copy: false,
                pay_own_cost: false,
                reduce_generic: 0,
            },
        }],
        ..creature(
            "Galvanoth",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Beast],
            3,
            3,
        )
    }
}

/// Distant Memories — tutor, then let an opponent choose which half you get.
pub fn distant_memories() -> CardDefinition {
    sorcery(
        "Distant Memories",
        cost(&[generic(2), u(), u()]),
        Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::Any,
                to: ZoneDest::ExileWithSourceStamp,
            },
            Effect::AnyPlayerMayAccept {
                who: PlayerRef::EachOpponent,
                prompt: "Give them the exiled card instead of three draws?".into(),
                accepted: Box::new(Effect::Move {
                    what: Selector::CardExiledWithSource,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
                otherwise: Box::new(draw(3)),
            },
        ]),
    )
}
