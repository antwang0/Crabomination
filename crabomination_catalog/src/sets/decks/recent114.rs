//! Enchantress / enchantment-matters package (Legacy & Modern staples). New
//! engine work: `StaticEffect::NonAuraEnchantmentsAreCreatures` (Opalescence /
//! Starfield of Nyx animate non-Aura enchantments to `MV/MV` creatures via a
//! layer-4 add-creature-type + layer-7 `SetPowerToughnessToManaValue`).
//! Tests in `tests/recent114.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, Predicate,
    SelectionRequirement, StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{LookPick, Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, w};

/// "Whenever you cast an enchantment spell, `body`." (Argothian Enchantress /
/// Enchantress's Presence shape.)
fn on_cast_enchantment(body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::HasCardType(CardType::Enchantment),
            },
        ),
        effect: body,
    }
}

/// "Whenever another enchantment you control enters, `body`." (Constellation.)
fn constellation(body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Enchantment,
            },
        ),
        effect: body,
    }
}

/// Selector for "other permanents you control" (or a filtered subset).
fn yours_other(extra: Option<SelectionRequirement>) -> Selector {
    let mut req = SelectionRequirement::ControlledByYou.and(SelectionRequirement::OtherThanSource);
    if let Some(e) = extra {
        req = req.and(e);
    }
    Selector::EachPermanent(req)
}

// ── Enchantress payoffs ──────────────────────────────────────────────────────

/// Enchantress's Presence — {2}{G} Enchantment. Whenever you cast an
/// enchantment spell, draw a card.
pub fn enchantresss_presence() -> CardDefinition {
    CardDefinition {
        name: "Enchantress's Presence",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![on_cast_enchantment(Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Herald of the Pantheon — {1}{G} 2/2 Centaur Shaman. Enchantment spells you
/// cast cost {1} less; whenever you cast an enchantment spell, gain 1 life.
pub fn herald_of_the_pantheon() -> CardDefinition {
    CardDefinition {
        name: "Herald of the Pantheon",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Enchantment spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::Enchantment,
                amount: 1,
            },
        }],
        triggered_abilities: vec![on_cast_enchantment(Effect::GainLife {
            who: Selector::You,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Sigil of the Empty Throne — {3}{W}{W} Enchantment. Whenever you cast an
/// enchantment spell, create a 4/4 white Angel creature token with flying.
pub fn sigil_of_the_empty_throne() -> CardDefinition {
    CardDefinition {
        name: "Sigil of the Empty Throne",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![on_cast_enchantment(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Angel".into(),
                power: 4,
                toughness: 4,
                keywords: vec![Keyword::Flying],
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Angel],
                    ..Default::default()
                },
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

/// Ajani's Chosen — {2}{W}{W} 3/3 Cat Soldier. Whenever an enchantment you
/// control enters, create a 2/2 white Cat creature token. (The Aura-attach
/// rider is dropped — the token still enters.)
pub fn ajanis_chosen() -> CardDefinition {
    CardDefinition {
        name: "Ajani's Chosen",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![constellation(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Cat".into(),
                power: 2,
                toughness: 2,
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Cat],
                    ..Default::default()
                },
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

// ── Animate-enchantment statics ──────────────────────────────────────────────

/// Opalescence — {2}{W}{W} Enchantment. Each other non-Aura enchantment is a
/// creature with base power and toughness each equal to its mana value.
pub fn opalescence() -> CardDefinition {
    CardDefinition {
        name: "Opalescence",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Each other non-Aura enchantment is a creature with power and toughness each equal to its mana value.",
            effect: StaticEffect::NonAuraEnchantmentsAreCreatures {
                yours_only: false,
                requires_five: false,
            },
        }],
        ..Default::default()
    }
}

/// Starfield of Nyx — {4}{W} Enchantment. At your upkeep you may return an
/// enchantment card from your graveyard. While you control 5+ enchantments,
/// each other non-Aura enchantment you control is an `MV/MV` creature.
pub fn starfield_of_nyx() -> CardDefinition {
    CardDefinition {
        name: "Starfield of Nyx",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::SelfSource,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::MayDo {
                description: "Return an enchantment card from your graveyard to the battlefield?"
                    .into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(SelectionRequirement::Enchantment),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                }),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "As long as you control five or more enchantments, each other non-Aura enchantment you control is an MV/MV creature.",
            effect: StaticEffect::NonAuraEnchantmentsAreCreatures {
                yours_only: true,
                requires_five: true,
            },
        }],
        ..Default::default()
    }
}

// ── Pillowfort / protection statics ──────────────────────────────────────────

/// Privileged Position — {2}{G/W}{G/W}{G/W} Enchantment. Other permanents you
/// control have hexproof.
pub fn privileged_position() -> CardDefinition {
    CardDefinition {
        name: "Privileged Position",
        cost: cost(&[
            generic(2),
            hybrid(Color::Green, Color::White),
            hybrid(Color::Green, Color::White),
            hybrid(Color::Green, Color::White),
        ]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Other permanents you control have hexproof.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours_other(None),
                keyword: Keyword::Hexproof,
            },
        }],
        ..Default::default()
    }
}

/// Greater Auramancy — {1}{W} Enchantment. Other enchantments you control have
/// shroud. (The "enchanted creatures you control have shroud" clause is
/// approximated to the enchantment-only grant.)
pub fn greater_auramancy() -> CardDefinition {
    CardDefinition {
        name: "Greater Auramancy",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Other enchantments you control have shroud.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours_other(Some(SelectionRequirement::Enchantment)),
                keyword: Keyword::Shroud,
            },
        }],
        ..Default::default()
    }
}

/// Nevermore — {1}{W}{W} Enchantment. As this enters, name a nonland card.
/// Spells with the chosen name can't be cast.
pub fn nevermore() -> CardDefinition {
    CardDefinition {
        name: "Nevermore",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::NameCard {
                what: Selector::This,
                restrict_to: None,
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Spells with the chosen name can't be cast.",
            effect: StaticEffect::NamedSpellCantBeCast,
        }],
        ..Default::default()
    }
}

/// Grasp of Fate — {1}{W}{W} Enchantment. ETB: exile a nonland permanent an
/// opponent controls until this leaves. (Printed "for each opponent, up to
/// one" — modeled as a single exile, faithful in 1v1.)
pub fn grasp_of_fate() -> CardDefinition {
    CardDefinition {
        name: "Grasp of Fate",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ExileUntilSourceLeaves {
                what: target_filtered(
                    SelectionRequirement::Nonland.and(SelectionRequirement::ControlledByOpponent),
                ),
                return_to: ExileReturnZone::Battlefield,
            },
        }],
        ..Default::default()
    }
}

// ── Constellation utility ────────────────────────────────────────────────────

/// Season of Growth — {1}{G} Enchantment. Whenever a creature you control
/// enters, scry 1. Whenever you cast a spell that targets a creature you
/// control, draw a card.
pub fn season_of_growth() -> CardDefinition {
    CardDefinition {
        name: "Season of Growth",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Creature,
                    }),
                effect: Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::CastSpellTargetsMatch(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                ),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

// ── Death-watching enchantments ──────────────────────────────────────────────

/// Sigil of the New Dawn — {3}{W} Enchantment. Whenever a creature you control
/// is put into your graveyard from the battlefield, you may pay {1}{W}; if you
/// do, return that card to your hand.
pub fn sigil_of_the_new_dawn() -> CardDefinition {
    CardDefinition {
        name: "Sigil of the New Dawn",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Return the dead creature to your hand?".into(),
                mana_cost: cost(&[generic(1), w()]),
                body: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Calix, Guided by Fate — {1}{G}{W} 2/2 legendary Human Druid. Constellation:
/// put a +1/+1 counter on target creature. (The combat-copy ability is dropped
/// — the constellation payoff is modeled.)
pub fn calix_guided_by_fate() -> CardDefinition {
    CardDefinition {
        name: "Calix, Guided by Fate",
        cost: cost(&[generic(1), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Enchantment,
                }),
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

// ── Batch 2: death-recursion, pillowfort, Auras ──────────────────────────────

/// Angelic Renewal — {1}{W} Enchantment. Whenever a creature you control dies,
/// you may sacrifice this enchantment; if you do, return that card to the
/// battlefield.
pub fn angelic_renewal() -> CardDefinition {
    CardDefinition {
        name: "Angelic Renewal",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Sacrifice Angelic Renewal to return the dead creature?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Graveyard,
                    },
                    Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Aura Fracture — {2}{W} Enchantment. Sacrifice a land: Destroy target
/// enchantment.
pub fn aura_fracture() -> CardDefinition {
    CardDefinition {
        name: "Aura Fracture",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::default(),
            sac_other_filter: Some((SelectionRequirement::Land, 1)),
            effect: Effect::Destroy {
                what: target_filtered(SelectionRequirement::Enchantment),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Solitary Confinement — {2}{W} Enchantment. Skip your draw step; you have
/// hexproof; prevent all damage that would be dealt to you. At your upkeep,
/// sacrifice this unless you discard a card. (Printed "shroud" is modeled as
/// hexproof — the self-target case is vanishingly rare.)
pub fn solitary_confinement() -> CardDefinition {
    CardDefinition {
        name: "Solitary Confinement",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::SelfSource,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::MayDiscard {
                description: "Discard a card to keep Solitary Confinement?".into(),
                count: Value::ONE,
                then: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Graveyard,
                })),
            },
        }],
        static_abilities: vec![
            StaticAbility {
                description: "Skip your draw step.",
                effect: StaticEffect::SkipStep {
                    step: TurnStep::Draw,
                    all_players: false,
                },
            },
            StaticAbility {
                description: "You have hexproof.",
                effect: StaticEffect::ControllerHasHexproof,
            },
            StaticAbility {
                description: "Prevent all damage that would be dealt to you.",
                effect: StaticEffect::PreventAllDamageToController,
            },
        ],
        ..Default::default()
    }
}

/// Shielded by Faith — {1}{W}{W} Aura. Enchanted creature has indestructible.
/// (The "attach on any creature entering" rider is dropped.)
pub fn shielded_by_faith() -> CardDefinition {
    CardDefinition {
        name: "Shielded by Faith",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Indestructible],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Unquestioned Authority — {2}{W} Aura. ETB: draw a card. Enchanted creature
/// has protection from creatures.
pub fn unquestioned_authority() -> CardDefinition {
    CardDefinition {
        name: "Unquestioned Authority",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::ProtectionFromCreatures],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Sacred Mesa — {2}{W} Enchantment. {1}{W}: Create a 1/1 white Pegasus with
/// flying. At your upkeep, sacrifice this unless you sacrifice a Pegasus.
pub fn sacred_mesa() -> CardDefinition {
    CardDefinition {
        name: "Sacred Mesa",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::SelfSource,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::MaySacrifice {
                description: "Sacrifice a Pegasus to keep Sacred Mesa?".into(),
                filter: SelectionRequirement::HasCreatureType(CreatureType::Pegasus),
                count: Value::ONE,
                then: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Graveyard,
                })),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Pegasus".into(),
                    power: 1,
                    toughness: 1,
                    keywords: vec![Keyword::Flying],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Pegasus],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Aegis of the Gods — {1}{W} 2/1 Human Soldier enchantment creature. You have
/// hexproof.
pub fn aegis_of_the_gods() -> CardDefinition {
    CardDefinition {
        name: "Aegis of the Gods",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "You have hexproof.",
            effect: StaticEffect::ControllerHasHexproof,
        }],
        ..Default::default()
    }
}

/// Frozen Aether — {3}{U} Enchantment. Artifacts, creatures, and lands your
/// opponents control enter the battlefield tapped.
pub fn frozen_aether() -> CardDefinition {
    use crate::mana::u;
    CardDefinition {
        name: "Frozen Aether",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Artifacts, creatures, and lands your opponents control enter tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::ControlledByOpponent.and(
                        SelectionRequirement::Artifact
                            .or(SelectionRequirement::Creature)
                            .or(SelectionRequirement::Land),
                    ),
                ),
            },
        }],
        ..Default::default()
    }
}

// ── Batch 3: Aura beats + enchantment removal ────────────────────────────────

/// Griffin Guide — {2}{W} Aura. Enchanted creature gets +2/+2 and has flying.
/// When it dies, create a 2/2 white Griffin with flying.
pub fn griffin_guide() -> CardDefinition {
    CardDefinition {
        name: "Griffin Guide",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Griffin".into(),
                    power: 2,
                    toughness: 2,
                    keywords: vec![Keyword::Flying],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Griffin],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Angelic Destiny — {2}{W}{W} Aura. Enchanted creature gets +4/+4 and has
/// flying and first strike. When it dies, return this to its owner's hand.
/// (The "is an Angel" type-add rider is dropped.)
pub fn angelic_destiny() -> CardDefinition {
    CardDefinition {
        name: "Angelic Destiny",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 4,
            toughness: 4,
            keywords: vec![Keyword::Flying, Keyword::FirstStrike],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Tranquil Grove — {1}{G} Enchantment. {1}{G}{G}: Destroy all other
/// enchantments.
pub fn tranquil_grove() -> CardDefinition {
    CardDefinition {
        name: "Tranquil Grove",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g(), g()]),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Enchantment.and(SelectionRequirement::OtherThanSource),
                ),
                body: Box::new(Effect::Destroy {
                    what: Selector::TriggerSource,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cho-Manno's Blessing — {W}{W} Aura. Flash. As it enters, choose a color;
/// enchanted creature has protection from that color.
pub fn cho_mannos_blessing() -> CardDefinition {
    CardDefinition {
        name: "Cho-Manno's Blessing",
        cost: cost(&[w(), w()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseColorForSelf,
        }],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has protection from the chosen color.",
            effect: StaticEffect::GrantProtectionFromChosenColor {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..Default::default()
    }
}

/// Flickering Ward — {W} Aura. As it enters, choose a color; enchanted creature
/// has protection from that color. {W}: Return this Aura to its owner's hand.
pub fn flickering_ward() -> CardDefinition {
    CardDefinition {
        name: "Flickering Ward",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseColorForSelf,
        }],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has protection from the chosen color.",
            effect: StaticEffect::GrantProtectionFromChosenColor {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Batch 4: constellation payoffs, enchantment recursion, ramp ──────────────

/// Doomwake Giant — {4}{B} 4/6 Giant. Constellation — whenever this or another
/// enchantment you control enters, creatures your opponents control get -1/-1
/// until end of turn.
pub fn doomwake_giant() -> CardDefinition {
    CardDefinition {
        name: "Doomwake Giant",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        triggered_abilities: vec![constellation(Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Auramancer — {2}{W} 2/2 Human Wizard. ETB: you may return an enchantment
/// card from your graveyard to your hand.
pub fn auramancer() -> CardDefinition {
    CardDefinition {
        name: "Auramancer",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Return an enchantment card from your graveyard to your hand?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(SelectionRequirement::Enchantment),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Monk Idealist — {2}{W} 2/2 Human Monk Cleric. ETB: return an enchantment
/// card from your graveyard to your hand.
pub fn monk_idealist() -> CardDefinition {
    CardDefinition {
        name: "Monk Idealist",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Monk,
                CreatureType::Cleric,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Enchantment),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Commune with the Gods — {1}{G} Sorcery. Reveal the top five cards; put a
/// creature or enchantment card among them into your hand, the rest into your
/// graveyard.
pub fn commune_with_the_gods() -> CardDefinition {
    CardDefinition {
        name: "Commune with the Gods",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(5),
            rest_to_graveyard: true,
            pick_filter: Some(SelectionRequirement::Creature.or(SelectionRequirement::Enchantment)),
    ..Default::default()
})),
        ..Default::default()
    }
}

/// Wildwood Rebirth — {1}{G} Instant. Return target creature card from your
/// graveyard to your hand.
pub fn wildwood_rebirth() -> CardDefinition {
    CardDefinition {
        name: "Wildwood Rebirth",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Nylea's Presence — {1}{G} Aura. Enchant land. ETB draw a card. Enchanted
/// land is every basic land type in addition to its other types.
pub fn nyleas_presence() -> CardDefinition {
    CardDefinition {
        name: "Nylea's Presence",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Land,
            },
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Enchanted land is every basic land type.",
            effect: StaticEffect::GrantAllBasicLandTypes {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..Default::default()
    }
}

/// Font of Fertility — {G} Enchantment. {1}{G}, Sacrifice this: Search your
/// library for a basic land card and put it onto the battlefield tapped, then
/// shuffle.
pub fn font_of_fertility() -> CardDefinition {
    CardDefinition {
        name: "Font of Fertility",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Batch 5: enchantment removal, Auras, and a God ───────────────────────────

/// Serene Heart — {1}{G} Instant. Destroy all Auras.
pub fn serene_heart() -> CardDefinition {
    CardDefinition {
        name: "Serene Heart",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::HasEnchantmentSubtype(
                EnchantmentSubtype::Aura,
            )),
            body: Box::new(Effect::Destroy {
                what: Selector::TriggerSource,
            }),
        },
        ..Default::default()
    }
}

/// Winds of Rath — {3}{W}{W} Sorcery. Destroy all creatures that aren't
/// enchanted.
pub fn winds_of_rath() -> CardDefinition {
    CardDefinition {
        name: "Winds of Rath",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature.and(
                SelectionRequirement::Not(Box::new(SelectionRequirement::IsEnchanted)),
            )),
            body: Box::new(Effect::Destroy {
                what: Selector::TriggerSource,
            }),
        },
        ..Default::default()
    }
}

/// Calming Verse — {3}{G} Sorcery. Destroy all enchantments you don't control.
/// (The reflexive "then destroy your own if you control an untapped land"
/// clause is dropped.)
pub fn calming_verse() -> CardDefinition {
    CardDefinition {
        name: "Calming Verse",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Enchantment.and(SelectionRequirement::ControlledByOpponent),
            ),
            body: Box::new(Effect::Destroy {
                what: Selector::TriggerSource,
            }),
        },
        ..Default::default()
    }
}

/// Root Out — {2}{G} Sorcery. Destroy target artifact or enchantment, then
/// investigate.
pub fn root_out() -> CardDefinition {
    CardDefinition {
        name: "Root Out",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
            crate::effect::shortcut::investigate(1),
        ]),
        ..Default::default()
    }
}
