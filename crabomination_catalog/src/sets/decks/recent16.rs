//! A sixteenth wave — tribal/artifact payoffs (a tap-count drain, a death-mana
//! Construct, a chosen-type land and lord). Tests in
//! `crabomination/src/tests/recent16.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    SelectionRequirement, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, ManaPayload, PlayerRef};
use crate::game::types::TurnStep;
use crate::mana::{cost, generic};

/// Throne of the God-Pharaoh — {2} Legendary Artifact. At your end step, each
/// opponent loses life equal to the number of tapped creatures you control.
pub fn throne_of_the_god_pharaoh() -> CardDefinition {
    CardDefinition {
        name: "Throne of the God-Pharaoh",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::Tapped),
                    )),
                    filter: SelectionRequirement::Any,
                },
            },
        }],
        ..Default::default()
    }
}

/// Su-Chi — {4} Artifact Creature — Construct 4/4. When it dies, add {C}{C}{C}{C}.
pub fn su_chi() -> CardDefinition {
    CardDefinition {
        name: "Su-Chi",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(4)),
            },
        }],
        ..Default::default()
    }
}

/// Secluded Courtyard — Land. As it enters, choose a creature type. {T}: Add
/// {C}. {T}: Add one mana of any color, spendable only on a creature spell of
/// the chosen type. (The "or activate an ability of a creature of the chosen
/// type" half of the restriction is approximated to the cast clause.)
pub fn secluded_courtyard() -> CardDefinition {
    CardDefinition {
        name: "Secluded Courtyard",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![etb(Effect::NameCreatureType { what: Selector::This })],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::RestrictedToChosenTypePlain(Box::new(
                        ManaPayload::AnyOneColor(Value::Const(1)),
                    )),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Aeolipile — {2} Artifact. {1}, {T}, Sacrifice this artifact: it deals 2
/// damage to any target.
pub fn aeolipile() -> CardDefinition {
    CardDefinition {
        name: "Aeolipile",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Phyrexian Vault — {3} Artifact. {2}, {T}, Sacrifice a creature: Draw a card.
pub fn phyrexian_vault() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Vault",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vanquisher's Banner — {5} Artifact. As it enters, choose a creature type.
/// Creatures you control of the chosen type get +1/+1; whenever you cast a
/// spell of the chosen type, draw a card.
pub fn vanquishers_banner() -> CardDefinition {
    CardDefinition {
        name: "Vanquisher's Banner",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            etb(Effect::NameCreatureType { what: Selector::This }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(crate::effect::Predicate::TriggerObjectIsChosenType),
                effect: crate::effect::shortcut::draw(1),
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control of the chosen type get +1/+1.",
            effect: StaticEffect::AnthemForChosenType {
                power: 1,
                toughness: 1,
                exclude_source: false,
                opponents: false, per_counter: None },
        }],
        ..Default::default()
    }
}

/// Icon of Ancestry — {3} Artifact. As it enters, choose a creature type.
/// Creatures you control of the chosen type get +1/+1. (The {3}, {T} dig for a
/// creature of the chosen type is dropped.)
pub fn icon_of_ancestry() -> CardDefinition {
    CardDefinition {
        name: "Icon of Ancestry",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::NameCreatureType { what: Selector::This })],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control of the chosen type get +1/+1.",
            effect: StaticEffect::AnthemForChosenType {
                power: 1,
                toughness: 1,
                exclude_source: false,
                opponents: false, per_counter: None },
        }],
        ..Default::default()
    }
}
