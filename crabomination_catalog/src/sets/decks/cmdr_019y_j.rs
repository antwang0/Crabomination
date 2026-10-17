//! Commander: 2026-10-17 (`019Y6J4R`) closers, tenth batch. Tests in
//! `tests/recent_b/cmdr_019y_j.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, prowess, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};
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

fn token(def: TokenDefinition, n: i32) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(def) }
}

/// Kree Commandos — {2}{W} 2/1 flying, vigilance, prowess Kree Soldier Villain.
pub fn kree_commandos() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Prowess],
        triggered_abilities: vec![prowess()],
        ..creature(
            "Kree Commandos",
            cost(&[generic(2), w()]),
            vec![CreatureType::Kree, CreatureType::Soldier, CreatureType::Villain],
            2,
            1,
        )
    }
}

/// Lightning Reaver — {3}{B}{R} 3/3 haste, fear Zombie Beast. Combat damage to
/// a player: a charge counter. Your end step: damage equal to its charge
/// counters to each opponent.
pub fn lightning_reaver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::Fear],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
                },
            },
        ],
        ..creature("Lightning Reaver", cost(&[generic(3), b(), r()]), vec![CreatureType::Zombie, CreatureType::Beast], 3, 3)
    }
}

/// Ghoulraiser — {1}{B}{B} 2/2 Zombie. ETB: return a Zombie card at random
/// from your graveyard to your hand.
pub fn ghoulraiser() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ReturnRandomFromGraveyard {
            who: PlayerRef::You,
            filter: R::HasCreatureType(CreatureType::Zombie),
            count: Value::ONE,
        })],
        ..creature("Ghoulraiser", cost(&[generic(1), b(), b()]), vec![CreatureType::Zombie], 2, 2)
    }
}

/// Hallway Heckler // Vicious Verse — {2}{R} 2/3 Elemental Sorcerer, enters
/// prepared. {T}, discard a card: draw a card. Vicious Verse ({B/R} sorcery):
/// 1 damage to target opponent.
pub fn hallway_heckler() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Prepared, Value::ONE)),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        prepare_spell: Some(Arc::new(CardDefinition {
            name: "Vicious Verse",
            cost: cost(&[hybrid(Color::Black, Color::Red)]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::DealDamage { to: target_filtered(R::OpponentPlayer), amount: Value::ONE },
            ..Default::default()
        })),
        ..creature("Hallway Heckler", cost(&[generic(2), r()]), vec![CreatureType::Elemental, CreatureType::Sorcerer], 2, 3)
    }
}

/// Craftwork Crusher — {3}{R}{R}{G}{G} 7/5 trample artifact creature Boar
/// Construct. ETB, choose two: 4 damage to target creature or planeswalker; a
/// 2/2 colorless Wizard Soldier token named Cadet; draw a card.
pub fn craftwork_crusher() -> CardDefinition {
    let cadet = TokenDefinition {
        name: "Cadet".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard, CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::ChooseN {
            picks: vec![0, 2],
            modes: vec![
                Effect::DealDamage { to: target_filtered(R::Creature.or(R::Planeswalker)), amount: Value::Const(4) },
                token(cadet, 1),
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ],
        })],
        ..creature(
            "Craftwork Crusher",
            cost(&[generic(3), r(), r(), g(), g()]),
            vec![CreatureType::Boar, CreatureType::Construct],
            7,
            5,
        )
    }
}

/// Head of the Hunt — {2}{B}{B} 4/3 flash Wolf. A creature an opponent
/// controls that would die is exiled instead; when you do, a 2/2 green Wolf.
pub fn head_of_the_hunt() -> CardDefinition {
    let wolf = TokenDefinition {
        name: "Wolf".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "If a creature an opponent controls would die, exile it instead. When you do, create a 2/2 green Wolf creature token.",
            effect: StaticEffect::ExileDyingOpponentCreatures { when_you_do: Some(Box::new(token(wolf, 1))), tokens_too: true },
        }],
        ..creature("Head of the Hunt", cost(&[generic(2), b(), b()]), vec![CreatureType::Wolf], 4, 3)
    }
}

/// Robotics Mastery — {4}{U} flash Aura. ETB: two 1/1 colorless flying Robot
/// artifact creature tokens. Enchanted creature gets +2/+2.
pub fn robotics_mastery() -> CardDefinition {
    let robot = TokenDefinition {
        name: "Robot".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Robotics Mastery",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 2, ..Default::default() }),
        triggered_abilities: vec![etb(token(robot, 2))],
        ..Default::default()
    }
}

/// Anointer of Valor — {5}{W} 3/5 flying Angel. Whenever a creature attacks,
/// you may pay {3}; when you do, a +1/+1 counter on it.
pub fn anointer_of_valor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer),
            effect: Effect::MayPay {
                description: "Pay {3} to put a +1/+1 counter on the attacker?".into(),
                mana_cost: cost(&[generic(3)]),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: None,
            },
        }],
        ..creature("Anointer of Valor", cost(&[generic(5), w()]), vec![CreatureType::Angel], 3, 5)
    }
}

/// Utrom Scientists — {2}{U} 2/2 artifact creature. ETB: tap up to one target
/// creature and put a stun counter on it.
pub fn utrom_scientists() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::Tap { what: Selector::Target(0) },
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
            ])),
        })],
        ..creature(
            "Utrom Scientists",
            cost(&[generic(2), u()]),
            vec![CreatureType::Utrom, CreatureType::Robot, CreatureType::Scientist],
            2,
            2,
        )
    }
}

/// Suspension Field — {1}{W} enchantment. ETB: you may exile target creature
/// with toughness 3 or greater until this leaves the battlefield.
pub fn suspension_field() -> CardDefinition {
    CardDefinition {
        name: "Suspension Field",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Exile target creature with toughness 3 or greater?".into(),
            body: Box::new(Effect::ExileUntilSourceLeaves {
                what: target_filtered(R::Creature.and(R::ToughnessAtLeast(3))),
                return_to: ExileReturnZone::Battlefield,
            }),
        })],
        ..Default::default()
    }
}

/// Crackdown Construct — {4} 2/2 artifact creature. Whenever you activate a
/// non-mana ability of an artifact or creature, it gets +1/+1 until end of
/// turn.
pub fn crackdown_construct() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivated, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact.or(R::Creature) },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Crackdown Construct", cost(&[generic(4)]), vec![CreatureType::Construct], 2, 2)
    }
}

/// Norwood Priestess — {2}{G}{G} 1/1 Elf Druid. {T}: you may put a green
/// creature card from your hand onto the battlefield; your turn, before
/// attackers are declared.
pub fn norwood_priestess() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::Any(vec![
                    Predicate::CurrentStepIs(TurnStep::Upkeep),
                    Predicate::CurrentStepIs(TurnStep::Draw),
                    Predicate::CurrentStepIs(TurnStep::PreCombatMain),
                    Predicate::CurrentStepIs(TurnStep::BeginCombat),
                ]),
            ])),
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Creature.and(R::HasColor(Color::Green)),
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
            ..Default::default()
        }],
        ..creature("Norwood Priestess", cost(&[generic(2), g(), g()]), vec![CreatureType::Elf, CreatureType::Druid], 1, 1)
    }
}
