//! An eighteenth wave — more Foundations (FDN) commons/uncommons: attack
//! triggers, an artifact-gated lord, an upkeep wheel, Auras and tokens. Tests
//! in `crabomination/src/tests/recent18.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, support, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color};

/// Charging Bandits — {4}{B} Human Rogue 3/3. Whenever it attacks, it gets
/// +2/+0 until end of turn.
pub fn charging_bandits() -> CardDefinition {
    CardDefinition {
        name: "Charging Bandits",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Dazzling Angel — {2}{W} Angel 2/3 with flying. Whenever another creature you
/// control enters, you gain 1 life.
pub fn dazzling_angel() -> CardDefinition {
    CardDefinition {
        name: "Dazzling Angel",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                },
            ),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Dragon Trainer — {3}{R}{R} Human 1/1. ETB: create a 4/4 red Dragon with
/// flying.
pub fn dragon_trainer() -> CardDefinition {
    let dragon = TokenDefinition {
        name: "Dragon".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Dragon Trainer",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: dragon,
        })],
        ..Default::default()
    }
}

/// Goblin Tomb Raider — {R} Goblin Pirate 1/2. As long as you control an
/// artifact, it gets +1/+0 and has haste.
pub fn goblin_tomb_raider() -> CardDefinition {
    CardDefinition {
        name: "Goblin Tomb Raider",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "+1/+0 and haste while you control an artifact.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(1),
                },
                power: 1,
                toughness: 0,
                keywords: vec![Keyword::Haste],
            },
        }],
        ..Default::default()
    }
}

/// Sanguine Syphoner — {1}{B} Vampire Warlock 1/3. Whenever it attacks, each
/// opponent loses 1 life and you gain 1 life.
pub fn sanguine_syphoner() -> CardDefinition {
    CardDefinition {
        name: "Sanguine Syphoner",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ]))],
        ..Default::default()
    }
}

/// Sky Crier — {1}{W} Bird Citizen 1/1 with flying and lifelink. {3}{W}: You and
/// target opponent each draw a card.
pub fn sky_crier() -> CardDefinition {
    CardDefinition {
        name: "Sky Crier",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Citizen],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Soulmender — {W} Human Cleric 1/1. {T}: You gain 1 life.
pub fn soulmender() -> CardDefinition {
    CardDefinition {
        name: "Soulmender",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stormfist Crusader — {B}{R} Human Knight 2/2 with menace. At your upkeep,
/// each player draws a card and loses 1 life.
pub fn stormfist_crusader() -> CardDefinition {
    CardDefinition {
        name: "Stormfist Crusader",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Run Away Together — {1}{U} Instant. Choose two target creatures controlled by
/// different players. Return those creatures to their owners' hands. (The
/// different-controllers restriction is approximated as any two creatures.)
pub fn run_away_together() -> CardDefinition {
    CardDefinition {
        name: "Run Away Together",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        },
        ..Default::default()
    }
}

/// Captured by Lagacs — {1}{G}{W} Aura. Enchant creature; it can't attack or
/// block. When this Aura enters, support 2.
pub fn captured_by_lagacs() -> CardDefinition {
    CardDefinition {
        name: "Captured by Lagacs",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(support(2))],
        ..Default::default()
    }
}

/// Battle Screech — {2}{W}{W} Sorcery. Create two 1/1 white Bird tokens with
/// flying. (The tap-three-white-creatures Flashback is dropped.)
pub fn battle_screech() -> CardDefinition {
    let bird = TokenDefinition {
        name: "Bird".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Battle Screech",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: bird },
        ..Default::default()
    }
}

/// Quag Vampires — {B} Vampire Rogue 1/1 with Multikicker {1}{B} and swampwalk.
/// Enters with a +1/+1 counter for each time it was kicked.
pub fn quag_vampires() -> CardDefinition {
    CardDefinition {
        name: "Quag Vampires",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![
            Keyword::Multikicker(cost(&[generic(1), b()])),
            Keyword::Landwalk(crate::card::LandType::Swamp),
        ],
        enters_with_counters: Some((crate::card::CounterType::PlusOnePlusOne, Value::TimesKicked)),
        ..Default::default()
    }
}

/// Bear Cub — {1}{G} Bear 2/2 vanilla.
pub fn bear_cub() -> CardDefinition {
    CardDefinition {
        name: "Bear Cub",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// Sworn Guardian — {1}{U} Merfolk Warrior 1/3 vanilla.
pub fn sworn_guardian() -> CardDefinition {
    CardDefinition {
        name: "Sworn Guardian",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        ..Default::default()
    }
}

/// Hunter's Edge — {3}{G} Sorcery. Put a +1/+1 counter on target creature you
/// control. Then that creature deals damage equal to its power to target
/// creature you don't control.
pub fn hunters_edge() -> CardDefinition {
    CardDefinition {
        name: "Hunter's Edge",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::DealDamage {
                to: Selector::Target(1),
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Kitsa, Otterball Elite — {1}{U} Legendary Otter Wizard 1/3 with vigilance and
/// prowess. {T}: Draw a card, then discard a card. (The copy-spell ability is
/// dropped — see TODO.md.)
pub fn kitsa_otterball_elite() -> CardDefinition {
    CardDefinition {
        name: "Kitsa, Otterball Elite",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Prowess],
        triggered_abilities: vec![crate::effect::shortcut::prowess()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
