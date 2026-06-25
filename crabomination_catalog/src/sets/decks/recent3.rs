//! A third wave of staples — high-demand reprints/format cards that filled
//! remaining gaps (Solphim, Atraxa, Deathrite Shaman, Grand Abolisher, …).
//! Each card has a functionality test in `crabomination/src/tests/recent3.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::{ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, phyrexian, r, u, w, Color};

/// Solphim, Mayhem Dominus — {2}{R}{R} 5/4. Doubles noncombat damage your
/// sources deal to opponents; {1}{R/P}{R/P}, discard two: gains an
/// indestructible counter.
pub fn solphim_mayhem_dominus() -> CardDefinition {
    CardDefinition {
        name: "Solphim, Mayhem Dominus",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "If a source you control would deal noncombat damage to \
                an opponent or a permanent an opponent controls, it deals double \
                that damage instead.",
            effect: StaticEffect::DoubleNoncombatDamageToOpponents,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), phyrexian(Color::Red), phyrexian(Color::Red)]),
            discard_cost: Some((SelectionRequirement::Any, 2)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Indestructible,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Atraxa, Praetors' Voice — {G}{W}{U}{B} 4/4 with flying, vigilance,
/// deathtouch, lifelink; proliferates at the beginning of your end step.
pub fn atraxa_praetors_voice() -> CardDefinition {
    CardDefinition {
        name: "Atraxa, Praetors' Voice",
        cost: cost(&[g(), w(), u(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Phyrexian,
                CreatureType::Angel,
                CreatureType::Horror,
            ],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![
            Keyword::Flying,
            Keyword::Vigilance,
            Keyword::Deathtouch,
            Keyword::Lifelink,
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::Proliferate,
        }],
        ..Default::default()
    }
}

/// Deathrite Shaman — {B/G} 1/2. Three graveyard-exile activated abilities:
/// land→any-color mana, instant/sorcery→drain 2, creature→gain 2.
pub fn deathrite_shaman() -> CardDefinition {
    let exile_target = |filter: SelectionRequirement| Effect::Move {
        what: Selector::TargetFiltered {
            slot: 0,
            filter: filter.and(SelectionRequirement::InGraveyard),
        },
        to: ZoneDest::Exile,
    };
    CardDefinition {
        name: "Deathrite Shaman",
        cost: cost(&[hybrid(Color::Black, Color::Green)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![
            // {T}: Exile target land card from a graveyard. Add one mana of any color.
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    exile_target(SelectionRequirement::Land),
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::AnyOneColor(Value::ONE),
                    },
                ]),
                ..Default::default()
            },
            // {B}, {T}: Exile target instant or sorcery from a graveyard. Each
            // opponent loses 2 life.
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[b()]),
                effect: Effect::Seq(vec![
                    exile_target(
                        SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    ),
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(2),
                    },
                ]),
                ..Default::default()
            },
            // {G}, {T}: Exile target creature card from a graveyard. Gain 2 life.
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[g()]),
                effect: Effect::Seq(vec![
                    exile_target(SelectionRequirement::Creature),
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Grand Abolisher — {W}{W} 2/2. During your turn, opponents can't cast spells
/// or activate abilities of artifacts, creatures, or enchantments.
pub fn grand_abolisher() -> CardDefinition {
    CardDefinition {
        name: "Grand Abolisher",
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "During your turn, your opponents can't cast spells or \
                activate abilities of artifacts, creatures, or enchantments.",
            effect: StaticEffect::OpponentsCantActDuringYourTurn,
        }],
        ..Default::default()
    }
}

/// Sundering Titan — {8} 7/10 artifact. On enter or leave, destroy a land of
/// each basic land type.
pub fn sundering_titan() -> CardDefinition {
    let destroy = |kind: EventKind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::SelfSource),
        effect: Effect::DestroyLandOfEachBasicType,
    };
    CardDefinition {
        name: "Sundering Titan",
        cost: cost(&[generic(8)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            ..Default::default()
        },
        power: 7,
        toughness: 10,
        triggered_abilities: vec![
            destroy(EventKind::EntersBattlefield),
            destroy(EventKind::PermanentLeavesBattlefield),
        ],
        ..Default::default()
    }
}

/// Arcane Laboratory — {2}{U} Enchantment. Each player can't cast more than one
/// spell each turn. (Reuses the existing `OneSpellPerTurn` static; the Rule of
/// Law family already ships.)
pub fn arcane_laboratory() -> CardDefinition {
    CardDefinition {
        name: "Arcane Laboratory",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Each player can't cast more than one spell each turn.",
            effect: StaticEffect::OneSpellPerTurn,
        }],
        ..Default::default()
    }
}

/// Flashfires — {3}{R} Sorcery. Destroy all Plains.
pub fn flashfires() -> CardDefinition {
    destroy_all_landtype("Flashfires", cost(&[generic(3), r()]), crate::card::LandType::Plains)
}

/// Tsunami — {3}{G} Sorcery. Destroy all Islands.
pub fn tsunami() -> CardDefinition {
    destroy_all_landtype("Tsunami", cost(&[generic(3), g()]), crate::card::LandType::Island)
}

/// Boiling Seas — {3}{R} Sorcery. Destroy all Islands.
pub fn boiling_seas() -> CardDefinition {
    destroy_all_landtype("Boiling Seas", cost(&[generic(3), r()]), crate::card::LandType::Island)
}

fn destroy_all_landtype(
    name: &'static str,
    cost: crate::mana::ManaCost,
    land_type: crate::card::LandType,
) -> CardDefinition {
    CardDefinition {
        name,
        cost,
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: Selector::EachPermanent(SelectionRequirement::HasLandType(land_type)),
        },
        ..Default::default()
    }
}

/// Shatterstorm — {2}{R}{R} Sorcery. Destroy all artifacts; they can't be regenerated.
pub fn shatterstorm() -> CardDefinition {
    CardDefinition {
        name: "Shatterstorm",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DestroyNoRegen {
            what: Selector::EachPermanent(SelectionRequirement::Artifact),
        },
        ..Default::default()
    }
}

/// Anarchy — {2}{R}{R} Sorcery. Destroy all white permanents.
pub fn anarchy() -> CardDefinition {
    CardDefinition {
        name: "Anarchy",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: Selector::EachPermanent(SelectionRequirement::HasColor(Color::White)),
        },
        ..Default::default()
    }
}

/// Creeping Mold — {2}{G}{G} Sorcery. Destroy target artifact, enchantment, or land.
pub fn creeping_mold() -> CardDefinition {
    CardDefinition {
        name: "Creeping Mold",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: crate::effect::shortcut::target_filtered(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Enchantment)
                    .or(SelectionRequirement::Land),
            ),
        },
        ..Default::default()
    }
}

/// Liliana's Caress — {1}{B} Enchantment. Whenever an opponent discards a card,
/// they lose 2 life.
pub fn lilianas_caress() -> CardDefinition {
    CardDefinition {
        name: "Liliana's Caress",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::OpponentControl),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Winter Orb — {2} Artifact. Lands don't untap during their controllers'
/// untap steps. (Reuses the `PreventUntap` static.)
pub fn winter_orb() -> CardDefinition {
    CardDefinition {
        name: "Winter Orb",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Lands don't untap during their controllers' untap steps.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::EachPermanent(SelectionRequirement::Land),
            },
        }],
        ..Default::default()
    }
}

/// Choke — {2}{G} Enchantment. Islands don't untap during their controllers'
/// untap steps.
pub fn choke() -> CardDefinition {
    CardDefinition {
        name: "Choke",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Islands don't untap during their controllers' untap steps.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::EachPermanent(SelectionRequirement::HasLandType(
                    crate::card::LandType::Island,
                )),
            },
        }],
        ..Default::default()
    }
}

/// Manalith — {3} Artifact. {T}: Add one mana of any color.
pub fn manalith() -> CardDefinition {
    CardDefinition {
        name: "Manalith",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![crate::sets::tap_add_any_color()],
        ..Default::default()
    }
}

/// Darksteel Ingot — {3} Artifact. Indestructible. {T}: Add one mana of any color.
pub fn darksteel_ingot() -> CardDefinition {
    CardDefinition {
        name: "Darksteel Ingot",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Indestructible],
        activated_abilities: vec![crate::sets::tap_add_any_color()],
        ..Default::default()
    }
}

/// Cultivator's Caravan — {3} 5/5 Vehicle. {T}: Add one mana of any color. Crew 3.
pub fn cultivators_caravan() -> CardDefinition {
    CardDefinition {
        name: "Cultivator's Caravan",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Crew(3)],
        activated_abilities: vec![crate::sets::tap_add_any_color()],
        ..Default::default()
    }
}

/// Spinning Wheel — {3} Artifact. {T}: Add one mana of any color. {5}, {T}: Tap
/// target creature.
pub fn spinning_wheel() -> CardDefinition {
    CardDefinition {
        name: "Spinning Wheel",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            crate::sets::tap_add_any_color(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(5)]),
                effect: Effect::Tap {
                    what: crate::effect::shortcut::target_filtered(SelectionRequirement::Creature),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Hurricane / Squall Line shape: deal X to each creature with flying and each player.
fn hurricane_effect() -> Effect {
    Effect::Seq(vec![
        Effect::DealDamage {
            to: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
            ),
            amount: Value::XFromCost,
        },
        Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::XFromCost,
        },
    ])
}

/// Hurricane — {X}{G} Sorcery. Deal X to each creature with flying and each player.
pub fn hurricane() -> CardDefinition {
    CardDefinition {
        name: "Hurricane",
        cost: cost(&[crate::mana::x(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: hurricane_effect(),
        ..Default::default()
    }
}

/// Squall Line — {X}{G}{G} Instant. Deal X to each creature with flying and each player.
pub fn squall_line() -> CardDefinition {
    CardDefinition {
        name: "Squall Line",
        cost: cost(&[crate::mana::x(), g(), g()]),
        card_types: vec![CardType::Instant],
        effect: hurricane_effect(),
        ..Default::default()
    }
}

/// Staff of Nin — {6} Artifact. Upkeep: draw a card. {T}: deal 1 to any target.
pub fn staff_of_nin() -> CardDefinition {
    CardDefinition {
        name: "Staff of Nin",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: Selector::Target(0), amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ivory Tower — {1} Artifact. Upkeep: gain life equal to cards in hand minus 4.
pub fn ivory_tower() -> CardDefinition {
    CardDefinition {
        name: "Ivory Tower",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::NonNeg(Box::new(Value::Diff(
                    Box::new(Value::HandSizeOf(PlayerRef::You)),
                    Box::new(Value::Const(4)),
                ))),
            },
        }],
        ..Default::default()
    }
}

/// Viridian Shaman — {2}{G} 2/2 Elf Shaman. ETB: destroy target artifact.
pub fn viridian_shaman() -> CardDefinition {
    CardDefinition {
        name: "Viridian Shaman",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Destroy {
            what: crate::effect::shortcut::target_filtered(SelectionRequirement::Artifact),
        })],
        ..Default::default()
    }
}

/// Caustic Caterpillar — {G} 1/1 Insect. {1}{G}, Sacrifice this: destroy target
/// artifact or enchantment.
pub fn caustic_caterpillar() -> CardDefinition {
    CardDefinition {
        name: "Caustic Caterpillar",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            sac_cost: true,
            effect: Effect::Destroy {
                what: crate::effect::shortcut::target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Noxious Revival — {G/P} Instant. Put target card from a graveyard on top of
/// its owner's library.
pub fn noxious_revival() -> CardDefinition {
    CardDefinition {
        name: "Noxious Revival",
        cost: cost(&[phyrexian(Color::Green)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::InGraveyard },
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: crate::effect::LibraryPosition::Top,
            },
        },
        ..Default::default()
    }
}

/// Bane of Progress — {4}{G}{G} 2/2 Elemental. ETB: destroy all artifacts and
/// enchantments; grow by one +1/+1 counter per permanent destroyed.
pub fn bane_of_progress() -> CardDefinition {
    CardDefinition {
        name: "Bane of Progress",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Seq(vec![
            Effect::Destroy {
                what: Selector::EachPermanent(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::PermanentsDestroyedThisResolution,
            },
        ]))],
        ..Default::default()
    }
}

/// Ramunap Ruins — Desert land. {T}: Add {C}. {T}, pay 1 life: Add {R}.
/// {2}{R}{R}, {T}, Sacrifice a Desert: deal 2 to each opponent.
pub fn ramunap_ruins() -> CardDefinition {
    CardDefinition {
        name: "Ramunap Ruins",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![crate::card::LandType::Desert],
            ..Default::default()
        },
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                life_cost: 1,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Red]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2), r(), r()]),
                sac_other_filter: Some((
                    SelectionRequirement::HasLandType(crate::card::LandType::Desert),
                    1,
                )),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Back to Nature — {1}{G} Instant. Destroy all enchantments.
pub fn back_to_nature() -> CardDefinition {
    CardDefinition {
        name: "Back to Nature",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: Selector::EachPermanent(SelectionRequirement::Enchantment),
        },
        ..Default::default()
    }
}

/// Whirlwind — {2}{G}{G} Sorcery. Destroy all creatures with flying.
pub fn whirlwind() -> CardDefinition {
    CardDefinition {
        name: "Whirlwind",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::HasKeyword(Keyword::Flying)),
            ),
        },
        ..Default::default()
    }
}

/// Fault Line — {X}{R}{R} Instant. Deal X to each creature without flying and each player.
pub fn fault_line() -> CardDefinition {
    CardDefinition {
        name: "Fault Line",
        cost: cost(&[crate::mana::x(), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying).negate()),
                ),
                amount: Value::XFromCost,
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::XFromCost,
            },
        ]),
        ..Default::default()
    }
}

/// Serenity — {1}{W} Enchantment. At the beginning of your upkeep, destroy all
/// artifacts and enchantments (they can't be regenerated) — itself included.
pub fn serenity() -> CardDefinition {
    CardDefinition {
        name: "Serenity",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::DestroyNoRegen {
                what: Selector::EachPermanent(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
        }],
        ..Default::default()
    }
}
