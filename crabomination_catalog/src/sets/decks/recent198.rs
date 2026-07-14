//! DSK gap batch on existing primitives: Baseball Bat (auto-attach Equipment
//! with an attack-tap granted trigger). Tests in `tests/recent198.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, EquipBonus, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, Selector};
use crate::mana::{cost, g, generic, w};

/// Baseball Bat — {G}{W} Equipment. ETB attach to a creature you control;
/// equipped creature gets +1/+1 and taps up to one target creature when it
/// attacks. Equip {3}.
pub fn baseball_bat() -> CardDefinition {
    CardDefinition {
        name: "Baseball Bat",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Tap { what: target_filtered(R::Creature) },
            }],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        ..Default::default()
    }
}
