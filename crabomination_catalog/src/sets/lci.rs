//! The Lost Caverns of Ixalan (LCI) — 2023. Introduces the Discover
//! (CR 701.57) keyword action.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement, Selector, Subtypes,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{craft, drain, etb, on_attack, on_dies, target_filtered};
use crate::effect::{Effect, PlayerRef, ZoneDest};
use crate::game::effects::{map_token, treasure_token};
use crate::mana::{b, cost, g, generic, r, u, w, x};
use crate::game::types::TurnStep;

/// Geological Appraiser — {2}{R}{R} 3/2 Human Artificer. "When this enters,
/// if you cast it, discover 3." (The "if you cast it" gate is approximated as
/// firing on any ETB — the engine doesn't tag cast-vs-put entries.)
pub fn geological_appraiser() -> CardDefinition {
    CardDefinition {
        name: "Geological Appraiser",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Discover { n: Value::Const(3), filter: None })],
        ..Default::default()
    }
}

/// Trumpeting Carnosaur — {4}{R}{R} 7/6 Dinosaur with trample. "When this
/// enters, discover 5." (The "{2}{R}, Discard this card: 3 damage" from-hand
/// ability is omitted — activated-from-hand abilities aren't modeled.)
pub fn trumpeting_carnosaur() -> CardDefinition {
    CardDefinition {
        name: "Trumpeting Carnosaur",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Discover { n: Value::Const(5), filter: None })],
        ..Default::default()
    }
}

/// Spyglass Siren — {U} 1/1 Siren Pirate with flying. "When this enters,
/// create a Map token." (Map tokens ship via `map_token()`.)
pub fn spyglass_siren() -> CardDefinition {
    CardDefinition {
        name: "Spyglass Siren",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Siren, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: map_token(),
        })],
        ..Default::default()
    }
}

/// Defossilize — {4}{B} Sorcery. "Return target creature card from your
/// graveyard to the battlefield. That creature explores, then it explores
/// again."
pub fn defossilize() -> CardDefinition {
    CardDefinition {
        name: "Defossilize",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::Explore { who: Selector::LastMoved },
            Effect::Explore { who: Selector::LastMoved },
        ]),
        ..Default::default()
    }
}

/// Goldvein Hydra — {X}{G} 0/0 Hydra with vigilance, trample, haste. Enters
/// with X +1/+1 counters. When it dies, create Treasures equal to its power
/// (its last-known counter-boosted power, via CR 603.10 leaves-battlefield LKI).
pub fn goldvein_hydra() -> CardDefinition {
    CardDefinition {
        name: "Goldvein Hydra",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hydra], ..Default::default() },
        keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Haste],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::PowerOf(Box::new(Selector::This)),
            definition: treasure_token(),
        })],
        ..Default::default()
    }
}

// ── Craft (CR 702.169) — LCI transforming artifacts ─────────────────────────
// The front face exiles itself and returns transformed via
// `Effect::ExileSelfReturnTransformed`; the "exile N other [type]" additional
// cost rides `craft_exile_cost` (graveyard cards first, then lowest-power
// battlefield permanents). All are sorcery-speed.

/// Tithing Blade // Consuming Sepulcher — {1}{B} Artifact; ETB each opponent
/// sacrifices a creature. Craft with creature {4}{B} → Consuming Sepulcher
/// (Artifact; your upkeep: drain 1).
pub fn tithing_blade() -> CardDefinition {
    let sepulcher = CardDefinition {
        name: "Consuming Sepulcher",
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: drain(1),
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Tithing Blade",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature,
        })],
        activated_abilities: vec![craft(cost(&[generic(4), b()]), SelectionRequirement::Creature, 1)],
        back_face: Some(Box::new(sepulcher)),
        ..Default::default()
    }
}

/// Visage of Dread // Dread Osseosaur — {1}{B} Artifact; ETB target opponent
/// reveals hand, you choose an artifact/creature card, they discard it. Craft
/// with two creatures {5}{B} → Dread Osseosaur (5/4 Menace; ETB/attack mill 2).
pub fn visage_of_dread() -> CardDefinition {
    let osseosaur = CardDefinition {
        name: "Dread Osseosaur",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur, CreatureType::Skeleton, CreatureType::Horror],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            etb(Effect::Mill { who: Selector::You, amount: Value::Const(2) }),
            on_attack(Effect::Mill { who: Selector::You, amount: Value::Const(2) }),
        ],
        ..Default::default()
    };
    CardDefinition {
        name: "Visage of Dread",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::DiscardChosen {
            from: target_filtered(SelectionRequirement::OpponentPlayer),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
        })],
        activated_abilities: vec![craft(cost(&[generic(5), b()]), SelectionRequirement::Creature, 2)],
        back_face: Some(Box::new(osseosaur)),
        ..Default::default()
    }
}

/// Spring-Loaded Sawblades // Bladewheel Chariot — {1}{W} Artifact, Flash; ETB
/// deal 5 to target tapped creature an opponent controls. Craft with artifact
/// {3}{W} → Bladewheel Chariot (Artifact Vehicle 5/5, Crew 1).
pub fn spring_loaded_sawblades() -> CardDefinition {
    let chariot = CardDefinition {
        name: "Bladewheel Chariot",
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Crew(1)],
        ..Default::default()
    };
    CardDefinition {
        name: "Spring-Loaded Sawblades",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::Tapped)
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
            amount: Value::Const(5),
        })],
        activated_abilities: vec![craft(cost(&[generic(3), w()]), SelectionRequirement::Artifact, 1)],
        back_face: Some(Box::new(chariot)),
        ..Default::default()
    }
}

/// Waterlogged Hulk // Watertight Gondola — {U} Artifact; {T}: mill a card.
/// Craft with Island {3}{U} → Watertight Gondola (Artifact Vehicle 4/4,
/// Vigilance, Crew 1; Descend 8 — unblockable while you have 8+ permanent
/// cards in your graveyard).
pub fn waterlogged_hulk() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    let gondola = CardDefinition {
        name: "Watertight Gondola",
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Crew(1)],
        static_abilities: vec![StaticAbility {
            description: "Descend 8 — can't be blocked while you have 8+ permanent cards in your graveyard.",
            effect: StaticEffect::SelfHasKeywordWhile {
                keyword: Keyword::Unblockable,
                condition: SelectionRequirement::ControllerDescend(8),
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Waterlogged Hulk",
        cost: cost(&[u()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Mill { who: Selector::You, amount: Value::Const(1) },
                ..Default::default()
            },
            craft(cost(&[generic(3), u()]), SelectionRequirement::HasLandType(LandType::Island), 1),
        ],
        back_face: Some(Box::new(gondola)),
        ..Default::default()
    }
}

/// Lodestone Needle // Guidestone Compass — {1}{U} Artifact, Flash; ETB tap a
/// target artifact/creature and put two stun counters on it. Craft with
/// artifact {2}{U} → Guidestone Compass ({1},{T}: target creature you control
/// explores; sorcery-speed).
pub fn lodestone_needle() -> CardDefinition {
    let compass = CardDefinition {
        name: "Guidestone Compass",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            sorcery_speed: true,
            effect: Effect::Explore {
                who: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Lodestone Needle",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature.or(SelectionRequirement::Artifact)),
            },
            Effect::AddCounter {
                what: crate::effect::shortcut::target(),
                kind: CounterType::Stun,
                amount: Value::Const(2),
            },
        ]))],
        activated_abilities: vec![craft(cost(&[generic(2), u()]), SelectionRequirement::Artifact, 1)],
        back_face: Some(Box::new(compass)),
        ..Default::default()
    }
}

/// Inverted Iceberg // Iceberg Titan — {1}{U} Artifact; ETB mill 1, then draw 1.
/// Craft with artifact {4}{U}{U} → Iceberg Titan (6/6 Golem). (The attack
/// tap/untap rider is omitted.)
pub fn inverted_iceberg() -> CardDefinition {
    let titan = CardDefinition {
        name: "Iceberg Titan",
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 6,
        toughness: 6,
        ..Default::default()
    };
    CardDefinition {
        name: "Inverted Iceberg",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(1) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]))],
        activated_abilities: vec![craft(cost(&[generic(4), u(), u()]), SelectionRequirement::Artifact, 1)],
        back_face: Some(Box::new(titan)),
        ..Default::default()
    }
}

/// Oteclan Landmark // Oteclan Levitator — {W} Artifact; ETB scry 2. Craft with
/// artifact {2}{W} → Oteclan Levitator (1/4 Golem with flying). (The
/// attack grant-flying rider is omitted.)
pub fn oteclan_landmark() -> CardDefinition {
    let levitator = CardDefinition {
        name: "Oteclan Levitator",
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Oteclan Landmark",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) })],
        activated_abilities: vec![craft(cost(&[generic(2), w()]), SelectionRequirement::Artifact, 1)],
        back_face: Some(Box::new(levitator)),
        ..Default::default()
    }
}

/// Dire Flail // Dire Blunderbuss — {R} Artifact Equipment, equipped +2/+0,
/// Equip {1}. Craft with artifact {3}{R}{R} → Dire Blunderbuss (equipped +3/+0,
/// Equip {1}). (The "attack-sac-artifact ping" granted ability is omitted.)
pub fn dire_flail() -> CardDefinition {
    use crate::card::EquipBonus;
    let blunderbuss = CardDefinition {
        name: "Dire Blunderbuss",
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus { power: 3, toughness: 0, ..Default::default() }),
        ..Default::default()
    };
    CardDefinition {
        name: "Dire Flail",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 0, ..Default::default() }),
        activated_abilities: vec![craft(cost(&[generic(3), r(), r()]), SelectionRequirement::Artifact, 1)],
        back_face: Some(Box::new(blunderbuss)),
        ..Default::default()
    }
}

/// Clay-Fired Bricks // Cosmium Kiln — {1}{W} Artifact; ETB tutor a basic Plains
/// to hand and gain 2 life. Craft with artifact {5}{W}{W} → Cosmium Kiln (ETB
/// make two 1/1 Gnome tokens; creatures you control get +1/+1).
pub fn clay_fired_bricks() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect, TokenDefinition};
    let gnome = TokenDefinition {
        name: "Gnome".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        ..Default::default()
    };
    let kiln = CardDefinition {
        name: "Cosmium Kiln",
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: gnome,
        })],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Clay-Fired Bricks",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasLandType(LandType::Plains)
                    .and(SelectionRequirement::HasSupertype(crate::card::Supertype::Basic)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]))],
        activated_abilities: vec![craft(cost(&[generic(5), w(), w()]), SelectionRequirement::Artifact, 1)],
        back_face: Some(Box::new(kiln)),
        ..Default::default()
    }
}
