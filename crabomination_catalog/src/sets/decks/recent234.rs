//! Gap batch — OTJ Spree + an Equipment, on existing primitives. Tests in
//! `tests/recent234.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, Keyword, SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, SpreeMode, Value,
};
use crate::mana::{b, cost, g, generic, w, Color, ManaCost};

fn spree(modes: Vec<SpreeMode>) -> Effect {
    Effect::Spree { modes }
}
fn mode(c: ManaCost, effect: Effect) -> SpreeMode {
    SpreeMode { cost: c, effect }
}

/// Trash the Town — {G} Instant. Spree: +{2} put two +1/+1 counters on target
/// creature; +{1} target creature gains trample; +{1} target creature gains
/// "Whenever this creature deals combat damage to a player, draw two cards."
pub fn trash_the_town() -> CardDefinition {
    CardDefinition {
        name: "Trash the Town",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: spree(vec![
            mode(
                cost(&[generic(2)]),
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ),
            mode(
                cost(&[generic(1)]),
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ),
            mode(
                cost(&[generic(1)]),
                Effect::GrantTriggeredAbility {
                    what: target_filtered(R::Creature),
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(
                            EventKind::DealsCombatDamageToPlayer,
                            EventScope::SelfSource,
                        ),
                        effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                    }),
                    duration: Duration::EndOfTurn,
                },
            ),
        ]),
        ..Default::default()
    }
}

fn mercenary_token() -> TokenDefinition {
    TokenDefinition {
        name: "Mercenary".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mercenary], ..Default::default() },
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Unfortunate Accident — {B} Instant. Spree: +{2}{B} destroy target creature;
/// +{1} create a 1/1 red Mercenary with "{T}: Target creature you control gets
/// +1/+0 until end of turn. Activate only as a sorcery."
pub fn unfortunate_accident() -> CardDefinition {
    CardDefinition {
        name: "Unfortunate Accident",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: spree(vec![
            mode(
                cost(&[generic(2), b()]),
                Effect::Destroy { what: target_filtered(R::Creature) },
            ),
            mode(
                cost(&[generic(1)]),
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: mercenary_token(),
                },
            ),
        ]),
        ..Default::default()
    }
}

/// Thunder Lasso — {2}{W} Artifact — Equipment. When it enters, attach it to
/// target creature you control. Equipped creature gets +1/+1. Whenever equipped
/// creature attacks, tap target creature defending player controls. Equip {2}.
pub fn thunder_lasso() -> CardDefinition {
    CardDefinition {
        name: "Thunder Lasso",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![on_attack(Effect::Tap {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
}
