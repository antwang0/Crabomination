//! The Lord of the Rings: Tales of Middle-earth (LTR) staples, anchored on
//! **The Ring tempts you** (CR 701.54 — `Effect::RingTempts`, per-player
//! temptation level + designated Ring-bearer). Each card has a functionality
//! test in `crabomination/src/tests/ltr.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, Predicate, Selector, SelectionRequirement, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, magecraft, on_attack, on_dies, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Birthday Escape — {U} Sorcery. Draw a card. The Ring tempts you.
pub fn birthday_escape() -> CardDefinition {
    CardDefinition {
        name: "Birthday Escape",
        cost: cost(&[u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::RingTempts { who: PlayerRef::You },
        ]),
        ..Default::default()
    }
}

/// The Black Breath — {2}{B} Sorcery. Creatures your opponents control get
/// -1/-1 until end of turn. The Ring tempts you.
pub fn the_black_breath() -> CardDefinition {
    CardDefinition {
        name: "The Black Breath",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                }),
            },
            Effect::RingTempts { who: PlayerRef::You },
        ]),
        ..Default::default()
    }
}

/// Rohirrim Lancer — {R} 1/1 Human Knight. Menace. When it dies, the Ring
/// tempts you.
pub fn rohirrim_lancer() -> CardDefinition {
    CardDefinition {
        name: "Rohirrim Lancer",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_dies(Effect::RingTempts { who: PlayerRef::You })],
        ..Default::default()
    }
}

/// Bilbo, Retired Burglar — {1}{U}{R} 1/3 Legendary Halfling Rogue. When Bilbo
/// enters or leaves the battlefield, the Ring tempts you. Whenever Bilbo deals
/// combat damage to a player, create a Treasure token.
pub fn bilbo_retired_burglar() -> CardDefinition {
    CardDefinition {
        name: "Bilbo, Retired Burglar",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Halfling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::RingTempts { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::RingTempts { who: PlayerRef::You },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::game::effects::treasure_token(),
                },
            },
        ],
        ..Default::default()
    }
}

/// Call of the Ring — {1}{B} Enchantment. At the beginning of your upkeep, the
/// Ring tempts you. Whenever you choose a creature as your Ring-bearer, you may
/// pay 2 life. If you do, draw a card.
pub fn call_of_the_ring() -> CardDefinition {
    CardDefinition {
        name: "Call of the Ring",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::RingTempts { who: PlayerRef::You },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::RingTempted, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Creature,
                    }),
                effect: Effect::MayDo {
                    description: "Call of the Ring: pay 2 life to draw a card?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
                        Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    ])),
                },
            },
        ],
        ..Default::default()
    }
}

/// Easterling Vanguard — {1}{B} 2/1 Human Warrior. When it dies, amass Orcs 1.
pub fn easterling_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Easterling Vanguard",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::Amass {
            who: PlayerRef::You,
            count: Value::Const(1),
            extra_type: Some(CreatureType::Orc),
        })],
        ..Default::default()
    }
}

/// Mirkwood Bats — {3}{B} 2/3 Bat. Flying. Whenever you create or sacrifice a
/// token, each opponent loses 1 life.
pub fn mirkwood_bats() -> CardDefinition {
    let drain = Effect::LoseLife {
        who: Selector::Player(PlayerRef::EachOpponent),
        amount: Value::Const(1),
    };
    CardDefinition {
        name: "Mirkwood Bats",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bat], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::TokenCreated, EventScope::YourControl),
                effect: drain.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::IsToken,
                    }),
                effect: drain,
            },
        ],
        ..Default::default()
    }
}

/// Battle-Scarred Goblin — {1}{R} 2/2 Goblin Warrior. Whenever it becomes
/// blocked, it deals 1 damage to each creature blocking it.
pub fn battle_scarred_goblin() -> CardDefinition {
    CardDefinition {
        name: "Battle-Scarred Goblin",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::BlockingCreatures,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Lothlórien Lookout — {1}{G} 1/3 Elf Scout. Whenever it attacks, scry 1.
pub fn lothlorien_lookout() -> CardDefinition {
    CardDefinition {
        name: "Lothlórien Lookout",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Banish from Edoras — {4}{W} Sorcery. Costs {2} less to cast if it targets a
/// tapped creature. Exile target creature.
pub fn banish_from_edoras() -> CardDefinition {
    CardDefinition {
        name: "Banish from Edoras",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Sorcery],
        self_cost_reduction_if_target: Some((SelectionRequirement::Tapped, 2)),
        effect: Effect::Exile { what: target_filtered(SelectionRequirement::Creature) },
        ..Default::default()
    }
}

/// Wizard's Rockets — {1} Artifact. Enters tapped. {X}, {T}, Sacrifice: add X
/// mana in any combination of colors. When it's put into a graveyard from the
/// battlefield, draw a card. (The X-mana ability is approximated as a single
/// any-color mana to keep within the activated-mana-ability primitive.)
pub fn wizards_rockets() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Wizard's Rockets",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Wizard's Rockets enters the battlefield tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Took Reaper — {1}{W} 2/1 Halfling Peasant. When it dies, the Ring tempts you.
pub fn took_reaper() -> CardDefinition {
    CardDefinition {
        name: "Took Reaper",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Halfling, CreatureType::Peasant],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::RingTempts { who: PlayerRef::You })],
        ..Default::default()
    }
}

/// Erebor Flamesmith — {1}{R} 2/1 Dwarf Artificer. Whenever you cast an instant
/// or sorcery spell, it deals 1 damage to each opponent.
pub fn erebor_flamesmith() -> CardDefinition {
    CardDefinition {
        name: "Erebor Flamesmith",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Prince Imrahil the Fair — {W}{U} 2/2 Legendary Human Noble. Whenever you
/// draw your second card each turn, create a 1/1 white Human Soldier.
pub fn prince_imrahil_the_fair() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let soldier = TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Prince Imrahil the Fair",
        cost: cost(&[w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::Triggerer, n: 2 })
                .once_per_turn(),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: soldier },
        }],
        ..Default::default()
    }
}

/// Slip On the Ring — {1}{W} Instant. Exile target creature you own, return it
/// under your control, then the Ring tempts you.
pub fn slip_on_the_ring() -> CardDefinition {
    CardDefinition {
        name: "Slip On the Ring",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
            },
            Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::RingTempts { who: PlayerRef::You },
        ]),
        ..Default::default()
    }
}

/// Rally at the Hornburg — {1}{R} Sorcery. Create two 1/1 white Human Soldier
/// tokens. Humans you control gain haste until end of turn.
pub fn rally_at_the_hornburg() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let soldier = TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Rally at the Hornburg",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: soldier },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Human)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Haradrim Spearmaster — {2}{R} 2/3 Human Warrior. Reach. At the beginning of
/// combat on your turn, another target creature you control gets +1/+0.
pub fn haradrim_spearmaster() -> CardDefinition {
    CardDefinition {
        name: "Haradrim Spearmaster",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Fog on the Barrow-Downs — {2}{W} Aura. Enchanted creature can't attack or
/// block. (Modeled as the can't-attack/can't-block grant; the "becomes a
/// Spirit, loses other types" rider is cosmetic.)
pub fn fog_on_the_barrow_downs() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Fog on the Barrow-Downs",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::Creature },
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Snarling Warg — {3}{B} 3/4 Wolf. Menace. Gets +1/+0 as long as you control
/// a Goblin or Orc.
pub fn snarling_warg() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Snarling Warg",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "As long as you control a Goblin or Orc, Snarling Warg gets +1/+0.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::ControlledByYou.and(
                            SelectionRequirement::HasCreatureType(CreatureType::Goblin)
                                .or(SelectionRequirement::HasCreatureType(CreatureType::Orc)),
                        ),
                    ),
                    n: Value::Const(1),
                },
                power: 1,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Wose Pathfinder — {1}{G} 1/1 Human Shaman. {T}: add one mana of any color.
/// {6}{G}, {T}: another target creature gets +3/+3 and gains trample until end
/// of turn.
pub fn wose_pathfinder() -> CardDefinition {
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Wose Pathfinder",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(6), g()]),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::OtherThanSource),
                        ),
                        power: Value::Const(3),
                        toughness: Value::Const(3),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Soldier of the Grey Host — {3}{W} 2/2 Spirit Soldier. Flash, flying. When it
/// enters, target creature gets +2/+0 until end of turn.
pub fn soldier_of_the_grey_host() -> CardDefinition {
    CardDefinition {
        name: "Soldier of the Grey Host",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Westfold Rider — {1}{W} 3/1 Human Knight. Sacrifice this creature: destroy
/// target artifact or enchantment. Activate only as a sorcery.
pub fn westfold_rider() -> CardDefinition {
    CardDefinition {
        name: "Westfold Rider",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bombadil's Song — {1}{G} Instant. Target creature you control gets +1/+1 and
/// gains hexproof until end of turn. The Ring tempts you.
pub fn bombadils_song() -> CardDefinition {
    CardDefinition {
        name: "Bombadil's Song",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::RingTempts { who: PlayerRef::You },
        ]),
        ..Default::default()
    }
}

/// Mordor Muster — {1}{B} Sorcery. You draw a card and lose 1 life. Amass Orcs 1.
pub fn mordor_muster() -> CardDefinition {
    CardDefinition {
        name: "Mordor Muster",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
            Effect::Amass { who: PlayerRef::You, count: Value::Const(1), extra_type: Some(CreatureType::Orc) },
        ]),
        ..Default::default()
    }
}

/// Bag End Porter — {3}{G} 4/4 Dwarf. Whenever it attacks, it gets +X/+X until
/// end of turn, where X is the number of legendary creatures you control.
pub fn bag_end_porter() -> CardDefinition {
    let x = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(SelectionRequirement::Any)),
        filter: SelectionRequirement::Creature
            .and(SelectionRequirement::HasSupertype(Supertype::Legendary))
            .and(SelectionRequirement::ControlledByYou),
    };
    CardDefinition {
        name: "Bag End Porter",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: x.clone(),
            toughness: x,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Hithlain Knots — {1}{U} Instant. Tap target creature. Scry 1. Draw a card.
pub fn hithlain_knots() -> CardDefinition {
    CardDefinition {
        name: "Hithlain Knots",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Lossarnach Captain — {3}{W} 3/1 Human Soldier. First strike. Whenever this
/// or another Human you control enters, tap target creature an opponent
/// controls. At the beginning of your upkeep, create a 1/1 white Human Soldier.
pub fn lossarnach_captain() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let soldier = TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Lossarnach Captain",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Human),
                    }),
                effect: Effect::Tap {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: soldier },
            },
        ],
        ..Default::default()
    }
}

/// Dúnedain Blade — {1}{W} Equipment. Equipped creature gets +2/+1. Equip {3}.
/// (The reduced "Equip Human {1}" alternative is dropped — minor.)
pub fn dunedain_blade() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Dúnedain Blade",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 1, ..Default::default() }),
        ..Default::default()
    }
}

/// Erkenbrand, Lord of Westfold — {3}{R} 3/3 Human Soldier. Whenever Erkenbrand
/// or another Human you control enters, creatures you control get +1/+0 EOT.
pub fn erkenbrand_lord_of_westfold() -> CardDefinition {
    CardDefinition {
        name: "Erkenbrand, Lord of Westfold",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Human),
                }),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Many Partings — {G} Sorcery. Search your library for a basic land card, put
/// it into your hand, then shuffle. Create a Food token.
pub fn many_partings() -> CardDefinition {
    CardDefinition {
        name: "Many Partings",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::food_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Goblin Fireleaper — {1}{R} 1/1 Goblin Warrior. {1}{R}: gets +1/+0 until end
/// of turn. When it dies, it deals damage equal to its power to target creature
/// an opponent controls.
pub fn goblin_fireleaper() -> CardDefinition {
    CardDefinition {
        name: "Goblin Fireleaper",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![on_dies(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            amount: Value::PowerOf(Box::new(Selector::This)),
        })],
        ..Default::default()
    }
}

/// Bitter Downfall — {3}{B} Instant. Costs {3} less if it targets a creature
/// dealt damage this turn. Destroy target creature. Its controller loses 2 life.
pub fn bitter_downfall() -> CardDefinition {
    CardDefinition {
        name: "Bitter Downfall",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((SelectionRequirement::DealtDamageThisTurn, 3)),
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(SelectionRequirement::Creature) },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Uruk-hai Berserker — {2}{B} 3/2 Orc Berserker. When it enters, the Ring
/// tempts you.
pub fn uruk_hai_berserker() -> CardDefinition {
    CardDefinition {
        name: "Uruk-hai Berserker",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::RingTempts { who: PlayerRef::You })],
        ..Default::default()
    }
}

/// Ranger's Firebrand — {R} Sorcery. Deals 2 damage to any target. The Ring
/// tempts you.
pub fn rangers_firebrand() -> CardDefinition {
    CardDefinition {
        name: "Ranger's Firebrand",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(SelectionRequirement::Any), amount: Value::Const(2) },
            Effect::RingTempts { who: PlayerRef::You },
        ]),
        ..Default::default()
    }
}





/// Andúril, Flame of the West — {3} Legendary Equipment. Equipped creature gets
/// +3/+1. Whenever it attacks, create two tapped 1/1 white Spirit tokens with
/// flying. (The legendary-equipped vigilance upgrade is dropped — minor.)
pub fn anduril_flame_of_the_west() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus, TokenDefinition};
    use crate::mana::Color;
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        keywords: vec![Keyword::Flying],
        tapped: true,
        ..Default::default()
    };
    CardDefinition {
        name: "Andúril, Flame of the West",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: spirit },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}
