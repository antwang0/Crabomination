//! A twenty-eighth wave — Duskmourn: House of Horror (DSK) commons/uncommons
//! on existing primitives (dies/attack triggers, typecycling, manifest dread,
//! and the Eerie ability word). Tests in `crabomination/src/tests/recent28.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    LandType, Predicate, SelectionRequirement, Selector, Subtypes, Value,
};
use crate::effect::PlayerRef;
use crate::effect::shortcut::{on_attack, on_dies, target_filtered};
use crate::game::effects::treasure_token;
use crate::mana::{b, cost, g, generic, r, u, w};

/// DSK Eerie ability word: "Whenever an enchantment you control enters and
/// whenever you fully unlock a Room, `effect`." Two triggers sharing one body.
fn eerie(effect: Effect) -> Vec<crate::card::TriggeredAbility> {
    use crate::card::TriggeredAbility;
    vec![
        TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Enchantment,
                }),
            effect: effect.clone(),
        },
        TriggeredAbility {
            event: EventSpec::new(EventKind::RoomFullyUnlocked, EventScope::YourControl),
            effect,
        },
    ]
}

/// Piggy Bank — {1}{R} 3/2 Boar Toy artifact creature. When it dies, create a
/// Treasure token.
pub fn piggy_bank() -> CardDefinition {
    CardDefinition {
        name: "Piggy Bank",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Boar, CreatureType::Toy],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: treasure_token(),
        })],
        ..Default::default()
    }
}

/// Appendage Amalgam — {2}{B} 3/2 Horror enchantment creature with flash.
/// Whenever it attacks, surveil 1.
pub fn appendage_amalgam() -> CardDefinition {
    CardDefinition {
        name: "Appendage Amalgam",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horror],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![on_attack(Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Skullsnap Nuisance — {U}{B} 1/4 Insect Skeleton with flying. Eerie —
/// surveil 1.
pub fn skullsnap_nuisance() -> CardDefinition {
    CardDefinition {
        name: "Skullsnap Nuisance",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Skeleton],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: eerie(Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::ONE,
        }),
        ..Default::default()
    }
}

/// Shepherding Spirits — {4}{W}{W} 4/5 Spirit with flying. Plainscycling {2}.
pub fn shepherding_spirits() -> CardDefinition {
    CardDefinition {
        name: "Shepherding Spirits",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![
            Keyword::Flying,
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Plains),
        ],
        ..Default::default()
    }
}

/// Slavering Branchsnapper — {4}{G}{G} 7/6 Lizard with trample. Forestcycling {2}.
pub fn slavering_branchsnapper() -> CardDefinition {
    CardDefinition {
        name: "Slavering Branchsnapper",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard],
            ..Default::default()
        },
        power: 7,
        toughness: 6,
        keywords: vec![
            Keyword::Trample,
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Forest),
        ],
        ..Default::default()
    }
}

/// Seized from Slumber — {4}{W} instant. Destroy target creature; costs {3}
/// less to cast if it targets a tapped creature.
pub fn seized_from_slumber() -> CardDefinition {
    CardDefinition {
        name: "Seized from Slumber",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Creature),
        },
        self_cost_reduction_if_target: Some((SelectionRequirement::Tapped, 3)),
        ..Default::default()
    }
}

/// Manifest Dread — {1}{G} sorcery. Manifest dread (CR 701.41).
pub fn manifest_dread_spell() -> CardDefinition {
    CardDefinition {
        name: "Manifest Dread",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ManifestDread {
            who: PlayerRef::You,
        },
        ..Default::default()
    }
}

/// Impossible Inferno — {4}{R} instant. Deals 6 damage to target creature;
/// Delirium — if there are four+ card types in your graveyard, exile the top
/// card of your library and you may play it until the end of your next turn.
pub fn impossible_inferno() -> CardDefinition {
    CardDefinition {
        name: "Impossible Inferno",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(6),
            },
            Effect::If {
                cond: Predicate::DeliriumActive {
                    who: PlayerRef::You,
                },
                then: Box::new(Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    pay_own_cost: false,
                    uncast_penalty: None,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Break Down the Door — {2}{G} instant. Choose one: exile target artifact;
/// exile target enchantment; or manifest dread.
pub fn break_down_the_door() -> CardDefinition {
    CardDefinition {
        name: "Break Down the Door",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Exile {
                what: target_filtered(SelectionRequirement::Artifact),
            },
            Effect::Exile {
                what: target_filtered(SelectionRequirement::Enchantment),
            },
            Effect::ManifestDread {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}

/// Found Footage — {1} Clue artifact. {2}, Sacrifice this artifact: surveil 2,
/// then draw a card.
pub fn found_footage() -> CardDefinition {
    use crate::card::{ActivatedAbility, ArtifactSubtype};
    CardDefinition {
        name: "Found Footage",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Clue],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::Surveil {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
