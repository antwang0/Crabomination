//! Prophecy (PCY), second wave — the Spellshaper legends, the land-eating
//! aggro shells, and the Aura cycle. Tests in `classic_sets/pcy2`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility,
    Value, WardCost,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, ZoneDest,
    shortcut::{target_any, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{ManaCost, b, cost, g, generic, r, u, w, x};

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

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
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

/// A flash Aura on a creature carrying a static bonus.
fn creature_aura(name: &'static str, c: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..enchantment(name, c)
    }
}

/// A legendary Spellshaper: `{cost}, {T}, Discard two cards: [effect]`.
fn spellshaper_legend(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    ability_cost: ManaCost,
    effect: Effect,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ability_cost,
            discard_cost: Some((R::Any, 2)),
            effect,
            ..Default::default()
        }],
        ..creature(name, c, types, 3, 3)
    }
}

/// Greel, Mind Raker — {3}{B}{B} 3/3. Two cards for X of theirs.
pub fn greel_mind_raker() -> CardDefinition {
    spellshaper_legend(
        "Greel, Mind Raker",
        cost(&[generic(3), b(), b()]),
        vec![CreatureType::Horror, CreatureType::Spellshaper],
        cost(&[x(), b()]),
        Effect::Discard {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::XFromCost,
            random: true,
        },
    )
}

/// Latulla, Keldon Overseer — {3}{R}{R} 3/3. Two cards for X damage.
pub fn latulla_keldon_overseer() -> CardDefinition {
    spellshaper_legend(
        "Latulla, Keldon Overseer",
        cost(&[generic(3), r(), r()]),
        vec![CreatureType::Human, CreatureType::Spellshaper],
        cost(&[x(), r()]),
        Effect::DealDamage { to: target_any(), amount: Value::XFromCost },
    )
}

/// Jolrael, Empress of Beasts — {3}{G}{G} 3/3. Stands their lands up so
/// they die to a sweeper — or block.
pub fn jolrael_empress_of_beasts() -> CardDefinition {
    spellshaper_legend(
        "Jolrael, Empress of Beasts",
        cost(&[generic(3), g(), g()]),
        vec![CreatureType::Human, CreatureType::Spellshaper],
        cost(&[generic(2), g()]),
        Effect::BecomeCreature {
            what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Land },
            power: Value::Const(3),
            toughness: Value::Const(3),
            creature_types: vec![],
            keywords: vec![],
            duration: Duration::EndOfTurn,
        },
    )
}

/// Mageta the Lion — {3}{W}{W} 3/3. A repeatable one-sided wrath.
pub fn mageta_the_lion() -> CardDefinition {
    spellshaper_legend(
        "Mageta the Lion",
        cost(&[generic(3), w(), w()]),
        vec![CreatureType::Human, CreatureType::Spellshaper],
        cost(&[generic(2), w(), w()]),
        Effect::Seq(vec![
            Effect::CantBeRegeneratedThisTurn {
                what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
            },
            Effect::Destroy {
                what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
            },
        ]),
    )
}

/// Hazy Homunculus — {1}{U} 1/1. Unblockable while they hold mana up.
pub fn hazy_homunculus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedIfDefenderControls(Box::new(
            R::Land.and(R::Untapped),
        ))],
        ..creature(
            "Hazy Homunculus",
            cost(&[generic(1), u()]),
            vec![CreatureType::Homunculus, CreatureType::Illusion],
            1,
            1,
        )
    }
}

/// Heightened Awareness — {3}{U}{U}. Pitches your hand for an extra draw
/// every turn.
pub fn heightened_awareness() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::Discard {
            who: Selector::You,
            amount: Value::HandSizeOf(PlayerRef::You),
            random: false,
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::YourControl),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..enchantment("Heightened Awareness", cost(&[generic(3), u(), u()]))
    }
}

/// Keldon Arsonist — {2}{R} 1/1. Two lands for one of theirs.
pub fn keldon_arsonist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Land, 2)),
            effect: Effect::Destroy { what: target_filtered(R::Land) },
            ..Default::default()
        }],
        ..creature(
            "Keldon Arsonist",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Keldon Berserker — {3}{R} 2/3. Bigger once you're tapped out.
pub fn keldon_berserker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::Not(Box::new(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::Land.and(R::Untapped).and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                })),
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Keldon Berserker",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Berserker],
            2,
            3,
        )
    }
}

/// Keldon Firebombers — {3}{R}{R} 3/3. Resets everyone to three lands.
pub fn keldon_firebombers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::EachPlayerSacrificesDownTo { filter: R::Land, keep: Value::Const(3) },
        }],
        ..creature(
            "Keldon Firebombers",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Lesser Gargadon — {2}{R}{R} 6/4. A land every time it fights.
pub fn lesser_gargadon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Land,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Land,
                },
            },
        ],
        ..creature("Lesser Gargadon", cost(&[generic(2), r(), r()]), vec![CreatureType::Beast], 6, 4)
    }
}

/// Living Terrain — {2}{G}{G}. Stands a land up as a 5/6.
pub fn living_terrain() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        equipped_bonus: Some(EquipBonus {
            set_base_pt: Some((5, 6)),
            set_card_types: Some(vec![CardType::Land, CardType::Creature]),
            set_creature_types: Some(vec![CreatureType::Treefolk]),
            set_colors: Some(vec![crate::mana::Color::Green]),
            ..Default::default()
        }),
        ..enchantment("Living Terrain", cost(&[generic(2), g(), g()]))
    }
}

/// Mageta's Boon — {1}{W}. Flash +1/+2.
pub fn magetas_boon() -> CardDefinition {
    creature_aura(
        "Mageta's Boon",
        cost(&[generic(1), w()]),
        EquipBonus { power: 1, toughness: 2, ..Default::default() },
    )
}

/// Jolrael's Favor — {1}{G}. Flash regeneration on tap.
pub fn jolraels_favor() -> CardDefinition {
    creature_aura(
        "Jolrael's Favor",
        cost(&[generic(1), g()]),
        EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(1), g()]),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Mana Vapors — {1}{U}. Costs them a whole untap step.
pub fn mana_vapors() -> CardDefinition {
    sorcery(
        "Mana Vapors",
        cost(&[generic(1), u()]),
        Effect::SkipNextUntap {
            what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Land },
        },
    )
}

/// Marsh Boa — {G} 1/1 swampwalk.
pub fn marsh_boa() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        ..creature("Marsh Boa", cost(&[g()]), vec![CreatureType::Snake], 1, 1)
    }
}

/// Mine Bearer — {2}{W} 1/1. Trades itself for an attacker.
pub fn mine_bearer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::IsAttacking)) },
            ..Default::default()
        }],
        ..creature(
            "Mine Bearer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Mirror Strike — {3}{W}. Sends an unblocked attacker's damage home.
pub fn mirror_strike() -> CardDefinition {
    CardDefinition {
        name: "Mirror Strike",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PreventAllDamageBetweenThisTurn {
            from: target_filtered(R::Creature.and(R::IsUnblocked)),
            to: Selector::Player(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Mungha Wurm — {2}{G}{G} 6/5. Huge, but your mana never comes back.
pub fn mungha_wurm() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You can't untap more than one land during your untap step.",
            effect: StaticEffect::MaxOneUntapPerStep { filter: R::Land },
        }],
        ..creature("Mungha Wurm", cost(&[generic(2), g(), g()]), vec![CreatureType::Wurm], 6, 5)
    }
}

/// Nakaya Shade — {1}{B} 1/1. A Shade anyone can tax.
pub fn nakaya_shade() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::EachOpponent,
                cost: WardCost::Mana(cost(&[generic(2)])),
                then: Box::new(Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                }),
            },
            ..Default::default()
        }],
        ..creature("Nakaya Shade", cost(&[generic(1), b()]), vec![CreatureType::Shade], 1, 1)
    }
}

/// Noxious Field — {1}{B}{B}. Turns a land into a board sweeper on a stick.
pub fn noxious_field() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::DealDamage {
                        to: Selector::EachPermanent(R::Creature),
                        amount: Value::ONE,
                    },
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachPlayer),
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..enchantment("Noxious Field", cost(&[generic(1), b(), b()]))
    }
}

/// Outbreak — {3}{B}. A type-wide shrink you can pitch a Swamp for.
pub fn outbreak() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            discard_filters: vec![(R::HasLandType(LandType::Swamp), 1)],
            ..Default::default()
        }),
        ..sorcery(
            "Outbreak",
            cost(&[generic(3), b()]),
            Effect::Seq(vec![
                Effect::NameCreatureType { what: Selector::This },
                Effect::PumpPT {
                    what: Selector::EachPermanent(R::IsSourceChosenCreatureType),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
            ]),
        )
    }
}

/// Overburden — {1}{U}. Every real creature costs a land.
pub fn overburden() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::Not(Box::new(R::IsToken))),
                }),
            effect: Effect::Move {
                what: Selector::take(
                    Selector::ControlledBy {
                        who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                        filter: R::Land,
                    },
                    Value::ONE,
                ),
                to: ZoneDest::Hand(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
            },
        }],
        ..enchantment("Overburden", cost(&[generic(1), u()]))
    }
}

/// Panic Attack — {2}{R}. Clears three blockers out of the way.
pub fn panic_attack() -> CardDefinition {
    sorcery(
        "Panic Attack",
        cost(&[generic(2), r()]),
        Effect::ApplyToTargets {
            filter: R::Creature,
            min_targets: 0,
            max_targets: 3,
            effect: Box::new(Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            }),
        },
    )
}

/// Pit Raptor — {2}{B}{B} 4/3. A great body on a rising rent.
pub fn pit_raptor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[generic(2), b(), b()]) },
        }],
        ..creature(
            "Pit Raptor",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Bird, CreatureType::Mercenary],
            4,
            3,
        )
    }
}

/// Plague Fiend — {1}{B} 1/1. Its bite kills unless they pay.
pub fn plague_fiend() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                cost: WardCost::Mana(cost(&[generic(2)])),
                then: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
        }],
        ..creature("Plague Fiend", cost(&[generic(1), b()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Pygmy Razorback — {1}{G} 2/1 trample.
pub fn pygmy_razorback() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature("Pygmy Razorback", cost(&[generic(1), g()]), vec![CreatureType::Boar], 2, 1)
    }
}

/// Quicksilver Wall — {2}{U} 1/6. A wall anyone can buy off.
pub fn quicksilver_wall() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            any_player: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
            ..Default::default()
        }],
        ..creature("Quicksilver Wall", cost(&[generic(2), u()]), vec![CreatureType::Wall], 1, 6)
    }
}

/// Jeweled Spirit — {3}{W}{W} 3/3 flier. Two lands buys it a colour.
pub fn jeweled_spirit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Land, 2)),
            effect: Effect::GrantProtectionFromChosenColor {
                what: Selector::This,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Jeweled Spirit", cost(&[generic(3), w(), w()]), vec![CreatureType::Spirit], 3, 3)
    }
}

/// Inflame — {R}. Finishes off everything that already took a hit.
pub fn inflame() -> CardDefinition {
    CardDefinition {
        name: "Inflame",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::DealtDamageThisTurn)),
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Latulla's Orders — {1}{R}. A flash Aura that eats their artifacts.
pub fn latullas_orders() -> CardDefinition {
    creature_aura(
        "Latulla's Orders",
        cost(&[generic(1), r()]),
        EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Destroy an artifact that player controls?".to_string(),
                    body: Box::new(Effect::Destroy {
                        what: Selector::take(
                            Selector::ControlledBy {
                                who: PlayerRef::TriggerEventPlayer,
                                filter: R::Artifact,
                            },
                            Value::ONE,
                        ),
                    }),
                },
            }],
            ..Default::default()
        },
    )
}
