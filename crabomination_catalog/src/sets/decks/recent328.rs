//! Bloomburrow (BLB) gaps — the Season cycle plus the legends. Tests in
//! `tests/recent_b/recent328.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn legend(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

/// The `{2}{R}: Level N. Activate only as a sorcery.` step, legal only from
/// level `n - 1`.
fn level_up(n: u8) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&[generic(2), r()]),
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(n - 1)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    }
}

fn token(name: &str, power: i32, toughness: i32, color: Color, ct: CreatureType) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power,
        toughness,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
        ..Default::default()
    }
}

/// Season of the Burrow — {3}{W}{W}. Five {P} across Rabbits, exile-and-draw
/// removal, and an indestructible reanimation.
pub fn season_of_the_burrow() -> CardDefinition {
    sorcery(
        "Season of the Burrow",
        cost(&[generic(3), w(), w()]),
        Effect::ChooseModesByPoints {
            points: vec![1, 2, 3],
            budget: 5,
            modes: vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: token("Rabbit", 1, 1, Color::White, CreatureType::Rabbit),
                },
                Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::TargetFiltered { slot: 0, filter: R::Nonland },
                        to: ZoneDest::Exile,
                    },
                    Effect::Draw {
                        who: Selector::Player(PlayerRef::ControllerOf(Box::new(
                            Selector::Target(0),
                        ))),
                        amount: Value::ONE,
                    },
                ]),
                Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::TargetFiltered {
                            slot: 0,
                            filter: R::PermanentCard
                                .and(R::InYourGraveyard)
                                .and(R::ManaValueAtMost(3)),
                        },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::AddCounter {
                        what: Selector::LastMoved,
                        kind: CounterType::Indestructible,
                        amount: Value::ONE,
                    },
                ]),
            ],
        },
    )
}

/// Season of Weaving — {4}{U}{U}. Cards, a token copy, or a full bounce.
pub fn season_of_weaving() -> CardDefinition {
    sorcery(
        "Season of Weaving",
        cost(&[generic(4), u(), u()]),
        Effect::ChooseModesByPoints {
            points: vec![1, 2, 3],
            budget: 5,
            modes: vec![
                Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::ONE },
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::one_of(Selector::EachPermanent(
                        R::Artifact.or(R::Creature).and(R::ControlledByYou),
                    )),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
                Effect::Move {
                    what: Selector::EachPermanent(R::Nonland.and(R::NotToken)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
            ],
        },
    )
}

/// Season of Loss — {3}{B}{B}. Edicts, death-fuelled draws, and a drain.
pub fn season_of_loss() -> CardDefinition {
    sorcery(
        "Season of Loss",
        cost(&[generic(3), b(), b()]),
        Effect::ChooseModesByPoints {
            points: vec![1, 2, 3],
            budget: 5,
            modes: vec![
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    count: Value::ONE,
                    filter: R::Creature,
                },
                Effect::Draw {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::CreaturesDiedThisTurn(PlayerRef::You),
                },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::CardsInGraveyardMatching {
                        who: PlayerRef::You,
                        filter: R::Creature,
                    },
                },
            ],
        },
    )
}

/// Season of the Bold — {3}{R}{R}. Treasure, impulse draw, or a two-turn
/// "whenever you cast a spell, ping something" rider.
pub fn season_of_the_bold() -> CardDefinition {
    sorcery(
        "Season of the Bold",
        cost(&[generic(3), r(), r()]),
        Effect::ChooseModesByPoints {
            points: vec![1, 2, 3],
            budget: 5,
            modes: vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        tapped: true,
                        ..crabomination_base::tokens::treasure_token()
                    },
                },
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
                Effect::OnEachSpellYouCastUntilEndOfYourNextTurn {
                    body: Box::new(Effect::OptionalTargets {
                        min: 0,
                        body: Box::new(Effect::DealDamage {
                            to: Selector::TargetFiltered { slot: 0, filter: R::Creature },
                            amount: Value::Const(2),
                        }),
                    }),
                },
            ],
        },
    )
}

/// Season of Gathering — {4}{G}{G}. Counters, a chosen-type wrath, or a big
/// draw off your fattest creature.
pub fn season_of_gathering() -> CardDefinition {
    sorcery(
        "Season of Gathering",
        cost(&[generic(4), g(), g()]),
        Effect::ChooseModesByPoints {
            points: vec![1, 2, 3],
            budget: 5,
            modes: vec![
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 1,
                    filter: R::Creature.and(R::ControlledByYou),
                    effect: Box::new(Effect::Seq(vec![
                        Effect::AddCounter {
                            what: Selector::Target(0),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        },
                        Effect::GrantKeywords {
                            what: Selector::Target(0),
                            keywords: vec![Keyword::Vigilance, Keyword::Trample],
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                },
                Effect::ChooseMode(vec![
                    Effect::Destroy { what: Selector::EachPermanent(R::Artifact) },
                    Effect::Destroy { what: Selector::EachPermanent(R::Enchantment) },
                ]),
                Effect::Draw {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::GreatestPowerControlled { who: PlayerRef::You },
                },
            ],
        },
    )
}

/// Helga, Skittish Seer — {G}{W}{U} 1/3. Big creature spells grow her and pay
/// you off; she taps for her power in ramp for more of them.
pub fn helga_skittish_seer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(R::Creature.and(R::ManaValueAtLeast(4))),
            ),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::ONE },
                Effect::GainLife { who: Selector::Player(PlayerRef::You), amount: Value::ONE },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::PowerOf(Box::new(Selector::This))),
            },
            ..Default::default()
        }],
        ..legend(
            "Helga, Skittish Seer",
            cost(&[g(), w(), u()]),
            vec![CreatureType::Frog, CreatureType::Druid],
            1,
            3,
        )
    }
}




/// Wick's entry payoff: the first Snail, then counters on it.
fn wick_snail() -> Effect {
    let snails = || {
        Selector::EachPermanent(
            R::HasCreatureType(CreatureType::Snail).and(R::ControlledByYou),
        )
    };
    Effect::If {
        cond: Predicate::SelectorExists(snails()),
        then: Box::new(Effect::AddCounter {
            what: Selector::one_of(snails()),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        }),
        else_: Box::new(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: token("Snail", 1, 1, Color::Black, CreatureType::Snail),
        }),
    }
}

/// Wick, the Whorled Mind — {3}{B} 2/4. Rats grow a Snail; the Snail can be
/// cashed in for a symmetric drain plus draws.
pub fn wick_the_whorled_mind() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(wick_snail()), TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Rat),
                }),
            effect: wick_snail(),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), b(), r()]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Snail), 1)),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::SacrificedPower,
                },
                Effect::Draw {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::SacrificedPower,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Wick, the Whorled Mind",
            cost(&[generic(3), b()]),
            vec![CreatureType::Rat, CreatureType::Warlock],
            2,
            4,
        )
    }
}

/// The Infamous Cruelclaw — {1}{B}{R} 3/3 menace. Connecting digs to the first
/// nonland card and lets you cast it by pitching a card instead.
pub fn the_infamous_cruelclaw() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            // The printed alt-cost is "discard a card"; the engine's
            // until-nonland impulse grants a free cast instead.
            effect: Effect::ExileTopUntilNonlandMayPlay {
                who: PlayerRef::You,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                free: true,
                hand_unless_mv_below: None,
                grant_to_exiling_player: true,
            },
        }],
        ..legend(
            "The Infamous Cruelclaw",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Weasel, CreatureType::Mercenary],
            3,
            3,
        )
    }
}

/// Muerra, Trash Tactician — {1}{R}{G} 2/4. Raccoons ramp you each main phase,
/// and spending mana pays off at four and eight.
pub fn muerra_trash_tactician() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::PreCombatMain),
                    EventScope::YourControl,
                ),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(
                        vec![Color::Red, Color::Green],
                        Value::count(Selector::EachPermanent(
                            R::HasCreatureType(CreatureType::Raccoon).and(R::ControlledByYou),
                        )),
                    ),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Expend, EventScope::YourControl)
                    .with_filter(Predicate::ExpendReached(4)),
                effect: Effect::GainLife {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::Const(3),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Expend, EventScope::YourControl)
                    .with_filter(Predicate::ExpendReached(8)),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
            },
        ],
        ..legend(
            "Muerra, Trash Tactician",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Raccoon, CreatureType::Warrior],
            2,
            4,
        )
    }
}



/// Dragonhawk, Fate's Tempest — {3}{R}{R} 5/5 flying. Entering or attacking
/// impulses a card per big creature, and burns for the ones you waste.
pub fn dragonhawk_fates_tempest() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: {
            let dig = || Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::count(Selector::EachPermanent(
                    R::Creature.and(R::PowerAtLeast(4)).and(R::ControlledByYou),
                )),
                duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: Some(Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                })),
            };
            vec![etb(dig()), on_attack(dig())]
        },
        ..legend(
            "Dragonhawk, Fate's Tempest",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Bird, CreatureType::Dragon],
            5,
            5,
        )
    }
}


/// Kotis, the Fangkeeper — {1}{B}{G}{U} 2/1 indestructible. Combat damage
/// exiles that much of the victim's library and free-casts what fits.
pub fn kotis_the_fangkeeper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Indestructible],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::Target(0),
                count: Value::TriggerEventAmount,
                duration: crate::card::MayPlayDuration::WhileExiled,
                pay_any_color: false,
                max_mana_value: Some(Value::TriggerEventAmount),
                pay_own_cost: false,
                uncast_penalty: None,
            },
        }],
        ..legend(
            "Kotis, the Fangkeeper",
            cost(&[generic(1), b(), g(), u()]),
            vec![CreatureType::Zombie, CreatureType::Warrior],
            2,
            1,
        )
    }
}

/// Artist's Talent — {1}{R} Class. Loot off noncreature spells, then discount
/// them, then add 2 to every noncombat hit you land.
pub fn artists_talent() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, StaticAbility};
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Artist's Talent",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Class],
            ..Default::default()
        },
        activated_abilities: vec![level_up(2), level_up(3)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Not(Box::new(R::Creature)))),
            effect: Effect::MayDo {
                description: "Discard a card to draw a card?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::You),
                        amount: Value::ONE,
                        random: false,
                    },
                    Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::ONE },
                ])),
            },
        }],
        static_abilities: vec![
            StaticAbility {
                description: "Level 2 — noncreature spells you cast cost {1} less.",
                effect: StaticEffect::WhileClassLevelAtLeast {
                    n: 2,
                    inner: Box::new(StaticEffect::CostReduction {
                        filter: R::Not(Box::new(R::Creature)),
                        amount: 1,
                    }),
                },
            },
            StaticAbility {
                description: "Level 3 — your noncombat damage to opponents gets +2.",
                effect: StaticEffect::WhileClassLevelAtLeast {
                    n: 3,
                    // `AddDamageToOpponents` covers the player half; damage to
                    // an opponent's permanents isn't scaled.
                    inner: Box::new(StaticEffect::AddDamageToOpponents {
                        source_color: None,
                        amount: 2,
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

