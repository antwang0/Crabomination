//! Oath of the Gatewatch (OGW) gap wave 1 — lands, Equipment, and the
//! small spells. Tests in `classic_sets/ogw`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, Keyword,
    SelectionRequirement as R, StaticAbility, Subtypes, Value,
};
use crate::effect::shortcut::{add_any_one_color, animate_land, draw, surge, target_filtered};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest,
};
use crate::mana::{Color, SpendRestriction, cost, generic, r, u, w};

fn types(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}

/// The four OGW enters-tapped duals ("This land enters tapped. {T}: Add {a} or
/// {b}."), which carry no basic land types.
fn ogw_tapland(name: &'static str, a: Color, b: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        activated_abilities: vec![super::super::tap_add(a), super::super::tap_add(b)],
        ..Default::default()
    }
}

pub fn meandering_river() -> CardDefinition {
    ogw_tapland("Meandering River", Color::White, Color::Blue)
}

pub fn submerged_boneyard() -> CardDefinition {
    ogw_tapland("Submerged Boneyard", Color::Blue, Color::Black)
}

pub fn timber_gorge() -> CardDefinition {
    ogw_tapland("Timber Gorge", Color::Red, Color::Green)
}

pub fn tranquil_expanse() -> CardDefinition {
    ogw_tapland("Tranquil Expanse", Color::Green, Color::White)
}

/// Holdout Settlement — Land. {T}: Add {C}. {T}, Tap an untapped creature you
/// control: Add one mana of any color.
pub fn holdout_settlement() -> CardDefinition {
    CardDefinition {
        name: "Holdout Settlement",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            super::super::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                tap_other_filter: Some(R::Creature.and(R::ControlledByYou)),
                effect: add_any_one_color(1),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Corrupted Crossroads — Land. {T}: Add {C}. {T}, Pay 1 life: Add one mana of
/// any color, spendable only on devoid spells.
pub fn corrupted_crossroads() -> CardDefinition {
    CardDefinition {
        name: "Corrupted Crossroads",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            super::super::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                life_cost: 1,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::Const(1))),
                        SpendRestriction::DevoidSpellsOnly,
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ruins of Oran-Rief — Land that enters tapped. {T}: Add {C}. {T}: Put a
/// +1/+1 counter on target colorless creature that entered this turn.
pub fn ruins_of_oran_rief() -> CardDefinition {
    CardDefinition {
        name: "Ruins of Oran-Rief",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        activated_abilities: vec![
            super::super::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::Colorless).and(R::EnteredThisTurn)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Mirrorpool — Land that enters tapped. {T}: Add {C}. Sacrifice it to copy an
/// instant/sorcery you control, or to token-copy a creature you control.
pub fn mirrorpool() -> CardDefinition {
    CardDefinition {
        name: "Mirrorpool",
        card_types: vec![CardType::Land],
        supertypes: vec![crate::card::Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        activated_abilities: vec![
            super::super::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), crate::mana::colorless(1)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::CopySpellMayChooseTargets {
                    what: target_filtered(
                        R::HasCardType(CardType::Instant)
                            .or(R::HasCardType(CardType::Sorcery))
                            .and(R::ControlledByYou),
                    ),
                    count: Value::Const(1),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4), crate::mana::colorless(1)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: target_filtered(R::Creature.and(R::ControlledByYou)),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ancient Crab — {1}{U}{U} 1/5 Crab.
pub fn ancient_crab() -> CardDefinition {
    CardDefinition {
        name: "Ancient Crab",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Crab]),
        power: 1,
        toughness: 5,
        ..Default::default()
    }
}

/// Makindi Aeronaut — {1}{W} 1/3 Kor Scout Ally with flying.
pub fn makindi_aeronaut() -> CardDefinition {
    CardDefinition {
        name: "Makindi Aeronaut",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Kor, CreatureType::Scout, CreatureType::Ally]),
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Umara Entangler — {1}{U} 2/1 Merfolk Rogue Ally with prowess.
pub fn umara_entangler() -> CardDefinition {
    CardDefinition {
        name: "Umara Entangler",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Merfolk, CreatureType::Rogue, CreatureType::Ally]),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Prowess],
        ..Default::default()
    }
}

/// Kor Sky Climber — {2}{W} 3/2 Kor Soldier Ally. {1}{W}: gains flying.
pub fn kor_sky_climber() -> CardDefinition {
    CardDefinition {
        name: "Kor Sky Climber",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Kor, CreatureType::Soldier, CreatureType::Ally]),
        power: 3,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Baloth Pup — {1}{G} 3/1 Beast with trample while it has a +1/+1 counter.
pub fn baloth_pup() -> CardDefinition {
    CardDefinition {
        name: "Baloth Pup",
        cost: cost(&[generic(1), crate::mana::g()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Beast]),
        power: 3,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "This creature has trample as long as it has a +1/+1 counter on it.",
            effect: StaticEffect::SelfHasKeywordWhile {
                keyword: Keyword::Trample,
                condition: R::WithCounter(CounterType::PlusOnePlusOne),
            },
        }],
        ..Default::default()
    }
}

/// Jwar Isle Avenger — {4}{U} 3/3 Sphinx with flying. Surge {2}{U}.
pub fn jwar_isle_avenger() -> CardDefinition {
    CardDefinition {
        name: "Jwar Isle Avenger",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Sphinx]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        alternative_cost: Some(surge(cost(&[generic(2), u()]), false)),
        ..Default::default()
    }
}

/// Chitinous Cloak — {3} Equipment. Equipped creature gets +2/+2 and has
/// menace. Equip {3}.
pub fn chitinous_cloak() -> CardDefinition {
    CardDefinition {
        name: "Chitinous Cloak",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Menace],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Seer's Lantern — {3} Artifact. {T}: Add {C}. {2}, {T}: Scry 1.
pub fn seers_lantern() -> CardDefinition {
    CardDefinition {
        name: "Seer's Lantern",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            super::super::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Immolating Glare — {1}{W} Instant. Destroy target attacking creature.
pub fn immolating_glare() -> CardDefinition {
    CardDefinition {
        name: "Immolating Glare",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(R::Creature.and(R::IsAttacking)),
        },
        ..Default::default()
    }
}

/// Expedite — {R} Instant. Target creature gains haste; draw a card.
pub fn expedite() -> CardDefinition {
    CardDefinition {
        name: "Expedite",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
        ..Default::default()
    }
}

/// Vines of the Recluse — {G} Instant. Target creature gets +1/+2, gains
/// reach, and untaps.
pub fn vines_of_the_recluse() -> CardDefinition {
    CardDefinition {
        name: "Vines of the Recluse",
        cost: cost(&[crate::mana::g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(1),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Reach,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
        ]),
        ..Default::default()
    }
}

/// Lead by Example — {1}{G} Instant. Support 2.
pub fn lead_by_example() -> CardDefinition {
    CardDefinition {
        name: "Lead by Example",
        cost: cost(&[generic(1), crate::mana::g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::SupportCounters {
            max_targets: 2,
            filter: R::Creature,
        },
        ..Default::default()
    }
}

/// Elemental Uprising — {1}{G} Instant. Target land you control becomes a 4/4
/// Elemental with haste that must be blocked this turn if able.
pub fn elemental_uprising() -> CardDefinition {
    CardDefinition {
        name: "Elemental Uprising",
        cost: cost(&[generic(1), crate::mana::g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            animate_land(0, 4),
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::MustBeBlocked,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Sparkmage's Gambit — {1}{R} Sorcery. 1 damage to each of up to two target
/// creatures; those creatures can't block this turn.
pub fn sparkmages_gambit() -> CardDefinition {
    CardDefinition {
        name: "Sparkmage's Gambit",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::Const(1),
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                },
            ])),
        },
        ..Default::default()
    }
}

/// Void Shatter — {1}{U}{U} Instant. Devoid. Counter target spell and exile it.
pub fn void_shatter() -> CardDefinition {
    CardDefinition {
        name: "Void Shatter",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::CounterSpellToZone {
            what: Selector::Target(0),
            zone: crate::effect::CounteredSpellZone::Exile,
        },
        ..Default::default()
    }
}

/// Call the Gatewatch — {2}{W} Sorcery. Search your library for a
/// planeswalker card and put it into your hand.
pub fn call_the_gatewatch() -> CardDefinition {
    CardDefinition {
        name: "Call the Gatewatch",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: R::HasCardType(CardType::Planeswalker),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}
