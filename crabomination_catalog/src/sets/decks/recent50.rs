//! Blue/Izzet/Simic spellslinger value. Tests in `tests/recent50.rs`.

use crate::card::{
    AlternativeCost, CardDefinition, CardType, CreatureType, DynamicPt, Effect, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes,
    TriggeredAbility,
};
use crate::effect::shortcut::{investigate, target_filtered};
use crate::mana::{cost, g, generic, r, u};

fn etb(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect,
    }
}

/// Enigma Drake — {1}{U}{R} */4 Drake with flying. Power = the number of instant
/// and sorcery cards in your graveyard.
pub fn enigma_drake() -> CardDefinition {
    CardDefinition {
        name: "Enigma Drake",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        dynamic_pt: Some(DynamicPt::InstantsSorceriesInControllerGraveyard { base_t: 4 }),
        ..Default::default()
    }
}

/// Niblis of Frost — {2}{U}{U} 3/3 Spirit with flying and prowess. Whenever you
/// cast an instant or sorcery spell, tap target creature an opponent controls and
/// it doesn't untap during its controller's next untap step.
pub fn niblis_of_frost() -> CardDefinition {
    CardDefinition {
        name: "Niblis of Frost",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Prowess],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                },
                Effect::SkipNextUntap {
                    what: Selector::Target(0),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Wavesifter — {3}{G}{U} 3/2 Elemental with flying. ETB investigate twice. Evoke
/// {G}{U}.
pub fn wavesifter() -> CardDefinition {
    CardDefinition {
        name: "Wavesifter",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(investigate(2))],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[g(), u()]),
            evoke_sacrifice: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}
