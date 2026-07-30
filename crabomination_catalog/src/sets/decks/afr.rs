//! Adventures in the Forgotten Realms — venture-into-the-dungeon cards
//! (CR 309 / 701.49, `base::dungeons`). Tests in `tests/afr.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w};

/// "You've completed a dungeon" (CR 701.49d).
fn completed_a_dungeon() -> Predicate {
    Predicate::ValueAtLeast(Value::DungeonsCompleted, Value::Const(1))
}

/// Shortcut Seeker — {3}{U} Human Rogue 2/5. Combat damage to a player:
/// venture into the dungeon.
pub fn shortcut_seeker() -> CardDefinition {
    CardDefinition {
        name: "Shortcut Seeker",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Venture,
        }],
        ..Default::default()
    }
}

/// Cloister Gargoyle — {2}{W} Artifact Creature — Gargoyle 0/4. ETB: venture.
/// While you've completed a dungeon it gets +3/+0 and has flying.
pub fn cloister_gargoyle() -> CardDefinition {
    CardDefinition {
        name: "Cloister Gargoyle",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Gargoyle],
            ..Default::default()
        },
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Venture)],
        static_abilities: vec![StaticAbility {
            description: "As long as you've completed a dungeon, this creature gets +3/+0 and has flying.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtLeast(Value::DungeonsCompleted, Value::Const(1)),
                power: 3,
                toughness: 0,
                keywords: vec![Keyword::Flying],
            },
        }],
        ..Default::default()
    }
}

/// Dungeon Crawler — {B} Zombie 2/1. Enters tapped. Whenever you complete a
/// dungeon, you may return this card from your graveyard to your hand.
pub fn dungeon_crawler() -> CardDefinition {
    CardDefinition {
        name: "Dungeon Crawler",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "This creature enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DungeonCompleted, EventScope::FromYourGraveyard),
            effect: Effect::MayDo {
                description: "Return Dungeon Crawler from your graveyard to your hand?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Nadaar, Selfless Paladin — {2}{W} Legendary Dragon Knight 3/3, vigilance.
/// Enters or attacks: venture. Other creatures you control get +1/+1 as long
/// as you've completed a dungeon.
pub fn nadaar_selfless_paladin() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Nadaar, Selfless Paladin",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            etb(Effect::Venture),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Venture,
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+1 as long as you've completed a dungeon.",
            effect: StaticEffect::PumpTeamIf {
                condition: completed_a_dungeon(),
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Gloom Stalker — {2}{W} Dwarf Ranger 2/3. Double strike as long as you've
/// completed a dungeon.
pub fn gloom_stalker() -> CardDefinition {
    CardDefinition {
        name: "Gloom Stalker",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Ranger],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "As long as you've completed a dungeon, this creature has double strike.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::DoubleStrike,
                condition: completed_a_dungeon(),
            },
        }],
        ..Default::default()
    }
}

/// Dungeon Map — {3} Artifact. {T}: Add {C}. {3}, {T}: Venture (sorcery only).
pub fn dungeon_map() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Dungeon Map",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                sorcery_speed: true,
                effect: Effect::Venture,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Dungeon Descent — Land. Enters tapped. {T}: Add {C}. {4}, {T}, tap an
/// untapped legendary creature you control: Venture (sorcery only).
pub fn dungeon_descent() -> CardDefinition {
    use crate::card::{ActivatedAbility, Supertype};
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Dungeon Descent",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(4)]),
                sorcery_speed: true,
                tap_other_filter: Some(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasSupertype(Supertype::Legendary)),
                ),
                effect: Effect::Venture,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Triumphant Adventurer — {W}{B} Human Knight 1/1, deathtouch. First strike
/// during your turn; attacks: venture.
pub fn triumphant_adventurer() -> CardDefinition {
    CardDefinition {
        name: "Triumphant Adventurer",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has first strike.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::FirstStrike,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Venture,
        }],
        ..Default::default()
    }
}

/// Yuan-Ti Malison — {1}{U} Snake Rogue 2/1. Can't be blocked while attacking
/// alone; combat damage to a player: venture.
pub fn yuan_ti_malison() -> CardDefinition {
    CardDefinition {
        name: "Yuan-Ti Malison",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "This creature can't be blocked as long as it's attacking alone.",
            effect: StaticEffect::SelfHasKeywordWhile {
                keyword: Keyword::Unblockable,
                condition: R::IsAttackingAlone,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Venture,
        }],
        ..Default::default()
    }
}

/// Precipitous Drop — {2}{B} Aura. ETB: venture. Enchanted creature gets
/// -2/-2, or -5/-5 as long as you've completed a dungeon.
pub fn precipitous_drop() -> CardDefinition {
    use crate::card::{ConditionalEquipBonus, EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Precipitous Drop",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature,
            },
        },
        triggered_abilities: vec![etb(Effect::Venture)],
        equipped_bonus: Some(EquipBonus {
            power: -2,
            toughness: -2,
            conditional: vec![ConditionalEquipBonus {
                host_filter: R::Creature,
                power: -3,
                toughness: -3,
                keywords: vec![],
                condition: Some(completed_a_dungeon()),
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Bar the Gate — {2}{U} Instant. Counter target creature or planeswalker
/// spell. Venture into the dungeon.
pub fn bar_the_gate() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Bar the Gate",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(
                    R::HasCardType(CardType::Creature).or(R::HasCardType(CardType::Planeswalker)),
                )),
            },
            Effect::Venture,
        ]),
        ..Default::default()
    }
}

/// Radiant Solar — {5}{W} Angel 3/6, flying, lifelink. This or another
/// nontoken creature you control enters: venture. {W}, Discard this card:
/// venture and gain 3 life.
pub fn radiant_solar() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Radiant Solar",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 3,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::NotToken),
                }),
            effect: Effect::Venture,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::Venture,
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fates' Reversal — {1}{B} Sorcery. Return up to one target creature card
/// from your graveyard to your hand; venture.
pub fn fates_reversal() -> CardDefinition {
    CardDefinition {
        name: "Fates' Reversal",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::InYourGraveyard),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
            Effect::Venture,
        ]),
        ..Default::default()
    }
}

/// Secret Door — {U} Artifact Creature — Wall 0/4, defender. {4}{U}: Venture
/// (sorcery only).
pub fn secret_door() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Secret Door",
        cost: cost(&[u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), u()]),
            sorcery_speed: true,
            effect: Effect::Venture,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Delver's Torch — {1}{W} Equipment. Equipped creature gets +1/+1; whenever
/// it attacks, venture. Equip {3}.
pub fn delvers_torch() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Delver's Torch",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Venture,
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Ranger's Hawk — {W} Bird 1/1, flying. {3}, {T}, tap another untapped
/// creature you control: Venture (sorcery only).
pub fn rangers_hawk() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Ranger's Hawk",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3)]),
            sorcery_speed: true,
            tap_other_filter: Some(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            effect: Effect::Venture,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fifty Feet of Rope — {1} Artifact. {T}: Target Wall can't block this turn.
/// {3}, {T}: Target creature skips its next untap. {4}, {T}: Venture (sorcery).
pub fn fifty_feet_of_rope() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Fifty Feet of Rope",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Wall))),
                    keyword: Keyword::CantBlock,
                    duration: crate::effect::Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                effect: Effect::SkipNextUntap {
                    what: target_filtered(R::Creature),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(4)]),
                sorcery_speed: true,
                effect: Effect::Venture,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Shessra, Death's Whisper — {2}{B}{G} Legendary Human Elf Warlock 1/3.
/// ETB: target creature blocks this turn if able. Your end step, if a
/// creature died this turn: you may pay 2 life to draw a card.
pub fn shessra_deaths_whisper() -> CardDefinition {
    use crate::card::Supertype;
    use crate::effect::shortcut::target_filtered;
    use crate::game::TurnStep;
    CardDefinition {
        name: "Shessra, Death's Whisper",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Elf,
                CreatureType::Warlock,
            ],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::MustBlockSource {
                what: target_filtered(R::Creature),
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::YourControl,
                )
                .with_filter(Predicate::ValueAtLeast(
                    Value::CreaturesDiedThisTurnTotal,
                    Value::Const(1),
                )),
                effect: Effect::MayPayLife {
                    description: "Pay 2 life to draw a card?".into(),
                    amount: Value::Const(2),
                    body: Box::new(Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    }),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Underdark Basilisk — {1}{G} Basilisk 1/2, deathtouch.
pub fn underdark_basilisk() -> CardDefinition {
    CardDefinition {
        name: "Underdark Basilisk",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Basilisk],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Devoted Paladin — {4}{W} Orc Knight 4/4. ETB: creatures you control get
/// +1/+1 and gain vigilance until end of turn.
pub fn devoted_paladin() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Devoted Paladin",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Ellywick Tumblestrum — {2}{G}{G} Legendary Planeswalker, loyalty 4.
/// +1: venture. −2: dig 6 for a creature to hand (legendary lifegain rider
/// omitted). −7: emblem — your creatures have trample, haste, and +2/+2
/// while you've completed a dungeon (per-dungeon scaling approximated).
pub fn ellywick_tumblestrum() -> CardDefinition {
    use crate::card::Supertype;
    use crate::effect::LoyaltyAbility;
    CardDefinition {
        name: "Ellywick Tumblestrum",
        cost: cost(&[generic(2), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Venture,
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::LookPickToHand {
                    who: PlayerRef::You,
                    count: Value::Const(6),
                    rest_to_graveyard: false,
                    pick_filter: Some(R::Creature),
                    take: None,
                    to_battlefield: false,
                    gain_life_if_pick: None,
                    gain_life_greatest_power_rest: false,
                    optional: false,
                    picked_lands_to_battlefield: false,
                    rest_bottom_random: false,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Ellywick Tumblestrum".into(),
                    triggered: vec![],
                    statics: vec![StaticAbility {
                        description: "Creatures you control have trample and haste and get +2/+2 for each dungeon you've completed.",
                        effect: StaticEffect::PumpTeamIf {
                            condition: completed_a_dungeon(),
                            applies_to: Selector::EachPermanent(
                                R::Creature.and(R::ControlledByYou),
                            ),
                            power: 2,
                            toughness: 2,
                            keywords: vec![Keyword::Trample, Keyword::Haste],
                        },
                    }],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Priest of Ancient Lore — {2}{W} Dwarf Cleric 2/1. ETB: gain 1 life, draw.
pub fn priest_of_ancient_lore() -> CardDefinition {
    CardDefinition {
        name: "Priest of Ancient Lore",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Circle of Dreams Druid — {G}{G}{G} Elf Druid 2/1. {T}: Add {G} for each
/// creature you control.
pub fn circle_of_dreams_druid() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ManaPayload;
    use crate::mana::Color;
    CardDefinition {
        name: "Circle of Dreams Druid",
        cost: cost(&[g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    Color::Green,
                    Value::CountOf(Box::new(Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou),
                    ))),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Manticore — {3}{B} Manticore 2/1, flash, flying. ETB: destroy target
/// opponent creature that was dealt damage this turn.
pub fn manticore() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Manticore",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Manticore],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(
                R::Creature
                    .and(R::ControlledByOpponent)
                    .and(R::DealtDamageThisTurn),
            ),
        })],
        ..Default::default()
    }
}

/// Plundering Barbarian — {2}{R} Dwarf Barbarian 2/2. ETB, choose one:
/// destroy target artifact, or create a Treasure.
pub fn plundering_barbarian() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Plundering Barbarian",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Barbarian],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(R::Artifact),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::treasure_token(),
            },
        ]))],
        ..Default::default()
    }
}

/// Half-Elf Monk — {3}{W} Human Elf Monk 1/4, vigilance. {1}{W}, {T}: Tap
/// target creature.
pub fn half_elf_monk() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Half-Elf Monk",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Elf, CreatureType::Monk],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::Tap {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dwarfhold Champion — {1}{W} Dwarf Warrior 3/1. +0/+2 while equipped.
pub fn dwarfhold_champion() -> CardDefinition {
    CardDefinition {
        name: "Dwarfhold Champion",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is equipped, it gets +0/+2.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::IsEquipped,
                },
                power: 0,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}
