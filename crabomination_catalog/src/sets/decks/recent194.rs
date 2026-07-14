//! OTJ gap batch on existing primitives: Double Down (copy outlaw spells),
//! Mystical Tether (O-Ring exile-until-leaves), High Noon (Rule-of-Law lock +
//! sac burn). Tests in `tests/recent194.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, ExileReturnZone, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect,
};
use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
use crate::effect::shortcut::{deal, etb, target_any, target_filtered};
use crate::effect::{Effect, Selector, Value};
use crate::mana::{cost, generic, r, u, w};

/// Double Down — {3}{U} Enchantment. Whenever you cast an outlaw spell, copy it.
pub fn double_down() -> CardDefinition {
    CardDefinition {
        name: "Double Down",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::IsOutlaw)),
            effect: Effect::CopySpellMayChooseTargets {
                what: Selector::TriggerSource,
                count: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Mystical Tether — {2}{W} Enchantment. ETB: exile target artifact or creature
/// an opponent controls until this leaves. (Flash-for-{2}-more is omitted, as
/// with the rest of the flash-rider enchantment cycle.)
pub fn mystical_tether() -> CardDefinition {
    CardDefinition {
        name: "Mystical Tether",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered((R::Artifact.or(R::Creature)).and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// High Noon — {1}{W} Enchantment. Each player can't cast more than one spell
/// each turn. {4}{R}, Sacrifice this enchantment: it deals 5 damage to any
/// target.
pub fn high_noon() -> CardDefinition {
    CardDefinition {
        name: "High Noon",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Each player can't cast more than one spell each turn.",
            effect: StaticEffect::OneSpellPerTurn,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), r()]),
            sac_cost: true,
            effect: deal(5, target_any()),
            ..Default::default()
        }],
        ..Default::default()
    }
}
