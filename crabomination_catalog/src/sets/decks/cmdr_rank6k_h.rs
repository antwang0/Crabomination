//! Commander: two-card gaps of EDHREC's commanders ranked 5501-6500 (the
//! 2026-10-14 scan), eighth file. Tests in `tests/recent_b/cmdr_rank6k_h.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    LandType, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::{Duration, Effect, ExtraManaKind, Selector};
use crate::game::types::TurnStep;
use crate::mana::{Color, cost, g, generic, u};

/// Utrom Monitor — {4}{U} 3/3 Utrom Scientist artifact creature, affinity for
/// artifacts, flying.
pub fn utrom_monitor() -> CardDefinition {
    CardDefinition {
        name: "Utrom Monitor",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Utrom, CreatureType::Scientist],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        affinity_filter: Some(R::Artifact),
        ..Default::default()
    }
}

/// Keeper of Progenitus — {3}{G} 1/3 Elf Druid. Whenever a player taps a
/// Mountain, Forest or Plains for mana, they add one more of a type it made.
pub fn keeper_of_progenitus() -> CardDefinition {
    CardDefinition {
        name: "Keeper of Progenitus",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Whenever a player taps a Mountain, Forest, or Plains for mana, that player adds one mana of any type that land produced.",
            effect: StaticEffect::ExtraManaOnLandTap {
                enchanted_only: false,
                filter: R::HasLandType(LandType::Mountain)
                    .or(R::HasLandType(LandType::Forest))
                    .or(R::HasLandType(LandType::Plains)),
                extra: ExtraManaKind::Mirror,
                while_monarch: false,
            },
        }],
        ..Default::default()
    }
}

/// Hydro-Man, Fluid Felon — {U}{U} 2/2 legendary Elemental Villain. Whenever
/// you cast a blue spell, if he's a creature, +1/+1 until end of turn. At
/// your end step, untap him; until your next turn he becomes a land with
/// "{T}: Add {U}". (Residual: he becomes a basic Island — the type change
/// adds the Island subtype and basic supertype and drops his name's
/// legendary supertype and abilities rather than only the creature type.)
pub fn hydro_man_fluid_felon() -> CardDefinition {
    CardDefinition {
        name: "Hydro-Man, Fluid Felon",
        cost: cost(&[u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Villain],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasColor(Color::Blue) },
                    Predicate::SourceIsCreature,
                ])),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::Untap { what: Selector::This, up_to: None },
                    Effect::BecomeBasicLand {
                        what: Selector::This,
                        land_type: LandType::Island,
                        duration: Duration::UntilNextTurn,
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}
