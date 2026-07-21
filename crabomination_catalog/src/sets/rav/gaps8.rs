//! Ravnica (RAV) gap wave 8: Radiance spells (now that `RadianceGroup` fans
//! out over any shared card type), utility lands, guild value creatures, and
//! a spread of artifacts/enchantments on existing primitives. Tests in
//! `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EntersAsCopy, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

fn saproling_token() -> TokenDefinition {
    TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Saproling], ..Default::default() },
        ..Default::default()
    }
}

/// Voja — legendary 2/2 green-and-white Wolf token (Tolsimir Wolfblood).
fn voja_token() -> TokenDefinition {
    TokenDefinition {
        name: "Voja".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::White],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        ..Default::default()
    }
}

/// 1/1 white Spirit with flying (Transluminant, Twilight Drover).
fn white_spirit_flyer() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        ..Default::default()
    }
}

// ── Radiance spells ──────────────────────────────────────────────────────────

/// Surge of Zeal — {R} Instant. Radiance — target creature and each other
/// creature that shares a color with it gain haste until end of turn.
pub fn surge_of_zeal() -> CardDefinition {
    CardDefinition {
        name: "Surge of Zeal",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GrantKeyword {
            what: Selector::RadianceGroup { subject: Box::new(target_filtered(R::Creature)) },
            keyword: Keyword::Haste,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Incite Hysteria — {2}{R} Sorcery. Radiance — until end of turn, target
/// creature and each other creature that shares a color with it can't block.
pub fn incite_hysteria() -> CardDefinition {
    CardDefinition {
        name: "Incite Hysteria",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::GrantKeyword {
            what: Selector::RadianceGroup { subject: Box::new(target_filtered(R::Creature)) },
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Leave No Trace — {1}{W} Instant. Radiance — destroy target enchantment and
/// each other enchantment that shares a color with it.
pub fn leave_no_trace() -> CardDefinition {
    CardDefinition {
        name: "Leave No Trace",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: Selector::RadianceGroup { subject: Box::new(target_filtered(R::Enchantment)) },
        },
        ..Default::default()
    }
}

/// Bathe in Light — {1}{W} Instant. Radiance — choose a color. Target creature
/// and each other creature that shares a color with it gain protection from the
/// chosen color until end of turn.
pub fn bathe_in_light() -> CardDefinition {
    CardDefinition {
        name: "Bathe in Light",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GrantProtectionFromChosenColor {
            what: Selector::RadianceGroup { subject: Box::new(target_filtered(R::Creature)) },
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

// ── Blue counters / mill ─────────────────────────────────────────────────────

/// Induce Paranoia — {2}{U}{U} Instant. Counter target spell. If {B} was spent
/// to cast this spell, that spell's controller mills X, where X is its mana value.
pub fn induce_paranoia() -> CardDefinition {
    CardDefinition {
        name: "Induce Paranoia",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::ManaSpentOfColorAtLeast { color: Color::Black, at_least: 1 },
                then: Box::new(Effect::Mill {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::CounterSpell { what: Selector::Target(0) },
        ]),
        ..Default::default()
    }
}

/// Mnemonic Nexus — {3}{U} Instant. Each player shuffles their graveyard into
/// their library.
pub fn mnemonic_nexus() -> CardDefinition {
    CardDefinition {
        name: "Mnemonic Nexus",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::EachPlayer },
        ..Default::default()
    }
}

// ── Red ──────────────────────────────────────────────────────────────────────

/// Flash Conscription — {5}{R} Instant. Untap target creature and gain control
/// of it until end of turn; it gains haste. If {W} was spent, it also gains
/// lifelink until end of turn.
pub fn flash_conscription() -> CardDefinition {
    CardDefinition {
        name: "Flash Conscription",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GainControl {
                what: Selector::Target(0),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::ManaSpentOfColorAtLeast { color: Color::White, at_least: 1 },
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Hex — {4}{B}{B} Sorcery. Destroy six target creatures.
pub fn hex() -> CardDefinition {
    CardDefinition {
        name: "Hex",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 6,
            min_targets: 6,
            filter: R::Creature,
            effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
        },
        ..Default::default()
    }
}

/// Excruciator — {6}{R}{R} 7/7 Avatar. Damage it would deal can't be prevented.
pub fn excruciator() -> CardDefinition {
    CardDefinition {
        name: "Excruciator",
        cost: cost(&[generic(6), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Avatar], ..Default::default() },
        power: 7,
        toughness: 7,
        static_abilities: vec![StaticAbility {
            description: "Damage that would be dealt by this creature can't be prevented.",
            effect: StaticEffect::SourceDamageCantBePrevented,
        }],
        ..Default::default()
    }
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Helldozer — {3}{B}{B}{B} 6/5 Zombie Giant. {B}{B}{B}, {T}: Destroy target
/// land. If it was nonbasic, untap this creature.
pub fn helldozer() -> CardDefinition {
    CardDefinition {
        name: "Helldozer",
        cost: cost(&[generic(3), b(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Giant],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[b(), b(), b()]),
            effect: Effect::Seq(vec![
                // Read the nonbasic clause before the land leaves the battlefield.
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: R::IsNonbasicLand,
                    },
                    then: Box::new(Effect::Untap { what: Selector::This, up_to: None }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::Destroy { what: target_filtered(R::Land) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tolsimir Wolfblood — {4}{G}{W} 3/4 Elf Warrior. Other green creatures you
/// control and other white creatures you control get +1/+1. {T}: Create Voja.
pub fn tolsimir_wolfblood() -> CardDefinition {
    let anthem = |color: Color| StaticAbility {
        description: "Other creatures you control of a color get +1/+1.",
        effect: StaticEffect::AnthemForFilter {
            filter: R::Creature.and(R::OtherThanSource).and(R::HasColor(color)),
            power: 1,
            toughness: 1,
            keywords: vec![],
            opponents: false,
            only_your_turn: false,
            scale_by_counters_on_self: None,
        },
    };
    CardDefinition {
        name: "Tolsimir Wolfblood",
        cost: cost(&[generic(4), g(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        static_abilities: vec![anthem(Color::Green), anthem(Color::White)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: voja_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Woodwraith Strangler — {2}{B}{G} 2/2 Plant Zombie. Exile a creature card
/// from your graveyard: Regenerate this creature.
pub fn woodwraith_strangler() -> CardDefinition {
    CardDefinition {
        name: "Woodwraith Strangler",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            exile_other_filter: Some((R::Creature, 1)),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Transluminant — {1}{G} 2/2 Dryad Shaman. {W}, Sacrifice this creature:
/// Create a 1/1 white Spirit with flying at the beginning of the next end step.
pub fn transluminant() -> CardDefinition {
    CardDefinition {
        name: "Transluminant",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dryad, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            mana_cost: cost(&[w()]),
            effect: Effect::AtNextEndStep {
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: white_spirit_flyer(),
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stone-Seeder Hierophant — {2}{G}{G} 1/1 Human Druid. Landfall — whenever a
/// land you control enters, untap this creature. {T}: Untap target land.
pub fn stone_seeder_hierophant() -> CardDefinition {
    CardDefinition {
        name: "Stone-Seeder Hierophant",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Untap { what: target_filtered(R::Land), up_to: None },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Utility lands ────────────────────────────────────────────────────────────

/// Duskmantle, House of Shadow — Land. {T}: Add {C}. {U}{B}, {T}: Target player
/// mills a card.
pub fn duskmantle_house_of_shadow() -> CardDefinition {
    CardDefinition {
        name: "Duskmantle, House of Shadow",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[u(), b()]),
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Sunhome, Fortress of the Legion — Land. {T}: Add {C}. {2}{R}{W}, {T}: Target
/// creature gains double strike until end of turn.
pub fn sunhome_fortress_of_the_legion() -> CardDefinition {
    CardDefinition {
        name: "Sunhome, Fortress of the Legion",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2), r(), w()]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Vitu-Ghazi, the City-Tree — Land. {T}: Add {C}. {2}{G}{W}, {T}: Create a
/// 1/1 green Saproling.
pub fn vitu_ghazi_the_city_tree() -> CardDefinition {
    CardDefinition {
        name: "Vitu-Ghazi, the City-Tree",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2), g(), w()]),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: saproling_token(),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Artifacts / enchantments ─────────────────────────────────────────────────

/// Copy Enchantment — {2}{U} Enchantment. You may have it enter as a copy of
/// any enchantment on the battlefield.
pub fn copy_enchantment() -> CardDefinition {
    CardDefinition {
        name: "Copy Enchantment",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        enters_as_copy: Some(EntersAsCopy { filter: R::Enchantment, ..Default::default() }),
        ..Default::default()
    }
}

/// Glare of Subdual — {2}{G}{W} Enchantment. Tap an untapped creature you
/// control: Tap target artifact or creature.
pub fn glare_of_subdual() -> CardDefinition {
    CardDefinition {
        name: "Glare of Subdual",
        cost: cost(&[generic(2), g(), w()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::Creature.and(R::ControlledByYou)),
            effect: Effect::Tap { what: target_filtered(R::Artifact.or(R::Creature)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Voyager Staff — {1} Artifact. {2}, Sacrifice this artifact: Exile target
/// creature, returning it under its owner's control at the next end step.
pub fn voyager_staff() -> CardDefinition {
    CardDefinition {
        name: "Voyager Staff",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::ExileReturnNextEndStep { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Twilight Drover — {2}{W} 1/1 Spirit. Whenever a creature token leaves the
/// battlefield, put a +1/+1 counter on it. {2}{W}, Remove a +1/+1 counter:
/// Create two 1/1 white Spirit tokens with flying.
pub fn twilight_drover() -> CardDefinition {
    CardDefinition {
        name: "Twilight Drover",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::IsToken),
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: white_spirit_flyer(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
