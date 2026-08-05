//! Duskmourn / Bloomburrow / Tarkir / Aetherdrift gap batch — Vehicles,
//! Equipment, Auras, a Leyline and graveyard engines. Tests in
//! `tests/recent_b/recent325.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword,
    OpeningHandEffect, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::card::ExileReturnZone;
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, ZoneDest,
};
use crate::game::types::TurnStep;
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

fn artifact(name: &'static str, c: ManaCost, sub: ArtifactSubtype) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![sub], ..Default::default() },
        ..Default::default()
    }
}

/// A 1/1 green Squirrel — Bloomburrow's Squirrel token.
fn squirrel_token() -> TokenDefinition {
    TokenDefinition {
        name: "Squirrel".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Your own Squirrels, Camellia excluded.
fn other_squirrels() -> R {
    R::HasCreatureType(CreatureType::Squirrel)
        .and(R::ControlledByYou)
        .and(R::Not(Box::new(R::IsSource)))
}

// ── Duskmourn ────────────────────────────────────────────────────────────────

/// Miasma Demon — discard any number on ETB, then shrink exactly that many.
pub fn miasma_demon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DiscardAnyNumber { who: Selector::You },
            Effect::CapTargetsAt {
                amount: Value::CardsDiscardedThisEffect,
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 5,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::PumpPT {
                        what: Selector::Target(0),
                        power: Value::Const(-2),
                        toughness: Value::Const(-2),
                        duration: Duration::EndOfTurn,
                    }),
                }),
            },
        ]))],
        ..creature("Miasma Demon", cost(&[generic(4), b(), b()]), vec![CreatureType::Demon], 5, 4)
    }
}

/// Stay Hidden, Stay Silent — a tap-down Aura that can bury its host.
pub fn stay_hidden_stay_silent() -> CardDefinition {
    CardDefinition {
        name: "Stay Hidden, Stay Silent",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered { slot: 0, filter: R::Creature },
        },
        triggered_abilities: vec![etb(Effect::Tap {
            what: Selector::AttachedTo(Box::new(Selector::This)),
        })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), u(), u()]),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOf(Box::new(Selector::AttachedTo(Box::new(
                            Selector::This,
                        )))),
                        pos: LibraryPosition::Shuffled,
                    },
                },
                Effect::ManifestDread { who: PlayerRef::You },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Chainsaw — a rev counter every time creatures die, and it all goes to power.
pub fn chainsaw() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![
            etb(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::DealDamage {
                    to: Selector::TargetFiltered { slot: 0, filter: R::Creature },
                    amount: Value::Const(3),
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Rev,
                    amount: Value::ONE,
                },
            },
        ],
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                filter: R::Any,
                per_power: 1,
                per_toughness: 0,
                count_self_counters: Some(CounterType::Rev),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..artifact("Chainsaw", cost(&[generic(1), r()]), ArtifactSubtype::Equipment)
    }
}

/// Dissection Tools — manifests its own wielder, then sharpens it.
pub fn dissection_tools() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ManifestDread { who: PlayerRef::You },
            Effect::Attach { what: Selector::This, to: Selector::LastMoved },
        ]))],
        equip_sacrifice_filter: Some(R::Creature),
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
            ..Default::default()
        }),
        ..artifact("Dissection Tools", cost(&[generic(5)]), ArtifactSubtype::Equipment)
    }
}

/// Unidentified Hovership — exiles a small creature until the ship leaves.
pub fn unidentified_hovership() -> CardDefinition {
    CardDefinition {
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Crew(1)],
        triggered_abilities: vec![
            etb(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::ExileUntilSourceLeaves {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::ToughnessAtMost(5)),
                    },
                    return_to: ExileReturnZone::Battlefield,
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::ManifestDread { who: PlayerRef::EachOpponent },
            },
        ],
        ..artifact("Unidentified Hovership", cost(&[generic(1), w(), w()]), ArtifactSubtype::Vehicle)
    }
}

/// Leyline of Mutation — {W}{U}{B}{R}{G} pays for anything, from turn zero.
pub fn leyline_of_mutation() -> CardDefinition {
    CardDefinition {
        name: "Leyline of Mutation",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "You may pay {W}{U}{B}{R}{G} rather than pay the mana cost for spells \
                          you cast.",
            effect: StaticEffect::FiveColorAlternativeCost,
        }],
        opening_hand: Some(OpeningHandEffect::StartInPlay { tapped: false, extra: Effect::Noop }),
        ..Default::default()
    }
}

// ── Bloomburrow ──────────────────────────────────────────────────────────────

/// Thornvault Forager — a Squirrel mana dork that pays off a forage.
pub fn thornvault_forager() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(Color::Green, Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Forage {
                    then: Box::new(Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::AnyColors(Value::Const(2)),
                    }),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3), g()]),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Squirrel),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Thornvault Forager",
            cost(&[generic(1), g()]),
            vec![CreatureType::Squirrel, CreatureType::Ranger],
            2,
            2,
        )
    }
}

/// Hoarder's Overflow — a stash counter per expend 4, cashed in for cards.
pub fn hoarders_overflow() -> CardDefinition {
    let stash = Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::Stash,
        amount: Value::ONE,
    };
    CardDefinition {
        name: "Hoarder's Overflow",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(stash.clone()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Expend, EventScope::YourControl)
                    .with_filter(Predicate::ExpendReached(4)),
                effect: stash,
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::You),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Stash,
                    },
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Festival of Embers — your graveyard becomes a second hand, and stays empty.
pub fn festival_of_embers() -> CardDefinition {
    CardDefinition {
        name: "Festival of Embers",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "During your turn, you may cast instant and sorcery spells from \
                              your graveyard by paying 1 life in addition to their other costs.",
                effect: StaticEffect::WhileYourTurn {
                    inner: Box::new(StaticEffect::GraveyardCastWithLifeSurcharge {
                        filter: R::HasCardType(CardType::Instant)
                            .or(R::HasCardType(CardType::Sorcery)),
                        life: 1,
                    }),
                },
            },
            StaticAbility {
                description: "If a card or token would be put into your graveyard from anywhere, \
                              exile it instead.",
                effect: StaticEffect::ExileCardsBoundForGraveyard {
                    opponents_only: false,
                    own_only: true,
                    colors: None,
                    card_types: None,
                    void_counter: false,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sac_cost: true,
            effect: Effect::Noop,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Camellia, the Seedmiser — menacing Squirrels, Food into bodies, forage to grow.
pub fn camellia_the_seedmiser() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Other Squirrels you control have menace.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(other_squirrels()),
                keyword: Keyword::Menace,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasArtifactSubtype(ArtifactSubtype::Food),
                }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: squirrel_token(),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Forage {
                then: Box::new(Effect::AddCounter {
                    what: Selector::EachPermanent(other_squirrels()),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Camellia, the Seedmiser",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Squirrel, CreatureType::Warlock],
            3,
            3,
        )
    }
}

// ── Tarkir: Dragonstorm ──────────────────────────────────────────────────────

/// Reverberating Summons — an enchantment that swings on a busy turn.
pub fn reverberating_summons() -> CardDefinition {
    CardDefinition {
        name: "Reverberating Summons",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer)
                .with_filter(Predicate::ValueAtLeast(
                    Value::SpellsCastThisTurn(PlayerRef::You),
                    Value::Const(2),
                )),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(3),
                creature_types: vec![CreatureType::Monk],
                keywords: vec![Keyword::Haste],
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            discard_hand_cost: true,
            sac_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stillness in Motion — mills you out, then rebuilds the top of your library.
pub fn stillness_in_motion() -> CardDefinition {
    CardDefinition {
        name: "Stillness in Motion",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(3) },
                Effect::If {
                    cond: Predicate::ValueAtMost(Value::LibrarySizeOf(PlayerRef::You), Value::ZERO),
                    then: Box::new(Effect::Seq(vec![
                        Effect::Move { what: Selector::This, to: ZoneDest::Exile },
                        Effect::ShuffleGraveyardCardsIntoLibrary {
                            who: PlayerRef::You,
                            filter: R::Any,
                            max: Value::Const(5),
                            to_top: true,
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Rite of Renewal — refills your hand from the graveyard, then exiles itself.
pub fn rite_of_renewal() -> CardDefinition {
    CardDefinition {
        name: "Rite of Renewal",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ReturnGraveyardCardsToHand {
                filter: R::Permanent,
                max: Value::Const(2),
            },
            Effect::ShuffleGraveyardCardsIntoLibrary {
                who: PlayerRef::Target(0),
                filter: R::Any,
                max: Value::Const(4),
                to_top: false,
            },
        ]),
        exile_on_resolve: true,
        ..Default::default()
    }
}

/// Dalkovan Encampment — a Warrior-making land that taps for two on demand.
pub fn dalkovan_encampment() -> CardDefinition {
    CardDefinition {
        name: "Dalkovan Encampment",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![Color::Red, Color::White], Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3), r(), w()]),
                sac_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: warrior_token(),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// A 1/1 red and white Warrior — Dalkovan Encampment's token.
fn warrior_token() -> TokenDefinition {
    TokenDefinition {
        name: "Warrior".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Warrior],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Silent Hallcreeper — unblockable, and each hit picks a fresh reward.
pub fn silent_hallcreeper() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ChooseUnchosenMode { modes: vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::BecomeCopyOf {
                    what: Selector::This,
                    source: target_filtered(R::Creature.and(R::ControlledByYou)),
                    extra_creature_types: vec![],
                    keep_own_triggered: false,
                },
            ] },
        }],
        ..creature(
            "Silent Hallcreeper",
            cost(&[generic(1), u()]),
            vec![CreatureType::Horror],
            1,
            1,
        )
    }
}

// ── Aetherdrift ──────────────────────────────────────────────────────────────

/// Thunderous Velocipede — everything else you play arrives bigger, and the
/// expensive things arrive much bigger.
pub fn thunderous_velocipede() -> CardDefinition {
    let others = R::Creature
        .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))
        .and(R::ControlledByYou)
        .and(R::Not(Box::new(R::IsSource)));
    let extra = |cheap: bool, amount: u32| StaticAbility {
        description: "Each other Vehicle and creature you control enters with an additional \
                      +1/+1 counter on it if its mana value is 4 or less, otherwise three.",
        effect: StaticEffect::MatchingEntersWithExtraCounters {
            filter: if cheap {
                others.clone().and(R::ManaValueAtMost(4))
            } else {
                others.clone().and(R::Not(Box::new(R::ManaValueAtMost(4))))
            },
            kind: CounterType::PlusOnePlusOne,
            amount,
        },
    };
    CardDefinition {
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample, Keyword::Crew(3)],
        static_abilities: vec![extra(true, 1), extra(false, 3)],
        ..artifact("Thunderous Velocipede", cost(&[generic(1), g(), g()]), ArtifactSubtype::Vehicle)
    }
}
