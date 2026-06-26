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
                opponents: false,
            },
        }],
        ..Default::default()
    }
}
