//! Zendikar Rising party (CR 700.18): payoffs that scale with `Value::PartyCount`.
//! Tests in `tests/recent58.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Selector, Subtypes, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, PlayerRef};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, w};

fn kor_warrior() -> TokenDefinition {
    TokenDefinition {
        name: "Kor Warrior".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Warrior], ..Default::default()
        },
        ..Default::default()
    }
}

/// Squad Commander — {3}{W} 3/3 Kor Warrior. ETB: make a 1/1 Kor Warrior for
/// each creature in your party. At combat on your turn, if you have a full
/// party, creatures you control get +1/+0 and gain indestructible until EOT.
pub fn squad_commander() -> CardDefinition {
    let team = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Squad Commander",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Warrior], ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You, count: Value::PartyCount, definition: kor_warrior(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
                effect: Effect::If {
                    cond: Predicate::ValueAtLeast(Value::PartyCount, Value::Const(4)),
                    then: Box::new(Effect::Seq(vec![
                        Effect::PumpPT {
                            what: team(), power: Value::Const(1), toughness: Value::Const(0),
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword {
                            what: team(), keyword: Keyword::Indestructible, duration: Duration::EndOfTurn,
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Kabira Outrider — {3}{W} 3/3 Human Warrior. ETB: target creature gets +1/+1
/// until end of turn for each creature in your party.
pub fn kabira_outrider() -> CardDefinition {
    CardDefinition {
        name: "Kabira Outrider",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior], ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::PartyCount,
            toughness: Value::PartyCount,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Tajuru Paragon — {1}{G} 3/2 Elf that's also a Cleric, Rogue, Warrior, and
/// Wizard, so it can fill any one party slot (still only one — CR 700.18).
/// Kicker {3}; if kicked, dig six for a creature card. (The "shares a creature
/// type" filter is approximated as any creature.)
pub fn tajuru_paragon() -> CardDefinition {
    CardDefinition {
        name: "Tajuru Paragon",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Elf, CreatureType::Cleric, CreatureType::Rogue,
                CreatureType::Warrior, CreatureType::Wizard,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Kicker(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(6),
                rest_to_graveyard: false,
                pick_filter: Some(R::Creature),
                take: None,
                to_battlefield: false,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}
