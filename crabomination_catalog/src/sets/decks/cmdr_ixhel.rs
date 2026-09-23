//! Commander: the cards the **Corrupting Influence** precon (ONC, Ixhel,
//! Scion of Atraxa) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Selector, Subtypes, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::catalog::sets::enters_tapped;
use crate::effect::shortcut::etb;
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate};
use crate::mana::{Color, b, cost, g, generic, w};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// "An opponent has three or more poison counters" (CR 702.166's ability
/// word, Corrupted).
fn corrupted() -> Predicate {
    Predicate::CorruptedActive { who: PlayerRef::You }
}

/// Carrion Call — two 1/1 infect Insects at instant speed.
pub fn carrion_call() -> CardDefinition {
    CardDefinition {
        name: "Carrion Call",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: Arc::new(TokenDefinition {
                name: "Phyrexian Insect".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Phyrexian, CreatureType::Insect],
                    ..Default::default()
                },
                keywords: vec![Keyword::Infect],
                ..Default::default()
            }),
        },
        ..Default::default()
    }
}

/// Glistening Sphere — a tapped mana rock that proliferates, and makes three
/// of one color while an opponent is corrupted. ⚠ The corrupted ability is a
/// conditional mana ability, which auto-payment does not tap; it is
/// activated directly.
pub fn glistening_sphere() -> CardDefinition {
    CardDefinition {
        name: "Glistening Sphere",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![enters_tapped()],
        triggered_abilities: vec![etb(Effect::Proliferate)],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(corrupted()),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(3)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ichor Rats — infect, and a poison counter for every player.
pub fn ichor_rats() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        triggered_abilities: vec![etb(Effect::AddPoison {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::ONE,
        })],
        ..creature(
            "Ichor Rats",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Rat],
            2,
            1,
        )
    }
}

/// Norn's Choirmaster — proliferate whenever a commander of yours enters or
/// attacks.
pub fn norns_choirmaster() -> CardDefinition {
    let commander = || Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsCommander };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(commander()),
                effect: Effect::Proliferate,
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(commander()),
                effect: Effect::Proliferate,
            },
        ],
        ..creature(
            "Norn's Choirmaster",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Phyrexian, CreatureType::Angel],
            5,
            4,
        )
    }
}
