//! Return to Ravnica (RTR) gap wave 9: the guildmages, counter-and-burn spells,
//! Overload's Counterflux, a doesn't-untap Aura, and Grove of the Guardian.
//! Tests in `classic_sets/rtr`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CreatureType, Effect,
    EnchantmentSubtype, Keyword, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest, ZoneRef};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Tower Drake — {2}{U} 2/1 Drake with flying. {W}: gets +0/+1 until end of turn.
pub fn tower_drake() -> CardDefinition {
    CardDefinition {
        name: "Tower Drake",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ZERO,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Paralyzing Grasp — {2}{U} Aura. Enchant creature; it doesn't untap during its
/// controller's untap step.
pub fn paralyzing_grasp() -> CardDefinition {
    CardDefinition {
        name: "Paralyzing Grasp",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..Default::default()
    }
}

/// Essence Backlash — {2}{U}{R} Instant. Counter target creature spell. It deals
/// damage equal to that spell's power to its controller.
pub fn essence_backlash() -> CardDefinition {
    CardDefinition {
        name: "Essence Backlash",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::Creature)),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Counterflux — {U}{U}{R} Instant. Can't be countered. Counter target spell you
/// don't control. Overload {1}{U}{U}{R} (counter each spell you don't control).
pub fn counterflux() -> CardDefinition {
    let overload = Effect::ForEach {
        selector: Selector::EachMatching {
            zone: ZoneRef::Stack,
            filter: R::Any,
        },
        body: Box::new(Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::ControlledByOpponent,
            },
            then: Box::new(Effect::CounterSpell {
                what: Selector::TriggerSource,
            }),
            else_: Box::new(Effect::Noop),
        }),
    };
    CardDefinition {
        name: "Counterflux",
        cost: cost(&[u(), u(), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::CounterSpell {
            what: target_filtered(R::IsSpellOnStack.and(R::ControlledByOpponent)),
        },
        alternative_cost: Some(AlternativeCost {
            awaken: false,
            mana_cost: cost(&[generic(1), u(), u(), r()]),
            effect_override: Some(overload),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Destroy the Evidence — {4}{B} Sorcery. Destroy target land; its controller
/// mills from the top until they reveal a land, putting those into their graveyard.
pub fn destroy_the_evidence() -> CardDefinition {
    CardDefinition {
        name: "Destroy the Evidence",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Land),
            },
            Effect::MillUntilLands {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                lands: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Mercurial Chemister — {3}{U}{R} 2/3 Human Wizard. {U}, {T}: Draw a card.
/// {R}, {T}, Exile an instant or sorcery card from your graveyard: deals damage
/// equal to the exiled card's mana value to any target.
pub fn mercurial_chemister() -> CardDefinition {
    CardDefinition {
        name: "Mercurial Chemister",
        cost: cost(&[generic(3), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                tap_cost: true,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(
                            (R::HasCardType(CardType::Instant)
                                .or(R::HasCardType(CardType::Sorcery)))
                            .and(R::InYourGraveyard),
                        ),
                        to: ZoneDest::Exile,
                    },
                    Effect::DealDamage {
                        to: Selector::TargetFiltered {
                            slot: 1,
                            filter: R::Creature.or(R::Player).or(R::Planeswalker),
                        },
                        amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Rix Maadi Guildmage — {B}{R} 2/2 Human Shaman. {B}{R}: target blocking
/// creature gets -1/-1 until end of turn. {B}{R}: target player loses 1 life.
pub fn rix_maadi_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Rix Maadi Guildmage",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b(), r()]),
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::IsBlocking)),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            // "target player who lost life this turn loses 1 life" — the
            // "lost life this turn" restriction isn't a targeting filter yet.
            ActivatedAbility {
                mana_cost: cost(&[b(), r()]),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Grove of the Guardian — Land. {T}: Add {C}. {3}{G}{W}, {T}, Tap two untapped
/// creatures you control, Sacrifice this land: Create an 8/8 green and white
/// Elemental token with vigilance.
pub fn grove_of_the_guardian() -> CardDefinition {
    CardDefinition {
        name: "Grove of the Guardian",
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
                mana_cost: cost(&[generic(3), g(), w()]),
                tap_cost: true,
                sac_cost: true,
                tap_n_filter: Some((R::Creature, 2)),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: crate::card::TokenDefinition {
                        name: "Elemental".into(),
                        power: 8,
                        toughness: 8,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Green, Color::White],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Elemental],
                            ..Default::default()
                        },
                        keywords: vec![Keyword::Vigilance],
                        ..Default::default()
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
