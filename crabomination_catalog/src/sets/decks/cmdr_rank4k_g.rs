//! Commander: two-card gaps of EDHREC's commanders ranked 3501-4500 (the
//! 2026-10-14 scan), seventh file. Tests in `tests/recent_b/cmdr_rank4k_g.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, PlayerStaticTarget, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, generic, r, u, w};
use std::sync::Arc;

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

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn make(n: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: n,
        definition: Arc::new(def),
    }
}

fn robot_villain() -> Vec<CreatureType> {
    vec![CreatureType::Robot, CreatureType::Villain]
}

// ── Ultron, Machine Overlord ─────────────────────────────────────────────────

/// A.I.M. Synthoids — {2} 1/3 Robot Villain artifact creature. ETB: surveil 2.
pub fn aim_synthoids() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
        ..creature("A.I.M. Synthoids", cost(&[generic(2)]), robot_villain(), 1, 3)
    }
}

/// Ultron Drone — {3} 2/3 Robot Villain artifact creature. Power-up — {6}: two
/// +1/+1 counters on it and a 2/2 colorless Robot Villain artifact creature.
pub fn ultron_drone() -> CardDefinition {
    let drone = TokenDefinition {
        name: "Robot Villain".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Villain],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            power_up: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                make(Value::ONE, drone),
            ]),
            ..Default::default()
        }],
        ..creature("Ultron Drone", cost(&[generic(3)]), robot_villain(), 2, 3)
    }
}

// ── Appa, the Vigilant / Hakoda, Selfless Commander ──────────────────────────

/// Aang, Air Nomad — {3}{W}{W} legendary 5/4 flying, vigilance Human Avatar
/// Ally. Other creatures you control have vigilance.
pub fn aang_air_nomad() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have vigilance.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature.and(R::OtherThanSource)),
                keyword: Keyword::Vigilance,
            },
        }],
        ..legend(
            "Aang, Air Nomad",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Avatar, CreatureType::Ally],
            5,
            4,
        )
    }
}

/// Katara, Heroic Healer — {4}{W} legendary 2/3 lifelink Human Warrior Ally.
/// ETB: a +1/+1 counter on each other creature you control.
pub fn katara_heroic_healer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: yours(R::Creature.and(R::OtherThanSource)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..legend(
            "Katara, Heroic Healer",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            2,
            3,
        )
    }
}

// ── Winter, Team Player ──────────────────────────────────────────────────────

/// Winter, Team Player — {4}{R} legendary 3/3 convoke Human Warrior. Whenever
/// you cast a noncreature spell, creatures you control get +1/+0 until end of
/// turn.
pub fn winter_team_player() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(crate::effect::shortcut::cast_is_noncreature()),
            effect: Effect::PumpPT {
                what: yours(R::Creature),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..legend(
            "Winter, Team Player",
            cost(&[generic(4), r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Goblin Wizardry — {3}{R} instant. Two 1/1 red Goblin Wizard tokens with
/// prowess.
pub fn goblin_wizardry() -> CardDefinition {
    let wizard = TokenDefinition {
        name: "Goblin Wizard".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Prowess],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Wizard],
            ..Default::default()
        },
        triggered_abilities: vec![crate::effect::shortcut::prowess()],
        ..Default::default()
    };
    spell(
        "Goblin Wizardry",
        cost(&[generic(3), r()]),
        CardType::Instant,
        make(Value::Const(2), wizard),
    )
}

// ── Layla Hassan ─────────────────────────────────────────────────────────────

/// Yuriko, Blade of the Mighty — {3}{W} legendary 2/3 Human Samurai. During
/// combat, players can't cast spells or activate non-mana abilities. Whenever a
/// creature you control attacks a player alone, it gains double strike until
/// end of turn. (Residual: the combat lock is Hand to Hand's, which still lets
/// a flash permanent be cast during combat.)
pub fn yuriko_blade_of_the_mighty() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During combat, players can't cast spells or activate abilities that aren't mana abilities.",
            effect: StaticEffect::NoInstantsOrAbilitiesDuringCombat,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsAttackingAlone,
                },
            ),
            effect: Effect::GrantKeyword {
                what: Selector::TriggerSource,
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
        }],
        ..legend(
            "Yuriko, Blade of the Mighty",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Samurai],
            2,
            3,
        )
    }
}

/// What Must Be Done — {3}{W}{W} sorcery. Choose one — destroy all artifacts
/// and creatures; or return target historic permanent card from your
/// graveyard to the battlefield, with two additional +1/+1 counters if it's a
/// creature.
pub fn what_must_be_done() -> CardDefinition {
    spell(
        "What Must Be Done",
        cost(&[generic(3), w(), w()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::Destroy {
                what: Selector::EachPermanent(R::Artifact.or(R::Creature)),
            },
            Effect::Seq(vec![
                Effect::ClearLastMoved,
                Effect::Move {
                    what: target_filtered(
                        R::historic().and(R::PermanentCard).and(R::InYourGraveyard),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::LastMoved,
                        filter: R::Creature,
                    },
                    then: Box::new(Effect::AddCounter {
                        what: Selector::LastMoved,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        ]),
    )
}

// ── Bilbo, Unexpected Adventurer ─────────────────────────────────────────────

/// Bilbo, Unexpected Adventurer — {3}{W} legendary 2/2 Halfling Rogue; can't
/// be blocked by creatures with power 3 or greater. Combat damage to a player:
/// put up to one target nonland permanent card with mana value 3 or less from
/// a graveyard onto the battlefield under its owner's control.
pub fn bilbo_unexpected_adventurer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::PowerAtLeast(3)))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::PermanentCard
                    .and(R::Nonland)
                    .and(R::ManaValueAtMost(3))
                    .and(R::InGraveyard),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                        tapped: false,
                    },
                }),
            },
        }],
        ..legend(
            "Bilbo, Unexpected Adventurer",
            cost(&[generic(3), w()]),
            vec![CreatureType::Halfling, CreatureType::Rogue],
            2,
            2,
        )
    }
}

/// Dáin, Lord of the Iron Hills — {1}{W} legendary 2/2 vigilance Dwarf Noble,
/// storied. With an enduring story, creatures can't attack you unless their
/// controller pays {1} for each of them.
pub fn dain_lord_of_the_iron_hills() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![
            StaticAbility {
                description: "Storied (If you control three or more artifacts, legendaries, and/or Sagas, you have an enduring story for the rest of the game.)",
                effect: StaticEffect::Storied,
            },
            StaticAbility {
                description: "As long as you have an enduring story, creatures can't attack you unless their controller pays {1} for each of those creatures.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::HasEnduringStory { who: PlayerRef::You },
                    inner: Box::new(StaticEffect::AttackTaxToController {
                        amount: Value::ONE,
                        protect_planeswalkers: false,
                        filter: None,
                    }),
                },
            },
        ],
        ..legend(
            "Dáin, Lord of the Iron Hills",
            cost(&[generic(1), w()]),
            vec![CreatureType::Dwarf, CreatureType::Noble],
            2,
            2,
        )
    }
}

// ── The Ninth Doctor ─────────────────────────────────────────────────────────

/// Kumena's Awakening — {2}{U}{U} enchantment, ascend. At the beginning of your
/// upkeep, each player draws a card; with the city's blessing, only you do.
pub fn kumenas_awakening() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![super::super::ascend()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::HasCityBlessing { who: PlayerRef::You },
                then: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Draw {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::ONE,
                }),
            },
        }],
        ..spell(
            "Kumena's Awakening",
            cost(&[generic(2), u(), u()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

/// Quakebringer — {3}{R}{R} 5/4 Giant Berserker; foretell {2}{R}{R}. Your
/// opponents can't gain life. At the beginning of your upkeep, 2 damage to each
/// opponent — from the battlefield, or from your graveyard while you control a
/// Giant.
pub fn quakebringer() -> CardDefinition {
    let quake = || Effect::DealDamage {
        to: Selector::Player(PlayerRef::EachOpponent),
        amount: Value::Const(2),
    };
    CardDefinition {
        foretell_cost: Some(cost(&[generic(2), r(), r()])),
        static_abilities: vec![StaticAbility {
            description: "Your opponents can't gain life.",
            effect: StaticEffect::PlayerCannotGainLife {
                target: PlayerStaticTarget::EachOpponent,
            },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: quake(),
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::FromYourGraveyard,
                )
                .with_filter(Predicate::SelectorExists(yours(R::HasCreatureType(
                    CreatureType::Giant,
                )))),
                effect: quake(),
            },
        ],
        ..creature(
            "Quakebringer",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Giant, CreatureType::Berserker],
            5,
            4,
        )
    }
}
