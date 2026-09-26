//! Commander: the cards the **Peace Offering** precon (BLC, Ms.
//! Bumbleflower) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_bumbleflower.rs`.
//!
//! Residuals (each also on its card):
//! - **Perch Protection** — the life lock lasts this turn, not until your
//!   next turn.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Gift, Keyword, LoyaltyAbility, PlaneswalkerSubtype,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, u, w};
use crate::sets::{enters_tapped, tap_add};
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn draw(who: PlayerRef, n: i32) -> Effect {
    Effect::Draw { who: Selector::Player(who), amount: Value::Const(n) }
}

fn blue_token(name: &str, ct: CreatureType, p: i32, t: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn octopus() -> TokenDefinition {
    blue_token("Octopus", CreatureType::Octopus, 8, 8, vec![])
}

/// A Thriving land: enters tapped, choose a color other than its own; {T}:
/// add its color or the chosen one.
pub(crate) fn thriving(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        as_enters_effect: Some(Effect::ChooseColorForSelfOtherThan(color)),
        activated_abilities: vec![
            tap_add(color),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::ChosenColorOfSource },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ms. Bumbleflower — vigilance; whenever you cast a spell, target opponent
/// draws, a +1/+1 counter and flying go on target creature, and the second
/// resolution this turn draws you two.
pub fn ms_bumbleflower() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::TargetFiltered { slot: 0, filter: R::OpponentPlayer },
                    amount: Value::ONE,
                },
                Effect::AddCounter {
                    what: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::GrantKeyword { what: Selector::Target(1), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
                Effect::NthResolutionThisTurn { branches: vec![Effect::Noop, draw(PlayerRef::You, 2)] },
            ]),
        }],
        ..legendary(creature(
            "Ms. Bumbleflower",
            cost(&[generic(1), g(), w(), u()]),
            vec![CreatureType::Rabbit, CreatureType::Citizen],
            1,
            5,
        ))
    }
}

/// Bloodroot Apothecary — toxic 2; on entry you and target opponent each
/// create a Treasure; an opponent sacrificing a noncreature token gets two
/// poison counters.
pub fn bloodroot_apothecary() -> CardDefinition {
    let treasure = || Arc::new(crabomination_base::tokens::treasure_token());
    CardDefinition {
        keywords: vec![Keyword::Toxic(2)],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: treasure() },
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 1,
                    filter: R::OpponentPlayer,
                    effect: Box::new(Effect::CreateToken {
                        who: PlayerRef::Target(0),
                        count: Value::ONE,
                        definition: treasure(),
                    }),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::OpponentControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsToken.and(R::Noncreature) },
                ),
                effect: Effect::AddPoison {
                    who: Selector::Player(PlayerRef::TriggerEventPlayer),
                    amount: Value::Const(2),
                },
            },
        ],
        ..creature(
            "Bloodroot Apothecary",
            cost(&[generic(2), g()]),
            vec![CreatureType::Squirrel, CreatureType::Druid],
            3,
            3,
        )
    }
}

/// Body of Knowledge — */* equal to the cards in your hand (CR 604.3); no
/// maximum hand size; damage dealt to it draws that many.
pub fn body_of_knowledge() -> CardDefinition {
    let hand = || Value::HandSizeOf(PlayerRef::You);
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Body of Knowledge's power and toughness are each equal to the number of cards in your hand.",
                effect: StaticEffect::SelfBasePtFromValue { power: hand(), toughness: hand() },
            },
            StaticAbility { description: "You have no maximum hand size.", effect: StaticEffect::NoMaximumHandSize },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..creature("Body of Knowledge", cost(&[generic(3), u(), u()]), vec![CreatureType::Avatar], 0, 0)
    }
}

/// Communal Brewing — on entry any number of target opponents each draw; an
/// ingredient counter, plus one per card drawn; your creature spells enter
/// with that many extra +1/+1 counters.
pub fn communal_brewing() -> CardDefinition {
    CardDefinition {
        name: "Communal Brewing",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::OpponentPlayer,
                    effect: Box::new(Effect::Draw { who: Selector::Target(0), amount: Value::ONE }),
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Ingredient,
                    amount: Value::Sum(vec![Value::ONE, Value::CardsDrawnThisEffect]),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellMatches(R::Creature)),
                effect: Effect::SpellEntersWithCounters {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Ingredient },
                },
            },
        ],
        ..Default::default()
    }
}

/// Fisher's Talent — Class. Level 1: at your upkeep, a land on top makes a
/// 1/1 Fish, then draw. Level 2 ({G}{U}): Fish become 3/3 Sharks. Level 3
/// ({2}{G}{U}): Sharks become 8/8 Octopuses.
pub fn fishers_talent() -> CardDefinition {
    let level_up = |mana: ManaCost, from: u8| ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    let becomes = |n: u8, from: &str, into: TokenDefinition, description: &'static str| StaticAbility {
        description,
        effect: StaticEffect::WhileClassLevelAtLeast {
            n,
            inner: Box::new(StaticEffect::TokenNamedBecomes { name: from.into(), into }),
        },
    };
    CardDefinition {
        name: "Fisher's Talent",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Class], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                        filter: R::Land,
                    },
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(blue_token("Fish", CreatureType::Fish, 1, 1, vec![])),
                    }),
                    else_: Box::new(Effect::Noop),
                },
                draw(PlayerRef::You, 1),
            ]),
        }],
        static_abilities: vec![
            becomes(2, "Fish", blue_token("Shark", CreatureType::Shark, 3, 3, vec![]), "Level 2 — Fish tokens become 3/3 Sharks."),
            becomes(3, "Shark", octopus(), "Level 3 — Shark tokens become 8/8 Octopuses."),
        ],
        activated_abilities: vec![
            level_up(cost(&[g(), u()]), 1),
            level_up(cost(&[generic(2), g(), u()]), 2),
        ],
        ..Default::default()
    }
}

/// Ghirapur Orrery — each player may play an additional land on each of
/// their turns; an empty-handed player draws three at their upkeep.
pub fn ghirapur_orrery() -> CardDefinition {
    let extra_land = || Effect::GrantExtraLandPlay { who: PlayerRef::ActivePlayer, count: Value::ONE };
    CardDefinition {
        name: "Ghirapur Orrery",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            etb(extra_land()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
                effect: Effect::Seq(vec![
                    extra_land(),
                    Effect::If {
                        cond: Predicate::ValueAtMost(Value::HandSizeOf(PlayerRef::ActivePlayer), Value::Const(0)),
                        then: Box::new(draw(PlayerRef::ActivePlayer, 3)),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Kwain, Itinerant Meddler — {T}: each player may draw; each who did gains
/// 1 life.
pub fn kwain_itinerant_meddler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::EachPlayerMayDrawThenTakersGainLife { life: 1 },
            ..Default::default()
        }],
        ..legendary(creature(
            "Kwain, Itinerant Meddler",
            cost(&[w(), u()]),
            vec![CreatureType::Rabbit, CreatureType::Wizard],
            1,
            3,
        ))
    }
}

/// Martial Impetus — +1/+1 and goaded; whenever it attacks, creatures
/// attacking your opponents get +1/+1 until end of turn.
pub fn martial_impetus() -> CardDefinition {
    CardDefinition {
        name: "Martial Impetus",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggers_on_equipment: true,
            triggered_abilities: vec![on_attack(Effect::PumpPT {
                // "each other creature": the Aura is the source here, so
                // "other" is its host.
                what: Selector::EachPermanent(R::Creature.and(R::IsAttackingAnOpponent).and(R::IsHostOfSource.negate())),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            })],
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature is goaded.",
            effect: StaticEffect::AttachedIsGoaded,
        }],
        ..Default::default()
    }
}

/// Mr. Foxglove — lifelink; attacking, draw up to the defending player's
/// hand size; if that drew nothing, you may put a creature from hand onto
/// the battlefield.
pub fn mr_foxglove() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Diff(
                    Box::new(Value::HandSizeOf(PlayerRef::DefendingPlayer)),
                    Box::new(Value::HandSizeOf(PlayerRef::You)),
                ),
            },
            Effect::If {
                cond: Predicate::ValueAtMost(Value::CardsDrawnThisEffect, Value::Const(0)),
                then: Box::new(Effect::MayDo {
                    description: "Put a creature card from your hand onto the battlefield?".into(),
                    body: Box::new(Effect::PutFromHandOntoBattlefield {
                        who: PlayerRef::You,
                        filter: R::Creature,
                        count: Value::ONE,
                        tapped: false,
                        haste: false,
                        sacrifice_eot: false,
                        return_eot: false,
                        then: None,
                    }),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..legendary(creature(
            "Mr. Foxglove",
            cost(&[generic(2), g(), w(), u()]),
            vec![CreatureType::Fox, CreatureType::Rogue],
            3,
            5,
        ))
    }
}

/// Octomancer — gift an Octopus; at the beginning of each end step, a copy
/// of target creature token that entered this turn.
pub fn octomancer() -> CardDefinition {
    CardDefinition {
        gift: Some(Box::new(Gift { label: "an Octopus", gifted_effect: Effect::Noop })),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(Predicate::SourceGiftPromised),
                effect: Effect::CreateToken { who: PlayerRef::ChosenPlayerOfSource, count: Value::ONE, definition: Arc::new(octopus()) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
                effect: Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: target_filtered(R::Creature.and(R::IsToken).and(R::EnteredThisTurn)),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
            },
        ],
        ..creature(
            "Octomancer",
            cost(&[generic(3), g(), u()]),
            vec![CreatureType::Frog, CreatureType::Druid],
            3,
            3,
        )
    }
}

/// Perch Protection — four 2/2 flying Birds; gift an extra turn: your
/// permanents phase out and you're protected until your next turn. Exile it.
///
/// ⚠ Residual: the life lock lasts this turn only.
pub fn perch_protection() -> CardDefinition {
    let birds = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(4),
        definition: Arc::new(blue_token("Bird", CreatureType::Bird, 2, 2, vec![Keyword::Flying])),
    };
    CardDefinition {
        name: "Perch Protection",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Instant],
        exile_on_resolve: true,
        effect: birds(),
        gift: Some(Box::new(Gift {
            label: "an extra turn",
            gifted_effect: Effect::Seq(vec![
                Effect::TakeExtraTurn { who: PlayerRef::ChosenPlayerOfSource, count: Value::ONE },
                birds(),
                Effect::PhaseOut { what: yours(R::Permanent), until_source_leaves: false },
                Effect::LifeLockUntilNextTurn { who: Selector::You },
                Effect::PlayerProtectionUntilNextTurn { who: PlayerRef::You },
            ]),
        })),
        ..Default::default()
    }
}

/// Promise of Loyalty — each player keeps one creature with a vow counter and
/// sacrifices the rest; the vowed can't attack you while it keeps its vow
/// counter.
pub fn promise_of_loyalty() -> CardDefinition {
    CardDefinition {
        name: "Promise of Loyalty",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::EachPlayerKeepsNSacrificesRest { keep: Value::ONE, filter: Some(R::Creature) },
            Effect::AddCounter { what: Selector::EachPermanent(R::Creature), kind: CounterType::Vow, amount: Value::ONE },
            Effect::GrantCantAttackYou {
                what: Selector::EachPermanent(R::Creature.and(R::WithCounter(CounterType::Vow))),
                duration: Duration::Permanent,
            },
        ]),
        ..Default::default()
    }
}

/// Steelburr Champion — offspring {1}{W}; vigilance; an opponent's
/// noncreature spell grows it.
pub fn steelburr_champion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Offspring(cost(&[generic(1), w()]))],
        triggered_abilities: vec![
            // CR 702.175 — the offspring copy, when its cost was paid.
            etb(Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::This,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: Some((1, 1)),
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                }),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Noncreature },
                ),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        ..creature(
            "Steelburr Champion",
            cost(&[generic(2), w()]),
            vec![CreatureType::Mouse, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Tamiyo, Field Researcher — loyalty 4. +1: up to two creatures draw you a
/// card for combat damage until your next turn. −2: tap up to two nonland
/// permanents; they skip their next untap. −7: draw three and an emblem
/// casting your hand spells for free.
pub fn tamiyo_field_researcher() -> CardDefinition {
    CardDefinition {
        name: "Tamiyo, Field Researcher",
        cost: cost(&[generic(1), g(), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Tamiyo], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Creature,
                    // A delayed trigger of Tamiyo's controller, not a grant:
                    // an opponent's creature draws *you* the card.
                    effect: Box::new(Effect::WatchCombatDamageUntilYourNextTurn {
                        what: Selector::Target(0),
                        body: Box::new(draw(PlayerRef::You, 1)),
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Permanent.and(R::Nonland),
                    effect: Box::new(Effect::Seq(vec![
                        Effect::Tap { what: Selector::Target(0) },
                        Effect::SkipNextUntap { what: Selector::Target(0) },
                    ])),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::Seq(vec![
                    draw(PlayerRef::You, 3),
                    Effect::CreateEmblem {
                        who: PlayerRef::You,
                        name: "Tamiyo, Field Researcher".into(),
                        triggered: vec![],
                        statics: vec![StaticAbility {
                            description: "You may cast spells from your hand without paying their mana costs.",
                            effect: StaticEffect::CastHandSpellsFree,
                        }],
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Tenuous Truce — enchant opponent; at their end step you and they each
/// draw; attacking between you and them sacrifices it.
pub fn tenuous_truce() -> CardDefinition {
    let attack_between = |attacker: PlayerRef, defender: PlayerRef| {
        Predicate::All(vec![
            Predicate::IsTurnOf(attacker),
            Predicate::AttackedDefenderWithCountAtLeast {
                who: PlayerRef::ActivePlayer,
                defender,
                at_least: 1,
                include_planeswalkers: true,
            },
        ])
    };
    CardDefinition {
        name: "Tenuous Truce",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::OpponentPlayer) },
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                    .with_filter(Predicate::IsTurnOf(PlayerRef::EnchantedPlayer)),
                effect: Effect::Seq(vec![draw(PlayerRef::You, 1), draw(PlayerRef::EnchantedPlayer, 1)]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer).with_filter(Predicate::Any(vec![
                    attack_between(PlayerRef::You, PlayerRef::EnchantedPlayer),
                    attack_between(PlayerRef::EnchantedPlayer, PlayerRef::You),
                ])),
                effect: Effect::SacrificeSource,
            },
        ],
        ..Default::default()
    }
}

/// Thriving Grove — enters tapped; {T}: {G} or a chosen other color.
pub fn thriving_grove() -> CardDefinition {
    thriving("Thriving Grove", Color::Green)
}

/// Thriving Heath — enters tapped; {T}: {W} or a chosen other color.
pub fn thriving_heath() -> CardDefinition {
    thriving("Thriving Heath", Color::White)
}

/// Thriving Isle — enters tapped; {T}: {U} or a chosen other color.
pub fn thriving_isle() -> CardDefinition {
    thriving("Thriving Isle", Color::Blue)
}

/// Twenty-Toed Toad — maximum hand size twenty; attacking with two or more
/// grows it and draws; attacking with twenty counters or twenty cards in
/// hand wins the game.
pub fn twenty_toed_toad() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Your maximum hand size is twenty.",
            effect: StaticEffect::ControllerMaxHandSize(20),
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                    .with_filter(Predicate::AttackedWithCountAtLeast { who: PlayerRef::You, at_least: 2 }),
                effect: Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    draw(PlayerRef::You, 1),
                ]),
            },
            on_attack(Effect::If {
                cond: Predicate::Any(vec![
                    Predicate::ValueAtLeast(Value::TotalCountersOn { what: Box::new(Selector::This) }, Value::Const(20)),
                    Predicate::ValueAtLeast(Value::HandSizeOf(PlayerRef::You), Value::Const(20)),
                ]),
                then: Box::new(Effect::WinGame { who: PlayerRef::You }),
                else_: Box::new(Effect::Noop),
            }),
        ],
        ..creature(
            "Twenty-Toed Toad",
            cost(&[generic(3), u()]),
            vec![CreatureType::Frog, CreatureType::Wizard],
            3,
            3,
        )
    }
}
