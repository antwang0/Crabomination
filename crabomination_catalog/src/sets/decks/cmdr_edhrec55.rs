//! Commander: EDHREC ranks 2001-2500, one-card gaps (first batch) — Rasputin,
//! Nylea Keen-Eyed, Old Gnawbone / Vorosh, Kamahl Fist of Krosa, Urabrask
//! Heretic Praetor, Kiora the Rising Tide, Silver Surfer, Tivash, Kalakscion.
//! Tests in `tests/recent_b/cmdr_edhrec55.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, ModesWiden, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_dies, on_you_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

/// Rasputin, the Oneiromancer — {1}{W}{U} legendary 4/1 Human Wizard. Enters:
/// a dream counter per opponent, and each opponent creates a 1/1 red Goblin.
/// {T}, remove one or more dream counters: add that much {C} (the engine
/// removes them all, the largest legal choice). {T}, remove a dream counter:
/// a 2/2 white Knight with protection from red.
pub fn rasputin_the_oneiromancer() -> CardDefinition {
    let dreams = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Dream };
    let has_dream = || Predicate::ValueAtLeast(dreams(), Value::ONE);
    let goblin = TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        ..Default::default()
    };
    let knight = TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Knight], ..Default::default() },
        keywords: vec![Keyword::Protection(Color::Red)],
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::Dream, amount: Value::OpponentCount },
            Effect::CreateToken { who: PlayerRef::EachOpponent, count: Value::ONE, definition: Arc::new(goblin) },
        ]))],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                condition: Some(has_dream()),
                effect: Effect::WithX {
                    x: dreams(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter { what: Selector::This, kind: CounterType::Dream, amount: Value::XFromCost },
                        Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::XFromCost) },
                    ])),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(has_dream()),
                effect: Effect::Seq(vec![
                    Effect::RemoveCounter { what: Selector::This, kind: CounterType::Dream, amount: Value::ONE },
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(knight) },
                ]),
                ..Default::default()
            },
        ],
        ..creature(
            "Rasputin, the Oneiromancer",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            4,
            1,
        )
    }
}

/// Sparring Construct — {1} 1/1 artifact creature Construct. When it dies, put
/// a +1/+1 counter on target creature you control.
pub fn sparring_construct() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![on_dies(Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature("Sparring Construct", cost(&[generic(1)]), vec![CreatureType::Construct], 1, 1)
    }
}

/// Skeletal Grimace — {1}{B} Aura. Enchanted creature gets +1/+1 and has
/// "{B}: Regenerate this creature."
pub fn skeletal_grimace() -> CardDefinition {
    CardDefinition {
        name: "Skeletal Grimace",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has \"{B}: Regenerate this creature.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    mana_cost: cost(&[b()]),
                    effect: Effect::Regenerate { what: Selector::This },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Deepchannel Mentor — {5}{U} 2/2 Merfolk Rogue. Blue creatures you control
/// can't be blocked.
pub fn deepchannel_mentor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Blue creatures you control can't be blocked.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasColor(Color::Blue)).and(R::ControlledByYou),
                ),
                keyword: Keyword::Unblockable,
            },
        }],
        ..creature("Deepchannel Mentor", cost(&[generic(5), u()]), vec![CreatureType::Merfolk, CreatureType::Rogue], 2, 2)
    }
}

/// Nightmare's Thirst — {B} instant. You gain 1 life. Target creature gets
/// -X/-X until end of turn, X the life you gained this turn.
pub fn nightmares_thirst() -> CardDefinition {
    let x = || Value::Negate(Box::new(Value::LifeGainedThisTurn(PlayerRef::You)));
    CardDefinition {
        name: "Nightmare's Thirst",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
            Effect::PumpPT { what: target_filtered(R::Creature), power: x(), toughness: x(), duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Visions of Phyrexia — {2}{R}{R} enchantment. Upkeep: exile the top card,
/// you may play it this turn. End step: if you didn't play a card from exile
/// this turn, create a tapped Powerstone.
pub fn visions_of_phyrexia() -> CardDefinition {
    let didnt_play = || Predicate::Not(Box::new(Predicate::PlayedCardFromExileThisTurn(PlayerRef::You)));
    CardDefinition {
        name: "Visions of Phyrexia",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(didnt_play()),
                effect: Effect::If {
                    cond: didnt_play(),
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(TokenDefinition { tapped: true, ..crabomination_base::tokens::powerstone_token() }),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Ancient Bronze Dragon — {5}{G}{G} 7/7 Elder Dragon, flying. Combat damage to
/// a player: roll a d20; put X +1/+1 counters on each of up to two target
/// creatures, X the result.
pub fn ancient_bronze_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::RollDie {
                sides: 20,
                count: Value::ONE,
                modifier: Value::Const(0),
                reroll_at_most: 0,
                results: vec![(
                    1,
                    20,
                    Effect::ApplyToTargets {
                        max_targets: 2,
                        min_targets: 0,
                        filter: R::Creature,
                        effect: Box::new(Effect::AddCounter {
                            what: Selector::Target(0),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::TriggerEventAmount,
                        }),
                    },
                )],
                ignore_lowest: 0,
                on_doubles: None,
            },
        }],
        ..creature("Ancient Bronze Dragon", cost(&[generic(5), g(), g()]), vec![CreatureType::Elder, CreatureType::Dragon], 7, 7)
    }
}

/// Kamahl's Will — {3}{G} instant. Choose one (both if you control a
/// commander as you cast it): any number of target lands you control become
/// 1/1 Elemental creatures with vigilance, indestructible and haste until end
/// of turn; or each creature you control deals damage equal to its power to
/// target creature you don't control. The land mode owns twenty optional
/// slots; with both modes, the unfilled ones are holes ahead of the creature's.
pub fn kamahls_will() -> CardDefinition {
    CardDefinition {
        name: "Kamahl's Will",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        modes_widen: Some(ModesWiden::If(Predicate::YouControlACommander)),
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::ApplyToTargets {
                    max_targets: 20,
                    min_targets: 0,
                    filter: R::Land.and(R::ControlledByYou),
                    effect: Box::new(Effect::BecomeCreature {
                        what: Selector::Target(0),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        creature_types: vec![CreatureType::Elemental],
                        keywords: vec![Keyword::Vigilance, Keyword::Indestructible, Keyword::Haste],
                        duration: Duration::EndOfTurn,
                    }),
                },
                Effect::EachDealsDamageEqualToPower {
                    dealers: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    target: target_filtered(R::Creature.and(R::ControlledByYou.negate())),
                },
            ],
            min: 1,
            max: 1,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Cosmic Cube — {5} artifact, ward {2}. Whenever you attack, look at the top
/// six cards; you may cast a spell from among them with mana value at most
/// the greatest power among your attackers without paying its mana cost; the
/// rest go to the bottom in a random order.
pub fn cosmic_cube() -> CardDefinition {
    CardDefinition {
        name: "Cosmic Cube",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![on_you_attack(Effect::RevealTopMayCastOneFree {
            count: Value::Const(6),
            max_mv: Value::PowerOf(Box::new(Selector::GreatestPowerControlledMatching(R::Creature.and(R::IsAttacking)))),
            filter: None,
        })],
        ..Default::default()
    }
}
