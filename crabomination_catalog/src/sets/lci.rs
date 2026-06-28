//! The Lost Caverns of Ixalan (LCI) — 2023. Introduces the Discover
//! (CR 701.57) keyword action.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, DynamicPt, EventKind, EventScope, EventSpec, Keyword, LandType, StaticAbility,
    StaticEffect, SelectionRequirement, Selector, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{craft, drain, etb, on_attack, on_dies, pump_target, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::effects::{map_token, treasure_token};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color, ManaCost};
use crate::game::types::TurnStep;

/// Geological Appraiser — {2}{R}{R} 3/2 Human Artificer. "When this enters,
/// if you cast it, discover 3." The `SourceWasCast` gate keeps token copies and
/// reanimated bodies from re-firing.
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
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SourceWasCast,
            then: Box::new(Effect::Discover { n: Value::Const(3), filter: None }),
            else_: Box::new(Effect::Noop),
        })],
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

// ── More LCI staples (Descend / fathomless descent) ─────────────────────────

/// Souls of the Lost — {1}{B} Spirit. Additional cost: discard a card (the "or
/// sacrifice a permanent" alternative is approximated as discard). Fathomless
/// descent — its P/T is */*+1 = permanent cards in your graveyard.
pub fn souls_of_the_lost() -> CardDefinition {
    CardDefinition {
        name: "Souls of the Lost",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        additional_cast_cost: vec![AdditionalCastCost::Discard { count: 1 }],
        dynamic_pt: Some(DynamicPt::PermanentCardsInControllerGraveyard { base_p: 0, base_t: 1 }),
        ..Default::default()
    }
}

/// Acolyte of Aclazotz — {2}{B} 1/4 Vampire Cleric. {T}, Sacrifice another
/// creature or artifact: each opponent loses 1 life and you gain 1 life.
pub fn acolyte_of_aclazotz() -> CardDefinition {
    CardDefinition {
        name: "Acolyte of Aclazotz",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((
                SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
                1,
            )),
            effect: drain(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cavern Stomper — {4}{G}{G} 7/7 Dinosaur. ETB scry 2. {3}{G}: this can't be
/// blocked by creatures with power 2 or less this turn.
pub fn cavern_stomper() -> CardDefinition {
    CardDefinition {
        name: "Cavern Stomper",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 7,
        toughness: 7,
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::CantBeBlockedByPowerAtMost(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Goldfury Strider — {4}{R} 3/5 Golem with trample. Tap two untapped artifacts
/// and/or creatures you control: target creature gets +2/+0 until end of turn.
/// Activate only as a sorcery.
pub fn goldfury_strider() -> CardDefinition {
    CardDefinition {
        name: "Goldfury Strider",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            tap_n_filter: Some((
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Artifact)
                    .and(SelectionRequirement::ControlledByYou),
                2,
            )),
            effect: pump_target(2, 0),
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── LCI Descend / explore commons ───────────────────────────────────────────

/// Frilled Cave-Wurm — {3}{U} 2/5 Salamander Wurm. Descend 4 — gets +2/+0
/// while you have 4+ permanent cards in your graveyard.
pub fn frilled_cave_wurm() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Frilled Cave-Wurm",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Salamander, CreatureType::Wurm],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        static_abilities: vec![StaticAbility {
            description: "Descend 4 — gets +2/+0 while you have 4+ permanent cards in your graveyard.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::DescendActive { who: PlayerRef::You, count: 4 },
                power: 2,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Basking Capybara — {1}{G} 1/3 Capybara. Descend 4 — gets +3/+0 while you
/// have 4+ permanent cards in your graveyard.
pub fn basking_capybara() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Basking Capybara",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Capybara], ..Default::default() },
        power: 1,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Descend 4 — gets +3/+0 while you have 4+ permanent cards in your graveyard.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::DescendActive { who: PlayerRef::You, count: 4 },
                power: 3,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Coati Scavenger — {2}{G} 3/2 Raccoon. Descend 4 — ETB, if you have 4+
/// permanent cards in your graveyard, return target permanent card from your
/// graveyard to your hand.
pub fn coati_scavenger() -> CardDefinition {
    CardDefinition {
        name: "Coati Scavenger",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Raccoon], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::DescendActive { who: PlayerRef::You, count: 4 },
            then: Box::new(Effect::Move {
                what: target_filtered(SelectionRequirement::PermanentCard),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Bitter Triumph — {1}{B} Instant. Additional cost: discard a card (the "or
/// pay 3 life" alternative is approximated as discard). Destroy target creature
/// or planeswalker.
pub fn bitter_triumph() -> CardDefinition {
    CardDefinition {
        name: "Bitter Triumph",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![AdditionalCastCost::Discard { count: 1 }],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
        },
        ..Default::default()
    }
}

/// Kinjalli's Dawnrunner — {2}{W} 1/1 Human Scout with double strike. ETB: it
/// explores.
pub fn kinjallis_dawnrunner() -> CardDefinition {
    CardDefinition {
        name: "Kinjalli's Dawnrunner",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![etb(Effect::Explore { who: Selector::This })],
        ..Default::default()
    }
}

/// Rampaging Ceratops — {4}{R} 5/4 Dinosaur. Can't be blocked except by three
/// or more creatures.
pub fn rampaging_ceratops() -> CardDefinition {
    CardDefinition {
        name: "Rampaging Ceratops",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::CantBeBlockedExceptByN(3)],
        ..Default::default()
    }
}

/// Mineshaft Spider — {3}{G} 3/4 Spider with reach. ETB: you may mill two cards.
pub fn mineshaft_spider() -> CardDefinition {
    CardDefinition {
        name: "Mineshaft Spider",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Mill two cards?".into(),
            body: Box::new(Effect::Mill { who: Selector::You, amount: Value::Const(2) }),
        })],
        ..Default::default()
    }
}

/// Poison Dart Frog — {1}{G} 1/1 Frog with reach. {T}: add one mana of any
/// color. {2}: this creature gains deathtouch until end of turn.
pub fn poison_dart_frog() -> CardDefinition {
    CardDefinition {
        name: "Poison Dart Frog",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Frog], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![
            crate::sets::tap_add_any_color(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Deathtouch,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── LCI explore / Map / token commons ───────────────────────────────────────

/// River Herald Scout — {1}{U} 1/2 Merfolk Scout. ETB: it explores.
pub fn river_herald_scout() -> CardDefinition {
    CardDefinition {
        name: "River Herald Scout",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Explore { who: Selector::This })],
        ..Default::default()
    }
}

/// Waterwind Scout — {2}{U} 2/2 Merfolk Scout with flying. ETB: create a Map.
pub fn waterwind_scout() -> CardDefinition {
    CardDefinition {
        name: "Waterwind Scout",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: map_token(),
        })],
        ..Default::default()
    }
}

/// Pathfinding Axejaw — {3}{G} 4/3 Dinosaur. ETB: it explores.
pub fn pathfinding_axejaw() -> CardDefinition {
    CardDefinition {
        name: "Pathfinding Axejaw",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Explore { who: Selector::This })],
        ..Default::default()
    }
}

/// Merfolk Cave-Diver — {2}{U} 2/4 Merfolk Scout. Whenever a creature you
/// control explores, this gets +1/+0 and can't be blocked this turn.
pub fn merfolk_cave_diver() -> CardDefinition {
    CardDefinition {
        name: "Merfolk Cave-Diver",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Explored, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Panicked Altisaur — {4}{R} 4/5 Dinosaur with reach. {T}: deals 2 damage to
/// each opponent.
pub fn panicked_altisaur() -> CardDefinition {
    CardDefinition {
        name: "Panicked Altisaur",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Plundering Pirate — {2}{R} 3/2 Orc Pirate. ETB: create a Treasure token.
pub fn plundering_pirate() -> CardDefinition {
    CardDefinition {
        name: "Plundering Pirate",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Pirate],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: treasure_token(),
        })],
        ..Default::default()
    }
}

/// Vito's Inquisitor — {3}{B} 3/3 Vampire Knight. {B}, Sacrifice another
/// creature or artifact: put a +1/+1 counter on this and it gains menace EOT.
pub fn vitos_inquisitor() -> CardDefinition {
    CardDefinition {
        name: "Vito's Inquisitor",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_other_filter: Some((
                SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
                1,
            )),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Oltec Cloud Guard — {3}{W} 3/2 Human Soldier with flying. ETB: create a 1/1
/// colorless Gnome artifact creature token.
pub fn oltec_cloud_guard() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Oltec Cloud Guard",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Gnome".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Artifact, CardType::Creature],
                subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

/// Miner's Guidewing — {W} 1/1 Bird with flying and vigilance. When it dies,
/// target creature you control explores.
pub fn miners_guidewing() -> CardDefinition {
    CardDefinition {
        name: "Miner's Guidewing",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![on_dies(Effect::Explore {
            who: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
        })],
        ..Default::default()
    }
}

// ── LCI Descend payoffs / commons ───────────────────────────────────────────

/// "At the beginning of your end step, if you descended this turn (CR 700.11),
/// put a +1/+1 counter on this creature."
fn end_step_descended_counter() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
            .with_filter(Predicate::DescendedThisTurn { who: PlayerRef::You }),
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
    }
}

/// Deep Goblin Skulltaker — {2}{B} 2/2 Goblin Warrior with menace. End step, if
/// you descended this turn, put a +1/+1 counter on it.
pub fn deep_goblin_skulltaker() -> CardDefinition {
    CardDefinition {
        name: "Deep Goblin Skulltaker",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![end_step_descended_counter()],
        ..Default::default()
    }
}

/// Child of the Volcano — {3}{R} 3/3 Elemental with trample. End step, if you
/// descended this turn, put a +1/+1 counter on it.
pub fn child_of_the_volcano() -> CardDefinition {
    CardDefinition {
        name: "Child of the Volcano",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![end_step_descended_counter()],
        ..Default::default()
    }
}

/// Echo of Dusk — {1}{B} 2/2 Vampire Spirit. Descend 4 — gets +1/+1 and has
/// lifelink while you have 4+ permanent cards in your graveyard.
pub fn echo_of_dusk() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Echo of Dusk",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Descend 4 — +1/+1 and lifelink while you have 4+ permanent cards in your graveyard.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::DescendActive { who: PlayerRef::You, count: 4 },
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Lifelink],
            },
        }],
        ..Default::default()
    }
}

/// Colossadactyl — {2}{G}{G} 4/5 Dinosaur with reach and trample.
pub fn colossadactyl() -> CardDefinition {
    CardDefinition {
        name: "Colossadactyl",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        ..Default::default()
    }
}

/// Hermitic Nautilus — {1}{U} 1/4 Artifact Creature — Nautilus with vigilance.
/// {1}{U}: this creature gets +3/-3 until end of turn.
pub fn hermitic_nautilus() -> CardDefinition {
    CardDefinition {
        name: "Hermitic Nautilus",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nautilus], ..Default::default() },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Screaming Phantom — {2}{B} 2/2 Spirit with flying. Attacks → mill a card.
pub fn screaming_phantom() -> CardDefinition {
    CardDefinition {
        name: "Screaming Phantom",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Mill { who: Selector::You, amount: Value::Const(1) })],
        ..Default::default()
    }
}

/// Deathcap Marionette — {1}{B} 1/1 Fungus with deathtouch. ETB: you may mill
/// two cards.
pub fn deathcap_marionette() -> CardDefinition {
    CardDefinition {
        name: "Deathcap Marionette",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fungus], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Mill two cards?".into(),
            body: Box::new(Effect::Mill { who: Selector::You, amount: Value::Const(2) }),
        })],
        ..Default::default()
    }
}

/// Greedy Freebooter — {B} 1/1 Human Pirate. When it dies, scry 1 and create a
/// Treasure token.
pub fn greedy_freebooter() -> CardDefinition {
    CardDefinition {
        name: "Greedy Freebooter",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: treasure_token() },
        ]))],
        ..Default::default()
    }
}

// ── LCI Gnome / artifact commons ────────────────────────────────────────────

/// Cartographer's Companion — {3} 2/1 Artifact Creature — Gnome. ETB: create a
/// Map token.
pub fn cartographers_companion() -> CardDefinition {
    CardDefinition {
        name: "Cartographer's Companion",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: map_token(),
        })],
        ..Default::default()
    }
}

/// Market Gnome — {W} 0/3 Artifact Creature — Gnome. When it dies, gain 1 life
/// and draw a card. (The duplicate "exiled while crafting" trigger is omitted.)
pub fn market_gnome() -> CardDefinition {
    CardDefinition {
        name: "Market Gnome",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        power: 0,
        toughness: 3,
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}

/// Adaptive Gemguard — {3}{W} 3/3 Artifact Creature — Gnome. Tap two untapped
/// artifacts and/or creatures you control: put a +1/+1 counter on this. Sorcery
/// speed.
pub fn adaptive_gemguard() -> CardDefinition {
    CardDefinition {
        name: "Adaptive Gemguard",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            tap_n_filter: Some((
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Artifact)
                    .and(SelectionRequirement::ControlledByYou),
                2,
            )),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dinotomaton — {3}{R} 4/3 Artifact Creature — Dinosaur Gnome with menace.
/// ETB: target creature you control gains menace until end of turn.
pub fn dinotomaton() -> CardDefinition {
    CardDefinition {
        name: "Dinotomaton",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur, CreatureType::Gnome],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou)),
            keyword: Keyword::Menace,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Oltec Archaeologists — {4}{W} 4/4 Human Artificer Scout. ETB, choose one —
/// return target artifact card from your graveyard to your hand; or scry 3.
pub fn oltec_archaeologists() -> CardDefinition {
    CardDefinition {
        name: "Oltec Archaeologists",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer, CreatureType::Scout],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Artifact),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(3) },
        ]))],
        ..Default::default()
    }
}

// ── Caves (LandType::Cave) and Caves-matter payoffs ─────────────────────────

/// `{T}: Add` the given mana pool — the basic tap ability shared by the Caves.
fn cave_tap(pool: ManaPayload) -> ActivatedAbility {
    ActivatedAbility { tap_cost: true, effect: Effect::AddMana { who: PlayerRef::You, pool }, ..Default::default() }
}

/// Captivating Cave — Land — Cave. {T}: Add {C}. {1}, {T}: Add one mana of any
/// color. {4}, {T}, Sacrifice: put two +1/+1 counters on target creature
/// (sorcery speed).
pub fn captivating_cave() -> CardDefinition {
    CardDefinition {
        name: "Captivating Cave",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Cave], ..Default::default() },
        activated_abilities: vec![
            cave_tap(ManaPayload::Colorless(Value::Const(1))),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(1)) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                mana_cost: cost(&[generic(4)]),
                effect: Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Volatile Fault — Land — Cave. {T}: Add {C}. {1}, {T}, Sacrifice: destroy
/// target nonbasic land an opponent controls (the "may search for a basic"
/// rider is dropped).
pub fn volatile_fault() -> CardDefinition {
    CardDefinition {
        name: "Volatile Fault",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Cave], ..Default::default() },
        activated_abilities: vec![
            cave_tap(ManaPayload::Colorless(Value::Const(1))),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::IsNonbasicLand
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Promising Vein — Land — Cave. {T}: Add {C}. {1}, {T}, Sacrifice: search your
/// library for a basic land, put it onto the battlefield tapped.
pub fn promising_vein() -> CardDefinition {
    CardDefinition {
        name: "Promising Vein",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Cave], ..Default::default() },
        activated_abilities: vec![
            cave_tap(ManaPayload::Colorless(Value::Const(1))),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasSupertype(Supertype::Basic)
                        .and(SelectionRequirement::Land),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Forgotten Monument — Land — Cave. {T}: Add {C}. Other Caves you control have
/// "{T}, Pay 1 life: Add one mana of any color."
pub fn forgotten_monument() -> CardDefinition {
    CardDefinition {
        name: "Forgotten Monument",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Cave], ..Default::default() },
        activated_abilities: vec![cave_tap(ManaPayload::Colorless(Value::Const(1)))],
        static_abilities: vec![StaticAbility {
            description: "Other Caves you control have \"{T}, Pay 1 life: Add one mana of any color.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasLandType(LandType::Cave)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                ability: ActivatedAbility {
                    tap_cost: true,
                    life_cost: 1,
                    effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(1)) },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// The "Hidden" Cave cycle — enters tapped, {T}: Add one color, {4}{color}, {T},
/// Sacrifice: Discover 4 (sorcery speed).
fn hidden_cave(name: &'static str, color: Color, color_cost: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Cave], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![
            cave_tap(ManaPayload::OfColor(color, Value::Const(1))),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                mana_cost: color_cost,
                effect: Effect::Discover { n: Value::Const(4), filter: None },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

pub fn hidden_courtyard() -> CardDefinition { hidden_cave("Hidden Courtyard", Color::White, cost(&[generic(4), w()])) }
pub fn hidden_cataract() -> CardDefinition { hidden_cave("Hidden Cataract", Color::Blue, cost(&[generic(4), u()])) }
pub fn hidden_necropolis() -> CardDefinition { hidden_cave("Hidden Necropolis", Color::Black, cost(&[generic(4), b()])) }
pub fn hidden_volcano() -> CardDefinition { hidden_cave("Hidden Volcano", Color::Red, cost(&[generic(4), r()])) }
pub fn hidden_nursery() -> CardDefinition { hidden_cave("Hidden Nursery", Color::Green, cost(&[generic(4), g()])) }

/// Spelunking — {2}{G} Enchantment. ETB: draw a card, then you may put a land
/// from your hand onto the battlefield (the "gain 4 if it's a Cave" rider is
/// dropped). Lands you control enter untapped.
pub fn spelunking() -> CardDefinition {
    CardDefinition {
        name: "Spelunking",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                count: Value::Const(1),
                tapped: false,
                haste: false,
                sacrifice_eot: false,
            },
        ]))],
        static_abilities: vec![StaticAbility {
            description: "Lands you control enter the battlefield untapped.",
            effect: StaticEffect::LandsEnterUntapped,
        }],
        ..Default::default()
    }
}

/// Sanguine Evangelist — {2}{W} 2/1 Vampire Cleric with battle cry. When it
/// enters or dies, create a 1/1 black Bat with flying.
pub fn sanguine_evangelist() -> CardDefinition {
    use crate::card::TokenDefinition;
    let bat = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: TokenDefinition {
            name: "Bat".into(),
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Flying],
            card_types: vec![CardType::Creature],
            colors: vec![Color::Black],
            subtypes: Subtypes { creature_types: vec![CreatureType::Bat], ..Default::default() },
            ..Default::default()
        },
    };
    CardDefinition {
        name: "Sanguine Evangelist",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![
            crate::effect::shortcut::battle_cry(1),
            etb(bat()),
            on_dies(bat()),
        ],
        ..Default::default()
    }
}

// ── More LCI staples (Map / explore / removal) ──────────────────────────────

/// Family Reunion — {1}{W} Instant. Choose one — creatures you control get
/// +1/+1 until end of turn; or creatures you control gain hexproof until end
/// of turn.
pub fn family_reunion() -> CardDefinition {
    let yours = || Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    );
    CardDefinition {
        name: "Family Reunion",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::PumpPT { what: yours(), power: Value::Const(1), toughness: Value::Const(1), duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: yours(), keyword: Keyword::Hexproof, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Bartolomé del Presidio — {W}{B} 2/1 Vampire Knight. Sacrifice another
/// creature or artifact: put a +1/+1 counter on Bartolomé.
pub fn bartolome_del_presidio() -> CardDefinition {
    CardDefinition {
        name: "Bartolomé del Presidio",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((SelectionRequirement::Creature.or(SelectionRequirement::Artifact), 1)),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── More LCI staples (Pirate / Dinosaur payoffs) ────────────────────────────

/// Captain Storm, Cosmium Raider — {U}{R} 2/2 legendary Human Pirate. Whenever
/// an artifact you control enters, put a +1/+1 counter on target Pirate you
/// control.
pub fn captain_storm_cosmium_raider() -> CardDefinition {
    CardDefinition {
        name: "Captain Storm, Cosmium Raider",
        cost: cost(&[u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                }),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::HasCreatureType(CreatureType::Pirate)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Bedrock Tortoise — {3}{G} 0/6 Turtle. During your turn, creatures you
/// control have hexproof. Each creature you control with toughness > power
/// assigns combat damage by its toughness.
pub fn bedrock_tortoise() -> CardDefinition {
    let yours = || Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    );
    CardDefinition {
        name: "Bedrock Tortoise",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Turtle], ..Default::default() },
        power: 0,
        toughness: 6,
        static_abilities: vec![
            StaticAbility {
                description: "During your turn, creatures you control have hexproof.",
                effect: StaticEffect::PumpTeamIf {
                    condition: Predicate::IsTurnOf(PlayerRef::You),
                    applies_to: yours(),
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::Hexproof],
                },
            },
            StaticAbility {
                description: "Your creatures with toughness > power assign combat damage by toughness.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::ToughnessGreaterThanPower),
                    ),
                    keyword: Keyword::AssignsCombatDamageByToughness,
                },
            },
        ],
        ..Default::default()
    }
}

/// Amalia Benavides Aguirre — {W}{B} 2/2 Vampire Scout, Ward—Pay 3 life.
/// Whenever you gain life, Amalia explores; then if her power is exactly 20,
/// destroy all other creatures.
pub fn amalia_benavides_aguirre() -> CardDefinition {
    CardDefinition {
        name: "Amalia Benavides Aguirre",
        cost: cost(&[w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Ward(crate::card::WardCost::Life(3))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Explore { who: Selector::This },
                Effect::If {
                    cond: Predicate::ValueEquals(
                        Value::PowerOf(Box::new(Selector::This)),
                        Value::Const(20),
                    ),
                    then: Box::new(Effect::Destroy {
                        what: Selector::EachPermanent(
                            SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                        ),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Jadelight Spelunker — {X}{G} 1/1 Merfolk Scout. When it enters, it explores
/// X times.
pub fn jadelight_spelunker() -> CardDefinition {
    CardDefinition {
        name: "Jadelight Spelunker",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Merfolk, CreatureType::Scout], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Repeat {
            count: Value::XFromCost,
            body: Box::new(Effect::Explore { who: Selector::This }),
        })],
        ..Default::default()
    }
}

/// Staggering Size — {1}{G} Instant. Target creature gets +3/+3 and gains
/// trample until end of turn.
pub fn staggering_size() -> CardDefinition {
    CardDefinition {
        name: "Staggering Size",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Compass Gnome — {2} 2/1 Gnome Artifact Creature. ETB: you may search your
/// library for a basic land or Cave card and put it on top.
pub fn compass_gnome() -> CardDefinition {
    CardDefinition {
        name: "Compass Gnome",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasSupertype(Supertype::Basic)
                .and(SelectionRequirement::Land)
                .or(SelectionRequirement::HasLandType(LandType::Cave)),
            to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top },
        })],
        ..Default::default()
    }
}

/// Gargantuan Leech — {7}{B} 5/5 with lifelink. Costs {1} less for each Cave
/// you control and each Cave card in your graveyard.
pub fn gargantuan_leech() -> CardDefinition {
    CardDefinition {
        name: "Gargantuan Leech",
        cost: cost(&[generic(7), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Leech], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Lifelink],
        affinity_filter: Some(
            SelectionRequirement::HasLandType(LandType::Cave)
                .and(SelectionRequirement::ControlledByYou),
        ),
        affinity_graveyard_filter: Some(SelectionRequirement::HasLandType(LandType::Cave)),
        ..Default::default()
    }
}

/// Terror Tide — {2}{B}{B} Sorcery. Fathomless descent — all creatures get
/// -X/-X until end of turn, where X is the number of permanent cards in your
/// graveyard.
pub fn terror_tide() -> CardDefinition {
    let descent = || Value::CardsInGraveyardMatching {
        who: PlayerRef::You,
        filter: SelectionRequirement::PermanentCard,
    };
    let neg = move || Value::Times(Box::new(descent()), Box::new(Value::Const(-1)));
    CardDefinition {
        name: "Terror Tide",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(SelectionRequirement::Creature),
            power: neg(),
            toughness: neg(),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Dusk Legion Duelist — {1}{W} 2/2 Vampire Soldier with vigilance. Whenever
/// one or more +1/+1 counters are put on it, draw a card (once each turn).
pub fn dusk_legion_duelist() -> CardDefinition {
    CardDefinition {
        name: "Dusk Legion Duelist",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                EventScope::SelfSource,
            ).once_per_turn(),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Over the Edge — {1}{G} Sorcery. Choose one — destroy target artifact or
/// enchantment; or target creature you control explores, then explores again.
pub fn over_the_edge() -> CardDefinition {
    CardDefinition {
        name: "Over the Edge",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment)),
            },
            Effect::Seq(vec![
                Effect::Explore {
                    who: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                },
                Effect::Explore { who: Selector::Target(0) },
            ]),
        ]),
        ..Default::default()
    }
}

/// Pugnacious Hammerskull — {2}{G} 6/6 Dinosaur. Whenever it attacks while you
/// control no other Dinosaur, put a stun counter on it.
pub fn pugnacious_hammerskull() -> CardDefinition {
    CardDefinition {
        name: "Pugnacious Hammerskull",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 6,
        toughness: 6,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::Not(Box::new(Predicate::SelectorExists(Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Dinosaur)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                )))),
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Stun,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Sentry of the Underworld — {3}{W}{B} 3/3 Griffin Skeleton with flying and
/// vigilance. {W}{B}, Pay 3 life: Regenerate it.
pub fn sentry_of_the_underworld() -> CardDefinition {
    CardDefinition {
        name: "Sentry of the Underworld",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Griffin, CreatureType::Skeleton],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), b()]),
            life_cost: 3,
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sunshot Militia — {1}{R} 1/3 Human Soldier. Tap two untapped artifacts
/// and/or creatures you control: deal 1 damage to each opponent (sorcery speed).
pub fn sunshot_militia() -> CardDefinition {
    CardDefinition {
        name: "Sunshot Militia",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Soldier], ..Default::default() },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            tap_n_filter: Some((SelectionRequirement::Artifact.or(SelectionRequirement::Creature), 2)),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}


// ── LCI batch (modern_decks): commons & uncommons on existing primitives ─────

/// Acrobatic Leap — {W} Instant. Target creature gets +1/+3 and gains flying
/// until end of turn. Untap it.
pub fn acrobatic_leap() -> CardDefinition {
    CardDefinition {
        name: "Acrobatic Leap",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
        ..Default::default()
    }
}

/// Petrify — {1}{W} Aura. Enchant artifact or creature. Enchanted permanent
/// can't attack or block, and its activated abilities can't be activated.
pub fn petrify() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Petrify",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
            ),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock, Keyword::CantActivateAbilities],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Ray of Ruin — {4}{B} Sorcery. Exile target creature, Vehicle, or nonbasic
/// land. Scry 1.
pub fn ray_of_ruin() -> CardDefinition {
    CardDefinition {
        name: "Ray of Ruin",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Vehicle))
                        .or(SelectionRequirement::IsNonbasicLand),
                ),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Scampering Surveyor — {4} 3/2 Gnome. When this enters, search your library
/// for a basic land or Cave card and put it onto the battlefield tapped.
pub fn scampering_surveyor() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Scampering Surveyor",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand
                .or(SelectionRequirement::HasLandType(LandType::Cave)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        })],
        ..Default::default()
    }
}

/// Seeker of Sunlight — {G} 1/1 Merfolk Scout. {2}{G}: This creature explores.
/// Activate only as a sorcery.
pub fn seeker_of_sunlight() -> CardDefinition {
    CardDefinition {
        name: "Seeker of Sunlight",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            sorcery_speed: true,
            effect: Effect::Explore { who: Selector::This },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mischievous Pup — {2}{W} 3/1 Dog, flash. When this enters, return up to one
/// other target permanent you control to its owner's hand.
pub fn mischievous_pup() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Mischievous Pup",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Permanent
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            },
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..Default::default()
    }
}

/// Nurturing Bristleback — {5}{G}{G} 5/5 Dinosaur. ETB: create a 3/3 green
/// Dinosaur token. Forestcycling {2}.
pub fn nurturing_bristleback() -> CardDefinition {
    use crate::card::TokenDefinition;
    let dino = TokenDefinition {
        name: "Dinosaur".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Nurturing Bristleback",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), LandType::Forest)],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: dino,
        })],
        ..Default::default()
    }
}

/// Soaring Sandwing — {4}{W}{W} 3/5 Dinosaur, flying. ETB: gain 3 life.
/// Plainscycling {2}.
pub fn soaring_sandwing() -> CardDefinition {
    CardDefinition {
        name: "Soaring Sandwing",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Landcycling(cost(&[generic(2)]), LandType::Plains)],
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(3) })],
        ..Default::default()
    }
}

/// Rampaging Spiketail — {4}{B}{B} 5/6 Dinosaur. ETB: target creature you
/// control gets +2/+0 and gains indestructible until end of turn.
/// Swampcycling {2}.
pub fn rampaging_spiketail() -> CardDefinition {
    CardDefinition {
        name: "Rampaging Spiketail",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), LandType::Swamp)],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Tinker's Tote — {2}{W} Artifact. ETB: create two 1/1 colorless Gnome
/// artifact creature tokens. {W}, Sacrifice this artifact: gain 3 life.
pub fn tinkers_tote() -> CardDefinition {
    use crate::card::TokenDefinition;
    let gnome = TokenDefinition {
        name: "Gnome".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Tinker's Tote",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: gnome,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            sac_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Primordial Gnawer — {4}{B} 5/2 Insect Horror. When this dies, discover 3.
pub fn primordial_gnawer() -> CardDefinition {
    CardDefinition {
        name: "Primordial Gnawer",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Horror],
            ..Default::default()
        },
        power: 5,
        toughness: 2,
        triggered_abilities: vec![on_dies(Effect::Discover { n: Value::Const(3), filter: None })],
        ..Default::default()
    }
}

/// Mephitic Draught — {1}{B} Artifact. When this enters or is put into a
/// graveyard from the battlefield, draw a card and lose 1 life.
pub fn mephitic_draught() -> CardDefinition {
    let payoff = || Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
    ]);
    CardDefinition {
        name: "Mephitic Draught",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            etb(payoff()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: payoff(),
            },
        ],
        ..Default::default()
    }
}

/// Staunch Crewmate — {1}{U} 2/1 Human Pirate. ETB: look at the top four cards
/// of your library; you may reveal an artifact or Pirate card from among them
/// and put it into your hand. Put the rest on the bottom.
pub fn staunch_crewmate() -> CardDefinition {
    CardDefinition {
        name: "Staunch Crewmate",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: false,
            pick_filter: Some(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::HasCreatureType(CreatureType::Pirate)),
            ),
            take: None,
            to_battlefield: false,
        })],
        ..Default::default()
    }
}

/// Malamet Brawler — {1}{G} 2/2 Cat Warrior. Whenever it attacks, target
/// attacking creature gains trample until end of turn.
pub fn malamet_brawler() -> CardDefinition {
    CardDefinition {
        name: "Malamet Brawler",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::IsAttacking),
            ),
            keyword: Keyword::Trample,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Malamet Veteran — {4}{G} 5/4 Cat Warrior, trample. Descend 4 — whenever it
/// attacks, if you have four or more permanent cards in your graveyard, put a
/// +1/+1 counter on target creature.
pub fn malamet_veteran() -> CardDefinition {
    CardDefinition {
        name: "Malamet Veteran",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![on_attack(Effect::If {
            cond: Predicate::DescendActive { who: PlayerRef::You, count: 4 },
            then: Box::new(Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Enterprising Scallywag — {1}{R} 2/2 Goblin Pirate. At the beginning of your
/// end step, if you descended this turn, create a Treasure token.
pub fn enterprising_scallywag() -> CardDefinition {
    CardDefinition {
        name: "Enterprising Scallywag",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
                .with_filter(Predicate::DescendedThisTurn { who: PlayerRef::You }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
            },
        }],
        ..Default::default()
    }
}

/// Careening Mine Cart — {3} 3/3 Artifact Vehicle, Crew 1. Whenever it attacks,
/// create a Treasure token.
pub fn careening_mine_cart() -> CardDefinition {
    CardDefinition {
        name: "Careening Mine Cart",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Crew(1)],
        triggered_abilities: vec![on_attack(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: treasure_token(),
        })],
        ..Default::default()
    }
}

/// Brazen Blademaster — {2}{R} 2/3 Orc Pirate. Whenever it attacks while you
/// control two or more artifacts, it gets +2/+1 until end of turn.
pub fn brazen_blademaster() -> CardDefinition {
    CardDefinition {
        name: "Brazen Blademaster",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                ),
                n: Value::Const(2),
            },
            then: Box::new(Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Burning Sun Cavalry — {1}{R} 2/2 Human Knight. Whenever it attacks or blocks
/// while you control a Dinosaur, it gets +1/+1 until end of turn.
pub fn burning_sun_cavalry() -> CardDefinition {
    let pump_if_dino = || Effect::If {
        cond: Predicate::SelectorCountAtLeast {
            sel: Selector::EachPermanent(
                SelectionRequirement::HasCreatureType(CreatureType::Dinosaur)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            n: Value::Const(1),
        },
        then: Box::new(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        }),
        else_: Box::new(Effect::Noop),
    };
    CardDefinition {
        name: "Burning Sun Cavalry",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            on_attack(pump_if_dino()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: pump_if_dino(),
            },
        ],
        ..Default::default()
    }
}

/// Hotfoot Gnome — {2}{R} 3/1 Artifact Creature — Gnome, haste. {T}: Another
/// target creature gains haste until end of turn.
pub fn hotfoot_gnome() -> CardDefinition {
    CardDefinition {
        name: "Hotfoot Gnome",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fungal Fortitude — {1}{B} Aura, flash. Enchanted creature gets +2/+0. When
/// enchanted creature dies, return it to the battlefield tapped under its
/// owner's control.
pub fn fungal_fortitude() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Fungal Fortitude",
        cost: cost(&[generic(1), b()]),
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
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 0, ..Default::default() }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)),
                    tapped: true,
                },
            },
        }],
        ..Default::default()
    }
}

/// Armored Kincaller — {2}{G} 3/3 Dinosaur. ETB: if you control another
/// Dinosaur, gain 3 life. (The "reveal a Dinosaur from hand" alternative is
/// approximated by the control check.)
pub fn armored_kincaller() -> CardDefinition {
    CardDefinition {
        name: "Armored Kincaller",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Dinosaur)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                n: Value::Const(1),
            },
            then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}


/// Brackish Blunder — {1}{U} Instant. Return target creature to its owner's
/// hand. If it was tapped, create a Map token.
pub fn brackish_blunder() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Brackish Blunder",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::Tapped,
                },
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: map_token(),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        ]),
        ..Default::default()
    }
}

/// Bloodthorn Flail — {B} Artifact Equipment. Equipped creature gets +2/+1.
/// Equip {3}. (The "or discard a card" equip alternative is omitted.)
pub fn bloodthorn_flail() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Bloodthorn Flail",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 1, ..Default::default() }),
        ..Default::default()
    }
}

/// Diamond Pick-Axe — {R} Artifact Equipment, indestructible. Equipped creature
/// gets +1/+1 and has "Whenever this creature attacks, create a Treasure
/// token." Equip {2}.
pub fn diamond_pick_axe() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Diamond Pick-Axe",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Indestructible, Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![on_attack(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Glowcap Lantern — {G} Artifact Equipment. Equipped creature has "Whenever
/// this creature attacks, it explores." Equip {2}. (The "look at the top card
/// any time" rider is omitted.)
pub fn glowcap_lantern() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Glowcap Lantern",
        cost: cost(&[g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![on_attack(Effect::Explore { who: Selector::This })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Pirate Hat — {1}{U} Artifact Equipment. Equipped creature gets +1/+1 and has
/// "Whenever this creature attacks, draw a card, then discard a card." Equip
/// {2}. (The cheaper Equip Pirate {1} is omitted.)
pub fn pirate_hat() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Pirate Hat",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![on_attack(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            ]))],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Triumphant Chomp — {R} Sorcery. Deals damage to target creature equal to 2
/// or the greatest power among Dinosaurs you control, whichever is greater.
pub fn triumphant_chomp() -> CardDefinition {
    CardDefinition {
        name: "Triumphant Chomp",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Max(
                Box::new(Value::Const(2)),
                Box::new(Value::PowerOf(Box::new(Selector::GreatestPowerControlledMatching(
                    SelectionRequirement::HasCreatureType(CreatureType::Dinosaur),
                )))),
            ),
        },
        ..Default::default()
    }
}

// ── LCI batch 2 (modern_decks): descend / discover / explore commons ─────────

/// Ruin-Lurker Bat — {W} 1/1 Bat, flying lifelink. End step, if you descended
/// this turn, scry 1.
pub fn ruin_lurker_bat() -> CardDefinition {
    CardDefinition {
        name: "Ruin-Lurker Bat",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bat], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
                .with_filter(Predicate::DescendedThisTurn { who: PlayerRef::You }),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Join the Dead — {1}{B}{B} Instant. Target creature gets -5/-5 until end of
/// turn — or -10/-10 instead if you have four or more permanent cards in your
/// graveyard (Descend 4).
pub fn join_the_dead() -> CardDefinition {
    CardDefinition {
        name: "Join the Dead",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::DescendActive { who: PlayerRef::You, count: 4 },
            then: Box::new(Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-10),
                toughness: Value::Const(-10),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(-5),
                toughness: Value::Const(-5),
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Ancestors' Aid — {1}{R} Instant. Target creature gets +2/+0 and gains first
/// strike until end of turn. Create a Treasure token.
pub fn ancestors_aid() -> CardDefinition {
    CardDefinition {
        name: "Ancestors' Aid",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: treasure_token() },
        ]),
        ..Default::default()
    }
}

/// River Herald Guide — {2}{G} 3/1 Merfolk Scout, vigilance. When this enters,
/// it explores.
pub fn river_herald_guide() -> CardDefinition {
    CardDefinition {
        name: "River Herald Guide",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::Explore { who: Selector::This })],
        ..Default::default()
    }
}

/// Might of the Ancestors — {2}{W} Enchantment. At the beginning of combat on
/// your turn, target creature you control gets +2/+0 and gains vigilance until
/// end of turn.
pub fn might_of_the_ancestors() -> CardDefinition {
    CardDefinition {
        name: "Might of the Ancestors",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Walk with the Ancestors — {4}{G} Sorcery. Return up to one target permanent
/// card from your graveyard to your hand. Discover 4.
pub fn walk_with_the_ancestors() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Walk with the Ancestors",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::PermanentCard),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Discover { n: Value::Const(4), filter: None },
        ]),
        ..Default::default()
    }
}

/// Vanguard of the Rose — {1}{W} 3/1 Vampire Knight. {1}, Sacrifice another
/// creature or artifact: this creature gains indestructible until end of turn.
/// Tap it.
pub fn vanguard_of_the_rose() -> CardDefinition {
    CardDefinition {
        name: "Vanguard of the Rose",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((
                SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
                1,
            )),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
                Effect::Tap { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Daring Discovery — {4}{R} Sorcery. Up to three target creatures can't block
/// this turn. Discover 4.
pub fn daring_discovery() -> CardDefinition {
    CardDefinition {
        name: "Daring Discovery",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 3,
                filter: SelectionRequirement::Creature,
                effect: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                }),
            },
            Effect::Discover { n: Value::Const(4), filter: None },
        ]),
        ..Default::default()
    }
}

/// Attentive Sunscribe — {1}{W} 2/2 Artifact Creature — Gnome. Whenever this
/// becomes tapped, scry 1.
pub fn attentive_sunscribe() -> CardDefinition {
    CardDefinition {
        name: "Attentive Sunscribe",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}


/// Self-Reflection — {4}{U}{U} Sorcery. Create a token that's a copy of target
/// creature you control. Flashback {3}{U}.
pub fn self_reflection() -> CardDefinition {
    CardDefinition {
        name: "Self-Reflection",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), u()]))],
        effect: Effect::CreateTokenCopyOf {
            who: PlayerRef::You,
            count: Value::Const(1),
            source: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            extra_creature_types: vec![],
            extra_card_types: vec![],
            override_pt: None,
            non_legendary: false,
            legendary: false,
        },
        ..Default::default()
    }
}

/// Canonized in Blood — {1}{B} Enchantment. End step, if you descended this
/// turn, put a +1/+1 counter on target creature you control. {5}{B}{B},
/// Sacrifice this enchantment: create a 4/3 white-black Vampire Demon with
/// flying.
pub fn canonized_in_blood() -> CardDefinition {
    use crate::card::TokenDefinition;
    let demon = TokenDefinition {
        name: "Vampire Demon".into(),
        power: 4,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Demon],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Canonized in Blood",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
                .with_filter(Predicate::DescendedThisTurn { who: PlayerRef::You }),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), b(), b()]),
            sac_cost: true,
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: demon },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Earthshaker Dreadmaw — {4}{G}{G} 6/6 Dinosaur, trample. When this enters,
/// draw a card for each other Dinosaur you control.
pub fn earthshaker_dreadmaw() -> CardDefinition {
    CardDefinition {
        name: "Earthshaker Dreadmaw",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(
                    SelectionRequirement::ControlledByYou.and(SelectionRequirement::OtherThanSource),
                )),
                filter: SelectionRequirement::HasCreatureType(CreatureType::Dinosaur),
            },
        })],
        ..Default::default()
    }
}

/// Threefold Thunderhulk — {7} 0/0 Artifact Creature — Gnome. Enters with three
/// +1/+1 counters. When it enters or attacks, create a number of 1/1 colorless
/// Gnome artifact creature tokens equal to its power. {2}, Sacrifice another
/// artifact: put a +1/+1 counter on it.
pub fn threefold_thunderhulk() -> CardDefinition {
    use crate::card::TokenDefinition;
    let gnome = TokenDefinition {
        name: "Gnome".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        ..Default::default()
    };
    let make_gnomes = move || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::PowerOf(Box::new(Selector::This)),
        definition: gnome.clone(),
    };
    CardDefinition {
        name: "Threefold Thunderhulk",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        triggered_abilities: vec![etb(make_gnomes()), on_attack(make_gnomes())],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((SelectionRequirement::Artifact, 1)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tectonic Hazard — {R} Sorcery. Deals 1 damage to each opponent and each
/// creature they control.
pub fn tectonic_hazard() -> CardDefinition {
    CardDefinition {
        name: "Tectonic Hazard",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(1) },
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Soulcoil Viper — {2}{B} 2/3 Snake. {B}, {T}, Sacrifice this creature: return
/// target creature card from your graveyard to the battlefield with a finality
/// counter on it (sorcery speed).
pub fn soulcoil_viper() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Soulcoil Viper",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            tap_cost: true,
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::InGraveyard.and(SelectionRequirement::Creature),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Finality,
                    amount: Value::Const(1),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Itzquinth, Firstborn of Gishath — {R}{G} 2/3 legendary Dinosaur, haste. ETB:
/// you may pay {2}; when you do, a Dinosaur you control bites another creature.
/// The bite is a `Reflexive` payoff so its two targets are chosen after the {2}
/// is paid (CR 603.7), not pre-validated at the ETB trigger.
pub fn itzquinth_firstborn_of_gishath() -> CardDefinition {
    CardDefinition {
        name: "Itzquinth, Firstborn of Gishath",
        cost: cost(&[r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {2}: a Dinosaur you control deals damage = its power \
                to another target creature".into(),
            mana_cost: cost(&[generic(2)]),
            body: Box::new(Effect::Reflexive {
                body: Box::new(Effect::DealDamageEqualToPower {
                    source: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Dinosaur)
                            .and(SelectionRequirement::ControlledByYou),
                    },
                    target: Selector::TargetFiltered {
                        slot: 1,
                        filter: SelectionRequirement::Creature,
                    },
                }),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Glorifier of Suffering — {2}{W} 3/2 Vampire Soldier. ETB: you may sacrifice
/// another creature or artifact; when you do, support 2 (a `Reflexive` payoff so
/// the two +1/+1-counter targets are chosen after the sacrifice).
pub fn glorifier_of_suffering() -> CardDefinition {
    CardDefinition {
        name: "Glorifier of Suffering",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice another creature or artifact: support 2".into(),
            filter: (SelectionRequirement::Creature.or(SelectionRequirement::Artifact))
                .and(SelectionRequirement::OtherThanSource),
            count: Value::ONE,
            then: Box::new(Effect::Reflexive {
                body: Box::new(Effect::SupportCounters {
                    filter: SelectionRequirement::Creature,
                    max_targets: 2,
                }),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Wary Thespian — {1}{G} 3/1 Cat Druid. When it enters or dies, surveil 1.
pub fn wary_thespian() -> CardDefinition {
    CardDefinition {
        name: "Wary Thespian",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) }),
            on_dies(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) }),
        ],
        ..Default::default()
    }
}

/// Huatli's Final Strike — {2}{G} Instant. Target creature you control gets
/// +1/+0, then deals damage equal to its power to target creature an opponent
/// controls.
pub fn huatlis_final_strike() -> CardDefinition {
    CardDefinition {
        name: "Huatli's Final Strike",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                },
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::DealDamageEqualToPower {
                source: Selector::Target(0),
                target: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Ghalta, Stampede Tyrant — {5}{G}{G}{G} 12/12 legendary Elder Dinosaur with
/// trample. ETB: put any number of creature cards from your hand onto the
/// battlefield.
pub fn ghalta_stampede_tyrant() -> CardDefinition {
    CardDefinition {
        name: "Ghalta, Stampede Tyrant",
        cost: cost(&[generic(5), g(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 12,
        toughness: 12,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature,
            count: Value::Const(99),
            tapped: false,
            haste: false,
            sacrifice_eot: false,
        })],
        ..Default::default()
    }
}

/// Deeproot Pilgrimage — {1}{U} Enchantment. Whenever one or more nontoken
/// Merfolk you control become tapped, create a 1/1 blue Merfolk with hexproof.
pub fn deeproot_pilgrimage() -> CardDefinition {
    use crate::card::TokenDefinition;
    let token = TokenDefinition {
        name: "Merfolk".into(),
        colors: vec![Color::Blue],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Merfolk], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Hexproof],
        ..Default::default()
    };
    CardDefinition {
        name: "Deeproot Pilgrimage",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Merfolk)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::NotToken),
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: token,
            },
        }],
        ..Default::default()
    }
}

/// Chupacabra Echo — {2}{B}{B} 3/2 Beast Horror Spirit. Fathomless descent —
/// ETB: target creature an opponent controls gets -X/-X, X = permanent cards in
/// your graveyard.
pub fn chupacabra_echo() -> CardDefinition {
    let descent = || Value::CardsInGraveyardMatching {
        who: PlayerRef::You,
        filter: SelectionRequirement::PermanentCard,
    };
    let neg = move || Value::Times(Box::new(descent()), Box::new(Value::Const(-1)));
    CardDefinition {
        name: "Chupacabra Echo",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast, CreatureType::Horror, CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            },
            power: neg(),
            toughness: neg(),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Huatli's Snubhorn — {1}{W} 2/2 Dinosaur with vigilance.
pub fn huatlis_snubhorn() -> CardDefinition {
    CardDefinition {
        name: "Huatli's Snubhorn",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    }
}

/// Pantlaza, Sun-Favored — {2}{R}{G}{W} 4/4 Dinosaur. When Pantlaza or another
/// Dinosaur you control enters, you may discover X = that creature's toughness
/// (once each turn).
pub fn pantlaza_sun_favored() -> CardDefinition {
    CardDefinition {
        name: "Pantlaza, Sun-Favored",
        cost: cost(&[generic(2), r(), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Dinosaur),
                })
                .once_per_turn(),
            effect: Effect::Discover {
                n: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                filter: None,
            },
        }],
        ..Default::default()
    }
}

/// Stalactite Stalker — {B} 1/1 Goblin Rogue with menace. End step, if you
/// descended this turn, +1/+1. {2}{B}, Sacrifice it: target creature gets -X/-X,
/// X = this creature's power (last-known).
pub fn stalactite_stalker() -> CardDefinition {
    let neg_pow = || Value::Times(
        Box::new(Value::PowerOf(Box::new(Selector::This))),
        Box::new(Value::Const(-1)),
    );
    CardDefinition {
        name: "Stalactite Stalker",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![end_step_descended_counter()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: neg_pow(),
                toughness: neg_pow(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Glimpse the Core — {1}{G} Sorcery. Choose one — search for a basic Forest and
/// put it onto the battlefield tapped; or return target Cave card from your
/// graveyard to the battlefield tapped.
pub fn glimpse_the_core() -> CardDefinition {
    CardDefinition {
        name: "Glimpse the Core",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand
                    .and(SelectionRequirement::HasLandType(LandType::Forest)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::InGraveyard
                        .and(SelectionRequirement::HasLandType(LandType::Cave)),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        ]),
        ..Default::default()
    }
}

/// Quicksand Whirlpool — {5}{W} Instant. Costs {3} less if it targets a tapped
/// creature. Exile target creature.
pub fn quicksand_whirlpool() -> CardDefinition {
    CardDefinition {
        name: "Quicksand Whirlpool",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((
            SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
            3,
        )),
        effect: Effect::Move {
            what: Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::Creature },
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Reckless Detective — {1}{R} 0/3 Devil Detective. Whenever it attacks, you may
/// discard a card; if you do, draw a card and it gets +2/+0. (The "sacrifice an
/// artifact" alternative cost is approximated as the discard.)
pub fn reckless_detective() -> CardDefinition {
    CardDefinition {
        name: "Reckless Detective",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Devil, CreatureType::Detective],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Discard a card: draw a card and +2/+0".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Idol of the Deep King // Sovereign's Macuahuitl — {2}{R} flash artifact; ETB
/// deal 2 to any target. Craft with artifact {2}{R} → an Equipment that attaches
/// on ETB, grants +2/+0, Equip {2}.
pub fn idol_of_the_deep_king() -> CardDefinition {
    use crate::card::EquipBonus;
    let macuahuitl = CardDefinition {
        name: "Sovereign's Macuahuitl",
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 0, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
        })],
        ..Default::default()
    };
    CardDefinition {
        name: "Idol of the Deep King",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(2),
        })],
        activated_abilities: vec![craft(cost(&[generic(2), r()]), SelectionRequirement::Artifact, 1)],
        back_face: Some(Box::new(macuahuitl)),
        ..Default::default()
    }
}

/// Calamitous Tide — {4}{U}{U} Sorcery. Return up to two target creatures to
/// their owners' hands. Draw two cards, then discard a card.
pub fn calamitous_tide() -> CardDefinition {
    CardDefinition {
        name: "Calamitous Tide",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                filter: SelectionRequirement::Creature,
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
        ]),
        ..Default::default()
    }
}

/// Hulking Bugbear — {1}{R}{R} 3/3 Goblin with haste.
pub fn hulking_bugbear() -> CardDefinition {
    CardDefinition {
        name: "Hulking Bugbear",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        ..Default::default()
    }
}

/// Etali's Favor — {2}{R} Aura. Enchant a creature you control; ETB discover 3;
/// enchanted creature gets +1/+1 and has trample.
pub fn etalis_favor() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Etali's Favor",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
        },
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, keywords: vec![Keyword::Trample], ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Discover { n: Value::Const(3), filter: None })],
        ..Default::default()
    }
}

/// Kindled Heroism — {R} Instant. Target creature gets +1/+0 and gains first
/// strike until end of turn. Scry 1.
pub fn kindled_heroism() -> CardDefinition {
    CardDefinition {
        name: "Kindled Heroism",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Dusk Rose Reliquary — {W} Artifact, Ward {2}. Additional cost: sacrifice an
/// artifact or creature. ETB: exile target artifact or creature an opponent
/// controls until this leaves.
pub fn dusk_rose_reliquary() -> CardDefinition {
    use crate::card::{AdditionalCastCost, ExileReturnZone, WardCost};
    CardDefinition {
        name: "Dusk Rose Reliquary",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
            count: 1,
        }],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                (SelectionRequirement::Artifact.or(SelectionRequirement::Creature))
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Digsite Conservator — {2} 2/1 artifact Gnome. Sacrifice it: exile up to four
/// target cards from a single graveyard (sorcery speed). When it dies, you may
/// pay {4}; if you do, discover 4.
pub fn digsite_conservator() -> CardDefinition {
    CardDefinition {
        name: "Digsite Conservator",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::ExileUpToNFromGraveyards { count: Value::Const(4) },
            ..Default::default()
        }],
        triggered_abilities: vec![on_dies(Effect::MayPay {
            description: "Pay {4}: discover 4".into(),
            mana_cost: cost(&[generic(4)]),
            body: Box::new(Effect::Discover { n: Value::Const(4), filter: None }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Malamet Battle Glyph — {G} Sorcery. Choose a creature you control and one you
/// don't; if yours entered this turn, put a +1/+1 counter on it; then they fight.
pub fn malamet_battle_glyph() -> CardDefinition {
    CardDefinition {
        name: "Malamet Battle Glyph",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::EnteredThisTurn,
                },
                then: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Fight {
                attacker: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                },
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Volatile Wanderglyph — {1}{R} 2/2 artifact Golem. Whenever it becomes tapped,
/// you may discard a card; if you do, draw a card.
pub fn volatile_wanderglyph() -> CardDefinition {
    CardDefinition {
        name: "Volatile Wanderglyph",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Loot: discard a card, then draw a card".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Deconstruction Hammer — {W} Equipment, equipped +1/+1, Equip {1}. (The
/// granted "{3}, {T}, sac: destroy artifact/enchantment" ability is omitted —
/// equipment can't grant activated abilities yet.)
pub fn deconstruction_hammer() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Deconstruction Hammer",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        ..Default::default()
    }
}

/// Cosmium Blast — {1}{W} Instant. Deal 4 damage to target attacking or
/// blocking creature.
pub fn cosmium_blast() -> CardDefinition {
    CardDefinition {
        name: "Cosmium Blast",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.and(
                    SelectionRequirement::IsAttacking.or(SelectionRequirement::IsBlocking),
                ),
            ),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Thousand Moons Crackshot — {1}{W} 2/2 Human Soldier. Whenever it attacks, you
/// may pay {2}{W}; when you do, tap target creature (reflexive, CR 603.7).
pub fn thousand_moons_crackshot() -> CardDefinition {
    CardDefinition {
        name: "Thousand Moons Crackshot",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![on_attack(Effect::MayPay {
            description: "Pay {2}{W}: tap target creature".into(),
            mana_cost: cost(&[generic(2), w()]),
            body: Box::new(Effect::Reflexive {
                body: Box::new(Effect::Tap {
                    what: target_filtered(SelectionRequirement::Creature),
                }),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Fanatical Offering — {1}{B} Instant. Additional cost: sacrifice an artifact or
/// creature. Draw two cards and create a Map token.
pub fn fanatical_offering() -> CardDefinition {
    use crate::card::AdditionalCastCost;
    CardDefinition {
        name: "Fanatical Offering",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
            count: 1,
        }],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: map_token() },
        ]),
        ..Default::default()
    }
}

/// Volcanic Geyser — {X}{R}{R} Instant. Deal X damage to any target.
pub fn volcanic_geyser() -> CardDefinition {
    CardDefinition {
        name: "Volcanic Geyser",
        cost: cost(&[x(), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::XFromCost,
        },
        ..Default::default()
    }
}

/// Akawalli, the Seething Tower — {1}{B}{G} 3/3 Fungus. Descend 4 — +2/+2 and
/// trample while you have 4+ permanent cards in your graveyard.
pub fn akawalli_the_seething_tower() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Akawalli, the Seething Tower",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fungus], ..Default::default() },
        power: 3,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Descend 4 — +2/+2 and trample while you have 4+ permanent cards in your graveyard.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::DescendActive { who: PlayerRef::You, count: 4 },
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Trample],
            },
        }],
        ..Default::default()
    }
}

/// Hidden Grotto — Land. ETB: surveil 1. {T}: Add {C}. {1}, {T}: Add one mana of
/// any color.
pub fn hidden_grotto() -> CardDefinition {
    CardDefinition {
        name: "Hidden Grotto",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![etb(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) })],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(1)) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(1)) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
