//! Duskmourn / Foundations / Bloomburrow grab-bag. Tests in
//! `tests/recent52.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, Effect,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, Predicate, RoomDoor, RoomDoors,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{investigate, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{Color, SpendRestriction, b, cost, g, generic, r, u};

/// Nethergoyf — {B} Lhurgoyf */1+*. Power = card types among cards in your
/// graveyard, toughness = that + 1. Escape—{2}{B}, exile four other cards.
pub fn nethergoyf() -> CardDefinition {
    CardDefinition {
        name: "Nethergoyf",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lhurgoyf],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        dynamic_pt: Some(DynamicPt::CardTypesInControllerGraveyard {
            base_p: 0,
            base_t: 1,
        }),
        keywords: vec![Keyword::Escape(cost(&[generic(2), b()]), 4)],
        ..Default::default()
    }
}

/// Omen Hawker — {U} 1/1 Octopus Advisor. {T}: Add {C}{U}, spend only to
/// activate abilities.
pub fn omen_hawker() -> CardDefinition {
    CardDefinition {
        name: "Omen Hawker",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Octopus, CreatureType::Advisor],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colorless(Value::ONE)),
                        SpendRestriction::AbilitiesOnly,
                    ),
                },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::Blue])),
                        SpendRestriction::AbilitiesOnly,
                    ),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hazardous Blast — {3}{R} Sorcery. 1 damage to each creature your opponents
/// control; those creatures can't block this turn.
pub fn hazardous_blast() -> CardDefinition {
    let opp_creatures = || R::Creature.and(R::ControlledByOpponent);
    CardDefinition {
        name: "Hazardous Blast",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(opp_creatures()),
                amount: Value::ONE,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(opp_creatures()),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Toxin Analysis — {B} Instant. Target creature gains deathtouch and lifelink
/// until end of turn. Investigate.
pub fn toxin_analysis() -> CardDefinition {
    CardDefinition {
        name: "Toxin Analysis",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            investigate(1),
        ]),
        ..Default::default()
    }
}

/// Enduring Courage — {2}{R}{R} Enchantment Creature — Dog Glimmer 3/3.
/// Whenever another creature you control enters, it gets +2/+0 and gains haste
/// until end of turn. Dies → returns as a noncreature enchantment.
pub fn enduring_courage() -> CardDefinition {
    CardDefinition {
        name: "Enduring Courage",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog, CreatureType::Glimmer],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    }),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::TriggerSource,
                        power: Value::Const(2),
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::TriggerSource,
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            },
            crate::effect::shortcut::on_dies(Effect::ReturnSelfAsEnchantment),
        ],
        ..Default::default()
    }
}

/// Vexing Bauble — {1} Artifact. Whenever a player casts a spell, if no mana
/// was spent to cast it, counter it. {1}, {T}, Sacrifice this: Draw a card.
pub fn vexing_bauble() -> CardDefinition {
    CardDefinition {
        name: "Vexing Bauble",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: Effect::If {
                cond: Predicate::ValueEquals(Value::CastSpellManaSpent, Value::Const(0)),
                then: Box::new(Effect::CounterSpell {
                    what: Selector::TriggerSource,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Loot, Exuberant Explorer — {2}{G} 1/4 Beast Noble. Extra land each turn.
/// {4}{G}{G}, {T}: dig 6, may put a creature with MV ≤ lands you control onto
/// the battlefield; rest on the bottom.
pub fn loot_exuberant_explorer() -> CardDefinition {
    CardDefinition {
        name: "Loot, Exuberant Explorer",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast, CreatureType::Noble],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "You may play an additional land on each of your turns.",
            effect: StaticEffect::ExtraLandPerTurn,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g(), g()]),
            tap_cost: true,
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(6),
                rest_to_graveyard: false,
                pick_filter: Some(R::Creature.and(R::ManaValueAtMostYourCount(Box::new(R::Land)))),
                take: None,
                to_battlefield: true,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: true,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
                rest_to_exile: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Roaring Furnace // Steaming Sauna — {1}{R} // {3}{U}{U} Room (DSK).
/// Furnace's unlock deals damage = cards in hand to an opponent's creature;
/// Sauna grants no max hand size and an end-step draw.
pub fn roaring_furnace_steaming_sauna() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Roaring Furnace // Steaming Sauna",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Room],
            ..Default::default()
        },
        room: Some(Box::new(RoomDoors {
            left: RoomDoor {
                name: "Roaring Furnace".to_string(),
                cost: cost(&[generic(1), r()]),
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::DoorUnlocked, EventScope::SelfSource),
                    effect: Effect::DealDamage {
                        to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                        amount: Value::HandSizeOf(PlayerRef::You),
                    },
                }],
                ..Default::default()
            },
            right: RoomDoor {
                name: "Steaming Sauna".to_string(),
                cost: cost(&[generic(3), u(), u()]),
                static_abilities: vec![StaticAbility {
                    description: "You have no maximum hand size.",
                    effect: StaticEffect::NoMaximumHandSize,
                }],
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::StepBegins(TurnStep::End),
                        EventScope::ActivePlayer,
                    ),
                    effect: Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                }],
                ..Default::default()
            },
        })),
        ..Default::default()
    }
}

/// Defiled Crypt // Cadaver Lab — {3}{B} // {B} Room (DSK). Crypt mints a 2/2
/// Horror enchantment when cards leave your graveyard (once/turn); Cadaver
/// Lab's unlock returns a creature card from your graveyard to hand.
pub fn defiled_crypt_cadaver_lab() -> CardDefinition {
    CardDefinition {
        name: "Defiled Crypt // Cadaver Lab",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Room],
            ..Default::default()
        },
        room: Some(Box::new(RoomDoors {
            left: RoomDoor {
                name: "Defiled Crypt".to_string(),
                cost: cost(&[generic(3), b()]),
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
                        .once_per_turn(),
                    effect: Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: TokenDefinition {
                            name: "Horror".to_string(),
                            power: 2,
                            toughness: 2,
                            card_types: vec![CardType::Enchantment, CardType::Creature],
                            colors: vec![Color::Black],
                            subtypes: Subtypes {
                                creature_types: vec![CreatureType::Horror],
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    },
                }],
                ..Default::default()
            },
            right: RoomDoor {
                name: "Cadaver Lab".to_string(),
                cost: cost(&[b()]),
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::DoorUnlocked, EventScope::SelfSource),
                    effect: Effect::Move {
                        what: target_filtered(R::Creature),
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                }],
                ..Default::default()
            },
        })),
        ..Default::default()
    }
}

/// Winter, Misanthropic Guide — {1}{B}{R}{G} 3/4 Human Warlock. Ward {2}. At
/// your upkeep, each player draws two. Delirium: opponents' max hand size = 7 −
/// the number of card types in your graveyard.
pub fn winter_misanthropic_guide() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Winter, Misanthropic Guide",
        cost: cost(&[generic(1), b(), r(), g()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Ward(WardCost::generic(2))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Zimone, All-Questioning — {1}{G}{U} 1/1 Human Wizard. At your end step, if a
/// land entered under your control this turn and you control a prime number of
/// lands, create Primo, the Indivisible (legendary 0/0 G/U Fractal) with that
/// many +1/+1 counters.
pub fn zimone_all_questioning() -> CardDefinition {
    use crate::game::types::TurnStep;
    let lands_you_control =
        || Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByYou)));
    CardDefinition {
        name: "Zimone, All-Questioning",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::All(vec![
                Predicate::ValueAtLeast(Value::LandsPlayedThisTurn(PlayerRef::You), Value::ONE),
                Predicate::ValueIsPrime(lands_you_control()),
            ])),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Primo, the Indivisible".to_string(),
                        power: 0,
                        toughness: 0,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Green, Color::Blue],
                        supertypes: vec![Supertype::Legendary],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Fractal],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: lands_you_control(),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Ghostly Dancers — {3}{W}{W} 2/5 Spirit. Flying. ETB return an enchantment
/// card from your graveyard to hand. Eerie — whenever an enchantment you
/// control enters, create a 3/1 white Spirit with flying.
pub fn ghostly_dancers() -> CardDefinition {
    let spirit_token = || TokenDefinition {
        name: "Spirit".to_string(),
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Ghostly Dancers",
        cost: cost(&[generic(3), crate::mana::w(), crate::mana::w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Move {
                    what: target_filtered(R::Enchantment),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Enchantment,
                    }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: spirit_token(),
                },
            },
        ],
        ..Default::default()
    }
}

/// Pirated Copy — {4}{U} 0/0 Shapeshifter Pirate. May enter as a copy of any
/// creature on the battlefield, except it's also a Pirate and has "whenever
/// this deals combat damage to a player, draw a card." (The "another creature
/// with the same name" half is approximated to the copy itself.)
pub fn pirated_copy() -> CardDefinition {
    use crate::card::EntersAsCopy;
    CardDefinition {
        name: "Pirated Copy",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter, CreatureType::Pirate],
            ..Default::default()
        },
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            extra_creature_types: vec![CreatureType::Pirate],
            extra_triggered: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Unwanted Remake — {W} Instant. Destroy target creature. Its controller
/// manifests dread.
pub fn unwanted_remake() -> CardDefinition {
    CardDefinition {
        name: "Unwanted Remake",
        cost: cost(&[crate::mana::w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Creature),
            },
            Effect::ManifestDread {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Fear of the Dark — {4}{B} 5/5 Enchantment Creature — Nightmare. When it
/// attacks it gains menace and deathtouch until end of turn. (The "if defending
/// player controls no Glimmer creatures" rider is approximated as
/// unconditional — Glimmers are rare.)
pub fn fear_of_the_dark() -> CardDefinition {
    CardDefinition {
        name: "Fear of the Dark",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::Seq(vec![
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Brimstone Roundup — {1}{R} Enchantment. Whenever you cast your second spell
/// each turn, create a 1/1 red Mercenary with "{T}: target creature you control
/// gets +1/+0 until end of turn; activate only as a sorcery." Plot {2}{R}.
pub fn brimstone_roundup() -> CardDefinition {
    let mercenary = || TokenDefinition {
        name: "Mercenary".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mercenary],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Brimstone Roundup",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        plot_cost: Some(cost(&[generic(2), r()])),
        triggered_abilities: vec![crate::effect::shortcut::flurry(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: mercenary(),
        })],
        ..Default::default()
    }
}

/// Vat Emergence — {4}{B} Sorcery. Put target creature card from a graveyard
/// onto the battlefield under your control. Proliferate.
pub fn vat_emergence() -> CardDefinition {
    CardDefinition {
        name: "Vat Emergence",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::Proliferate,
        ]),
        ..Default::default()
    }
}

/// Shardmage's Rescue — {W} Aura with flash. Enchant a creature you control;
/// it gets +1/+1 and (approximated as ongoing) hexproof. The printed
/// "hexproof only while this entered this turn" window is granted for the
/// duration instead — the protect-the-target intent is preserved.
pub fn shardmages_rescue() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Shardmage's Rescue",
        cost: cost(&[crate::mana::w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature.and(R::ControlledByYou),
            },
        },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Hexproof],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Trail of Crumbs — {1}{G} Enchantment. ETB create a Food. Whenever you
/// sacrifice a Food, you may pay {1} to look at the top two cards and put a
/// permanent card from among them into your hand (rest to the bottom).
pub fn trail_of_crumbs() -> CardDefinition {
    use crate::card::ArtifactSubtype;
    use crabomination_base::tokens::food_token;
    CardDefinition {
        name: "Trail of Crumbs",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: food_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Food),
                    }),
                effect: Effect::MayPay {
                    description: "Pay {1} to dig two for a permanent?".to_string(),
                    mana_cost: cost(&[generic(1)]),
                    body: Box::new(Effect::LookPickToHand {
                        who: PlayerRef::You,
                        count: Value::Const(2),
                        rest_to_graveyard: false,
                        pick_filter: Some(R::PermanentCard),
                        take: None,
                        to_battlefield: false,
                        gain_life_if_pick: None,
                        gain_life_greatest_power_rest: false,
                        optional: false,
                        picked_lands_to_battlefield: false,
                        rest_bottom_random: false,
                        rest_to_exile: false,
                    }),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Macabre Reconstruction — {3}{B} Sorcery. Costs {2} less if a creature died
/// this turn (the printed "a creature card was put into your graveyard from
/// anywhere" is approximated to the died case). Return up to two target
/// creature cards from your graveyard to your hand.
pub fn macabre_reconstruction() -> CardDefinition {
    CardDefinition {
        name: "Macabre Reconstruction",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {2} less to cast if a creature died this turn.",
            effect: StaticEffect::SelfCostReducedIfCreatureDiedThisTurn { amount: 2 },
        }],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
        ..Default::default()
    }
}
