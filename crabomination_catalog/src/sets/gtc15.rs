//! Gatecrash (GTC) wave 15: Gate-matters payoffs, combat-static beaters, and
//! the blue Primordial. Tests in `classic_sets/gtc`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
    StaticAbility, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_any, target_filtered};
use crate::effect::{Effect, Predicate, Selector, StaticEffect};
use crate::mana::{b, cost, generic, r, u, w};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: t, ..Default::default() }
}

fn aura() -> Subtypes {
    Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() }
}

/// Alms Beast — {2}{W}{B} 6/6 Beast. Creatures blocking or blocked by it have
/// lifelink.
pub fn alms_beast() -> CardDefinition {
    CardDefinition {
        name: "Alms Beast",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Beast]),
        power: 6,
        toughness: 6,
        static_abilities: vec![StaticAbility {
            description: "Creatures blocking or blocked by this creature have lifelink.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::Permanent },
                applies_to: Selector::CreaturesInCombatWith(Box::new(Selector::This)),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Lifelink],
            },
        }],
        ..Default::default()
    }
}

/// Hold the Gates — {2}{W} Enchantment. Creatures you control get +0/+1 for
/// each Gate you control and have vigilance.
pub fn hold_the_gates() -> CardDefinition {
    CardDefinition {
        name: "Hold the Gates",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control get +0/+1 for each Gate you control.",
                effect: StaticEffect::PumpTeamByControlledPermanents {
                    applies_to: R::Creature.and(R::ControlledByYou),
                    count_filter: R::HasLandType(crate::card::LandType::Gate),
                    per_power: 0,
                    per_toughness: 1,
                    count_graveyard: false,
                },
            },
            StaticAbility {
                description: "Creatures you control have vigilance.",
                effect: StaticEffect::PumpTeamIf {
                    condition: Predicate::EntityMatches { what: Selector::This, filter: R::Permanent },
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::Vigilance],
                },
            },
        ],
        ..Default::default()
    }
}

/// Way of the Thief — {3}{U} Aura. Enchant creature; it gets +2/+2 and can't be
/// blocked as long as you control a Gate.
pub fn way_of_the_thief() -> CardDefinition {
    CardDefinition {
        name: "Way of the Thief",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 2, ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature can't be blocked as long as you control a Gate.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasLandType(crate::card::LandType::Gate).and(R::ControlledByYou),
                    ),
                    n: Value::Const(1),
                },
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Unblockable],
            },
        }],
        ..Default::default()
    }
}

/// Diluvian Primordial — {5}{U}{U} 5/5 Avatar, Flying. ETB: for each opponent,
/// you may cast up to one target instant or sorcery card from that player's
/// graveyard without paying its mana cost; exile it if it would leave the stack.
pub fn diluvian_primordial() -> CardDefinition {
    CardDefinition {
        name: "Diluvian Primordial",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Avatar]),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::HasCardType(CardType::Instant)
                .or(R::HasCardType(CardType::Sorcery))
                .and(R::InOpponentGraveyard),
            effect: Box::new(Effect::CastWithoutPayingImmediate {
                what: Selector::Target(0),
                source_zone: crate::card::Zone::Graveyard,
                exile_after: true,
                copy: false,
            }),
        })],
        ..Default::default()
    }
}

/// Five-Alarm Fire — {1}{R}{R} Enchantment. Whenever a creature you control
/// deals combat damage, put a blaze (charge) counter on this; remove five to
/// deal 5 damage to any target.
pub fn five_alarm_fire() -> CardDefinition {
    let gain_counter = |kind: EventKind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::YourControl),
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Charge,
            amount: Value::ONE,
        },
    };
    CardDefinition {
        name: "Five-Alarm Fire",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            gain_counter(EventKind::DealsCombatDamageToPlayer),
            gain_counter(EventKind::DealsCombatDamageToCreature),
        ],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Charge, 5)),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(5) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

