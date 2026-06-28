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
