//! Commander: one-card gaps of the 2026-10-15 full-list EDHREC scan. Tests
//! in `tests/recent_b/cmdr_scan7k_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, landfall, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, u, w};
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

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn plus_one(what: Selector, n: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: n }
}

/// Iron Man Armor — {3} Equipment. ETB: attach to target creature you
/// control. Equipped creature gets +2/+1 and has flying. {2}: if it isn't a
/// creature, it becomes a 0/0 flying Construct Hero artifact creature that
/// gets +1/+1 per artifact you control until end of turn. Equip {2}. (Iron
/// Man, Futurist Paragon.)
pub fn iron_man_armor() -> CardDefinition {
    let artifacts = || Value::count(yours(R::HasCardType(CardType::Artifact)));
    CardDefinition {
        name: "Iron Man Armor",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 1, keywords: vec![Keyword::Flying], ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::If {
                cond: Predicate::EntityMatches { what: Selector::This, filter: R::Not(Box::new(R::Creature)) },
                then: Box::new(Effect::Seq(vec![
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(0),
                        toughness: Value::Const(0),
                        creature_types: vec![CreatureType::Construct, CreatureType::Hero],
                        keywords: vec![Keyword::Flying],
                        duration: Duration::EndOfTurn,
                    },
                    // "+1/+1 for each artifact you control", read live.
                    Effect::SetBasePtLive { what: Selector::This, power: artifacts(), toughness: artifacts(), duration: Duration::EndOfTurn },
                ])),
                else_: Box::new(Effect::Noop),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ajani Goldmane — {2}{W}{W} planeswalker, loyalty 4. +1: gain 2 life. −1:
/// a +1/+1 counter on each creature you control; they gain vigilance until
/// end of turn. −6: a white Avatar whose power and toughness are your life
/// total. (Yoshimaru, Beloved Companion.)
pub fn ajani_goldmane() -> CardDefinition {
    let avatar = TokenDefinition {
        name: "Avatar".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Avatar], ..Default::default() },
        dynamic_pt: Some((Value::LifeOf(PlayerRef::You), Value::LifeOf(PlayerRef::You))),
        ..Default::default()
    };
    CardDefinition {
        name: "Ajani Goldmane",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Ajani], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::Seq(vec![
                    plus_one(yours(R::Creature), Value::ONE),
                    Effect::GrantKeyword { what: yours(R::Creature), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(avatar) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Saved by the Shell — {1}{G} instant, {1} less with a Turtle: a +1/+1
/// counter on target creature you control; it gains trample, hexproof and
/// indestructible until end of turn. (Mikey & Leo, Chaos & Order.)
pub fn saved_by_the_shell() -> CardDefinition {
    let grant = |k: Keyword| Effect::GrantKeyword { what: Selector::Target(0), keyword: k, duration: Duration::EndOfTurn };
    CardDefinition {
        name: "Saved by the Shell",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast if you control a Turtle.",
            effect: StaticEffect::SelfCostReducedIfPredicate {
                amount: 1,
                condition: Predicate::SelectorCountAtLeast { sel: yours(R::HasCreatureType(CreatureType::Turtle)), n: Value::ONE },
            },
        }],
        effect: Effect::Seq(vec![
            plus_one(target_filtered(R::Creature.and(R::ControlledByYou)), Value::ONE),
            grant(Keyword::Trample),
            grant(Keyword::Hexproof),
            grant(Keyword::Indestructible),
        ]),
        ..Default::default()
    }
}

/// Avatar of Burgeoning Echoes — {G}{U} 2/3 Avatar. Landfall: empower Jace 2.
/// Planeswalkers you control have "[−10]: a +1/+1 counter on target creature
/// for each land you control." (Tam, Observant Sequencer.)
pub fn avatar_of_burgeoning_echoes() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![landfall(Effect::EmpowerJace { who: PlayerRef::You, count: Value::Const(2) })],
        static_abilities: vec![StaticAbility {
            description: "Planeswalkers you control have \"[−10]: Put a +1/+1 counter on target creature for each land you control.\"",
            effect: StaticEffect::PlaneswalkersHaveLoyaltyAbilities {
                abilities: vec![LoyaltyAbility {
                    loyalty_cost: -10,
                    effect: plus_one(target_filtered(R::Creature), Value::count(yours(R::Land))),
                    ..Default::default()
                }],
            },
        }],
        ..creature("Avatar of Burgeoning Echoes", cost(&[g(), u()]), vec![CreatureType::Avatar], 2, 3)
    }
}

/// Fire Nation Salvagers — {3}{B}{B} 3/3 Human Soldier; menace. ETB: a +1/+1
/// counter on target creature or Vehicle you control. Whenever creatures you
/// control with counters deal combat damage to a player, put target creature
/// or Vehicle card from that player's graveyard onto the battlefield under
/// your control. (The Fire Nation Drill.)
pub fn fire_nation_salvagers() -> CardDefinition {
    let creature_or_vehicle = || R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle));
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            etb(plus_one(target_filtered(creature_or_vehicle().and(R::ControlledByYou)), Value::ONE)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::WithAnyCounter })
                    .once_per_batch(),
                effect: Effect::Move {
                    what: target_filtered(creature_or_vehicle().and(R::InOpponentGraveyard).and(R::ControlledByTriggerPlayer)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            },
        ],
        ..creature("Fire Nation Salvagers", cost(&[generic(3), b(), b()]), vec![CreatureType::Human, CreatureType::Soldier], 3, 3)
    }
}

/// Gimli, Mournful Avenger — {1}{R}{G} legendary 3/2 Dwarf Warrior;
/// indestructible while two or more creatures died under your control this
/// turn. Whenever another creature you control dies, a +1/+1 counter on him;
/// the third time that resolves in a turn, he fights up to one target
/// creature you don't control.
pub fn gimli_mournful_avenger() -> CardDefinition {
    use crate::mana::r;
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Gimli has indestructible as long as two or more creatures died under your control this turn.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Indestructible,
                condition: Predicate::CreaturesDiedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::Const(2) },
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::OtherThanSource,
            }),
            effect: Effect::Seq(vec![
                plus_one(Selector::This, Value::ONE),
                Effect::NthResolutionThisTurn {
                    branches: vec![
                        Effect::Noop,
                        Effect::Noop,
                        Effect::ReflexiveTrigger {
                            body: Box::new(Effect::OptionalTargets {
                                min: 0,
                                body: Box::new(Effect::Fight {
                                    attacker: Selector::This,
                                    defender: target_filtered(R::Creature.and(R::Not(Box::new(R::ControlledByYou)))),
                                }),
                            }),
                        },
                    ],
                },
            ]),
        }],
        supertypes: vec![Supertype::Legendary],
        ..creature("Gimli, Mournful Avenger", cost(&[generic(1), r(), g()]), vec![CreatureType::Dwarf, CreatureType::Warrior], 3, 2)
    }
}
