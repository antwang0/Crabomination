//! Commander: two-card gaps of EDHREC's commanders ranked 5501-6500 (the
//! 2026-10-14 scan), second file. Tests in `tests/recent_b/cmdr_rank6k_b.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Duration, Effect, ExtraManaKind, LookPick, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};
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

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

fn power_up(mana: ManaCost, effect: Effect) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        power_up: true,
        effect,
        ..Default::default()
    }
}

fn counters(n: i32) -> Effect {
    Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(n),
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

fn merfolk(hexproof: bool) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Merfolk".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        keywords: if hexproof {
            vec![Keyword::Hexproof]
        } else {
            vec![]
        },
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk],
            ..Default::default()
        },
        ..Default::default()
    })
}

fn another_of_yours_enters(filter: R) -> EventSpec {
    EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
        Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: filter.and(R::OtherThanSource),
        },
    )
}

// ── White Tiger, Ava Ayala ───────────────────────────────────────────────────

/// White Tiger, Ava Ayala — {1}{G} legendary 2/2 Human Hero. Power-up {5}{G}:
/// a +1/+1 counter on her and The Tiger God, a legendary 4/4 green Cat God
/// token that can't be blocked by more than one creature.
pub fn white_tiger_ava_ayala() -> CardDefinition {
    let god = Arc::new(TokenDefinition {
        name: "The Tiger God".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        colors: vec![Color::Green],
        keywords: vec![Keyword::CantBeBlockedByMoreThanOne],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::God],
            ..Default::default()
        },
        ..Default::default()
    });
    CardDefinition {
        activated_abilities: vec![power_up(
            cost(&[generic(5), g()]),
            Effect::Seq(vec![
                counters(1),
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: god,
                },
            ]),
        )],
        ..legend(
            "White Tiger, Ava Ayala",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Hero],
            2,
            2,
        )
    }
}

/// Greenhouse Propagator — {2}{G} 2/3 Cat Druid. Whenever another creature you
/// control enters, gain 1 life. {T}: add {G}.
pub fn greenhouse_propagator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: another_of_yours_enters(R::Creature),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![crate::sets::tap_add(Color::Green)],
        ..creature(
            "Greenhouse Propagator",
            cost(&[generic(2), g()]),
            vec![CreatureType::Cat, CreatureType::Druid],
            2,
            3,
        )
    }
}

// ── Ant-Man, Scott Lang ──────────────────────────────────────────────────────

/// Ant-Man, Scott Lang — {1}{W} legendary 2/2 Human Hero. {4}: a +1/+1 counter
/// on him.
pub fn ant_man_scott_lang() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            effect: counters(1),
            ..Default::default()
        }],
        ..legend(
            "Ant-Man, Scott Lang",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Hero],
            2,
            2,
        )
    }
}

/// Kayla's Reconstruction — {X}{W}{W}{W} sorcery. Look at the top seven; put up
/// to X artifact and/or creature cards with mana value 3 or less from among
/// them onto the battlefield; the rest on the bottom in a random order.
pub fn kaylas_reconstruction() -> CardDefinition {
    spell(
        "Kayla's Reconstruction",
        cost(&[x(), w(), w(), w()]),
        CardType::Sorcery,
        Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(7),
            pick_filter: Some(R::Artifact.or(R::Creature).and(R::ManaValueAtMost(3))),
            take: Some(Value::XFromCost),
            to_battlefield: true,
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        })),
    )
}

// ── Fixer, Techno Terror ─────────────────────────────────────────────────────

/// Fixer, Techno Terror — {1}{B} legendary 2/1 Human Artificer Villain. {T},
/// pay 2 life: draw a card; activate only if an artifact entered the
/// battlefield under your control this turn.
pub fn fixer_techno_terror() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 2,
            condition: Some(Predicate::ArtifactEnteredThisTurn {
                who: PlayerRef::You,
            }),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..legend(
            "Fixer, Techno Terror",
            cost(&[generic(1), b()]),
            vec![
                CreatureType::Human,
                CreatureType::Artificer,
                CreatureType::Villain,
            ],
            2,
            1,
        )
    }
}

/// Glaze Fiend — {1}{B} 0/1 flying Illusion artifact creature. Whenever
/// another artifact you control enters, it gets +2/+2 until end of turn.
pub fn glaze_fiend() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: another_of_yours_enters(R::Artifact),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Glaze Fiend",
            cost(&[generic(1), b()]),
            vec![CreatureType::Illusion],
            0,
            1,
        )
    }
}

// ── Goliath, Mass Manipulator ────────────────────────────────────────────────

/// Goliath, Mass Manipulator — {1}{G} legendary 2/2 reach Human Scientist Hero.
/// Power-up {4}{G}: two +1/+1 counters on him, then draw a card for each
/// creature you control with power 4 or greater.
pub fn goliath_mass_manipulator() -> CardDefinition {
    let big = Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)));
    CardDefinition {
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![power_up(
            cost(&[generic(4), g()]),
            Effect::Seq(vec![
                counters(2),
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::CountOf(Box::new(big)),
                },
            ]),
        )],
        ..legend(
            "Goliath, Mass Manipulator",
            cost(&[generic(1), g()]),
            vec![
                CreatureType::Human,
                CreatureType::Scientist,
                CreatureType::Hero,
            ],
            2,
            2,
        )
    }
}

/// Garruk's Harbinger — {1}{G}{G} 4/3 Beast; hexproof from black. Whenever it
/// deals combat damage to a player, look at that many cards from the top; you
/// may reveal a creature or Garruk planeswalker card and put it into your
/// hand; the rest on the bottom in a random order. (Residual: combat damage
/// to a planeswalker doesn't trigger it.)
pub fn garruks_harbinger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::HexproofFromColor(Color::Black)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::TriggerEventAmount,
                pick_filter: Some(R::Creature.or(R::HasPlaneswalkerType(
                    crate::card::PlaneswalkerSubtype::Garruk,
                ))),
                optional: true,
                rest_bottom_random: true,
                ..Default::default()
            })),
        }],
        ..creature(
            "Garruk's Harbinger",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Beast],
            4,
            3,
        )
    }
}

// ── Namora, the Sea Queen ────────────────────────────────────────────────────

/// Namora, the Sea Queen — {2}{U} legendary 2/3 flying Mutant Merfolk Hero.
/// Power-up {5}{U}: a +1/+1 counter on her and two 1/1 blue Merfolk.
pub fn namora_the_sea_queen() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![power_up(
            cost(&[generic(5), u()]),
            Effect::Seq(vec![
                counters(1),
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: merfolk(false),
                },
            ]),
        )],
        ..legend(
            "Namora, the Sea Queen",
            cost(&[generic(2), u()]),
            vec![
                CreatureType::Mutant,
                CreatureType::Merfolk,
                CreatureType::Hero,
            ],
            2,
            3,
        )
    }
}

/// Aquatic Incursion — {3}{U} enchantment. ETB: two 1/1 blue Merfolk with
/// hexproof. {3}{U}: target Merfolk can't be blocked this turn.
pub fn aquatic_incursion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: merfolk(true),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Merfolk))),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..spell(
            "Aquatic Incursion",
            cost(&[generic(3), u()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

// ── Jack of Hearts, Volatile Hero ────────────────────────────────────────────

/// Jack of Hearts, Volatile Hero — {1}{R}{R} legendary 2/2 flying, haste Alien
/// Human Hero. When he dies, he deals damage equal to his power to each
/// creature. Power-up {4}{R}{R}: discard your hand, draw three cards, and two
/// +1/+1 counters on him.
pub fn jack_of_hearts_volatile_hero() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![on_dies(Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature),
            amount: Value::PowerOf(Box::new(Selector::This)),
        })],
        activated_abilities: vec![power_up(
            cost(&[generic(4), r(), r()]),
            Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::You),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
                counters(2),
            ]),
        )],
        ..legend(
            "Jack of Hearts, Volatile Hero",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Alien, CreatureType::Human, CreatureType::Hero],
            2,
            2,
        )
    }
}

/// Gauntlet of Might — {4} artifact. Red creatures get +1/+1. Whenever a
/// Mountain is tapped for mana, its controller adds an additional {R}.
pub fn gauntlet_of_might() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Red creatures get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Red))),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Whenever a Mountain is tapped for mana, its controller adds an additional {R}.",
                effect: StaticEffect::ExtraManaOnLandTap {
                    enchanted_only: false,
                    filter: R::HasLandType(LandType::Mountain),
                    extra: ExtraManaKind::Fixed(Color::Red),
                    while_monarch: false,
                },
            },
        ],
        ..spell(
            "Gauntlet of Might",
            cost(&[generic(4)]),
            CardType::Artifact,
            Effect::Noop,
        )
    }
}

// ── White Widow, Free Agent ──────────────────────────────────────────────────

/// White Widow, Free Agent — {3}{W} legendary 2/3 Human Hero Villain. ETB,
/// choose one: a +1/+1 counter on each of up to two target creatures; or
/// return target artifact or enchantment card from your graveyard to hand.
pub fn white_widow_free_agent() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
            Effect::Move {
                what: target_filtered(R::Artifact.or(R::Enchantment).from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]))],
        ..legend(
            "White Widow, Free Agent",
            cost(&[generic(3), w()]),
            vec![
                CreatureType::Human,
                CreatureType::Hero,
                CreatureType::Villain,
            ],
            2,
            3,
        )
    }
}

/// Promise of Tomorrow — {2}{W} enchantment. Whenever a creature you control
/// dies, exile it. At the beginning of each end step, if you control no
/// creatures, sacrifice it and return all cards exiled with it to the
/// battlefield under your control.
pub fn promise_of_tomorrow() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
                effect: Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::ExileWithSourceStamp,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::End),
                    EventScope::AnyPlayer,
                )
                .with_filter(Predicate::Not(Box::new(Predicate::SelectorExists(
                    Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                )))),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::CardExiledWithSource,
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    },
                    Effect::SacrificeSource,
                ]),
            },
        ],
        ..spell(
            "Promise of Tomorrow",
            cost(&[generic(2), w()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

// ── Rescue, Pepper Potts ─────────────────────────────────────────────────────

/// Covert Technician — {2}{U} 2/4 Human Ninja artifact creature; ninjutsu
/// {1}{U}. Whenever it deals combat damage to a player, you may put an
/// artifact card with mana value at most that damage from your hand onto the
/// battlefield.
pub fn covert_technician() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(1), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::WithX {
                x: Value::TriggerEventAmount,
                body: Box::new(Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Artifact.and(R::ManaValueAtMostXFromCost),
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                }),
            },
        }],
        ..creature(
            "Covert Technician",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Ninja],
            2,
            4,
        )
    }
}

/// Replication Specialist — {4}{U} 3/4 flying Moonfolk Artificer. Whenever a
/// nontoken artifact you control enters, you may pay {1}{U}; if you do, create
/// a token copy of it.
pub fn replication_specialist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact.and(R::NotToken),
                }),
            effect: Effect::MayPay {
                description: "Pay {1}{U} to copy the artifact?".into(),
                mana_cost: cost(&[generic(1), u()]),
                body: Box::new(crate::effect::shortcut::token_copy_of(
                    PlayerRef::You,
                    Value::ONE,
                    Selector::TriggerSource,
                )),
                else_: None,
            },
        }],
        ..creature(
            "Replication Specialist",
            cost(&[generic(4), u()]),
            vec![CreatureType::Moonfolk, CreatureType::Artificer],
            3,
            4,
        )
    }
}
