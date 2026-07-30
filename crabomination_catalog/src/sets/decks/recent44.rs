//! Artifact/enchantment hate, ability-tax hate-bears, and an ETB-suppressor.
//! Tests in `tests/recent44.rs`.

use crate::card::{
    AlternativeCost, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{ActivatedAbility, PlayerRef};
use crate::game::types::TurnStep;
use crate::mana::{Color, cost, g, generic, hybrid, r, u, w};

fn etb(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect,
    }
}

fn destroy_artifact_target() -> Effect {
    Effect::Destroy {
        what: target_filtered(R::Artifact),
    }
}

/// Energy Flux — {1}{U} Enchantment. All artifacts have "At the beginning of
/// your upkeep, sacrifice this artifact unless you pay {2}."
pub fn energy_flux() -> CardDefinition {
    CardDefinition {
        name: "Energy Flux",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "All artifacts have \"At the beginning of your upkeep, sacrifice this artifact unless you pay {2}.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Artifact,
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::StepBegins(TurnStep::Upkeep),
                        EventScope::YourControl,
                    ),
                    effect: Effect::UnlessPlayerPays {
                        who: PlayerRef::You,
                        cost: WardCost::generic(2),
                        then: Box::new(Effect::SacrificeSource),
                    },
                }),
            },
        }],
        ..Default::default()
    }
}

/// Uktabi Orangutan — {2}{G} 2/2 Ape. ETB destroy target artifact.
pub fn uktabi_orangutan() -> CardDefinition {
    CardDefinition {
        name: "Uktabi Orangutan",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ape],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(destroy_artifact_target())],
        ..Default::default()
    }
}

/// Ingot Chewer — {4}{R} 3/2 Elemental. ETB destroy target artifact. Evoke {R}.
pub fn ingot_chewer() -> CardDefinition {
    CardDefinition {
        name: "Ingot Chewer",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(destroy_artifact_target())],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[r()]),
            evoke_sacrifice: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Manglehorn — {1}{G}{G} 2/2 Beast. ETB destroy target artifact. Artifacts your
/// opponents control enter the battlefield tapped.
pub fn manglehorn() -> CardDefinition {
    CardDefinition {
        name: "Manglehorn",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(destroy_artifact_target())],
        static_abilities: vec![StaticAbility {
            description: "Artifacts your opponents control enter the battlefield tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::EachPermanent(R::Artifact.and(R::ControlledByOpponent)),
            },
        }],
        ..Default::default()
    }
}

/// Viridian Zealot — {1}{G} 2/2 Elf Warrior. {1}{G}, Sacrifice: Destroy
/// target artifact or enchantment.
pub fn viridian_zealot() -> CardDefinition {
    CardDefinition {
        name: "Viridian Zealot",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Artifact.or(R::Enchantment)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sundering Growth — {1}{W} Sorcery. Destroy target artifact or enchantment.
/// Populate.
pub fn sundering_growth() -> CardDefinition {
    CardDefinition {
        name: "Sundering Growth",
        cost: cost(&[hybrid(Color::Green, Color::White), hybrid(Color::Green, Color::White)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Artifact.or(R::Enchantment)),
            },
            Effect::Populate {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}

/// Glowrider — {1}{W} 1/1 Cleric. Noncreature spells cost {1} more to cast.
pub fn glowrider() -> CardDefinition {
    CardDefinition {
        name: "Glowrider",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "Noncreature spells cost {1} more to cast.",
            effect: StaticEffect::AdditionalCost {
                filter: R::Noncreature,
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Harsh Mentor — {1}{W} 2/2 Human Cleric. Whenever an opponent activates a
/// non-mana, non-loyalty ability, deal 2 damage to that player.
pub fn harsh_mentor() -> CardDefinition {
    CardDefinition {
        name: "Harsh Mentor",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivated, EventScope::OpponentControl),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Hushwing Gryff — {2}{W} 2/1 Griffin with flash and flying. Creatures entering
/// the battlefield don't cause abilities to trigger.
pub fn hushwing_gryff() -> CardDefinition {
    CardDefinition {
        name: "Hushwing Gryff",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Griffin],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Creatures entering the battlefield don't cause abilities to trigger.",
            effect: StaticEffect::SuppressCreatureEtbTriggers {
                also_dies: false,
                also_artifacts: false,
            },
        }],
        ..Default::default()
    }
}
