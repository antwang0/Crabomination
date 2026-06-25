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
