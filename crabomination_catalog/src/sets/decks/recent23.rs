//! A twenty-third wave. The headline mechanic is
//! `Keyword::AssignsCombatDamageByToughness` (CR 510.1c — "assigns combat
//! damage equal to its toughness rather than its power"): Doran, the Siege
//! Tower (all creatures), Tapestry Warden / Ancient Lumberknot (your creatures
//! with toughness > power), Bill the Pony (a sacrifice-a-Food temporary grant).
//! Plus Affinity for a type (Thrumming Hivepool — Slivers; Gearseeker Serpent
//! — artifacts; Chitin Gravestalker — graveyard) and ~20 DSK/DFT/EOE staples on
//! existing primitives (cycling, devoid burn, modal counter/manifest, threaten,
//! manifest-dread bounce, cycle triggers, …). Tests in
//! `crabomination/src/tests/recent23.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, LandType, SelectionRequirement, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{deal, etb, on_dies, target_any, target_filtered};
use crate::effect::{Duration, PlayerRef};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Doran, the Siege Tower — {W}{B}{G} 0/5 legendary Treefolk Shaman. Each
/// creature assigns combat damage equal to its toughness rather than its power.
pub fn doran_the_siege_tower() -> CardDefinition {
    CardDefinition {
        name: "Doran, the Siege Tower",
        cost: cost(&[w(), b(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk, CreatureType::Shaman],
            ..Default::default()
        },
        power: 0,
        toughness: 5,
        static_abilities: vec![StaticAbility {
            description: "Each creature assigns combat damage equal to its toughness",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(SelectionRequirement::Creature),
                keyword: Keyword::AssignsCombatDamageByToughness,
            },
        }],
        ..Default::default()
    }
}

/// Tapestry Warden — {3}{G} 3/4 artifact Robot Soldier with vigilance. Each
/// creature you control with toughness greater than its power assigns combat
/// damage equal to its toughness rather than its power. (The "stations using
/// toughness" half is dropped.)
pub fn tapestry_warden() -> CardDefinition {
    CardDefinition {
        name: "Tapestry Warden",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Your creatures with toughness > power assign damage by toughness",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::ToughnessGreaterThanPower),
                ),
                keyword: Keyword::AssignsCombatDamageByToughness,
            },
        }],
        ..Default::default()
    }
}

/// Ancient Lumberknot — {2}{B}{G} 1/4 Treefolk. Each creature you control with
/// toughness greater than its power assigns combat damage equal to its
/// toughness rather than its power.
pub fn ancient_lumberknot() -> CardDefinition {
    CardDefinition {
        name: "Ancient Lumberknot",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Your creatures with toughness > power assign damage by toughness",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::ToughnessGreaterThanPower),
                ),
                keyword: Keyword::AssignsCombatDamageByToughness,
            },
        }],
        ..Default::default()
    }
}

/// Thrumming Hivepool — {6} Artifact with Affinity for Slivers. Slivers you
/// control have double strike and haste. At the beginning of your upkeep,
/// create two 1/1 colorless Sliver creature tokens.
pub fn thrumming_hivepool() -> CardDefinition {
    let sliver_lord = |keyword| StaticAbility {
        description: "Slivers you control have double strike and haste",
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::EachPermanent(
                SelectionRequirement::ControlledByYou
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Sliver)),
            ),
            keyword,
        },
    };
    CardDefinition {
        name: "Thrumming Hivepool",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        affinity_filter: Some(
            SelectionRequirement::ControlledByYou
                .and(SelectionRequirement::HasCreatureType(CreatureType::Sliver)),
        ),
        static_abilities: vec![
            sliver_lord(Keyword::DoubleStrike),
            sliver_lord(Keyword::Haste),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: Box::new(TokenDefinition {
                    name: "Sliver".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Sliver],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            },
        }],
        ..Default::default()
    }
}

/// Bill the Pony — {3}{W} 1/4 legendary Horse. ETB: create two Food. Sacrifice
/// a Food: until end of turn, target creature you control assigns combat damage
/// equal to its toughness rather than its power.
pub fn bill_the_pony() -> CardDefinition {
    let food = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(2),
        definition: Box::new(crabomination_base::tokens::food_token()),
    };
    CardDefinition {
        name: "Bill the Pony",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horse],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![etb(food())],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((
                SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Food),
                1,
            )),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::AssignsCombatDamageByToughness,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bedhead Beastie — {4}{R}{R} 5/6 Beast with menace and Mountaincycling {2}.
pub fn bedhead_beastie() -> CardDefinition {
    CardDefinition {
        name: "Bedhead Beastie",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        keywords: vec![
            Keyword::Menace,
            Keyword::Typecycling(Box::new((
                cost(&[generic(2)]),
                SelectionRequirement::HasLandType(LandType::Mountain),
            ))),
        ],
        ..Default::default()
    }
}

/// Daggermaw Megalodon — {4}{U}{U} 5/7 Shark with vigilance and Islandcycling {2}.
pub fn daggermaw_megalodon() -> CardDefinition {
    CardDefinition {
        name: "Daggermaw Megalodon",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shark],
            ..Default::default()
        },
        power: 5,
        toughness: 7,
        keywords: vec![
            Keyword::Vigilance,
            Keyword::Typecycling(Box::new((
                cost(&[generic(2)]),
                SelectionRequirement::HasLandType(LandType::Island),
            ))),
        ],
        ..Default::default()
    }
}

/// Boilerbilges Ripper — {4}{R} 4/4 Human Assassin. When it enters, you may
/// sacrifice another creature or enchantment; if you do, it deals 2 damage to
/// any target.
pub fn boilerbilges_ripper() -> CardDefinition {
    CardDefinition {
        name: "Boilerbilges Ripper",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice another creature or enchantment? (deal 2 to any target)".into(),
            filter: SelectionRequirement::Creature
                .or(SelectionRequirement::Enchantment)
                .and(SelectionRequirement::OtherThanSource),
            count: Value::Const(1),
            then: Box::new(deal(2, target_any())),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Bashful Beastie — {4}{G} 5/4 Beast. When it dies, manifest dread.
pub fn bashful_beastie() -> CardDefinition {
    CardDefinition {
        name: "Bashful Beastie",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![on_dies(Effect::ManifestDread {
            who: PlayerRef::You,
        })],
        ..Default::default()
    }
}

/// Bear Trap — {1} Artifact with flash. {3}, {T}, Sacrifice this: it deals 3
/// damage to target creature.
pub fn bear_trap() -> CardDefinition {
    CardDefinition {
        name: "Bear Trap",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Flash],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            sac_cost: true,
            effect: deal(3, target_filtered(SelectionRequirement::Creature)),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Frantic Strength — {2}{G} Aura with flash. Enchant creature. Enchanted
/// creature gets +2/+2 and has trample.
pub fn frantic_strength() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Frantic Strength",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Trample],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Most Valuable Slayer — {3}{R} 2/4 Human Warrior. Whenever you attack, target
/// attacking creature gets +1/+0 and gains first strike until end of turn.
pub fn most_valuable_slayer() -> CardDefinition {
    CardDefinition {
        name: "Most Valuable Slayer",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::IsAttacking),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Twist Reality — {1}{U}{U} Instant. Choose one — counter target spell; or
/// manifest dread.
pub fn twist_reality() -> CardDefinition {
    CardDefinition {
        name: "Twist Reality",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
            },
            Effect::ManifestDread {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}

/// Vengeful Possession — {2}{R} Sorcery. Gain control of target creature until
/// end of turn, untap it, it gains haste. Then you may discard a card; if you
/// do, draw a card.
pub fn vengeful_possession() -> CardDefinition {
    CardDefinition {
        name: "Vengeful Possession",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(SelectionRequirement::Creature),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::MayDo {
                description: "Discard a card to draw a card?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                ])),
            },
        ]),
        ..Default::default()
    }
}

/// Unstoppable Plan — {2}{U} Enchantment. At the beginning of your end step,
/// untap all nonland permanents you control.
pub fn unstoppable_plan() -> CardDefinition {
    CardDefinition {
        name: "Unstoppable Plan",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Untap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Nonland.and(SelectionRequirement::ControlledByYou),
                ),
                up_to: None,
            },
        }],
        ..Default::default()
    }
}

/// Gearseeker Serpent — {5}{U}{U} 5/6 Serpent with Affinity for artifacts.
/// {5}{U}: it can't be blocked this turn.
pub fn gearseeker_serpent() -> CardDefinition {
    CardDefinition {
        name: "Gearseeker Serpent",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Serpent],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        affinity_filter: Some(
            SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
        ),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), u()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Aetherjacket — {3} 2/1 artifact Thopter with flying and vigilance. {2}, {T},
/// Sacrifice this creature: destroy another target artifact (sorcery speed).
pub fn aetherjacket() -> CardDefinition {
    CardDefinition {
        name: "Aetherjacket",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.and(SelectionRequirement::OtherThanSource),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dynamite Diver — {R} 1/1 Goblin Pilot. When it dies, it deals 1 damage to
/// any target. (Its saddle/crew power bonus is dropped — no engine hook yet.)
pub fn dynamite_diver() -> CardDefinition {
    CardDefinition {
        name: "Dynamite Diver",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Pilot],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_dies(deal(1, target_any()))],
        ..Default::default()
    }
}

/// Gas Guzzler — {B} 2/1 Vampire Rogue with "Start your engines!". Enters
/// tapped. Max speed — {B}, Sacrifice another creature: draw a card.
pub fn gas_guzzler() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Gas Guzzler",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::StartYourEngines],
        static_abilities: vec![StaticAbility {
            description: "This creature enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_other_filter: Some((
                SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                1,
            )),
            condition: Some(Predicate::SpeedAtLeast {
                who: PlayerRef::You,
                speed: 4,
            }),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Chitin Gravestalker — {5}{B} 5/4 Insect Warrior. This spell costs {1} less to
/// cast for each artifact and/or creature card in your graveyard. Cycling {2}.
pub fn chitin_gravestalker() -> CardDefinition {
    CardDefinition {
        name: "Chitin Gravestalker",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        affinity_graveyard_filter: Some(
            SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
        ),
        ..Default::default()
    }
}

/// Unnerving Grasp — {2}{U} Sorcery. Return up to one target nonland permanent
/// to its owner's hand. Manifest dread.
pub fn unnerving_grasp() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Unnerving Grasp",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: SelectionRequirement::Nonland,
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            },
            Effect::ManifestDread {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}

/// Fanged Flames — {1}{R} Sorcery with devoid. Deals 4 damage to target creature
/// or planeswalker. If it would die this turn, exile it instead.
pub fn fanged_flames() -> CardDefinition {
    CardDefinition {
        name: "Fanged Flames",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Devoid],
        // Install the exile replacement before the damage so the lethal-damage
        // SBA is caught (CR 614).
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            },
            deal(4, Selector::Target(0)),
        ]),
        ..Default::default()
    }
}

/// Splitskin Doll — {1}{W} 2/1 artifact Toy. When it enters, draw a card. Then
/// discard a card unless you control another creature with power 2 or less.
pub fn splitskin_doll() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Splitskin Doll",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Toy],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::PowerAtMost(2))
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    n: Value::Const(1),
                },
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                }),
            },
        ]))],
        ..Default::default()
    }
}

/// Skittering Surveyor — {3} 1/2 artifact Construct. When it enters, you may
/// search your library for a basic land card and put it into your hand.
pub fn skittering_surveyor() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Skittering Surveyor",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for a basic land?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..Default::default()
    }
}

/// Agonasaur Rex — {3}{G}{G} 8/8 Dinosaur with trample and Cycling {2}{G}. When
/// you cycle it, put two +1/+1 counters on up to one target creature; it gains
/// trample and indestructible until end of turn.
pub fn agonasaur_rex() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Agonasaur Rex",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 8,
        toughness: 8,
        keywords: vec![Keyword::Trample, Keyword::Cycling(cost(&[generic(2), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: SelectionRequirement::Creature,
                effect: Box::new(Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Indestructible,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Marketwatch Phantom — {1}{W} 2/2 Spirit Detective. Whenever another creature
/// you control with power 2 or less enters, this creature gains flying until
/// end of turn.
pub fn marketwatch_phantom() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Marketwatch Phantom",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Detective],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::PowerAtMost(2))
                        .and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}
