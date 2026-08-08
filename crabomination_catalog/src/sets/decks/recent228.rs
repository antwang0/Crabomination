//! Gap batch — MH3 Medallions/Cascade staples, WOE/BIG/OTJ value cards, all on
//! existing primitives. Tests in `tests/recent228.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EquipBonus, ExileReturnZone, Keyword,
    SelectionRequirement as R, StaticAbility, Subtypes, TokenDefinition,
};
use crate::effect::shortcut::{
    add_any_one_color, add_colorless, cascade, deal, etb, flurry, on_dies, target_any,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector, StaticEffect, Value,
};
use crate::mana::{Color, cost, g, generic, r, u, w};

fn medallion(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Spells of the chosen color cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: R::HasColor(color),
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Pearl Medallion — {2} Artifact. White spells you cast cost {1} less.
pub fn pearl_medallion() -> CardDefinition {
    medallion("Pearl Medallion", Color::White)
}
/// Sapphire Medallion — {2} Artifact. Blue spells you cast cost {1} less.
pub fn sapphire_medallion() -> CardDefinition {
    medallion("Sapphire Medallion", Color::Blue)
}
/// Jet Medallion — {2} Artifact. Black spells you cast cost {1} less.
pub fn jet_medallion() -> CardDefinition {
    medallion("Jet Medallion", Color::Black)
}
/// Ruby Medallion — {2} Artifact. Red spells you cast cost {1} less.
pub fn ruby_medallion() -> CardDefinition {
    medallion("Ruby Medallion", Color::Red)
}
/// Emerald Medallion — {2} Artifact. Green spells you cast cost {1} less.
pub fn emerald_medallion() -> CardDefinition {
    medallion("Emerald Medallion", Color::Green)
}

/// Annoyed Altisaur — {5}{G}{G} 6/5 Dinosaur. Reach, trample, cascade.
pub fn annoyed_altisaur() -> CardDefinition {
    CardDefinition {
        name: "Annoyed Altisaur",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        triggered_abilities: vec![cascade(7)],
        ..Default::default()
    }
}

/// Meteoric Mace — {4}{R}{R} Artifact Equipment. Equipped creature gets +4/+0
/// and has trample. Equip {4}. Cascade.
pub fn meteoric_mace() -> CardDefinition {
    CardDefinition {
        name: "Meteoric Mace",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            power: 4,
            toughness: 0,
            keywords: vec![Keyword::Trample],
            ..Default::default()
        }),
        triggered_abilities: vec![cascade(6)],
        ..Default::default()
    }
}

fn fish_token() -> TokenDefinition {
    let whale = TokenDefinition {
        name: "Whale".into(),
        power: 6,
        toughness: 6,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Whale],
            ..Default::default()
        },
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: Box::new(TokenDefinition {
                name: "Kraken".into(),
                power: 9,
                toughness: 9,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Blue],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Kraken],
                    ..Default::default()
                },
                ..Default::default()
            }),
        })],
        ..Default::default()
    };
    TokenDefinition {
        name: "Fish".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fish],
            ..Default::default()
        },
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: Box::new(whale),
        })],
        ..Default::default()
    }
}

/// Reef Worm — {3}{U} 0/1 Worm. When it dies, make a 3/3 Fish that makes a 6/6
/// Whale that makes a 9/9 Kraken.
pub fn reef_worm() -> CardDefinition {
    CardDefinition {
        name: "Reef Worm",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Worm],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: Box::new(fish_token()),
        })],
        ..Default::default()
    }
}

/// Deserted Temple — Land. {T}: Add {C}. {1}, {T}: Untap target land.
pub fn deserted_temple() -> CardDefinition {
    CardDefinition {
        name: "Deserted Temple",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: add_colorless(1),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Untap {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Land,
                    },
                    up_to: None,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Barbarian Ring — Land. {T}: Add {R}, deal 1 to you. {R}, {T}, Sacrifice,
/// threshold: deal 2 to any target.
pub fn barbarian_ring() -> CardDefinition {
    CardDefinition {
        name: "Barbarian Ring",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Colors(vec![Color::Red]),
                    },
                    deal(1, Selector::You),
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[r()]),
                sac_cost: true,
                condition: Some(Predicate::ThresholdActive {
                    who: PlayerRef::You,
                }),
                effect: deal(2, target_any()),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Glass Casket — {1}{W} Artifact. ETB: exile target creature an opponent
/// controls with mana value 3 or less until this leaves.
pub fn glass_casket() -> CardDefinition {
    CardDefinition {
        name: "Glass Casket",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature
                    .and(R::ControlledByOpponent)
                    .and(R::ManaValueAtMost(3)),
            },
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Crystal Grotto — Land. ETB: scry 1. {T}: Add {C}. {1}, {T}: Add one mana of
/// any color.
pub fn crystal_grotto() -> CardDefinition {
    CardDefinition {
        name: "Crystal Grotto",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![etb(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: add_colorless(1),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: add_any_one_color(1),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Molten Duplication — {1}{R} Sorcery. Create a token copy of target artifact
/// or creature you control (also an artifact), with haste; sacrifice it next
/// end step.
pub fn molten_duplication() -> CardDefinition {
    CardDefinition {
        name: "Molten Duplication",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::Const(1),
                source: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Artifact.or(R::Creature).and(R::ControlledByYou),
                },
                extra_creature_types: vec![],
                extra_card_types: vec![CardType::Artifact],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![Keyword::Haste],
            },
            Effect::SacrificeLastCreatedTokensAtNextEndStep,
        ]),
        ..Default::default()
    }
}

/// Shackle Slinger — {2}{U} 3/2 Human Soldier. When you cast your second spell
/// each turn, tap target opponent's creature, or stun it if already tapped.
pub fn shackle_slinger() -> CardDefinition {
    let target = Selector::TargetFiltered {
        slot: 0,
        filter: R::Creature.and(R::ControlledByOpponent),
    };
    CardDefinition {
        name: "Shackle Slinger",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![flurry(Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: R::Tapped,
            },
            then: Box::new(Effect::AddCounter {
                what: target.clone(),
                kind: crate::card::CounterType::Stun,
                amount: Value::Const(1),
            }),
            else_: Box::new(Effect::Tap { what: target }),
        })],
        ..Default::default()
    }
}

/// Fledgling Dragon — {2}{R}{R} 2/2 Dragon. Flying; with threshold it gets
/// +3/+3 and has "{R}: +1/+0". (The firebreathing is modeled as always
/// available; it's only relevant once the +3/+3 threshold bonus is live.)
pub fn fledgling_dragon() -> CardDefinition {
    CardDefinition {
        name: "Fledgling Dragon",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Threshold — +3/+3.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ThresholdActive {
                    who: PlayerRef::You,
                },
                power: 3,
                toughness: 3,
                keywords: vec![],
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Thunder Salvo — {1}{R} Instant. Deals X damage to target creature, where X
/// is 2 plus the number of other spells you've cast this turn.
pub fn thunder_salvo() -> CardDefinition {
    CardDefinition {
        name: "Thunder Salvo",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature,
            },
            amount: Value::Sum(vec![
                Value::Const(2),
                Value::OtherSpellsCastThisTurn(PlayerRef::You),
            ]),
        },
        ..Default::default()
    }
}
