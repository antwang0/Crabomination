//! Commander: 2026-10-19 (`01DJHkWk`) — the party pair (The Destined
//! Warrior, Tazri, Beacon of Unity: Base Camp, The Destined Warrior, Skyclave
//! Plunder, Practiced Tactics) and Captain America, Unbowed's Infinity
//! Formula and Mandroid Squadron.
//! Tests in `tests/recent_b/cmdr_01dj_b.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Subtypes, Supertype,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, LookPick, ManaPayload, PlayerRef, Predicate, Selector, StaticAbility,
    StaticEffect, Value,
};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, SpendRestriction, b, cost, generic, u, w};

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// CR 700.8 — the four party types.
fn party_type() -> R {
    R::HasCreatureType(CreatureType::Cleric)
        .or(R::HasCreatureType(CreatureType::Rogue))
        .or(R::HasCreatureType(CreatureType::Warrior))
        .or(R::HasCreatureType(CreatureType::Wizard))
}

/// Base Camp — land; enters tapped. {T}: Add {C}. {T}: Add one mana of any
/// color, spent only on a Cleric, Rogue, Warrior or Wizard spell or on an
/// ability of one.
pub fn base_camp() -> CardDefinition {
    CardDefinition {
        name: "Base Camp",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                        SpendRestriction::PartySpellsOrAbilities,
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// The Destined Warrior — {1}{W}{U}{B} legendary 3/3 first strike,
/// vigilance, menace Human Warrior. Cleric, Rogue, Warrior and Wizard spells
/// you cast cost {1} less. At the beginning of combat on your turn, creatures
/// you control get +1/+0 until end of turn — +3/+0 instead with a full party.
pub fn the_destined_warrior() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance, Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Cleric, Rogue, Warrior, and Wizard spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: party_type(),
                amount: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::IfPred {
                    pred: Box::new(Predicate::ValueAtLeast(Value::PartyCount, Value::Const(4))),
                    then: Box::new(Value::Const(3)),
                    else_: Box::new(Value::ONE),
                },
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "The Destined Warrior",
            cost(&[generic(1), w(), u(), b()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Skyclave Plunder — {4}{U} sorcery. Look at the top 3 + party cards; put
/// three into your hand and the rest on the bottom in a random order.
pub fn skyclave_plunder() -> CardDefinition {
    CardDefinition {
        name: "Skyclave Plunder",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Sum(vec![Value::Const(3), Value::PartyCount]),
            take: Some(Value::Const(3)),
            rest_bottom_random: true,
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Practiced Tactics — {W} instant. It deals damage equal to twice the number
/// of creatures in your party to target attacking or blocking creature.
pub fn practiced_tactics() -> CardDefinition {
    CardDefinition {
        name: "Practiced Tactics",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
            amount: Value::Times(Box::new(Value::Const(2)), Box::new(Value::PartyCount)),
        },
        ..Default::default()
    }
}

/// Infinity Formula — {2}{W} Equipment. Enters: attach it to target creature
/// you control. Equipped creature gets +1/+2 and has "Whenever this creature
/// attacks, you gain 2 life." Equip {2}.
pub fn infinity_formula() -> CardDefinition {
    CardDefinition {
        name: "Infinity Formula",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
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

/// Mandroid Squadron — {1}{W} 0/4 artifact creature Human Soldier. Enters:
/// gain 2 life.
pub fn mandroid_squadron() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        ..creature(
            "Mandroid Squadron",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            0,
            4,
        )
    }
}
