//! Commander: 2026-10-25 (`01Y6ZRFx`) — the Stockman, Mad Fly-entist deck's
//! closers, mostly Ice Age / Coldsnap blue and artifacts. Tests in
//! `tests/recent_b/cmdr_01y6_c.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, CumulativeUpkeepCost, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
    Value, WardCost,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{DelayedTriggerKind, Duration, Effect, PlayerRef, Selector};
use crate::game::types::TurnStep;
use crate::mana::{Color, cost, generic, u, x};

fn artifact_creature(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn non_wall() -> R {
    R::Not(Box::new(R::HasCreatureType(CreatureType::Wall)))
}

/// "Activate only before attackers are declared" (with `DeclareAttackers`
/// too: "before blockers are declared").
fn before_attackers(and_attackers: bool) -> Predicate {
    let mut steps = vec![TurnStep::Untap, TurnStep::Upkeep, TurnStep::Draw, TurnStep::PreCombatMain, TurnStep::BeginCombat];
    if and_attackers {
        steps.push(TurnStep::DeclareAttackers);
    }
    Predicate::Any(steps.into_iter().map(Predicate::CurrentStepIs).collect())
}

/// "{2}: Target [color] creature gains [keyword] until end of turn."
fn skulkin(name: &'static str, mana: crate::mana::ManaCost, p: i32, t: i32, color: Color, keyword: Keyword) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::HasColor(color))),
                keyword,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact_creature(name, mana, vec![CreatureType::Scarecrow], p, t)
    }
}

/// Antler Skulkin — {5} 3/3 Scarecrow. {2}: target white creature gains
/// persist until end of turn.
pub fn antler_skulkin() -> CardDefinition {
    skulkin("Antler Skulkin", cost(&[generic(5)]), 3, 3, Color::White, Keyword::Persist)
}

/// Fang Skulkin — {2} 2/1 Scarecrow. {2}: target black creature gains wither
/// until end of turn.
pub fn fang_skulkin() -> CardDefinition {
    skulkin("Fang Skulkin", cost(&[generic(2)]), 2, 1, Color::Black, Keyword::Wither)
}

/// Clockwork Beast — {6} 0/4 Beast, enters with seven +1/+0 counters; sheds
/// one at the end of each combat it fought in; {X}, {T} during your upkeep:
/// up to X back, seven at most.
pub fn clockwork_beast() -> CardDefinition {
    let shed = |kind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::SelfSource),
        effect: Effect::DelayUntil {
            kind: DelayedTriggerKind::EndOfCombat,
            body: Box::new(Effect::RemoveCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusZero,
                amount: Value::ONE,
            }),
        },
    };
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusZero, Value::Const(7))),
        triggered_abilities: vec![shed(EventKind::Attacks), shed(EventKind::Blocks)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            condition: Some(Predicate::All(vec![
                Predicate::CurrentStepIs(TurnStep::Upkeep),
                Predicate::IsTurnOf(PlayerRef::You),
            ])),
            effect: Effect::AddCounterCapped {
                what: Selector::This,
                kind: CounterType::PlusOnePlusZero,
                amount: Value::XFromCost,
                cap: Value::Const(7),
            },
            ..Default::default()
        }],
        ..artifact_creature("Clockwork Beast", cost(&[generic(6)]), vec![CreatureType::Beast], 0, 4)
    }
}

/// Ronom Serpent — {5}{U} 5/6 snow Serpent. Can't attack unless defending
/// player controls a snow land. When you control no snow lands, sacrifice it.
pub fn ronom_serpent() -> CardDefinition {
    let snow_land = || R::Land.and(R::IsSnow);
    CardDefinition {
        name: "Ronom Serpent",
        cost: cost(&[generic(5), u()]),
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Serpent], ..Default::default() },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::CanAttackOnlyIfDefenderControls(Box::new(snow_land()))],
        // CR 603.8 — a state trigger.
        sacrifice_when: Some(Predicate::Not(Box::new(Predicate::SelectorExists(Selector::EachPermanent(
            snow_land().and(R::ControlledByYou),
        ))))),
        ..Default::default()
    }
}

/// Bazaar of Wonders — {3}{U}{U} world enchantment. ETB: exile all
/// graveyards. Whenever a player casts a spell, counter it if a card with the
/// same name is in a graveyard or a nontoken permanent with the same name is
/// on the battlefield.
pub fn bazaar_of_wonders() -> CardDefinition {
    CardDefinition {
        name: "Bazaar of Wonders",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![Supertype::World],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::ExileAllGraveyards { filter: None, opponents_only: false },
            },
            // CR 603.4 — the name check is made again on resolution.
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
                effect: Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::SameNameAsAGraveyardCard.or(R::SameNameAsANontokenPermanent),
                    },
                    then: Box::new(Effect::CounterSpell { what: Selector::TriggerSource }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Balduvian Shaman — {U} 1/1 Human Cleric Shaman. {T}: change the text of
/// target white enchantment you control without cumulative upkeep by
/// replacing one color word with another; it gains cumulative upkeep {1}.
pub fn balduvian_shaman() -> CardDefinition {
    CardDefinition {
        name: "Balduvian Shaman",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::ReplaceColorWord {
                    what: target_filtered(
                        R::Enchantment
                            .and(R::HasColor(Color::White))
                            .and(R::ControlledByYou)
                            .and(R::Not(Box::new(R::HasCumulativeUpkeep))),
                    ),
                    duration: Duration::Permanent,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[generic(1)]))),
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Arcum's Whistle — {3} artifact. {3}, {T}, before attackers: target
/// non-Wall creature the active player has controlled since the turn began;
/// unless that player pays {X} (its mana value), it attacks this turn if able
/// and is destroyed at the next end step if it didn't attack.
pub fn arcums_whistle() -> CardDefinition {
    CardDefinition {
        name: "Arcum's Whistle",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            condition: Some(before_attackers(false)),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                cost: WardCost::GenericTargetManaValue,
                then: Box::new(Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_filtered(
                            R::Creature
                                .and(non_wall())
                                .and(R::ControlledByActivePlayer)
                                .and(R::Not(Box::new(R::EnteredThisTurn))),
                        ),
                        keyword: Keyword::MustAttack,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::DelayUntilWithCapture {
                        kind: DelayedTriggerKind::NextEndStep,
                        capture: Selector::Target(0),
                        body: Box::new(Effect::If {
                            cond: Predicate::EntityMatches {
                                what: Selector::Target(0),
                                filter: R::Not(Box::new(R::AttackedThisTurn)),
                            },
                            then: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                            else_: Box::new(Effect::Noop),
                        }),
                    },
                ])),
                if_paid: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Acidic Dagger — {4} artifact. {4}, {T}, before blockers: whenever target
/// creature deals combat damage to a non-Wall creature this turn, destroy
/// that creature; when the target leaves the battlefield this turn,
/// sacrifice the Dagger.
pub fn acidic_dagger() -> CardDefinition {
    CardDefinition {
        name: "Acidic Dagger",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            condition: Some(before_attackers(true)),
            effect: Effect::Seq(vec![
                Effect::GrantTriggeredAbility {
                    what: target_filtered(R::Creature),
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource)
                            .with_filter(Predicate::EntityMatches { what: Selector::Target(0), filter: non_wall() }),
                        effect: Effect::Destroy { what: Selector::Target(0) },
                    }),
                    duration: Duration::EndOfTurn,
                },
                Effect::WhenTargetLeavesBattlefieldThisTurn {
                    what: Selector::Target(0),
                    body: Box::new(Effect::SacrificePermanent { what: Selector::This }),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
